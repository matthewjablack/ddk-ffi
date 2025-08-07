#!/bin/bash

# Setup script for generating Uniffi bindings

echo "Building Rust library..."
cargo build --release

echo "Installing npm dependencies..."
npm install

echo "Compiling TypeScript example..."
npm run build

echo "Setup complete! You can now run: node dist/example.js"