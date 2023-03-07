use std::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000")
        .expect("the provided address should be valid");

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener).await.unwrap();
}
