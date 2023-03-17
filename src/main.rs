use anyhow::Context;
use newsletter::{build_app, configuration, telemetry::init_subscriber};
use tracing::debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_subscriber(std::io::stdout);

    let configuration = configuration::build()
        .expect("app configuration should be present and valid");

    debug!("Detected the following configuration: {configuration:?}");

    let app = build_app(configuration)
        .await
        .context("something went wrong building the app")?;
    app.run()
        .await
        .context("something went wrong running the app")?;

    Ok(())
}
