export RUST_BACKTRACE := "full"

build:
    cargo build --release

build-static-armv7:
    cross build --target armv7-unknown-linux-musleabihf --release
