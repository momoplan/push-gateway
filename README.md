# push-gateway

`push-gateway` 是独立的内网推送入口，负责两件事：

- `POST /push`：按 Redis 用户路由把下行消息投递到在线 edge 节点

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

默认监听：`127.0.0.1:4013`

## 配置

```toml
[server]
bind_address = "127.0.0.1"
port = 4013

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
export PUSH_GATEWAY_SERVER_PORT=4013
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

## 部署文件

- `deploy/push-gateway.service`: `systemd` 服务定义

## 部署建议

- `push-gateway` 只监听 `127.0.0.1` 或内网地址，不暴露公网
- `push-gateway` 建议监听 `4013`，仅承接 `/push`
- `lowcode-websocket` 监听 `4012`，仅承接 `/agent/message`
