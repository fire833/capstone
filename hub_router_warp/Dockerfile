FROM docker.io/node:latest AS frontend_builder

ADD hub_router_ui /usr/src/hub_router_warp/hub_router_ui
WORKDIR /usr/src/hub_router_warp/hub_router_ui

RUN ls -la
RUN npm i
RUN npm run build

FROM docker.io/rust:latest AS builder

ADD src /usr/src/hub_router_warp/src
ADD Cargo.lock /usr/src/hub_router_warp
ADD Cargo.toml /usr/src/hub_router_warp
WORKDIR /usr/src/hub_router_warp

COPY --from=frontend_builder /usr/src/hub_router_warp/hub_router_ui/dist /usr/src/hub_router_warp/hub_router_ui/dist

RUN cargo build --release

FROM docker.io/alpine:latest

COPY --from=builder /usr/src/hub_router_warp/target/release/hub_router_warp /usr/local/bin/hub_router
USER 1000:1000

LABEL org.opencontainers.image.author="Kendall Tauser"
LABEL org.opencontainers.image.title="Hub Router Warp - Test Distributor of Selenium tests amongst multiple Hubs."
# LABEL org.opencontainers.image.source="https://github.com/fire833/capstone"

ENTRYPOINT [ "hub_router" ]
