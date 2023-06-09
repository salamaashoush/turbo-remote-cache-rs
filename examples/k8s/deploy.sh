#!/bin/sh

COMMAND=$1

# check if the command is apply
if [ "$COMMAND" = "apply" ]; then
  # 1. apply the storage
  kubectl apply -f storage.yaml

  # 2. apply the deployment
  kubectl apply -f deployment.yaml

  # 3. apply the service
  kubectl apply -f service.yaml

  # 4. apply the ingress
  kubectl apply -f ingress.yaml
fi

# check if the command is delete

if [ "$COMMAND" = "delete" ]; then
  # 1. delete the ingress
  kubectl delete -f ingress.yaml

  # 2. delete the service
  kubectl delete -f service.yaml

  # 3. delete the deployment
  kubectl delete -f deployment.yaml

  # 4. delete the storage
  kubectl delete -f storage.yaml
fi
