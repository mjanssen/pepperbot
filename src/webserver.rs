pub mod libs;

use axum::{
    body::{self, Body, Empty, Full},
    extract::{Path, State},
    http::{header, HeaderValue, Request},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{include_dir, Dir};
use libs::redis::{get_config, get_subscriber_amount};
use log::{error, info};
use std::env;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Span;

use crate::libs::version::print_version;

static STATIC_DIR: Dir<'_> = include_dir!("./html/build");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    print_version();

    info!("Starting webserver service");
    
    if let Ok(redis_domain) = env::var("REDIS_URL") {
        match redis::Client::open(redis_domain.clone()) {
            Ok(redis_client) => {
                let app = axum::Router::new()
                    .route("/", get(render_index))
                    .route("/index.html", get(render_index))
                    .route("/*path", get(static_path))
                    .with_state(redis_client)
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(DefaultMakeSpan::new().include_headers(true))
                            .on_request(|request: &Request<Body>, _span: &Span| {
                                info!("{} {}", request.method(), request.uri().path())
                            }),
                    );

                let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await?;
            }
            Err(_) => error!("Redis connection failed"),
        }
    }

    Ok(())
}

async fn render_index(State(redis_service): State<redis::Client>) -> impl IntoResponse {
    match STATIC_DIR.get_file("index.html") {
        None => Response::builder()
            .status(axum::http::status::StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => {
            let content = file.contents_utf8();
            let template = set_template_values(content, redis_service).await;

            Response::builder()
                .status(axum::http::status::StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("text/html").unwrap(),
                )
                .body(body::boxed(Full::from(template)))
                .unwrap()
        }
    }
}

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(axum::http::status::StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => {
            let content = file.contents();

            Response::builder()
                .status(axum::http::status::StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                )
                .body(body::boxed(Full::from(content)))
                .unwrap()
        }
    }
}

async fn set_template_values(contents: Option<&str>, redis_client: redis::Client) -> String {
    match redis_client.get_connection() {
        Ok(mut con) => {
            let subscriber_count = get_subscriber_amount(&mut con).await;
            let message_count: String = get_config(
                &mut con,
                libs::redis::Config::MessagesSentKey,
                libs::redis::Database::CONFIG,
            )
            .unwrap_or("1337".to_string());

            let deals_count: String = get_config(
                &mut con,
                libs::redis::Config::DealsSentKey,
                libs::redis::Database::CONFIG,
            )
            .unwrap_or("1337".to_string());

            let template = contents.unwrap_or("");
            template
                .replace(
                    "__SUBSCRIBER_COUNT__",
                    format!("{}", subscriber_count).as_str(),
                )
                .replace("__MESSAGES_SENT__", &message_count)
                .replace("__DEALS_SENT__", &deals_count)
        }
        _ => contents.unwrap_or("").to_string(),
    }
}
