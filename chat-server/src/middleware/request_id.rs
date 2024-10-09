use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::warn;

use super::X_REQUEST_ID;

pub async fn x_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id exists, use it; otherwise, generate a new one
    let req_id = match req.headers().get(X_REQUEST_ID) {
        Some(val) => Some(val.clone()),
        None => {
            let id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&id) {
                Ok(val) => {
                    req.headers_mut().append(X_REQUEST_ID, val.clone());
                    Some(val)
                }
                Err(e) => {
                    warn!("failed to parse request id: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;

    let Some(req_id) = req_id else {
        return res;
    };

    res.headers_mut()
        .append(header::HeaderName::from_static(X_REQUEST_ID), req_id);
    res
}
