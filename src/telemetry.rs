use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn init_subscriber() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "zero2prod=DEBUG,tower_http=DEBUG".into());
    let formatting_layer =
        BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .init();
}
