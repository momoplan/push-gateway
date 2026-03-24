use crate::config::RedisConfig;
use crate::models::AgentBroadcastEnvelope;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RedisDelivery {
    connection: ConnectionManager,
    route_key_prefix: String,
    queue_key_prefix: String,
    node_alive_key_prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueuedDeliveryMessage {
    user_id: String,
    envelope: AgentBroadcastEnvelope,
}

impl RedisDelivery {
    pub async fn new(config: RedisConfig) -> Result<Self, String> {
        let client = redis::Client::open(config.url.as_str())
            .map_err(|err| format!("初始化 Redis 失败: {}", err))?;
        let connection = client
            .get_connection_manager()
            .await
            .map_err(|err| format!("创建 Redis 连接失败: {}", err))?;

        Ok(Self {
            connection,
            route_key_prefix: config.route_key_prefix,
            queue_key_prefix: config.queue_key_prefix,
            node_alive_key_prefix: config.node_alive_key_prefix,
        })
    }

    pub async fn dispatch_to_user(
        &self,
        user_id: &str,
        envelope: &AgentBroadcastEnvelope,
    ) -> Result<usize, String> {
        let route_key = self.route_key(user_id);
        let mut connection = self.connection.clone();

        let node_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&route_key)
            .query_async(&mut connection)
            .await
            .map_err(|err| format!("查询用户路由失败: {}", err))?;

        if node_ids.is_empty() {
            return Ok(0);
        }

        let payload = serde_json::to_string(&QueuedDeliveryMessage {
            user_id: user_id.to_string(),
            envelope: envelope.clone(),
        })
        .map_err(|err| format!("序列化下行消息失败: {}", err))?;

        let mut delivered_nodes = 0usize;
        for node_id in node_ids {
            if !self.is_node_alive(&node_id).await? {
                let _: i64 = redis::cmd("SREM")
                    .arg(&route_key)
                    .arg(&node_id)
                    .query_async(&mut connection)
                    .await
                    .unwrap_or(0);
                continue;
            }

            let queue_key = self.queue_key_for_node(&node_id);
            let _: i64 = redis::cmd("RPUSH")
                .arg(queue_key)
                .arg(&payload)
                .query_async(&mut connection)
                .await
                .map_err(|err| format!("写入下行队列失败: {}", err))?;
            delivered_nodes += 1;
        }

        Ok(delivered_nodes)
    }

    async fn is_node_alive(&self, node_id: &str) -> Result<bool, String> {
        let mut connection = self.connection.clone();
        let exists: i64 = redis::cmd("EXISTS")
            .arg(self.node_alive_key(node_id))
            .query_async(&mut connection)
            .await
            .map_err(|err| format!("检查节点心跳失败: {}", err))?;
        Ok(exists > 0)
    }

    fn route_key(&self, user_id: &str) -> String {
        format!("{}:{}", self.route_key_prefix, user_id)
    }

    fn queue_key_for_node(&self, node_id: &str) -> String {
        format!("{}:{}", self.queue_key_prefix, node_id)
    }

    fn node_alive_key(&self, node_id: &str) -> String {
        format!("{}:{}", self.node_alive_key_prefix, node_id)
    }
}
