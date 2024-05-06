use crate::settings::{Database, DatabaseSSLMode, DatabaseType};

pub fn database_url(db_settings: &Database) -> String {
    match db_settings.database_type {
        DatabaseType::Postgres => {
            format!(
                "postgres://{}{}{}@{}:{}{}?sslmode={}",
                db_settings.username.clone().unwrap_or_default(),
                if db_settings.password.is_none() {
                    ""
                } else {
                    ":"
                },
                db_settings.password.clone().unwrap_or_default(),
                db_settings.host,
                db_settings.port,
                if db_settings.name.is_none() {
                    "".to_string()
                } else {
                    format!("/{}", db_settings.name.clone().unwrap())
                },
                match db_settings.ssl_mode.clone().unwrap_or_default() {
                    DatabaseSSLMode::Disable => "disable",
                    DatabaseSSLMode::Allow => "allow",
                    DatabaseSSLMode::Prefer => "prefer",
                    DatabaseSSLMode::Require => "require",
                    DatabaseSSLMode::VerifyCA => "verify-ca",
                    DatabaseSSLMode::VerifyFull => "verify-full",
                }
            )
        }
        DatabaseType::Sqlite => {
            format!("sqlite://{}", db_settings.name.clone().unwrap_or_default())
        }
        DatabaseType::Mysql | DatabaseType::MariaDB => {
            format!(
                "mysql://{}{}{}@{}:{}{}?charset={}",
                db_settings.username.clone().unwrap_or_default(),
                if db_settings.password.is_none() {
                    ""
                } else {
                    ":"
                },
                db_settings.password.clone().unwrap_or_default(),
                db_settings.host,
                db_settings.port,
                if db_settings.name.is_none() {
                    "".to_string()
                } else {
                    format!("/{}", db_settings.name.clone().unwrap())
                },
                db_settings.charset.clone().unwrap_or("utf8".to_string())
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url() {
        let db_settings = Database {
            database_type: DatabaseType::Postgres,
            username: Some("user".to_string()),
            password: Some("password".to_string()),
            host: "localhost".to_string(),
            port: 5432,
            name: Some("database".to_string()),
            charset: None,
            ssl_mode: None,
        };
        assert_eq!(
            database_url(&db_settings),
            "postgres://user:password@localhost:5432/database?sslmode=prefer"
        );

        let db_settings = Database {
            database_type: DatabaseType::Sqlite,
            username: None,
            password: None,
            host: "localhost".to_string(),
            port: 5432,
            name: Some("database".to_string()),
            charset: None,
            ssl_mode: None,
        };
        assert_eq!(database_url(&db_settings), "sqlite://database");

        let db_settings = Database {
            database_type: DatabaseType::Mysql,
            username: Some("user".to_string()),
            password: Some("password".to_string()),
            host: "localhost".to_string(),
            port: 3306,
            name: Some("database".to_string()),
            charset: Some("utf8".to_string()),
            ssl_mode: None,
        };
        assert_eq!(
            database_url(&db_settings),
            "mysql://user:password@localhost:3306/database?charset=utf8"
        );
    }
}
