FROM --platform=$TARGETPLATFORM debian:stable-slim
ARG TARGETPLATFORM


COPY target/armv7-unknown-linux-musleabihf/release/tesla-metrics /app/
COPY log4rs.yml /log4rs.yaml

EXPOSE 3000

ENTRYPOINT exec /app/tesla-metrics