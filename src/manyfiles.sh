#!/bin/bash

# Define your bucket name
BUCKET_NAME="your-bucket-name"

# Get the list of files sorted by modification date
aws s3api list-objects-v2 --bucket $BUCKET_NAME --query 'Contents | sort_by(@, &LastModified)[-100:].{Key: Key}' --output text > filelist.txt

# Download the last 100 modified files
tail -n 100 filelist.txt | while read -r line; do
  aws s3 cp s3://$BUCKET_NAME/"$line" ./downloaded/
done

# Cleanup
rm filelist.txt
