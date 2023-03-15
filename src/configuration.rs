use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};
use std::env;

/// Load the configuration for the app.
///
/// # Errors
///
/// Will return an error when:
///
/// - the specified configuration file cannot be found;
/// - the loaded configuration cannot be deserialized into [`Settings`].
///
/// # Panics
///
/// Will panic if the `APP_ENVIRONMENT` environment variable couldn't be detected
/// or parsed.
#[allow(clippy::unwrap_used)]
pub fn build() -> Result<Settings, ConfigError> {
    // The configuration files should be located inside the "configuration"
    // directory.
    let configuration_directory =
        env::current_dir().unwrap().join("configuration");

    // Determine the operating environment and load the appropriate configuration.
    let environment: AppEnvironment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .unwrap();

    // `config` will look for a file named, "configuration," in the top-level
    // directory with any extension it knows how to parse: yaml, json, toml, etc.
    let mut settings: Settings = Config::builder()
        .add_source(
            config::File::from(configuration_directory.join("base"))
                .required(true),
        )
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str()))
                .required(true),
        )
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?
        .try_deserialize()?;

    // Railway generates a `PORT` env. variable at runtime. Read it.
    if let Ok(port) = env::var("PORT") {
        settings.application.port =
            port.parse().expect("the port should be a valid number");
    }

    Ok(settings)
}

enum AppEnvironment {
    Local,
    Production,
}

impl AppEnvironment {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either 'local' or \
                'production'."
            )),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
}

impl DatabaseSettings {
    #[must_use]
    pub fn connect_options(&self) -> PgConnectOptions {
        let mut options = PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .database(&self.name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}
