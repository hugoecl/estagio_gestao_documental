#!/bin/bash

USE_HTTPS=false
SHOW_HELP=false
ADDITIONAL_ARGS=()

show_help() {
    echo "Backend Development Script"
    echo "Usage: $0 [OPTIONS] [-- PROGRAM_ARGS]"
    echo
    echo "Options:"
    echo "  -h, --help        Show this help message and exit"
    echo "  --https           Run with HTTPS support (disables cranelift)"
    echo
    echo "Examples:"
    echo "  $0                         # Run with default settings"
    echo "  $0 --https                 # Run with HTTPS support"
    echo "  $0 --https -- --port=8443  # Run with HTTPS and pass --port=8443"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        --https)
            USE_HTTPS=true
            shift
            ;;
        --)
            shift
            ADDITIONAL_ARGS=("$@")
            break
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help to see available options"
            exit 1
            ;;
    esac
done

if [ "$SHOW_HELP" = true ]; then
    show_help
fi

if [ "$USE_HTTPS" = true ]; then
    echo "Running in HTTPS mode (cranelift disabled)"
    RUSTFLAGS="-Z threads=4" cargo run --features https -- "${ADDITIONAL_ARGS[@]}"
else
    echo "Running in standard mode with cranelift"
    RUSTFLAGS="-Z threads=4 -C link-arg=-fuse-ld=/usr/bin/mold" CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift cargo +nightly run -Zcodegen-backend -- "${ADDITIONAL_ARGS[@]}"
fi
