#!/bin/bash

RUSTFLAGS="-Z threads=4 -C link-arg=-fuse-ld=/usr/bin/mold" CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift cargo build -Zcodegen-backend