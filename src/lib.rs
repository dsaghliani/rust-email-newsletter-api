#![allow(clippy::unused_async)]

pub mod configuration;
pub mod telemetry;

mod domain;
mod extractors;
mod routes;
mod startup;

pub use startup::run;
