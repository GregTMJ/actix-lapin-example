# actix-lapin-example

A Rust service that bridges RabbitMQ with an HTTP layer using [Actix Web](https://github.com/actix/actix-web) and [Lapin](https://github.com/amqp-rs/lapin). It consumes messages from RabbitMQ and exposes HTTP endpoints to send/receive messages through the message broker.

## How it works

`POST /api/v1/request-servicehub` publishes a JSON request to the configured ServiceHub exchange and queue. The request includes a UUID correlation ID and the address of this instance's reply queue. A background RabbitMQ consumer matches each reply to the waiting HTTP request by correlation ID. The handler returns the reply, or a `504 Gateway Timeout` after `SERVICEHUB_TIMEOUT` seconds.

Each instance declares its own exclusive, auto-deleting response queue. The reply target is built once from the configuration at startup and shared by the HTTP handlers. Pending requests are tracked in a shared map of one-shot channels. The RabbitMQ connection uses Lapin's automatic recovery.

## Prerequisites

- **RabbitMQ server** – A running RabbitMQ instance is required. Configure the connection details in `.env` (see [Configuration](#configuration)). You can bring one up with Docker:

  ```bash
  docker run -d --name rabbitmq -p 5672:5672 -p 15672:15672 \
    -e RABBITMQ_DEFAULT_USER=user \
    -e RABBITMQ_DEFAULT_PASS=bitnami \
    rabbitmq:3-management
  ```

## Stack

- **Rust 2024** edition
- **Actix Web 4** – HTTP server
- **Lapin 4** – AMQP 0-9-1 client for RabbitMQ
- **Tokio** – Async runtime

## Project structure

```
src/
├── api/          # HTTP handlers and request/response schemas
├── configs.rs    # Environment-based configuration
├── errors/       # Custom error types (API + RMQ)
├── lib.rs        # Library root
├── main.rs       # Application entry point
├── prelude.rs    # Re-exports
├── state.rs      # Shared config, channel, reply target, and pending requests
└── rmq/          # RabbitMQ connection builder and message handler
```

## Configuration

Copy the template environment file and adjust values as needed:

```bash
cp .env.tmp .env
```

| Variable                       | Description                             | Default                   |
| ------------------------------ | --------------------------------------- | ------------------------- |
| `RMQ_USER`                     | RabbitMQ username                       | `user`                    |
| `RMQ_PASSWORD`                 | RabbitMQ password                       | `bitnami`                 |
| `RMQ_HOST`                     | RabbitMQ host                           | `rabbitmq`                |
| `RMQ_PORT`                     | RabbitMQ port                           | `5672`                    |
| `RMQ_VHOST`                    | RabbitMQ virtual host                   | `%2F`                     |
| `RMQ_EXCHANGE`                 | Main exchange name                      | `servicehub`              |
| `RMQ_RESPONSE_QUEUE`           | Prefix for per-instance response queues | `json_adapter.q.response` |
| `RMQ_SERVICEHUB_EXCHANGE`      | ServiceHub exchange name                | `servicehub`              |
| `RMQ_SERVICEHUB_REQUEST_QUEUE` | ServiceHub request queue                | `servicehub.q.request`    |
| `SERVICEHUB_TIMEOUT`           | Response timeout in seconds             | `30`                      |

## Running

### Local

```bash
cargo run
```

The server starts on `0.0.0.0:8000`.

### Docker Compose

The compose file references two external Docker networks. Create them once if they don't exist:

```bash
docker network create no-internet
docker network create has-internet
```

Then:

```bash
docker compose up --build
```

The service is exposed on port `7085`.

## API

| Method | Path                         | Description                                         |
| ------ | ---------------------------- | --------------------------------------------------- |
| `GET`  | `/api/v1/ping`               | Health check — returns 503 if RabbitMQ disconnected |
| `POST` | `/api/v1/request-servicehub` | Send a message to RabbitMQ and wait for a response  |

## Tests

```bash
cargo test
```

The existing unit tests cover request and response schemas and RabbitMQ exchange and queue types. A running RabbitMQ server is needed to test the full HTTP-to-RabbitMQ flow.

## Build

```bash
cargo build --release
```

The release binary is named `actix-lapin`.
