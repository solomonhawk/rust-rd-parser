#!/usr/bin/env bash
set -e

cd $(dirname "$0")

echo "Building WASM package..."

echo which wasm-pack

echo "$PATH"
# # Build for web (ES modules)
# wasm-pack build --target web --features wasm --out-dir pkg-web

# # Build for Node.js
# wasm-pack build --target nodejs --features wasm --out-dir pkg-node

# # Build for bundlers (webpack, etc.)
# wasm-pack build --target bundler --features wasm --out-dir pkg

# echo "WASM build complete!"
# echo "Web package: pkg-web/"
# echo "Node.js package: pkg-node/"
# echo "Bundler package: pkg/"
