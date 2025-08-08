# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a React Native library that provides Rust-powered DLC (Discreet Log Contract) functionality through UniFFI bindings. The project bridges Rust DLC implementation to React Native using `uniffi-bindgen-react-native`.

## Architecture

The project consists of two main components:

- **ddk-ffi/**: Rust crate containing DLC business logic with UniFFI interface definitions
- **ddk-rn/**: React Native library with generated TypeScript, C++, iOS, and Android bindings

The architecture follows this flow:
1. Rust code defines structs, functions, and errors in `ddk-ffi/src/lib.rs`
2. UniFFI interface is declared in `ddk-ffi/src/ddk_ffi.udl`
3. Bindings are generated for TypeScript (JSI), C++, iOS (Swift/Objective-C), and Android (Kotlin/JNI)
4. React Native consumes the generated TypeScript API

## Build System & Commands

This project uses `just` as the primary build orchestrator. All build commands should be run from the project root.

### Core Build Commands
- `just uniffi`: Complete build pipeline (generates all bindings + builds iOS/Android)
- `just uniffi-jsi`: Generate TypeScript and C++ JSI bindings from UDL
- `just uniffi-turbo`: Generate React Native TurboModule specifications
- `just build-ios`: Build iOS static libraries and create XCFramework
- `just build-android`: Build Android native libraries (JNI)

### Example App Commands
- `just example-ios`: Install iOS dependencies with new architecture enabled
- `just example-android`: Build Android example app
- `cd ddk-rn/example && npx react-native run-ios`: Run iOS example
- `cd ddk-rn/example && npx react-native run-android`: Run Android example

### React Native Library Commands (run from ddk-rn/)
- `pnpm test`: Run Jest tests
- `pnpm typecheck`: Run TypeScript type checking
- `pnpm lint`: Run ESLint
- `pnpm prepare`: Build library with react-native-builder-bob

### Rust Commands (run from ddk-ffi/)
- `cargo build`: Build Rust crate
- `cargo test`: Run Rust tests

## Development Workflow

1. **Modify Rust Code**: Edit `ddk-ffi/src/lib.rs` with new functions/structs
2. **Update Interface**: Add corresponding definitions to `ddk-ffi/src/ddk_ffi.udl`
3. **Generate Bindings**: Run `just uniffi` to regenerate all language bindings
4. **Manual Fix**: Fix the include path in `ddk-rn/cpp/bennyhodl-ddk-rn.cpp` from `#include "/ddk_ffi.hpp"` to `#include "ddk_ffi.hpp"`
5. **Test Changes**: Use example app or run tests

## Key Files & Locations

### Rust FFI Layer
- `ddk-ffi/src/lib.rs`: Core Rust implementation
- `ddk-ffi/src/ddk_ffi.udl`: UniFFI interface definitions
- `ddk-ffi/Cargo.toml`: Rust project configuration

### React Native Layer
- `ddk-rn/src/`: Generated TypeScript bindings
- `ddk-rn/cpp/`: Generated C++ bindings for JSI
- `ddk-rn/ios/`: iOS native module and XCFramework
- `ddk-rn/android/`: Android native module and JNI libraries
- `ddk-rn/ubrn.config.yaml`: UniFFI React Native configuration

### Generated Files (do not edit manually)
- TypeScript bindings in `ddk-rn/src/`
- C++ bindings in `ddk-rn/cpp/`
- iOS frameworks in `ddk-rn/ios/*.xcframework`
- Android libraries in `ddk-rn/android/src/main/jniLibs/`

## Known Issues

### Manual Post-Build Fix Required
After running `just uniffi`, manually fix the include path:
```cpp
// In ddk-rn/cpp/bennyhodl-ddk-rn.cpp, change:
#include "/ddk_ffi.hpp"
// To:
#include "ddk_ffi.hpp"
```

### Dependencies
- Requires `uniffi-bindgen-react-native` globally installed
- Uses pnpm as package manager (not npm/yarn)
- React Native new architecture enabled by default

## Testing

- Rust tests: `cargo test` (in ddk-ffi/)
- TypeScript tests: `pnpm test` (in ddk-rn/)
- Integration testing via example app

## Code Generation

All TypeScript, C++, iOS, and Android code is automatically generated from the Rust code and UDL definitions. Do not manually edit generated files as they will be overwritten on the next build.