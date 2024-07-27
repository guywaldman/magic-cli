#!/usr/bin/env bash

set -e

RETRIES=3

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(realpath $SCRIPT_DIR/..)

source $ROOT_DIR/scripts/utils.sh

set +e

pushd $ROOT_DIR

info "Activating Python virtual environment..."
python -m venv venv
source venv/bin/activate
info "Successfully activated Python virtual environment."

info "Installing Python dependencies..."
pip install -r requirements.txt 1> /dev/null
info "Successfully installed Python dependencies."

python -m pytest $ROOT_DIR/tests --retries 3
if [ $? -ne 0 ]; then
	error-and-exit "E2E tests failed"
else
	success "E2E tests passed"
fi
