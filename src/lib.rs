#![allow(clippy::unused_async)]

pub mod configuration;
pub mod telemetry;

mod domain;
mod email_client;
mod extractors;
mod routes;
mod startup;
mod state;

pub use email_client::EmailClient;
pub use startup::{create_email_client, run};

use state::AppState;
