# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-01-15

### Added

#### Core DLC Transaction Functions
- **`create_dlc_transactions`** - Create complete DLC transaction set (funding, CETs, refund)
- **`create_spliced_dlc_transactions`** - Create spliced DLC transactions for complex scenarios
- **`create_cet`** - Create single Contract Execution Transaction
- **`create_cets`** - Create multiple CETs for different outcomes
- **`create_refund_transaction`** - Create refund transaction with locktime
- **`create_fund_tx_locking_script`** - Generate funding transaction locking script

#### Utility Functions
- **`is_dust_output`** - Check if transaction output is below dust limit
- **`get_change_output_and_fees`** - Calculate change outputs and fee requirements
- **`get_total_input_vsize`** - Calculate virtual size for fee estimation
- **`verify_fund_tx_signature`** - Verify funding transaction signatures

#### Signing & Cryptographic Functions
- **`get_raw_funding_transaction_input_signature`** - Get raw ECDSA signature for funding input
- **`sign_fund_transaction_input`** - Sign funding transaction input with private key
- **`create_cet_adaptor_signature_from_oracle_info`** - Create adaptor signatures for oracle-based CETs

#### Data Structures
- **`Transaction`** - UniFFI-compatible Bitcoin transaction representation
- **`TxInput`** / **`TxOutput`** - Transaction input/output structures
- **`TxInputInfo`** - Input metadata for DLC construction
- **`DlcOutcome`** - Outcome definition with local/remote payouts
- **`PartyParams`** - Party-specific DLC parameters and inputs
- **`DlcTransactions`** - Complete DLC transaction set container
- **`AdaptorSignature`** - Adaptor signature with proof
- **`ChangeOutputAndFees`** - Change output calculation results
- **`OracleInfo`** - Oracle public key and nonces for DLC attestation

#### Error Handling
- **`DLCError`** enum with comprehensive error types:
  - `InvalidSignature`, `InvalidPublicKey`, `InvalidTransaction`
  - `InsufficientFunds`, `InvalidArgument`, `SerializationError`
  - `Secp256k1Error`, `MiniscriptError`, `NetworkError`
- Automatic conversion from `rust-dlc` and `secp256k1-zkp` errors

### Implementation Details

#### Architecture
- **Pure wrapper approach**: All functions delegate to `rust-dlc` crate
- **Zero code duplication**: No reimplementation of DLC logic
- **Type conversion layer**: Seamless conversion between UniFFI and rust-dlc types
- **Forward compatibility**: Updates to `rust-dlc` automatically propagate

#### Conversion Functions
- Bidirectional conversion between Bitcoin and UniFFI transaction types
- Safe conversion of cryptographic primitives (keys, signatures, scripts)
- Proper error handling for invalid data formats

#### Testing
- Comprehensive test suite covering all major functions
- Wrapper function validation against direct `rust-dlc` calls
- Error condition testing for invalid inputs
- Realistic DLC scenario testing with proper cryptographic data

### Documentation
- Extensive inline documentation for all public functions
- Code examples and usage patterns
- Error handling guidance
- DLC specification references

### Dependencies
- **rust-dlc**: Core DLC implementation (with newly public functions)
- **bitcoin**: Bitcoin transaction and script handling
- **secp256k1-zkp**: Cryptographic operations and zero-knowledge proofs
- **uniffi**: Foreign function interface generation

### Notes
This release provides complete feature parity with `cfd-dlc/dlc_transactions.cpp` while maintaining a clean wrapper architecture. All functionality leverages the battle-tested `rust-dlc` library without code duplication, ensuring reliability and maintainability.

## [0.1.0] - 2025-01-15

### Added
- Initial project structure
- Basic UniFFI integration
- Demo functions (`hello_world`, `do_the_dlc`, `lygos`)
- Project documentation and build system setup