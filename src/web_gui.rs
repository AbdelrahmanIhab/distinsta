use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;

use super::Client;
use super::discovery::UserInfo;

// API Response types
#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

// API Request types
#[derive(Deserialize)]
struct ShareRequest {
    image_path: String,
    image_id: String,
    target_user: String,
    views: u32,
}

#[derive(Deserialize)]
struct RequestImageRequest {
    owner: String,
    image_id: String,
    p2p_address: String,
}

// API Handlers
async fn get_peers_handler(State(client): State<Arc<Client>>) -> Json<Vec<UserInfo>> {
    let peers = client.get_peers().await.unwrap_or_default();
    Json(peers)
}

async fn share_image_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<ShareRequest>,
) -> Json<ApiResponse> {
    match client
        .share_image(&req.image_path, req.image_id, req.target_user, req.views)
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Image shared successfully".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn request_image_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<RequestImageRequest>,
) -> Json<ApiResponse> {
    match client
        .request_image_from_peer(&req.owner, &req.image_id, &req.p2p_address)
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Image received".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn list_my_images_handler(State(client): State<Arc<Client>>) -> Json<Vec<String>> {
    let owned = client.owned_images.read().await;
    let images: Vec<String> = owned.keys().cloned().collect();
    Json(images)
}

async fn list_received_images_handler(State(client): State<Arc<Client>>) -> Json<Vec<String>> {
    let received = client.received_images.read().await;
    let images: Vec<String> = received.keys().cloned().collect();
    Json(images)
}

pub async fn start_web_server(client: Arc<Client>) {
    // Build router
    let app = Router::new()
        .route("/api/peers", get(get_peers_handler))
        .route("/api/share", post(share_image_handler))
        .route("/api/request", post(request_image_handler))
        .route("/api/my_images", get(list_my_images_handler))
        .route("/api/received", get(list_received_images_handler))
        .nest_service("/", ServeDir::new("static"))
        .with_state(client);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("🌐 GUI available at: http://localhost:8080");

    axum::serve(listener, app).await.unwrap();
}
