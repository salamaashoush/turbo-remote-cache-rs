docker buildx build --push \
  --platform linux/arm64/v8,linux/amd64 \
  --tag salamaashoush/turbo-remote-cache-rs:latest \
  .
