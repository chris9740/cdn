#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path_to_image>"
    exit 1
fi

IMAGE_PATH="$1"
PRIVATE_KEY_PATH="./certs/staging.pem"

if [ ! -f "$IMAGE_PATH" ]; then
    echo >&2 "Error: Image file not found"
    exit 1
fi

if [ ! -f "$PRIVATE_KEY_PATH" ]; then
    echo >&2 "Error: Private key file not found"
    exit 1
fi

SIGNATURE=$(openssl dgst -md5 -sign "$PRIVATE_KEY_PATH" "$IMAGE_PATH" | base64 -w0)

if [ ! -z "$SIGNATURE" ]; then
    echo $SIGNATURE
fi
