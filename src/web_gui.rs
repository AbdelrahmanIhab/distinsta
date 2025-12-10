use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
    response::IntoResponse,
    http::{header, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use tower_http::services::ServeDir;
use base64::Engine;

use super::{Client, ViewRequest, ViewableImageInfo};
use super::discovery::{UserInfo, ImageInfo};

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
struct UploadRequest {
    image_path: String,
    image_id: String,
}

#[derive(Deserialize)]
struct RequestAccessRequest {
    owner_p2p_address: String,
    image_id: String,
    requested_views: u32,
}

#[derive(Deserialize)]
struct ApproveRequestRequest {
    requester: String,
    requester_p2p_address: String,
    image_id: String,
    granted_views: u32,
}

#[derive(Deserialize)]
struct RejectRequestRequest {
    requester: String,
    image_id: String,
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
    let image_path = req.image_path;
    let image_id = req.image_id;
    let target_user = req.target_user;
    let views = req.views;

    let share_result = client.share_image(&image_path, image_id.clone(), target_user.clone(), views).await;

    if share_result.is_ok() {
        // Spawn publish in background
        let client_clone = client.clone();
        let id_clone = image_id.clone();
        let path_clone = image_path.clone();
        let user_clone = target_user.clone();
        tokio::spawn(async move {
            let _ = client_clone.publish_image(id_clone, path_clone, vec![user_clone]).await;
        });
    }

    match share_result {
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

async fn upload_image_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<UploadRequest>,
) -> Json<ApiResponse> {
    match client.upload_image(&req.image_path, req.image_id).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Image uploaded successfully".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn request_access_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<RequestAccessRequest>,
) -> Json<ApiResponse> {
    match client
        .request_access(&req.owner_p2p_address, req.image_id, req.requested_views)
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Access request sent".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn approve_request_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<ApproveRequestRequest>,
) -> Json<ApiResponse> {
    match client
        .approve_request(
            req.requester,
            req.requester_p2p_address,
            req.image_id,
            req.granted_views,
        )
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Request approved".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn reject_request_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<RejectRequestRequest>,
) -> Json<ApiResponse> {
    match client.reject_request(req.requester, req.image_id).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "Request rejected".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

async fn get_pending_requests_handler(
    State(client): State<Arc<Client>>,
) -> Json<Vec<ViewRequest>> {
    let requests = client.get_pending_requests().await;
    Json(requests)
}

async fn get_all_images_handler(State(client): State<Arc<Client>>) -> Json<Vec<ImageInfo>> {
    let images = client.get_all_images().await.unwrap_or_default();
    Json(images)
}

async fn get_viewable_images_handler(
    State(client): State<Arc<Client>>,
) -> Json<Vec<ViewableImageInfo>> {
    let images = client.get_viewable_images().await;
    Json(images)
}

#[derive(Deserialize)]
struct ViewImageRequest {
    image_id: String,
}

async fn view_image_handler(
    State(client): State<Arc<Client>>,
    Json(req): Json<ViewImageRequest>,
) -> Json<ApiResponse> {
    match client.view_image_with_tracking(&req.image_id).await {
        Ok(path) => {
            // Read the image file and encode as base64
            match fs::read(&path) {
                Ok(image_bytes) => {
                    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
                    Json(ApiResponse {
                        success: true,
                        message: format!("Image viewed successfully"),
                        data: Some(serde_json::json!({
                            "path": path,
                            "image_data": base64_image
                        })),
                    })
                }
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: format!("Failed to read image: {}", e),
                    data: None,
                }),
            }
        }
        Err(e) => Json(ApiResponse {
            success: false,
            message: e.to_string(),
            data: None,
        }),
    }
}

pub async fn start_web_server(client: Arc<Client>) {
    // Build router
    let app = Router::new()
        .route("/api/peers", get(get_peers_handler))
        .route("/api/share", post(share_image_handler))
        .route("/api/upload", post(upload_image_handler))
        .route("/api/request_access", post(request_access_handler))
        .route("/api/approve", post(approve_request_handler))
        .route("/api/reject", post(reject_request_handler))
        .route("/api/pending_requests", get(get_pending_requests_handler))
        .route("/api/all_images", get(get_all_images_handler))
        .route("/api/viewable_images", get(get_viewable_images_handler))
        .route("/api/view_image", post(view_image_handler))
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
