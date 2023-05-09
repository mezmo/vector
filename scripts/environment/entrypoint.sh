#! /usr/bin/env bash

# set HOSTNAME to container id for `cross`
HOSTNAME="$(head -1 /proc/self/cgroup|cut -d/ -f3)"
export HOSTNAME

git config --global url."https://${GITHUB_TOKEN}@github.com".insteadOf ssh://git@github.com

exec "$@"
