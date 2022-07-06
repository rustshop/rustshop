#!/usr/bin/env bash

binary_tag_name=aws-bootstrap-build-binary
docker build -t="$binary_tag_name" .
container_id=$(docker create $(docker images -q "$binary_tag_name" | head -n 1))
docker cp "$container_id:/usr/local/bin/aws-bootstrap" "aws-bootstrap"
docker rm $container_id

echo "The ./aws-bootstrap binary is ready."
