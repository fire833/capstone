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

# Compile for musl so it works on alpine: https://stackoverflow.com/questions/59766239/how-to-build-a-rust-app-free-of-shared-libraries#59766875
RUN rustup target add x86_64-unknown-linux-musl
RUN RUSTFLAGS='-C link-arg=-s' cargo build --release --target x86_64-unknown-linux-musl

FROM docker.io/alpine:latest

RUN wget -O /usr/local/bin/dumb-init https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64
RUN chmod +x /usr/local/bin/dumb-init

COPY --from=builder /usr/src/hub_router_warp/target/x86_64-unknown-linux-musl/release/hub_router_warp /usr/local/bin/hub_router
USER 1000:1000

LABEL org.opencontainers.image.author="Kendall Tauser, Aaron Pierce"
LABEL org.opencontainers.image.title="Hub Router Warp - Test Distributor of Selenium tests amongst multiple Hubs."
LABEL org.opencontainers.image.source="https://github.com/fire833/capstone"

ENTRYPOINT [ "dumb-init", "hub_router" ]
