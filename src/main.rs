use tokio::net::TcpListener;

use axum::{extract::Request, routing::get, Router};
use vad::run_file;

mod audio;
mod ffmpeg;
mod g711;
mod server;
mod vad;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if &args[1] == "listen" {
        listen().await;
    } else if &args[1] == "analyse" {
        // lode_and_decode(&args[2]);
        run_file(&args[2]).unwrap();
    } else {
        println!("Unrecognised arg {:?}", args);
    }
}

async fn listen() {
    let app = Router::new().route("/health", get(health));

    let listener = TcpListener::bind("0.0.0.0:3210")
        .await
        .expect("Failed to bind to port 3210");

    axum::serve(listener, app).await.unwrap()
}

async fn health() -> &'static str {
    "hello"
}

async fn analyse(request: Request) {
    //
}
