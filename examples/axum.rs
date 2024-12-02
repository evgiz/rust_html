use axum::{response::IntoResponse, routing::get, Router};
use rand::Rng;

use rust_html::*;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start axum serve");
}

async fn root() -> impl IntoResponse {
    let random_number: i32 = rand::thread_rng().gen_range(0..100);
    let page = rhtml! {r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>rust_html axum</title>
            </head>
            <body>
                <h1>Welcome to rust_html!</h1>
                <p>
                    Here's a random number: {random_number}
                    <br/>
                    Refresh to get another one.
                </p>
            </body>
        </html>
    "#};
    page
}
