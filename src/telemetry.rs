use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn init_subscriber<Sink>(sink: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "newsletter=DEBUG,tower_http=DEBUG".into());
    let formatting_layer = BunyanFormattingLayer::new("newsletter".into(), sink);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .init();
}

pub use request_id_make_span::RequestIdMakeSpan;

mod request_id_make_span {
    use axum::http::Request;
    use tower_http::trace::MakeSpan;
    use tracing::{Level, Span};
    use uuid::Uuid;

    #[derive(Clone, Debug)]
    pub struct RequestIdMakeSpan {
        level: Level,
        include_headers: bool,
    }

    impl RequestIdMakeSpan {
        /// Create a new `DefaultMakeSpan`.
        #[must_use]
        pub const fn new() -> Self {
            Self {
                level: Level::DEBUG,
                include_headers: false,
            }
        }

        /// Set the [`Level`] used for the [tracing span].
        ///
        /// Defaults to [`Level::DEBUG`].
        ///
        /// [tracing span]: https://docs.rs/tracing/latest/tracing/#spans
        #[must_use]
        pub const fn level(mut self, level: Level) -> Self {
            self.level = level;
            self
        }

        /// Include request headers on the [`Span`].
        ///
        /// By default headers are not included.
        ///
        /// [`Span`]: tracing::Span
        #[must_use]
        pub const fn include_headers(mut self, include_headers: bool) -> Self {
            self.include_headers = include_headers;
            self
        }
    }

    impl Default for RequestIdMakeSpan {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<B> MakeSpan<B> for RequestIdMakeSpan {
        #[allow(clippy::cognitive_complexity)]
        fn make_span(&mut self, request: &Request<B>) -> Span {
            // This ugly macro is needed, unfortunately, because `tracing::span!`
            // required the level argument to be static. Meaning we can't just pass
            // `self.level`.
            macro_rules! make_span {
                ($level:expr) => {
                    if self.include_headers {
                        tracing::span!(
                            $level,
                            "request",
                            request_id = %Uuid::new_v4(),
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                            headers = ?request.headers(),
                        )
                    } else {
                        tracing::span!(
                            $level,
                            "request",
                            request_id = %Uuid::new_v4(),
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                        )
                    }
                }
            }

            match self.level {
                Level::ERROR => {
                    make_span!(Level::ERROR)
                }
                Level::WARN => {
                    make_span!(Level::WARN)
                }
                Level::INFO => {
                    make_span!(Level::INFO)
                }
                Level::DEBUG => {
                    make_span!(Level::DEBUG)
                }
                Level::TRACE => {
                    make_span!(Level::TRACE)
                }
            }
        }
    }
}
