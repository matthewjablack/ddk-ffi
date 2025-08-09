# DDK-RN Development

This document covers development practices specific to the React Native bindings (ddk-rn).

## Quick Start

```bash
# Generate all bindings and build native libraries
just uniffi

# Fix the include path (required after every uniffi generation)
sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' cpp/bennyblader-ddk-rn.cpp

# Build and test
pnpm prepare
pnpm test
pnpm typecheck
```

## Development Workflow

1. **Modify Rust code** in `../ddk-ffi/src/`
2. **Update interface** in `../ddk-ffi/src/ddk_ffi.udl`
3. **Generate bindings**: `just uniffi`
4. **Fix include path**: Manual fix required in `cpp/bennyblader-ddk-rn.cpp`
5. **Test changes**: Use example app or run tests

## Release Process

### Automated Release (Recommended)

```bash
# 1. Ensure Rust version matches package.json
vim ../ddk-ffi/Cargo.toml

# 2. Generate bindings
just uniffi
sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' cpp/bennyblader-ddk-rn.cpp

# 3. Create binary archives
pnpm create-archives

# 4. Run automated release
npm login  # First time only
pnpm release

# 5. Upload archives to GitHub release
gh release upload v<version> ../release-archives/*.tar.gz
```

## Testing

- **Unit tests**: `pnpm test`
- **Type checking**: `pnpm typecheck`
- **Example app iOS**: `cd example && npx react-native run-ios`
- **Example app Android**: `cd example && npx react-native run-android`

## Project Structure

- `src/`: Generated TypeScript bindings
- `cpp/`: Generated C++ JSI bindings
- `ios/`: iOS native module and XCFramework
- `android/`: Android native module and JNI libraries
- `example/`: React Native example app