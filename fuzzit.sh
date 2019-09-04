#!/bin/bash
set -xe

# Validate arguments
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <fuzz-type>"
    exit 1
fi

# Configure
NAME=wasmer
TYPE=$1

# Setup
if [ ! -f fuzzit ]; then
    wget -q -O fuzzit https://github.com/fuzzitdev/fuzzit/releases/download/v2.4.29/fuzzit_Linux_x86_64
    chmod a+x fuzzit
fi

# Fuzz
TARGET=simple-instantiate
FUZZER=simple_instantiate
cargo fuzz run $FUZZER -- -runs=0
./fuzzit create job --type $TYPE $NAME/$TARGET fuzz/target/x86_64-unknown-linux-gnu/debug/$FUZZER
