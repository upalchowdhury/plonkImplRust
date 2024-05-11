#!/bin/bash

# Variables
BUCKET_NAME="your-bucket-name"
PREFIX="your-folder-prefix/"  # Include the trailing slash if looking within a specific folder
DOWNLOAD_DIR="your-local-download-directory"
mkdir -p "$DOWNLOAD_DIR"  # Ensure download directory exists

# Fetch the most recent file
echo "Fetching the most recent file from the bucket..."
LATEST_FILE=$(aws s3api list-objects-v2 --bucket "$BUCKET_NAME" --prefix "$PREFIX" \
    --query 'Contents | sort_by(@, &LastModified) | last(@).Key' \
    --output text)

# Check if a file was found
if [[ -z "$LATEST_FILE" || "$LATEST_FILE" == "None" ]]; then
    echo "No files found or returned empty filename. Exiting..."
    exit 1
fi

echo "Processing file: $LATEST_FILE"
DOWNLOAD_PATH="$DOWNLOAD_DIR/$LATEST_FILE"

# Create directory for the file if it doesn't exist
mkdir -p "$(dirname "$DOWNLOAD_PATH")"

echo "Attempting to download $LATEST_FILE to $DOWNLOAD_PATH..."
if ! aws s3 cp "s3://$BUCKET_NAME/$LATEST_FILE" "$DOWNLOAD_PATH"; then
    echo "ERROR: Failed to download $LATEST_FILE - Check permissions and file path"
    exit 1
else
    echo "Download successful: $LATEST_FILE"
fi

# Place your file processing logic here
# Example: If it's an image file and you need to resize or modify it:
# convert "$DOWNLOAD_PATH" -resize 800x800 "$DOWNLOAD_PATH.modified"

# Optionally, upload the processed file back to S3, for example:
# UPLOAD_DESTINATION="s3://$BUCKET_NAME/processed/$LATEST_FILE"
# aws s3 cp "$DOWNLOAD_PATH.modified" "$UPLOAD_DESTINATION"

# Cleanup: remove the downloaded file
rm "$DOWNLOAD_PATH"
echo "Cleanup complete for $LATEST_FILE"
