#!/bin/bash

set -e

if [ $# -eq 0 ]; then
  echo "Usage: $0 <pod name>|<heap tarball> <image version>"
  exit 1
fi

if [ ! -f "$1" ]; then
  POD=$1
  STS=$(echo "$POD" | cut -d"-" -f1,2)
  NS=pipeline
  NAME="heap-$NS-$POD-$(date +%F-%H-%M-%S)"
  FILE=$NAME.tar.gz
  VERSION=$(kubectl -n pipeline get statefulset "$STS" -o=jsonpath='{$.spec.template.spec.containers[:1].image}' | cut -d':' -f2)

  # shellcheck disable=SC2016
  kubectl exec -n "$NS" "$POD" -- bash -c 'tar -zcf /tmp/heap.tar.gz  $(ls -t *.heap | head -1 | cut -d" " -f2)'
  kubectl cp "$NS/$POD:tmp/heap.tar.gz" "$FILE"
else
  if [ $# -lt 2 ]; then
    echo "Usage: $0 <heap tarball> <image version>"
    exit 1
  fi
  FILE=$1
  VERSION=$2

  shift # Remove image version
fi

TEMP=$(mktemp -d)
tar -zxf "$FILE" -C "$TEMP"

shift # Remove pod name or heap tarball

docker build build -t "jeprof-$VERSION" --build-arg VERSION="$VERSION"
docker run -v "$TEMP:/heap" "jeprof-$VERSION" "$@"
