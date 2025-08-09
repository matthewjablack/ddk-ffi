# @bennyblader/ddk-ts

TypeScript/Node.js bindings for the DLC Dev Kit (DDK) - NAPI-RS based native bindings for Node.js applications.

## Installation

```bash
npm install @bennyblader/ddk-ts
# or
pnpm add @bennyblader/ddk-ts
```

The package includes prebuilt binaries for:

- macOS ARM64 (Apple Silicon)
- Linux x64 (glibc)

## Quick Start

```typescript
import { version, createFundTxLockingScript } from '@bennyblader/ddk-ts'

console.log(`DDK Version: ${version()}`)
```

For complete API documentation, see the [main README](../README.md#api-reference).

## Development

### Prerequisites

- Node.js >= 14
- Rust >= 1.70
- pnpm
- NAPI-RS CLI: `npm install -g @napi-rs/cli`

### Building from Source

```bash
# Install dependencies
pnpm install

# Build for current platform
pnpm build

# Build for all supported platforms (Darwin ARM64 and Linux x64)
pnpm build:all
```

### Just Commands

```bash
# Build TypeScript bindings for current platform
just ts-build

# Build for all supported platforms
just ts-build-all

# Run example
just ts-example

# Run tests
just ts-test

# Development setup (build + example setup)
just ts-dev

# Release new version to npm
just ts-release <version>
```

### Project Structure

```
ddk-ts/
├── src-napi/           # Rust NAPI-RS source code
│   ├── lib.rs          # Main library wrapper functions
│   ├── types.rs        # Type definitions matching UDL
│   └── conversions.rs  # Type conversions between Rust and JS
├── src/                # Generated TypeScript/JavaScript files
│   ├── index.js        # Main entry point (generated)
│   └── index.d.ts      # TypeScript definitions (generated)
├── npm/                # Platform-specific packages
│   ├── darwin-arm64/   # macOS ARM64 package
│   └── linux-x64-gnu/  # Linux x64 package
├── example/            # Example TypeScript application
├── __test__/           # Test files
└── scripts/            # Build and verification scripts
    └── verify-parity.js # Ensures API compatibility with UDL
```

### Testing

```bash
# Run all tests
pnpm test

# Run verification scripts
pnpm verify        # Run all verification checks
pnpm verify:parity # Check API parity with UDL definitions
pnpm verify:types  # Verify TypeScript types
```

### Platform Support

| Platform | Architecture          | Status          |
| -------- | --------------------- | --------------- |
| macOS    | ARM64 (Apple Silicon) | ✅ Supported    |
| Linux    | x64 (glibc)           | ✅ Supported    |
| macOS    | x64 (Intel)           | ❌ Not included |
| Windows  | x64                   | ❌ Not included |
| Linux    | ARM64                 | ❌ Not included |

### Release Process

To release a new version:

```bash
just ts-release 0.2.0
```

This will:

1. Check working directory is clean
2. Update version in package.json
3. Build for all supported platforms
4. Run tests
5. Commit and tag as `ddk-ts-v0.2.0`
6. Push to GitHub
7. Publish to npm

### API Compatibility

The TypeScript bindings maintain 100% API compatibility with the UniFFI definitions. The [verify-parity.js](scripts/verify-parity.js) script ensures that all functions defined in the UDL file are properly exposed in the TypeScript bindings.

## Troubleshooting

### Missing Binary

If you get an error about missing binaries, ensure your platform is supported or build from source:

```bash
pnpm build
```

### BigInt Support

All 64-bit integers are represented as JavaScript `BigInt`. Make sure your Node.js version supports BigInt (Node.js 10.4.0+).

## License

MIT
