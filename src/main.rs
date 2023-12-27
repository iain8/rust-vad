use std::sync::Arc;

use ort::Session;
use tokio::net::TcpListener;

use axum::{
    body::Bytes,
    debug_handler,
    extract::State,
    routing::{get, post},
    Json, Router,
};
use vad::{analyse_data, run_file, Marker};

mod g711;
mod vad;

#[derive(Clone)]
struct AppState {
    vad_model: Arc<Session>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if &args[1] == "listen" {
        listen().await;
    } else if &args[1] == "analyse" {
        run_file(&args[2]).unwrap();
    } else {
        println!("Unrecognised arg {:?}", args);
    }
}

async fn listen() {
    let vad_model = vad::load_model().expect("failed to load VAD model");

    let shared_state = AppState {
        vad_model: Arc::new(vad_model),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/analyse", post(analyse))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3210")
        .await
        .expect("Failed to bind to port 3210");

    println!("Listening on port 3210");

    axum::serve(listener, app).await.unwrap()
}

async fn health() -> &'static str {
    "hello"
}

#[debug_handler]
async fn analyse(State(state): State<AppState>, body: Bytes) -> Json<Vec<Marker>> {
    let result = analyse_data(state.vad_model, body.to_vec()).unwrap();

    Json(result)

    // match analyse_data(state.vad_model, body.to_vec()) {
    //     Ok(result) => (StatusCode::OK, Json(result)),
    //     Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())),
    // }
}
