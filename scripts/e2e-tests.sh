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
pip install -r requirements.txt
info "Successfully installed Python dependencies."

PASSED=false
for i in $(seq 1 $RETRIES); do
	python -m pytest -s $ROOT_DIR/tests
	if [ $? -ne 0 ]; then
		warn "E2E tests failed (try $i/$RETRIES), retrying..."
		continue
	fi
	if [ $? -eq 0 ]; then
		PASSED=true
		break
	fi
done

set -e

popd

if [ "$PASSED" = false ]; then
	error-and-exit "E2E tests failed"
else
	success "E2E tests passed"
fi
