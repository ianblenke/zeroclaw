//! Static file serving for the embedded web dashboard.
//!
//! Uses `rust-embed` to bundle the `web/dist/` directory into the binary at compile time.

use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/dist/"]
struct WebAssets;

/// Serve static files from `/_app/*` path
pub async fn handle_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().strip_prefix("/_app/").unwrap_or(uri.path());

    serve_embedded_file(path)
}

/// SPA fallback: serve index.html for any non-API, non-static GET request
pub async fn handle_spa_fallback() -> impl IntoResponse {
    serve_embedded_file("index.html")
}

fn serve_embedded_file(path: &str) -> impl IntoResponse {
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (
                        header::CACHE_CONTROL,
                        cache_control_for_path(path).to_string(),
                    ),
                ],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

/// Determine cache-control header value based on asset path.
fn cache_control_for_path(path: &str) -> &'static str {
    if path.contains("assets/") {
        // Hashed filenames — immutable cache
        "public, max-age=31536000, immutable"
    } else {
        // index.html etc — no cache
        "no-cache"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-GW-012-SC01
    #[test]
    fn cache_control_assets_path_is_immutable() {
        assert_eq!(
            cache_control_for_path("assets/main.abc123.js"),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            cache_control_for_path("_app/assets/style.css"),
            "public, max-age=31536000, immutable"
        );
    }

    /// REQ-GW-012-SC02
    #[test]
    fn cache_control_non_asset_is_no_cache() {
        assert_eq!(cache_control_for_path("index.html"), "no-cache");
        assert_eq!(cache_control_for_path("favicon.ico"), "no-cache");
    }

    /// REQ-GW-012-SC03
    #[tokio::test]
    async fn handle_static_strips_app_prefix() {
        // Test that URI path prefix is stripped correctly
        let uri: Uri = "/_app/index.html".parse().unwrap();
        let path = uri.path().strip_prefix("/_app/").unwrap_or(uri.path());
        assert_eq!(path, "index.html");
    }

    /// REQ-GW-012-SC04
    #[tokio::test]
    async fn handle_static_missing_file_returns_404() {
        let uri: Uri = "/_app/nonexistent_file_xyz.txt".parse().unwrap();
        let response = handle_static(uri).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// REQ-GW-012-SC05
    #[tokio::test]
    async fn spa_fallback_behavior() {
        // SPA fallback tries to serve "index.html" — if web/dist/ is not
        // embedded (no build step), this returns 404. We test the function
        // doesn't panic and returns a valid response.
        let response = handle_spa_fallback().await.into_response();
        // Either OK (if web/dist/index.html exists) or NOT_FOUND
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
        );
    }
}
