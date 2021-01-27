FROM --platform=$BUILDPLATFORM alpine:latest
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN apk --no-cache add ca-certificates

COPY target/armv7-unknown-linux-musleabihf/release/tesla-metrics /app/

CMD ["/app/tesla-metrics"]