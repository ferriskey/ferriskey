#!/bin/bash

set -e

bash -c "docker run --rm -v $(pwd):/helm-docs -u $(id -u) jnorwood/helm-docs:latest"
git diff --exit-code --quiet -- charts/ferriskey/README.md
