# rocket-opentelemetry-demo

[![Rust](https://github.com/SF-Zhou/rocket-opentelemetry-demo/actions/workflows/rust.yml/badge.svg)](https://github.com/SF-Zhou/rocket-opentelemetry-demo/actions/workflows/rust.yml)

### 1. Setup SigNoz

```bash
# 1. download latest code.
git clone https://github.com/SigNoz/signoz.git --depth 1
cd signoz/deploy/docker

# 2. use docker to start a new instance.
docker compose up -d --remove-orphans
```

![SigNoz running inside OrbStack](./images/orbstack.png)

### 2. Launch Rocket

```bash
# 1. start the demo server.
cargo run --release --bin server

# 2. run in another terminal.
cargo run --release --bin client
```

![SigNoz tracing demo](./images/signoz.png)
