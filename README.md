# DLC-FFI: React Native Uniffi Example

A simple Rust library with React Native bindings using Mozilla's Uniffi.

## Structure

- `src/lib.rs` - Rust implementation with hello_world function
- `src/dlc_ffi.udl` - Uniffi interface definition
- `build.rs` - Build script for generating Rust scaffolding
- `example.ts` - TypeScript example demonstrating usage
- `setup.sh` - Setup script to generate bindings

## Quick Start

1. Run the setup script:
```bash
./setup.sh
```

2. Run the TypeScript example:
```bash
node dist/example.js
```

## Manual Setup

1. Install uniffi-bindgen:
```bash
cargo install uniffi-bindgen
```

2. Build the Rust library:
```bash
cargo build --release
```

3. Generate TypeScript bindings:
```bash
uniffi-bindgen generate src/dlc_ffi.udl --language typescript --out-dir bindings
```

4. Install npm dependencies and compile TypeScript:
```bash
npm install
npm run build
```

## React Native Integration

To use in React Native:

1. Copy the generated bindings to your React Native project
2. Add the compiled Rust library (`.so`/`.dylib`/`.dll`) to your app bundle
3. Import and use the generated TypeScript bindings

The `hello_world()` function returns "Hello, World from Rust!" when called.