use lapin::Connection;
use lapin::uri::{AMQPAuthority, AMQPUri, AMQPUserInfo};

use crate::settings::Settings;

pub async fn new_amqp_connection(settings: &Settings) -> Connection {
    let mq_settings = settings.mq.as_ref().unwrap();

    Connection::connect_uri(
        AMQPUri {
            authority: AMQPAuthority {
                host: mq_settings.host.clone().unwrap_or("localhost".into()),
                port: mq_settings.port.unwrap_or(5672),
                userinfo: AMQPUserInfo {
                    username: mq_settings.username.clone().unwrap_or_default(),
                    password: mq_settings.password.clone().unwrap_or_default(),
                },
            },
            vhost: if let Some(vhost) = mq_settings.vhost.clone() {
                if vhost.starts_with('/') {
                    vhost
                } else {
                    format!("/{}", vhost)
                }
            } else {
                "/".into()
            },
            ..Default::default()
        },
        lapin::ConnectionProperties::default().with_connection_name("lapin".into()),
    )
    .await
    .unwrap()
}
