#!/bin/bash

# Variables
BUCKET_NAME="your-bucket-name"
PREFIX="your-folder-prefix/"
DOWNLOAD_DIR="your-local-download-directory"
mkdir -p "$DOWNLOAD_DIR"

# Process files
aws s3api list-objects-v2 --bucket "$BUCKET_NAME" --prefix "$PREFIX" \
    --query 'Contents | sort_by(@, &LastModified)[-10:].Key' \
    --output text | while read FILE; do
    echo "Processing file: $FILE"
    DOWNLOAD_PATH="$DOWNLOAD_DIR/$FILE"
    mkdir -p "$(dirname "$DOWNLOAD_PATH")"
    
    echo "Downloading $FILE to $DOWNLOAD_PATH..."
    if ! aws s3 cp "s3://$BUCKET_NAME/$FILE" "$DOWNLOAD_PATH"; then
        echo "ERROR: Failed to download $FILE - Check permissions and file path"
        continue
    else
        echo "Download successful: $FILE"
    fi

    # Processing logic goes here
    
    # Cleanup
    rm "$DOWNLOAD_PATH"
    echo "Cleanup complete for $FILE"
done
