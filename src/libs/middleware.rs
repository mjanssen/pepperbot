use log::info;
use axum::{
    http::header::USER_AGENT,
    http::Request,
    middleware::Next,
    response::Response,
    RequestPartsExt,
};
use axum_client_ip::InsecureClientIp;

pub async fn request_logger<B>(req: Request<B>, next: Next<B>) -> Response {
    let (mut parts, body) = req.into_parts();

    if parts.uri.path() != "/_health" {
        let remote_addr: InsecureClientIp = parts.extract().await.unwrap();
        let user_agent = parts.headers.get(USER_AGENT);

        if let Some(user_agent_header) = user_agent {
            info!(
                "{} - \"{} {} {:?}\" {:?}",
                remote_addr.0,
                parts.method,
                parts.uri.path(),
                parts.version,
                user_agent_header
            )
        } else {
            info!(
                "{} - \"{} {} {:?}\" \"\"",
                remote_addr.0,
                parts.method,
                parts.uri.path(),
                parts.version
            )
        }
    }

    let req = Request::from_parts(parts, body);
    next.run(req).await
}
