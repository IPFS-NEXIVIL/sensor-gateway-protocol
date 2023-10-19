use axum::{http::StatusCode, routing::post, Router};
use tower_http::cors::CorsLayer;

pub fn startWifi() {
    tokio::spawn({
        async move {
            // build our application with a single route
            let app = Router::new()
                .route("/", post(recv_data))
                .layer(CorsLayer::permissive());

            // run it with hyper on localhost:3000
            axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap()
        }
    });
}

async fn recv_data(data: String) -> Result<(), (StatusCode, String)> {
    println!("{}", data);
    Ok(())
}
