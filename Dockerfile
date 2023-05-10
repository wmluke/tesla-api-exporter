FROM --platform=$TARGETPLATFORM debian:stable-slim
ARG TARGETPLATFORM


COPY target/armv7-unknown-linux-musleabihf/release/tesla-api-exporter /app/
COPY log4rs.yml /log4rs.yaml

EXPOSE 3001

ENTRYPOINT exec /app/tesla-api-exporter
