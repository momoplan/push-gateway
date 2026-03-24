mod config;
mod delivery;
mod models;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use config::AppConfig;
use delivery::RedisDelivery;
use models::{AgentBroadcastEnvelope, ErrorResponse, PushRequest, PushResponse};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    redis_delivery: Arc<RedisDelivery>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app_config = match AppConfig::load() {
        Ok(config) => {
            println!("✓ 配置文件加载成功");
            config
        }
        Err(err) => {
            println!("⚠ 配置文件加载失败,使用默认配置: {}", err);
            AppConfig::default()
        }
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(app_config.logging.level.clone()));

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(env_filter)
        .init();

    let state = AppState {
        redis_delivery: Arc::new(
            RedisDelivery::new(app_config.redis.clone())
                .await
                .map_err(std::io::Error::other)?,
        ),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/push", post(push))
        .with_state(state);

    let addr: SocketAddr = format!(
        "{}:{}",
        app_config.server.bind_address, app_config.server.port
    )
    .parse()?;

    info!("Push Gateway 启动在 {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn push(
    State(state): State<AppState>,
    Json(payload): Json<PushRequest>,
) -> Result<Json<PushResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.user_id.trim().is_empty() {
        return Err(bad_request("userId 不能为空"));
    }
    if payload.project_id.trim().is_empty() {
        return Err(bad_request("projectId 不能为空"));
    }
    if payload.data.is_null() {
        return Err(bad_request("data 不能为空"));
    }

    let delivered_nodes = state
        .redis_delivery
        .dispatch_to_user(
            payload.user_id.trim(),
            &AgentBroadcastEnvelope {
                project_id: payload.project_id.trim().to_string(),
                request_id: payload.request_id,
                session_id: payload.session_id,
                data: payload.data,
            },
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(PushResponse {
        status: "ok",
        delivered: delivered_nodes > 0,
        delivered_nodes,
    }))
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "INVALID_REQUEST",
            message: message.into(),
        }),
    )
}

fn internal_error(message: String) -> (StatusCode, Json<ErrorResponse>) {
    error!("push gateway internal error: {}", message);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: "INTERNAL_ERROR",
            message,
        }),
    )
}
