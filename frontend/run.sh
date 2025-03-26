#!/bin/bash

bun run build

sudo -E HOST="0.0.0.0" PORT=447 /home/aplic/.bun/bin/bun run dist/server/entry.mjs
