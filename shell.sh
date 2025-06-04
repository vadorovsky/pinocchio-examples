#!/bin/sh

if command -v podman >/dev/null 2>&1; then
  CONTAINER_ENGINE=podman
elif command -v docker >/dev/null 2>&1; then
  CONTAINER_ENGINE=docker
else
  echo "Could not find a supported container engine (docker, podman)"
  exit 1
fi

IMAGE_URI=${IMAGE_URI:-docker.io/vadorovsky/pinocchio}

"${CONTAINER_ENGINE}" build -t "${IMAGE_URI}" .
"${CONTAINER_ENGINE}" run -it --rm -v .:/src "${IMAGE_URI}"
