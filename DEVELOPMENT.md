# Development Practices

This document outlines the key development practices and workflow for the ddk-ffi library.

## Core Principles

### 1. Pure Wrapper Architecture

- **This library should be PURELY a wrapper of rust-dlc**
- **AVOID copying logic** from rust-dlc into this codebase
- All DLC functionality must delegate to the rust-dlc crate
- Only implement type conversions and UniFFI interface bindings
- When rust-dlc updates, this library should continue working without code changes (only recompilation)

### 2. No Code Duplication

- Do not reimplement any DLC logic that exists in rust-dlc
- If functionality is missing from rust-dlc, contribute it upstream rather than implementing it here
- Keep conversion functions minimal and focused only on type transformation

## Required Workflow for Changes

### Before Every Commit/Tag

When making changes to `src/lib.rs` or `src/ddk_ffi.udl`, you MUST:

1. **Generate bindings**: Run `just uniffi` to regenerate all language bindings
2. **Fix include path**: Manually fix the include path in `ddk-rn/cpp/bennyblader-ddk-rn.cpp`:
   ```cpp
   // Change this:
   #include "/ddk_ffi.hpp"
   // To this:
   #include "ddk_ffi.hpp"
   ```
3. **Test build**: Verify the generated bindings compile correctly
4. **Commit together**: Include both Rust changes AND generated bindings in the same commit
   ```bash
   git add .  # Add all changes to the current directory
   git commit -m "feat: description of changes"
   ```

### Release Process

#### Automated Release (Recommended)

5. **Update Rust version only**: Update version in Cargo.toml

   ```bash
   # Update Rust crate version to match package.json
   vim ddk-ffi/Cargo.toml  # Change version = "0.1.1" to "0.1.2"
   ```

6. **Regenerate bindings**: Run `just uniffi` to update version in generated bindings

   ```bash
   just uniffi
   # Fix include path as usual
   sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' ddk-rn/cpp/bennyblader-ddk-rn.cpp
   ```

7. **Commit Rust version change**: Commit the Rust version bump

   ```bash
   git add .
   git commit -m "chore: sync Rust version with package.json"
   ```

8. **Automated release with npm publishing**: Use release-it for everything else

   ```bash
   cd ddk-rn

   # Authenticate with npm (first time only)
   npm login

   # Run automated release (handles versioning, tagging, GitHub release, npm publish)
   pnpm release
   ```

This will automatically:

- Prompt for version bump in package.json
- Build the library with react-native-builder-bob
- Create git tag and GitHub release
- Publish to npm registry
- Generate conventional changelog

#### Manual Release (If needed)

Alternatively, you can do it manually:

5. **Update Version Numbers**: Update version in both package manifests

   ```bash
   # Update Rust crate version
   vim ddk-ffi/Cargo.toml  # Change version = "0.1.1" to "0.1.2"

   # Update React Native package version
   vim ddk-rn/package.json  # Change "version": "0.1.1" to "0.1.2"
   ```

6. **Regenerate bindings**: Run `just uniffi` to update version in generated bindings

   ```bash
   just uniffi
   sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' ddk-rn/cpp/bennyblader-ddk-rn.cpp
   ```

7. **Build and test package**: Verify the npm package builds correctly

   ```bash
   cd ddk-rn
   pnpm prepare  # Build with react-native-builder-bob
   npm pack --dry-run  # Preview what will be published
   ```

8. **Commit version changes**: Include version bumps in the release commit

   ```bash
   git add .
   git commit -m "chore: bump version to v<version>"
   ```

9. **Create and push tag**: Create git tag and push to GitHub

   ```bash
   git tag -a v<version> -m "Release v<version>: <description>"
   git push origin master
   git push origin --tags
   ```

10. **Publish to npm**: Publish the package

    ```bash
    cd ddk-rn
    npm publish
    ```

11. **Create GitHub Release**: Use GitHub CLI to create a release
    ```bash
    gh release create v<version> --generate-notes --title "Release v<version>: <title>"
    ```

### Complete Development Cycle (Automated)

