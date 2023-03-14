use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

/// Load the configuration for the app.
///
/// # Errors
///
/// Will return an error when:
///
/// - the specified configuration file cannot be found;
/// - the loaded configuration cannot be deserialized into [`Settings`].
pub fn build() -> Result<Settings, ConfigError> {
    // `config` will look for a file named, "configuration," in the top-level
    // directory with any extension it knows how to parse: yaml, json, toml, etc.
    Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?
        .try_deserialize()
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
    pub database_name: String,
}

impl DatabaseSettings {
    #[must_use]
    pub fn connection_string(&self) -> Secret<String> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        );

        Secret::new(connection_string)
    }
}
