#!/usr/bin/env bash

export GEMINI_API_KEY=$(op item get vg55xz2n6pfdkhrljiupfrlpfq --fields "credential" --reveal)

docker run \
    -v $(pwd)/config.yaml:/app/config.yaml \
    -e GEMINI_API_KEY=${GEMINI_API_KEY} \
    -p 4000:4000 \
    ghcr.io/berriai/litellm:main-latest \
    --config /app/config.yaml --detailed_debug

# RUNNING on http://0.0.0.0:4000
