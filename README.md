# actix-lapin-example

A Rust service that bridges RabbitMQ with an HTTP layer using [Actix Web](https://github.com/actix/actix-web) and [Lapin](https://github.com/amqp-rs/lapin). It consumes messages from RabbitMQ and exposes HTTP endpoints to send/receive messages through the message broker.

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

## Project Structure

```
src/
├── api/          # HTTP handlers and request/response schemas
├── configs.rs    # Environment-based configuration
├── errors/       # Custom error types (API + RMQ)
├── lib.rs        # Library root
├── main.rs       # Application entry point
├── prelude.rs    # Re-exports
└── rmq/          # RabbitMQ connection builder and message handler
```

## Configuration

Copy the template environment file and adjust values as needed:

```bash
cp .env.tmp .env
```

| Variable                       | Description                  | Default                   |
| ------------------------------ | ---------------------------- | ------------------------- |
| `RMQ_USER`                     | RabbitMQ username            | `user`                    |
| `RMQ_PASSWORD`                 | RabbitMQ password            | `bitnami`                 |
| `RMQ_HOST`                     | RabbitMQ host                | `rabbitmq`                |
| `RMQ_PORT`                     | RabbitMQ port                | `5672`                    |
| `RMQ_VHOST`                    | RabbitMQ virtual host        | `%2F`                     |
| `RMQ_EXCHANGE`                 | Main exchange name           | `servicehub`              |
| `RMQ_RESPONSE_QUEUE`           | Queue for incoming responses | `json_adapter.q.response` |
| `RMQ_SERVICEHUB_EXCHANGE`      | ServiceHub exchange name     | `servicehub`              |
| `RMQ_SERVICEHUB_EXCHANGE_TYPE` | ServiceHub exchange type     | `direct`                  |
| `RMQ_SERVICEHUB_REQUEST_QUEUE` | ServiceHub request queue     | `servicehub.q.request`    |
| `SERVICEHUB_TIMEOUT`           | Response timeout in seconds  | `30`                      |

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

## Build

```bash
cargo build --release
```

The release binary is named `actix-lapin`.
