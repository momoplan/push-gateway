# push-gateway

`push-gateway` 是独立的内网推送入口，负责两件事：

- `POST /push`：按 Redis 用户路由把下行消息投递到在线 edge 节点
- `POST /agent/message`：透明代理到 `agent-service`，给现有 lowcode / edge 调用方保留统一入口

它本身不维护任何前端连接，也不负责 Agent 会话池。

## 运行

1. 复制配置：

```bash
cp config.example.toml config.toml
```

2. 启动：

```bash
cargo run
```

默认监听：`127.0.0.1:4012`

默认上游 agent 入口：`http://127.0.0.1:4013/agent/message`

## 配置

```toml
[server]
bind_address = "127.0.0.1"
port = 4012

[agent_upstream]
api_base_url = "http://127.0.0.1:4013"
agent_message_path = "/agent/message"
request_timeout_secs = 30

[logging]
level = "info"

[redis]
url = "redis://127.0.0.1:6379/"
route_key_prefix = "ws:route:user"
queue_key_prefix = "ws:queue:node"
node_alive_key_prefix = "ws:node:alive"
```

也支持环境变量覆盖，例如：

```bash
export PUSH_GATEWAY_SERVER_PORT=4012
export PUSH_GATEWAY_AGENT_UPSTREAM_API_BASE_URL=http://127.0.0.1:4013
export PUSH_GATEWAY_REDIS_URL=redis://127.0.0.1:6379/
```

## 接口

### `GET /healthz`

返回：

```json
{"status":"ok"}
```

### `POST /push`

请求：

```json
{
  "userId": "123",
  "projectId": "project_123",
  "requestId": "req-1",
  "sessionId": "sess-1",
  "data": {
    "type": "notification",
    "message": "hello"
  }
}
```

响应：

```json
{
  "status": "ok",
  "delivered": true,
  "deliveredNodes": 1
}
```

说明：

- `delivered=false` 表示当前没有活跃 edge 节点持有该用户连接
- 失活节点会在路由分发时自动从用户路由中清理

### `POST /agent/message`

该接口不改写请求体，直接代理到上游 `agent-service` 的 `/agent/message`。

这样 lowcode / edge 侧仍可只配置一个统一内网地址。

## 部署文件

- `deploy/push-gateway.service`: `systemd` 服务定义

## 部署建议

- `push-gateway` 只监听 `127.0.0.1` 或内网地址，不暴露公网
- 让 `push-gateway` 占用原先统一入口端口，例如 `4012`
- 将 `agent-service` 内部 `push_server` 端口后移到 `4013`
- lowcode 与 edge 继续指向 `push-gateway` 的统一入口地址
