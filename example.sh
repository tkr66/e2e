#!/bin/sh

cargo build --release

docker run -d -p 4444:4444 --rm --shm-size="2g" selenium/standalone-edge:4.32.0-20250505

./target/release/e2e -f example-e2e.yaml
