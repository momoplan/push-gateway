use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub agent_upstream: AgentUpstreamConfig,
    pub logging: LoggingConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentUpstreamConfig {
    pub api_base_url: String,
    #[serde(default = "default_agent_message_path")]
    pub agent_message_path: String,
    #[serde(default = "default_agent_request_timeout_secs")]
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_route_key_prefix")]
    pub route_key_prefix: String,
    #[serde(default = "default_queue_key_prefix")]
    pub queue_key_prefix: String,
    #[serde(default = "default_node_alive_key_prefix")]
    pub node_alive_key_prefix: String,
}

fn default_agent_message_path() -> String {
    "/agent/message".to_string()
}

fn default_agent_request_timeout_secs() -> u64 {
    30
}

fn default_route_key_prefix() -> String {
    "ws:route:user".to_string()
}

fn default_queue_key_prefix() -> String {
    "ws:queue:node".to_string()
}

fn default_node_alive_key_prefix() -> String {
    "ws:node:alive".to_string()
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(
                config::Environment::with_prefix("PUSH_GATEWAY")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 4012,
            },
            agent_upstream: AgentUpstreamConfig {
                api_base_url: "http://127.0.0.1:4013".to_string(),
                agent_message_path: default_agent_message_path(),
                request_timeout_secs: default_agent_request_timeout_secs(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            redis: RedisConfig {
                url: "redis://127.0.0.1:6379/".to_string(),
                route_key_prefix: default_route_key_prefix(),
                queue_key_prefix: default_queue_key_prefix(),
                node_alive_key_prefix: default_node_alive_key_prefix(),
            },
        }
    }
}
