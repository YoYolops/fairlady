#!/bin/sh
# ./kubo_node/init-config.sh

echo "Applying custom Kubo configurations..."

# Permissive CORS config for api
ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT", "POST", "GET"]'

# Permissive CORS config for gateway
ipfs config --json Gateway.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
ipfs config --json Gateway.HTTPHeaders.Access-Control-Allow-Methods '["GET"]'

echo "Custom configurations applied successfully."