# DDK-FFI React Native

A React Native library that provides Rust-powered functionality through UniFFI bindings. This project demonstrates how to integrate Rust code into React Native applications using `uniffi-bindgen-react-native`.

## 📁 Project Structure

```
.
├── ddk-ffi/                 # Rust crate with UniFFI definitions
│   ├── src/
│   │   ├── lib.rs          # Rust implementation
│   │   └── ddk_ffi.udl     # UniFFI interface definitions
│   ├── Cargo.toml
│   └── uniffi.toml         # UniFFI configuration for Kotlin/Swift
│
├── ddk-rn/                  # React Native library
│   ├── src/                # Generated TypeScript bindings
│   ├── cpp/                # Generated C++ bindings for JSI
│   ├── ios/                # iOS native module
│   ├── android/            # Android native module
│   ├── example/            # Example React Native app
│   └── ubrn.config.yaml    # UniFFI React Native configuration
│
└── justfile                 # Build automation commands
```

## 🚀 Prerequisites

- **Rust** (with `cargo`)
- **Node.js** (v18+) and **pnpm**
- **React Native development environment**
  - iOS: Xcode, CocoaPods
  - Android: Android Studio, JDK 11+
- **uniffi-bindgen-react-native** (`npm i -g uniffi-bindgen-react-native`)
- **just** (`cargo install just`)

## 🛠️ Installation

1. **Clone the repository**

   ```bash
   git clone <repository-url>
   cd ddk-ffi
   ```

2. **Install dependencies**

   ```bash
   # Install Rust dependencies
   cd ddk-ffi
   cargo build

   # Install React Native dependencies
   cd ../ddk-rn
   pnpm install
   ```

3. **Generate bindings and build**
   ```bash
   # From project root
   just uniffi
   ```

## 🔧 Development Workflow

### 1. Write Rust Code

Edit your Rust implementation in `ddk-ffi/src/lib.rs`:

```rust
pub fn hello_world() -> String {
    "Hello from Rust!".to_string()
}

pub fn do_the_dlc() -> String {
    "DLC functionality here".to_string()
}
```

### 2. Define UniFFI Interface

Add function signatures to `ddk-ffi/src/ddk_ffi.udl`:

```udl
namespace ddk_ffi {
    string hello_world();
    string do_the_dlc();
};
```

### 3. Generate Bindings

```bash
# Generate TypeScript and C++ bindings
just uniffi-jsi

# Generate TurboModule specs
just uniffi-turbo

# Build native libraries for iOS
just build-ios

# Build native libraries for Android
just build-android
```

### 4. Run Example App

```bash
# iOS (with new architecture)
just example-ios
cd ddk-rn/example
npx react-native run-ios

# Android
just example-android
cd ddk-rn/example
npx react-native run-android
```

## 📝 Build Commands Reference

| Command                | Description                                                 |
| ---------------------- | ----------------------------------------------------------- |
| `just uniffi`          | Runs the complete build pipeline (jsi, turbo, ios, android) |
| `just uniffi-jsi`      | Generates TypeScript and C++ JSI bindings from the UDL file |
| `just uniffi-turbo`    | Generates React Native TurboModule specifications           |
| `just build-ios`       | Builds iOS static libraries and creates XCFramework         |
| `just build-android`   | Builds Android native libraries (JNI)                       |
| `just example-ios`     | Installs iOS dependencies with new architecture enabled     |
| `just example-android` | Builds the Android example app                              |
| `just clean`           | Removes all generated files and build artifacts             |

## ⚠️ Known Issues & Fixes

### Manual Fix Required

After running `just uniffi`, you need to manually fix the include path in `ddk-rn/cpp/bennyhodl-ddk-rn.cpp`:

```cpp
// Change from:
#include "/ddk_ffi.hpp"
// To:
#include "ddk_ffi.hpp"
```

### Bob 0.30 Babel Runtime Issue

If you encounter `@babel/runtime` errors, ensure it's installed:

```bash
cd ddk-rn
pnpm add @babel/runtime
```

## 🏗️ Architecture

This project uses UniFFI to bridge Rust code to React Native:

1. **Rust Layer** (`ddk-ffi/`): Core business logic written in Rust
2. **UniFFI Bindings**: Automatically generated C/C++ bindings from UDL definitions
3. **JSI Layer**: Direct JavaScript Interface for synchronous Rust function calls
4. **TurboModule**: React Native's new architecture module system
5. **TypeScript**: Type-safe API for JavaScript/React Native consumers

## 📦 Publishing

Before publishing to npm:

```bash
cd ddk-rn
pnpm build  # Runs Bob to build CommonJS/ESM modules
npm publish
```
