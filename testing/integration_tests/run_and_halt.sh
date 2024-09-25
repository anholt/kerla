#!/bin/sh
set -e

if [ -z "$TESTS_DIR" ]; then
    TESTS_DIR=/integration_tests
fi

$TESTS_DIR/run.sh
halt
