use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn init_subscriber() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zero2prod=DEBUG,tower_http=DEBUG".into()),
        )
        .with(fmt::layer())
        .init();
}
