# @bennyblader/ddk-rn

**Rust-powered DLC (Discreet Log Contracts) bindings for React Native**

A React Native library that provides complete DLC transaction functionality through high-performance Rust bindings. Built and tested with React Native 0.75 using the new architecture.

[![npm version](https://badge.fury.io/js/@bennyblader%2Fddk-rn.svg)](https://badge.fury.io/js/@bennyblader%2Fddk-rn)
[![GitHub](https://img.shields.io/github/license/bennyhodl/ddk-ffi)](https://github.com/bennyhodl/ddk-ffi/blob/master/LICENSE)

## ✨ Features

- **Complete DLC Support**: Full feature parity with industry-standard DLC implementations
- **High Performance**: Rust-powered core with zero-copy data transfer via JSI
- **Type Safe**: Full TypeScript definitions with comprehensive error handling
- **Production Ready**: Built on battle-tested [rust-dlc](https://github.com/p2pderivatives/rust-dlc) library
- **React Native 0.75+**: Optimized for the new architecture with TurboModules

### 🔥 DLC Capabilities
- **Transaction Creation**: Funding, CET, and refund transaction generation
- **Adaptor Signatures**: Oracle-based conditional execution
- **Fee Management**: Intelligent fee calculation and dust handling  
- **Multi-Party Support**: Full support for complex DLC scenarios
- **Signing & Verification**: Complete cryptographic operations

## 📦 Installation

```bash
# Using npm
npm install @bennyblader/ddk-rn

# Using yarn
yarn add @bennyblader/ddk-rn

# Using pnpm
pnpm add @bennyblader/ddk-rn
```

### Platform Setup

#### iOS
```bash
cd ios && pod install
```

#### Android
No additional setup required - native libraries are included.

## 🚀 Quick Start

```typescript
import { 
  createDlcTransactions, 
  createFundTxLockingScript,
  DlcOutcome,
  PartyParams 
} from '@bennyblader/ddk-rn';

// Initialize DLC parties
const localParams: PartyParams = {
  fundPubkey: localPublicKey,
  changeScriptPubkey: localChangeScript,
  changeSerialId: 1n,
  payoutScriptPubkey: localPayoutScript,
  payoutSerialId: 2n,
  inputs: localInputs,
  inputAmount: 1000000n, // 0.01 BTC in sats
  collateral: 500000n,   // 0.005 BTC in sats
  dlcInputs: []
};

// Define contract outcomes
const outcomes: DlcOutcome[] = [
  { localPayout: 1000000n, remotePayout: 0n },      // Local wins
  { localPayout: 500000n, remotePayout: 500000n },  // Split
  { localPayout: 0n, remotePayout: 1000000n }       // Remote wins
];

// Create complete DLC transaction set
try {
  const dlcTxs = createDlcTransactions(
    outcomes,
    localParams,
    remoteParams,
    144,        // refund locktime (blocks)
    2n,         // fee rate (sat/vB)
    0,          // fund lock time
    0,          // CET lock time
    0n          // fund output serial ID
  );
  
  console.log('✅ DLC transactions created:', {
    funding: dlcTxs.fund.rawBytes.length,
    cets: dlcTxs.cets.length,
    refund: dlcTxs.refund.rawBytes.length
  });
} catch (error) {
  console.error('❌ DLC creation failed:', error);
}
```

## 📖 API Reference

### Core Transaction Functions

#### `createDlcTransactions(outcomes, localParams, remoteParams, refundLocktime, feeRate, fundLockTime, cetLockTime, fundOutputSerialId)`
Creates a complete set of DLC transactions (funding, CETs, refund).

#### `createFundTxLockingScript(localFundPubkey, remoteFundPubkey)`
Generates the multisig locking script for the funding transaction.

#### `createCets(fundTxId, fundVout, localScript, remoteScript, outcomes, lockTime, localSerialId, remoteSerialId)`
Creates multiple Contract Execution Transactions for different outcomes.

#### `createRefundTransaction(localScript, remoteScript, localAmount, remoteAmount, lockTime, fundTxId, fundVout)`
Creates a refund transaction with CSV timelock.

### Signing Functions

#### `signFundTransactionInput(fundTx, privkey, prevTxId, prevTxVout, value)`
Signs a funding transaction input with the provided private key.

#### `verifyFundTxSignature(fundTx, signature, pubkey, txid, vout, inputAmount)`
Verifies a signature on a funding transaction input.

#### `createCetAdaptorSignatureFromOracleInfo(cet, oracleInfo, fundingSk, fundingScript, totalCollateral, msgs)`
Creates adaptor signatures for oracle-based contract execution.

### Utility Functions

#### `getChangeOutputAndFees(params, feeRate)`
Calculates change outputs and fee requirements for a party.

#### `isDustOutput(output)`
Checks if a transaction output is below the dust threshold.

#### `getTotalInputVsize(inputs)`
Calculates the virtual size of inputs for fee estimation.

### Error Handling

All functions return detailed error information:

```typescript
import { DLCError } from '@bennyblader/ddk-rn';

try {
  const result = createDlcTransactions(/* ... */);
} catch (error) {
  if (error instanceof DLCError) {
    switch (error.message) {
      case 'InvalidPublicKey':
        console.log('Invalid public key provided');
        break;
      case 'InsufficientFunds':
        console.log('Not enough funds for transaction');
        break;
      default:
        console.log('DLC error:', error.message);
    }
  }
}
```

## 🏗️ Architecture

This library uses a **pure wrapper approach** around the [rust-dlc](https://github.com/p2pderivatives/rust-dlc) library:

1. **Rust Core**: All DLC logic implemented in rust-dlc (battle-tested)
2. **Zero Duplication**: No reimplemented DLC functionality
3. **Type Conversion**: Seamless bridging between TypeScript and Rust types
4. **JSI Performance**: Direct memory access for maximum performance
5. **Forward Compatible**: Automatic updates when rust-dlc improves

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────┐
│   React Native  │    │  TypeScript  │    │    Rust     │
│      App        │───▶│   Bindings   │───▶│   rust-dlc  │
│                 │    │   (Generated)│    │   (Core)    │
└─────────────────┘    └──────────────┘    └─────────────┘
       ▲                       ▲                   ▲
       │                       │                   │
   App Logic              Type Safety        DLC Implementation
```

## 🧪 Example Usage Patterns

### Complete DLC Flow
```typescript
import { 
  createDlcTransactions,
  signFundTransactionInput,
  createCetAdaptorSignatureFromOracleInfo,
  version
} from '@bennyblader/ddk-rn';

console.log('📦 Using ddk-rn version:', version());

// 1. Create DLC transactions
const dlcTxs = createDlcTransactions(outcomes, localParams, remoteParams, ...);

// 2. Sign funding transaction
const signedFundTx = signFundTransactionInput(
  dlcTxs.fund, 
  privateKey, 
  inputTxId, 
  inputVout, 
  inputValue
);

// 3. Create adaptor signatures for CETs
const adaptorSigs = dlcTxs.cets.map(cet => 
  createCetAdaptorSignatureFromOracleInfo(
    cet, 
    oracleInfo, 
    fundingPrivkey,
    dlcTxs.fundingScriptPubkey,
    totalCollateral,
    messages
  )
);
```

### Fee Estimation
```typescript
import { getChangeOutputAndFees, getTotalInputVsize } from '@bennyblader/ddk-rn';

// Calculate fees and change
const feeInfo = getChangeOutputAndFees(partyParams, 2n); // 2 sat/vB

console.log('💰 Fee breakdown:', {
  fundingFee: feeInfo.fundFee,
  cetFee: feeInfo.cetFee,
  changeAmount: feeInfo.changeOutput.value
});

// Estimate input size for fee calculation
const inputSize = getTotalInputVsize(inputs);
console.log('📏 Input vsize:', inputSize, 'vBytes');
```

## ⚡ Performance

- **Zero-copy operations** via React Native JSI
- **Rust-level performance** for cryptographic operations  
- **Minimal overhead** type conversions
- **Synchronous execution** - no promise overhead for core operations

## 🛠️ Development

Want to contribute or modify this library? See our comprehensive development guide:

### Prerequisites
- **Rust** (latest stable)
- **Node.js** 18+ and **pnpm**
- **React Native development environment**
- **just** (`cargo install just`)
- **uniffi-bindgen-react-native** (`npm i -g uniffi-bindgen-react-native`)

### Development Workflow

1. **Clone and setup**
   ```bash
   git clone https://github.com/bennyhodl/ddk-ffi.git
   cd ddk-ffi
   ```

2. **Make changes to Rust code**
   ```bash
   # Edit Rust implementation
   vim ddk-ffi/src/lib.rs
   
   # Update UniFFI interface
   vim ddk-ffi/src/ddk_ffi.udl
   ```

3. **Generate and test**
   ```bash
   # Generate all bindings
   just uniffi
   
   # Fix include path (required after generation)
   sed -i '' 's|#include "/ddk_ffi.hpp"|#include "ddk_ffi.hpp"|' ddk-rn/cpp/bennyblader-ddk-rn.cpp
   
   # Test changes
   cd ddk-ffi && cargo test
   cd ../ddk-rn && pnpm test
   ```

4. **Run example app**
   ```bash
   # iOS
   cd ddk-rn/example && npx react-native run-ios
   
   # Android  
   cd ddk-rn/example && npx react-native run-android
   ```

### Release Process

This library uses automated releases with `release-it`:

```bash
cd ddk-rn
pnpm release  # Handles versioning, building, tagging, and npm publishing
```

For detailed development instructions, see [DEVELOPMENT.md](./DEVELOPMENT.md).

## 📋 Requirements

- **React Native**: 0.75+
- **New Architecture**: Required (TurboModules/Fabric)
- **iOS**: 11.0+
- **Android**: API 23+

## 📄 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🤝 Contributing

Contributions welcome! Please read our [development guide](./DEVELOPMENT.md) and ensure:

1. All tests pass (`cargo test` and `pnpm test`)
2. Bindings are regenerated (`just uniffi`)
3. Code follows the **pure wrapper** principle
4. Changes include appropriate documentation

## 🔗 Links

- **GitHub**: https://github.com/bennyhodl/ddk-ffi
- **npm Package**: https://www.npmjs.com/package/@bennyblader/ddk-rn  
- **Issues**: https://github.com/bennyhodl/ddk-ffi/issues
- **rust-dlc**: https://github.com/p2pderivatives/rust-dlc

---

Built with ❤️ using [rust-dlc](https://github.com/p2pderivatives/rust-dlc) and [UniFFI](https://mozilla.github.io/uniffi-rs/)