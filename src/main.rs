use std::net::TcpListener;

use zero2prod::get_configuration;

#[tokio::main]
async fn main() {
    let configuration =
        get_configuration().expect("app configuration should be present");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener =
        TcpListener::bind(&address).expect("the provided address should be valid");

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener).await.unwrap();
}
