use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushRequest {
    pub user_id: String,
    pub project_id: String,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBroadcastEnvelope {
    pub project_id: String,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default, rename = "sessionId")]
    pub session_id: Option<String>,
    pub data: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushResponse {
    pub status: &'static str,
    pub delivered: bool,
    pub delivered_nodes: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
}
