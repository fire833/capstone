
FROM docker.io/golang:alpine AS builder

ADD . /usr/src/exporter
WORKDIR /usr/src/exporter

RUN go build -o grid_exporter -trimpath cmd/grid_exporter/main.go

FROM docker.io/alpine:latest

COPY --from=builder /usr/src/exporter/grid_exporter /usr/local/bin/grid_exporter
USER 1000:1000

LABEL org.opencontainers.image.author="Kendall Tauser"
LABEL org.opencontainers.image.title="Grid Exporter - Prometheus Exporter for Selenium Grid instances"
# LABEL org.opencontainers.image.source="https://github.com/fire833/capstone"

ENTRYPOINT [ "grid_exporter" ]
