use serde::{Deserialize, Serialize};
use crate::discovery::ImageInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PRequest {
    /// Request an image from a peer
    RequestImage {
        requester: String,
        image_id: String,
    },
    /// Get list of images owned by a peer
    GetImageList {
        requester: String,
    },
    /// Request permission to view an image
    RequestAccess {
        requester: String,
        requester_p2p_address: String,
        image_id: String,
        requested_views: u32,
    },
    /// Approve a view request
    ApproveRequest {
        requester: String,
        image_id: String,
        granted_views: u32,
        owner: String,
        owner_p2p_address: String,
    },
    /// Request thumbnail/preview of an image
    RequestThumbnail {
        requester: String,
        image_id: String,
    },
    /// Notify owner that a view was consumed
    NotifyViewConsumed {
        viewer: String,
        image_id: String,
        remaining_views: u32,
    },
    /// Update viewer's permissions (from owner to viewer)
    UpdateViewerPermissions {
        viewer: String,
        image_id: String,
        new_view_count: u32,
    },
    /// Revoke access to an image
    RevokeAccess {
        viewer: String,
        image_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PResponse {
    /// Image data with embedded metadata
    ImageData {
        image_id: String,
        data: Vec<u8>,
    },
    /// List of available images
    ImageList {
        images: Vec<ImageInfo>,
    },
    /// Access denied
    AccessDenied {
        reason: String,
    },
    /// Error occurred
    Error {
        message: String,
    },
    /// Request received successfully
    RequestReceived {
        message: String,
    },
    /// Approval sent successfully
    ApprovalSent {
        message: String,
    },
    /// Thumbnail data (low resolution preview)
    ThumbnailData {
        image_id: String,
        data: Vec<u8>,
    },
    /// Acknowledgment of notification
    Acknowledged {
        message: String,
    },
    /// Access revoked
    AccessRevoked {
        message: String,
    },
}
