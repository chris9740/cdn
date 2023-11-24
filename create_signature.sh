#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path_to_image>"
    exit 1
fi

IMAGE_FILE="$1"
PRIVATE_KEY="./certs/staging.pem"

if [ ! -f "$IMAGE_FILE" ]; then
    echo >&2 "Error: Image file not found"
    exit 1
fi

if [ ! -f "$PRIVATE_KEY" ]; then
    echo >&2 "Error: Private key file not found"
    exit 1
fi

TEMP_FILE=$(mktemp)

openssl dgst -md5 -sign "$PRIVATE_KEY" -out $TEMP_FILE "$IMAGE_FILE"

if [ $? -eq 0 ]; then
    SIGNATURE=$(base64 $TEMP_FILE -w0)

    echo $SIGNATURE
    rm $TEMP_FILE
else
    echo >&2 "Signing failed."
    exit 1
fi
