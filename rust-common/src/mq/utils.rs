use crate::settings::Settings;

pub(crate) fn parse_amqp_settings(settings: &Settings) -> String {
    let mq_settings = settings.mq.as_ref().unwrap();
    let credentials = match (mq_settings.username.as_ref(), mq_settings.password.as_ref()) {
        (None, Some(password)) => format!("{}@", password), // password only
        (Some(username), Some(password)) => format!("{}:{}@", username, password), // username and password
        _ => "".to_string(),
    };

    format!(
        "amqp://{}{}:{}{}",
        credentials,
        mq_settings.host.clone().unwrap_or("localhost".to_string()),
        mq_settings.port.unwrap_or(5672),
        match mq_settings.vhost.as_ref() {
            Some(vhost) => format!(
                "/{}",
                if vhost.starts_with("/") {
                    &vhost[1..]
                } else {
                    vhost
                }
            ),
            None => "/".into(),
        }
    )
}

mod tests {
    use crate::settings::Mq;

    use super::*;

    #[test]
    fn test_parse_amqp_settings() {
        {
            let settings = Settings {
                mq: Some(Mq {
                    host: Some("localhost".to_string()),
                    port: Some(5672),
                    username: Some("user".to_string()),
                    password: Some("password".to_string()),
                    vhost: Some("/".to_string()),
                }),
                ..Default::default()
            };

            assert_eq!(
                parse_amqp_settings(&settings),
                "amqp://user:password@localhost:5672/"
            );
        }
        {
            let settings = Settings {
                mq: Some(Mq {
                    host: Some("localhost".to_string()),
                    port: Some(5672),
                    username: None,
                    password: Some("password".to_string()),
                    vhost: Some("/".to_string()),
                }),
                ..Default::default()
            };

            assert_eq!(
                parse_amqp_settings(&settings),
                "amqp://password@localhost:5672/"
            );
        }
        {
            let settings = Settings {
                mq: Some(Mq {
                    host: Some("localhost".to_string()),
                    port: Some(5672),
                    username: None,
                    password: None,
                    vhost: Some("/".to_string()),
                }),
                ..Default::default()
            };

            assert_eq!(parse_amqp_settings(&settings), "amqp://localhost:5672/");
        }
    }
}
