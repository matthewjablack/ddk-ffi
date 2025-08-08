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
2. **Fix include path**: Manually fix the include path in `ddk-rn/cpp/bennyhodl-ddk-rn.cpp`:
   ```cpp
   // Change this:
   #include "/ddk_ffi.hpp"
   // To this:  
   #include "ddk_ffi.hpp"
   ```
3. **Test build**: Verify the generated bindings compile correctly
4. **Commit together**: Include both Rust changes AND generated bindings in the same commit

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

## Memory
- **CRITICAL**: Always run `just uniffi` and fix the include path before committing changes to `.udl` or `.rs` files
- **PRINCIPLE**: This is a pure wrapper - delegate to rust-dlc, never reimplement