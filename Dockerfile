FROM --platform=$TARGETPLATFORM debian:stable-slim
ARG TARGETPLATFORM

WORKDIR /app

COPY target/armv7-unknown-linux-musleabihf/release/tesla-metrics .
COPY log4rs.yml .

CMD ["/app/tesla-metrics"]