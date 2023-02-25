
FROM docker.io/rust:latest AS builder

ADD src /usr/src/hub_router_warp/src
ADD Cargo.lock /usr/src/hub_router_warp
ADD Cargo.toml /usr/src/hub_router_warp
WORKDIR /usr/src/hub_router_warp

RUN cargo build --release

FROM docker.io/alpine:latest

COPY --from=builder /usr/src/hub_router_warp/target/release/hub_router_warp /usr/local/bin/hub_router
USER 1000:1000

LABEL org.opencontainers.image.author="Kendall Tauser"
LABEL org.opencontainers.image.title="Hub Router Warp - Test Distributor of Selenium tests amongst multiple Hubs."
# LABEL org.opencontainers.image.source="https://github.com/fire833/capstone"

ENTRYPOINT [ "hub_router" ]
