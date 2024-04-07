use std::pin::Pin;

use futures::{FutureExt, StreamExt};
use lapin::options::BasicCancelOptions;
use lapin::protocol::constants::REPLY_SUCCESS;
use opentelemetry::trace::TraceContextExt;
use teloxide::prelude::Update;
use teloxide::stop::{mk_stop_token, StopFlag, StopToken};
use teloxide::update_listeners::{AsUpdateStream, UpdateListener};

use crate::bot::utils::extract_span_from_delivery;
use crate::settings::Settings;

pub struct MqUpdateListener {
    channel: lapin::Channel,
    consumer: lapin::Consumer,
    consumer_tag: String,
    token: StopToken,
    flag: StopFlag,
}

impl<'a> AsUpdateStream<'a> for MqUpdateListener {
    type StreamErr = lapin::Error;
    type Stream =
        Pin<Box<dyn futures::Stream<Item = Result<Update, Self::StreamErr>> + Unpin + Send + 'a>>;

    fn as_stream(&'a mut self) -> Self::Stream {
        let flag = self.flag.clone();
        let consumer = self.consumer.clone();
        
        Box::pin(consumer.filter_map(move |delivery| {
            assert!(!flag.is_stopped(), "Update listener stopped");
            if self.consumer.state() != lapin::ConsumerState::Active
                && self.consumer.state() != lapin::ConsumerState::ActiveWithDelegate
            {
                panic!("Consumer state is not Active.");
            }

            async move {
                match delivery {
                    Ok(delivery) => {
                        let cx = extract_span_from_delivery(&delivery);

                        match serde_json::from_slice::<Update>(&delivery.data) {
                            Ok(mut update) => {
                                update.cx = Some(cx);
                                Some(Ok(update))
                            }
                            Err(e) => {
                                log::error!("Error deserializing message: {}", e);
                                cx.span().record_error(&e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving message: {}", e);
                        None
                    }
                }
            }.boxed()
        }))
    }
}

impl UpdateListener for MqUpdateListener {
    type Err = lapin::Error;

    fn stop_token(&mut self) -> StopToken {
        self.token.clone()
    }
}

static EXCHANGE_NAME: &str = "bot_updates";

impl MqUpdateListener {
    pub async fn new(
        service_name: &str,
        amqp_conn: lapin::Connection,
        settings: &Settings,
    ) -> Result<Self, lapin::Error> {
        let channel = amqp_conn.create_channel().await.unwrap();
        log::debug!("Created amqp channel");

        let queue_name = format!("{}:queue.{}", EXCHANGE_NAME, service_name);
        channel
            .exchange_declare(
                EXCHANGE_NAME,
                lapin::ExchangeKind::Fanout,
                lapin::options::ExchangeDeclareOptions {
                    auto_delete: true,
                    ..Default::default()
                },
                Default::default(),
            )
            .await?;
        log::debug!("Declared exchange: {}", EXCHANGE_NAME);

        channel
            .queue_declare(
                &queue_name,
                lapin::options::QueueDeclareOptions {
                    auto_delete: true,
                    ..Default::default()
                },
                Default::default(),
            )
            .await?;
        log::debug!("Declared queue: {}", queue_name);

        channel
            .queue_bind(
                &queue_name,
                EXCHANGE_NAME,
                "",
                lapin::options::QueueBindOptions::default(),
                Default::default(),
            )
            .await?;
        log::debug!("Bound queue: {}", queue_name);

        let consumer = channel
            .basic_consume(
                &queue_name,
                settings.instance_id.as_ref().unwrap().as_str(),
                lapin::options::BasicConsumeOptions {
                    no_ack: true,
                    ..Default::default()
                },
                Default::default(),
            )
            .await?;
        log::debug!(
            "Created consumer: {}",
            settings.instance_id.as_ref().unwrap()
        );

        let (token, flag) = mk_stop_token();

        Ok(MqUpdateListener {
            channel,
            consumer,
            consumer_tag: settings.instance_id.as_ref().unwrap().clone(),
            token,
            flag,
        })
    }

    pub async fn stop(&mut self) -> Result<(), lapin::Error> {
        self.channel
            .basic_cancel(&self.consumer_tag, BasicCancelOptions::default())
            .await?;

        self.channel
            .close(REPLY_SUCCESS, "application stopped")
            .await?;

        Ok(())
    }
}
