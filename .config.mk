APP_NAME=vector
APP_VERSION:=$(shell jq -r .version package.json | tr -d v)
ENABLE_COMMITLINT = true
DEFAULT_BRANCH = master
