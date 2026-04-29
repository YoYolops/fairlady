#!/bin/sh
# ./kubo_node/init-config.sh

echo "Applying custom Kubo configurations..."

# Permissive CORS config for api
ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT", "POST", "GET"]'

# Permissive CORS config for gateway
ipfs config --json Gateway.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
ipfs config --json Gateway.HTTPHeaders.Access-Control-Allow-Methods '["GET"]'


# Garbage Collection Thresholds
ipfs config Datastore.StorageMax "10GB"

# If disk usage hits 90% of StorageMax, GC is triggered.
ipfs config Datastore.StorageGCWatermark 90

# How frequently the background daemon checks the storage size
ipfs config Datastore.GCPeriod "1h"

echo "Custom configurations applied successfully."