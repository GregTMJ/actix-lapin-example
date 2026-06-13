# CLAUDE.md

## Project purpose

HTTP-to-RabbitMQ adapter. Clients POST to `/api/v1/request-servicehub`, the service publishes to RabbitMQ with a UUID `correlation_id`, and a background consumer routes the reply back to the waiting HTTP handler via a `tokio::sync::mpsc` channel keyed on `correlation_id`. A timeout task fires after `SERVICEHUB_TIMEOUT` seconds if no reply arrives.

## Commands

```bash
cargo build --release          # build
cargo run                      # run locally (requires .env + RabbitMQ)
cargo test                     # run tests
docker compose up --build      # run via Docker (see network note below)
```

Docker requires two external networks created once:

```bash
docker network create no-internet
docker network create has-internet
```

## Module map

| Path                  | Responsibility                                                                       |
| --------------------- | ------------------------------------------------------------------------------------ |
| `src/main.rs`         | Wires up Actix, spawns RMQ consumer task, shares channel via `Data<Channel>`         |
| `src/configs.rs`      | `PROJECT_CONFIG` (LazyLock env config), `RESPONSE_CHANNELS` (global mpsc sender map) |
| `src/api/base.rs`     | `POST /request-servicehub` — creates mpsc pair, stores sender, publishes, waits      |
| `src/api/ping.rs`     | `GET /ping` — returns 503 if Lapin channel disconnected                              |
| `src/api/schemas.rs`  | `IncomingRequest`, `Request`, `ServiceResponse` — request/response shapes            |
| `src/rmq/builder.rs`  | `ConnectionBuilder` — constructs the Lapin `AMQPConnection`                          |
| `src/rmq/handlers.rs` | `RmqHandler::consume_main` (consumer loop), `send_message` (publish)                 |
| `src/rmq/schemas.rs`  | `Exchange`, `Queue` — thin wrappers with type conversion                             |
| `src/errors/`         | `ProjectError` wrapping `ApiErrors` / `RmqErrors` via `thiserror`                    |

## Architecture notes

- `RESPONSE_CHANNELS` is a `Mutex<HashMap<ShortString, mpsc::Sender<ServiceResponse>>>`. Entries are inserted before publish and removed by whichever wins the race: the consumer or the timeout task.
- The mpsc channel capacity is 2 — one slot for the consumer reply, one for the timeout — so neither blocks.
- The Lapin `Channel` is shared across all HTTP workers via `Data<Arc<Channel>>`. The consumer runs on its own dedicated channel.
- Auto-recovery is enabled on the connection (`ConnectionBuilder`). The consumer loop will resume after transient RabbitMQ restarts.

## Testing approach

- **Schema tests** — unit test `ServiceResponse::try_from(Vec<u8>)` and `Request::from(IncomingRequest)` in-module with `#[cfg(test)]`.
- **HTTP handler tests** — use `actix_web::test::init_service` + `TestRequest` to test the ping endpoint and error paths without a real RabbitMQ.
- **Integration tests** — in `tests/integration.rs`, gated with `#[ignore = "requires running RabbitMQ"]`, covering the full send-and-receive flow.
