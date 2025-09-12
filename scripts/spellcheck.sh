#!/bin/bash

# Check if typos is installed
if ! command -v typos &> /dev/null; then
    echo "Error: typos is not installed."
    echo "Please install it using one of the following methods:"
    echo ""
    echo "  Using Cargo:"
    echo "    cargo install typos-cli"
    echo ""
    echo ""
    echo "For more installation options, see: https://github.com/crate-ci/typos"
    exit 1
fi

# Run typos with the provided arguments, defaulting to current directory
if [ $# -eq 0 ]; then
    typos --config ./typos.toml
else
    typos --config ./typos.toml "$@"
fi
