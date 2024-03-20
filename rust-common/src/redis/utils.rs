use crate::settings::Settings;

macro_rules! none_if_not_exist {
    ($value:expr) => {
        if $value.is_none() || $value.as_ref().unwrap().is_empty() {
            None
        } else {
            $value
        }
    };
}

pub(crate) fn parse_redis_settings(settings: &Settings) -> String {
    let redis_settings = settings.redis.as_ref().unwrap();

    let username = none_if_not_exist!(redis_settings.username.clone());
    let password = none_if_not_exist!(redis_settings.password.clone());

    let credentials = match (username.as_ref(), password.as_ref()) {
        (None, Some(password)) => format!("{}@", password), // password only
        (Some(username), Some(password)) => format!("{}:{}@", username, password), // username and password
        _ => "".to_string(),
    };

    format!(
        "redis://{}{}:{}/{}",
        credentials,
        redis_settings
            .host
            .clone()
            .unwrap_or("localhost".to_string()),
        redis_settings.port.unwrap_or(6379),
        redis_settings.db.unwrap_or(0)
    )
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::settings::RedisMode;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_redis_settings() {
        {
            let settings = Settings {
                redis: Some(crate::settings::Redis {
                    mode: Some(RedisMode::Standalone),
                    host: Some("localhost".to_string()),
                    port: Some(6379),
                    username: Some("user".to_string()),
                    password: Some("password".to_string()),
                    db: Some(0),
                }),
                ..Default::default()
            };

            assert_eq!(
                parse_redis_settings(&settings),
                "redis://user:password@localhost:6379/0"
            );
        }
        {
            let settings = Settings {
                redis: Some(crate::settings::Redis {
                    mode: Some(RedisMode::Standalone),
                    host: Some("localhost".to_string()),
                    port: Some(6379),
                    username: None,
                    password: Some("password".to_string()),
                    db: Some(0),
                }),
                ..Default::default()
            };

            assert_eq!(
                parse_redis_settings(&settings),
                "redis://password@localhost:6379/0"
            );
        }
        {
            let settings = Settings {
                redis: Some(crate::settings::Redis {
                    mode: Some(RedisMode::Standalone),
                    host: Some("localhost".to_string()),
                    port: Some(6379),
                    username: None,
                    password: None,
                    db: Some(0),
                }),
                ..Default::default()
            };

            assert_eq!(parse_redis_settings(&settings), "redis://localhost:6379/0");
        }
    }
}
