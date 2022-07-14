#!/usr/bin/env bash
# TODO: rewrite in Rust
# export RELEASE_VERSION=`curl -sL https://api.github.com/repos/aws-controllers-k8s/$SERVICE-controller/releases/latest | grep '"tag_name":' | cut -d'"' -f4`
# export RELEASE_VERSION="v0.1.3" # s3
# export SERVICE=s3
# export RELEASE_VERSION="v0.1.3" # s3
export SERVICE=rds
export RELEASE_VERSION="v0.0.27" # rds
export ACK_SYSTEM_NAMESPACE=ack-system
export AWS_REGION=us-east-1

aws ecr-public get-login-password --region us-east-1 | helm registry login --username AWS --password-stdin public.ecr.aws

helm install --create-namespace -n $ACK_SYSTEM_NAMESPACE ack-$SERVICE-controller \
  oci://public.ecr.aws/aws-controllers-k8s/$SERVICE-chart --version=$RELEASE_VERSION --set=aws.region=$AWS_REGION

# helm uninstall -n $ACK_SYSTEM_NAMESPACE ack-$SERVICE-controller \
#   oci://public.ecr.aws/aws-controllers-k8s/$SERVICE-chart
