#!/bin/sh
set -e

if [ -z "$TEST_DIR" ]; then
    TEST_DIR=/integration_tests
fi

./run.sh
halt
