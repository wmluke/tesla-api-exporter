export RUST_BACKTRACE := "full"

build:
    cargo build --release

build-static-armv7:
    cross build --target armv7-unknown-linux-musleabihf --release

build-docker-arm:
    docker buildx build --push --platform linux/arm64 -t wmluke/tesla-metrics .
