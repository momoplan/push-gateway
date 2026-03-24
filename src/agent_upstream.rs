use crate::config::AgentUpstreamConfig;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct AgentUpstreamClient {
    client: Client,
    agent_message_path: String,
    api_base_url: String,
}

impl AgentUpstreamClient {
    pub fn new(config: AgentUpstreamConfig) -> Result<Self, String> {
        let timeout = Duration::from_secs(config.request_timeout_secs.max(1));
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| format!("创建上游 HTTP 客户端失败: {}", err))?;

        Ok(Self {
            client,
            agent_message_path: config.agent_message_path,
            api_base_url: config.api_base_url,
        })
    }

    pub async fn forward_agent_message(&self, payload: &Value) -> Result<Value, String> {
        let target_url = format!(
            "{}/{}",
            self.api_base_url.trim_end_matches('/'),
            self.agent_message_path.trim_start_matches('/')
        );

        let response = self
            .client
            .post(&target_url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|err| format!("请求上游 agent 接口失败: {}", err))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|err| format!("读取上游响应失败: {}", err))?;

        if !status.is_success() {
            return Err(format!("上游 agent 接口返回错误状态 {}: {}", status, raw));
        }

        serde_json::from_str::<Value>(&raw)
            .map_err(|err| format!("解析上游响应失败: {}, body={}", err, raw))
    }
}