```bash
# 1. Make changes to Rust code
vim ddk-ffi/src/lib.rs ddk-ffi/src/ddk_ffi.udl

# 2. Test changes
cd ddk-ffi && cargo test

# 3. Generate bindings
just uniffi

# 4. Fix include path
sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' ddk-rn/cpp/bennyblader-ddk-rn.cpp

# 5. Commit feature changes
git add .
git commit -m "feat: description of changes"

# 6. Update Rust version to match package.json
vim ddk-ffi/Cargo.toml    # Update version = "0.1.2" (match package.json)

# 7. Regenerate bindings with new version
just uniffi
sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' ddk-rn/cpp/bennyblader-ddk-rn.cpp

# 8. Commit Rust version sync
git add .
git commit -m "chore: sync Rust version with package.json"

# 9. Automated release with npm publishing
cd ddk-rn
npm login  # First time only
pnpm release  # Handles everything: versioning, tagging, GitHub release, npm publish
```

### What `pnpm release` does automatically:

- Prompts for version bump (patch, minor, major)
- Updates package.json version
- Builds library with react-native-builder-bob
- Generates conventional changelog
- Creates git commit and tag
- Pushes to GitHub
- Creates GitHub release
- Publishes to npm registry

### Why This Matters

- Generated bindings must stay in sync with Rust code
- Consumers of the library need both Rust logic and bindings to work together
- Prevents broken builds when someone pulls only partial changes

## Development Workflow

1. **Make Rust changes** in `src/lib.rs` or `src/ddk_ffi.udl`
2. **Run tests**: `cargo test` to verify Rust functionality
3. **Generate bindings**: `just uniffi` to update all language bindings
4. **Fix include path** in generated C++ file
5. **Test bindings**: Verify iOS/Android/TypeScript bindings compile
6. **Commit everything**: Include Rust + generated bindings in single commit

## Code Standards

### Wrapper Functions

```rust
// GOOD: Pure wrapper that delegates to rust-dlc
pub fn create_dlc_transactions(/* params */) -> Result<DlcTransactions, DLCError> {
    // Convert UniFFI types to rust-dlc types
    let rust_params = convert_params(params)?;

    // Call rust-dlc function
    let result = dlc::create_dlc_transactions(&rust_params)?;

    // Convert result back to UniFFI types
    Ok(convert_result(result))
}

// BAD: Reimplementing DLC logic
pub fn create_dlc_transactions(/* params */) -> Result<DlcTransactions, DLCError> {
    // Don't do this - reimplementing DLC transaction creation logic
    let mut tx = Transaction::new();
    // ... hundreds of lines of DLC logic copied from rust-dlc
}
```

### Error Handling

- Convert rust-dlc errors to UniFFI errors using `From` traits
- Don't create new error conditions that rust-dlc doesn't have
- Preserve error semantics from the underlying library

### Testing

- Test wrapper functions by comparing results with direct rust-dlc calls
- Verify type conversions work correctly with realistic data
- Test error handling paths

## Architecture Validation

Ask these questions for every change:

1. **Am I copying logic from rust-dlc?** → If yes, find a way to call rust-dlc instead
2. **Will this break when rust-dlc updates?** → If yes, make it more generic
3. **Could this be contributed to rust-dlc instead?** → Consider upstream contribution
4. **Am I generating bindings after Rust changes?** → Always required

## NPM Publishing Setup

### First Time Setup

Before you can publish to npm, you need:

1. **npm account**: Create account at https://www.npmjs.com/
2. **Access to @bennyblader scope**: Ensure you have publish permissions
3. **Authentication**: Run `npm login` in the `ddk-rn/` directory
4. **Verify access**: Test with `npm whoami` and `npm access ls-packages @bennyblader`

### Publishing Requirements

- Package builds successfully with `pnpm prepare`
- All tests pass with `pnpm test`
- Version in `package.json` is higher than published version
- Git working tree is clean

## Memory

- **CRITICAL**: Always run `just uniffi` and fix the include path before committing changes to `.udl` or `.rs` files
- **PRINCIPLE**: This is a pure wrapper - delegate to rust-dlc, never reimplement
- **RELEASES**: Use `pnpm release` for automated npm publishing with proper versioning
