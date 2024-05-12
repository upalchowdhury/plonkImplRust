aws s3api list-objects-v2 \
    --bucket your-bucket-name \
    --query "Contents | sort_by(@, &LastModified)[::-1][:100].{Key: Key}" \
    --output text > files_to_download.txt
while IFS= read -r file_key; do
    aws s3 cp "s3://your-bucket-name/$file_key" "/local/path/to/download/$file_key"
done < files_to_download.txt
