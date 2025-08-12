#![allow(clippy::too_many_arguments)]
use bip39::{Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, IntoDerivationPath, Xpriv, Xpub};
use bitcoin::hashes::Hash;
use bitcoin::sighash::EcdsaSighashType;
use bitcoin::{
    Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction as BtcTransaction, TxIn,
    TxOut as BtcTxOut, Txid, Witness,
};
use bitcoin::{Script, WPubkeyHash};
use dlc::{
    self, dlc_input::DlcInputInfo as RustDlcInputInfo, DlcTransactions as RustDlcTransactions,
    OracleInfo as DlcOracleInfo, PartyParams as DlcPartyParams, Payout as DlcPayout,
    TxInputInfo as DlcTxInputInfo,
};
use secp256k1_zkp::{
    ecdsa::Signature as EcdsaSignature, Message, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey,
};
use secp256k1_zkp::{schnorr::Signature as SchnorrSignature, All, EcdsaAdaptorSignature};
use std::str::FromStr;
use std::sync::OnceLock;

uniffi::include_scaffolding!("ddk_ffi");

static SECP_CONTEXT: OnceLock<Secp256k1<All>> = OnceLock::new();

pub fn get_secp_context() -> &'static Secp256k1<All> {
    SECP_CONTEXT.get_or_init(Secp256k1::new)
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Minimum value that can be included in a transaction output. Under this value,
/// outputs are discarded
/// See: https://github.com/discreetlogcontracts/dlcspecs/blob/master/Transactions.md#change-outputs
const DUST_LIMIT: u64 = 1000;

/// The witness size of a P2WPKH input
/// See: <https://github.com/discreetlogcontracts/dlcspecs/blob/master/Transactions.md#fees>
pub const P2WPKH_WITNESS_SIZE: usize = 107;

// Error type implementation
#[derive(Debug, thiserror::Error)]
pub enum DLCError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Serialization error")]
    SerializationError,
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(String),
    #[error("Miniscript error")]
    MiniscriptError,
    #[error("Invalid network")]
    InvalidNetwork,
    #[error("Extended key error: {0}")]
    KeyError(ExtendedKey),
}

#[derive(Debug, thiserror::Error)]
pub enum ExtendedKey {
    #[error("Invalid mnemonic")]
    InvalidMnemonic,
    #[error("Invalid xpriv")]
    InvalidXpriv,
    #[error("Invalid xpub")]
    InvalidXpub,
    #[error("Invalid derivation path")]
    InvalidDerivationPath,
}

impl From<dlc::Error> for DLCError {
    fn from(err: dlc::Error) -> Self {
        match err {
            dlc::Error::Secp256k1(_) => DLCError::Secp256k1Error(err.to_string()),
            dlc::Error::InvalidArgument => {
                DLCError::InvalidArgument("Error from rust-dlc".to_string())
            }
            dlc::Error::Miniscript(_) => DLCError::MiniscriptError,
            dlc::Error::P2wpkh(_) => DLCError::InvalidTransaction,
            dlc::Error::InputsIndex(_) => {
                DLCError::InvalidArgument("Error from rust-dlc: InputsIndex".to_string())
            }
        }
    }
}

impl From<secp256k1_zkp::Error> for DLCError {
    fn from(err: secp256k1_zkp::Error) -> Self {
        DLCError::Secp256k1Error(err.to_string())
    }
}

// UniFFI struct definitions (as defined in UDL)
#[derive(Clone)]
pub struct Transaction {
    pub version: i32,
    pub lock_time: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub raw_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct TxInput {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Clone)]
pub struct TxInputInfo {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub max_witness_length: u32,
    pub serial_id: u64,
}

#[derive(Clone)]
pub struct Payout {
    pub offer: u64,
    pub accept: u64,
}

#[derive(Clone)]
pub struct DlcInputInfo {
    pub fund_tx: Transaction,
    pub fund_vout: u32,
    pub local_fund_pubkey: Vec<u8>,
    pub remote_fund_pubkey: Vec<u8>,
    pub fund_amount: u64,
    pub max_witness_len: u32,
    pub input_serial_id: u64,
    pub contract_id: Vec<u8>,
}

#[derive(Clone)]
pub struct PartyParams {
    pub fund_pubkey: Vec<u8>,
    pub change_script_pubkey: Vec<u8>,
    pub change_serial_id: u64,
    pub payout_script_pubkey: Vec<u8>,
    pub payout_serial_id: u64,
    pub inputs: Vec<TxInputInfo>,
    pub input_amount: u64,
    pub collateral: u64,
    pub dlc_inputs: Vec<DlcInputInfo>,
}

#[derive(Clone)]
pub struct DlcTransactions {
    pub fund: Transaction,
    pub cets: Vec<Transaction>,
    pub refund: Transaction,
    pub funding_script_pubkey: Vec<u8>,
}

#[derive(Clone)]
pub struct AdaptorSignature {
    pub signature: Vec<u8>,
    pub proof: Vec<u8>,
}

#[derive(Clone)]
pub struct ChangeOutputAndFees {
    pub change_output: TxOutput,
    pub fund_fee: u64,
    pub cet_fee: u64,
}

#[derive(Clone)]
pub struct OracleInfo {
    pub public_key: Vec<u8>,
    pub nonces: Vec<Vec<u8>>,
}

// Conversion helpers
pub fn btc_tx_to_transaction(tx: &BtcTransaction) -> Transaction {
    use bitcoin::consensus::Encodable;
    let mut raw_bytes = Vec::new();
    tx.consensus_encode(&mut raw_bytes).unwrap();

    Transaction {
        version: tx.version.0,
        lock_time: tx.lock_time.to_consensus_u32(),
        inputs: tx
            .input
            .iter()
            .map(|input| TxInput {
                txid: input.previous_output.txid.to_string(),
                vout: input.previous_output.vout,
                script_sig: input.script_sig.to_bytes(),
                sequence: input.sequence.0,
                witness: input.witness.iter().map(|w| w.to_vec()).collect(),
            })
            .collect(),
        outputs: tx
            .output
            .iter()
            .map(|output| TxOutput {
                value: output.value.to_sat(),
                script_pubkey: output.script_pubkey.to_bytes(),
            })
            .collect(),
        raw_bytes,
    }
}

pub fn plz_work() -> String {
    "heyhowareya".to_string()
}

pub fn transaction_to_btc_tx(tx: &Transaction) -> Result<BtcTransaction, DLCError> {
    use bitcoin::consensus::Decodable;
    BtcTransaction::consensus_decode(&mut &tx.raw_bytes[..])
        .map_err(|_| DLCError::SerializationError)
}

pub fn dlc_input_info_to_rust(input: &DlcInputInfo) -> Result<RustDlcInputInfo, DLCError> {
    let btc_tx = transaction_to_btc_tx(&input.fund_tx)?;
    let local_fund_pubkey =
        PublicKey::from_slice(&input.local_fund_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;
    let remote_fund_pubkey =
        PublicKey::from_slice(&input.remote_fund_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;
    let contract_id: [u8; 32] = input.contract_id.as_slice().try_into().map_err(|_| {
        DLCError::InvalidArgument("Contract id length must be 32 bytes.".to_string())
    })?;
    Ok(RustDlcInputInfo {
        fund_tx: btc_tx,
        fund_vout: input.fund_vout,
        local_fund_pubkey,
        remote_fund_pubkey,
        fund_amount: Amount::from_sat(input.fund_amount),
        max_witness_len: input.max_witness_len as usize,
        input_serial_id: input.input_serial_id,
        contract_id,
    })
}

/// Convert UniFFI TxInputInfo to rust-dlc TxInputInfo
pub fn tx_input_info_to_rust(input: &TxInputInfo) -> Result<DlcTxInputInfo, DLCError> {
    let txid = Txid::from_str(&input.txid)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;
    Ok(DlcTxInputInfo {
        outpoint: OutPoint {
            txid,
            vout: input.vout,
        },
        max_witness_len: input.max_witness_length as usize,
        redeem_script: ScriptBuf::from(input.script_sig.clone()),
        serial_id: input.serial_id,
    })
}

/// Convert UniFFI PartyParams to rust-dlc PartyParams
pub fn party_params_to_rust(params: &PartyParams) -> Result<DlcPartyParams, DLCError> {
    let fund_pubkey =
        PublicKey::from_slice(&params.fund_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;

    let inputs: Result<Vec<_>, _> = params.inputs.iter().map(tx_input_info_to_rust).collect();

    let dlc_inputs: Result<Vec<_>, _> = params
        .dlc_inputs
        .iter()
        .map(dlc_input_info_to_rust)
        .collect();

    Ok(DlcPartyParams {
        fund_pubkey,
        change_script_pubkey: ScriptBuf::from(params.change_script_pubkey.clone()),
        change_serial_id: params.change_serial_id,
        payout_script_pubkey: ScriptBuf::from(params.payout_script_pubkey.clone()),
        payout_serial_id: params.payout_serial_id,
        inputs: inputs?,
        dlc_inputs: dlc_inputs?,
        input_amount: Amount::from_sat(params.input_amount),
        collateral: Amount::from_sat(params.collateral),
    })
}

/// Convert rust-dlc DlcTransactions to UniFFI DlcTransactions
pub fn rust_dlc_transactions_to_uniffi(dlc_txs: RustDlcTransactions) -> DlcTransactions {
    DlcTransactions {
        fund: btc_tx_to_transaction(&dlc_txs.fund),
        cets: dlc_txs.cets.iter().map(btc_tx_to_transaction).collect(),
        refund: btc_tx_to_transaction(&dlc_txs.refund),
        funding_script_pubkey: dlc_txs.funding_script_pubkey.to_bytes(),
    }
}

/// Create a funding script pubkey for DLC transactions
pub fn create_fund_tx_locking_script(
    local_fund_pubkey: Vec<u8>,
    remote_fund_pubkey: Vec<u8>,
) -> Result<Vec<u8>, DLCError> {
    let local_pk =
        PublicKey::from_slice(&local_fund_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;
    let remote_pk =
        PublicKey::from_slice(&remote_fund_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;

    let script = dlc::make_funding_redeemscript(&local_pk, &remote_pk);
    Ok(script.to_bytes())
}

/// Create complete DLC transactions
pub fn create_dlc_transactions(
    outcomes: Vec<Payout>,
    local_params: PartyParams,
    remote_params: PartyParams,
    refund_locktime: u32,
    fee_rate: u64,
    fund_lock_time: u32,
    cet_lock_time: u32,
    fund_output_serial_id: u64,
) -> Result<DlcTransactions, DLCError> {
    // Convert UniFFI types to rust-dlc types
    let rust_local_params = party_params_to_rust(&local_params)?;
    let rust_remote_params = party_params_to_rust(&remote_params)?;

    // Convert outcomes to payouts
    let payouts: Vec<DlcPayout> = outcomes
        .iter()
        .map(|outcome| DlcPayout {
            offer: Amount::from_sat(outcome.offer),
            accept: Amount::from_sat(outcome.accept),
        })
        .collect();

    // Use rust-dlc library to create transactions
    let dlc_txs = dlc::create_dlc_transactions(
        &rust_local_params,
        &rust_remote_params,
        &payouts,
        refund_locktime,
        fee_rate,
        fund_lock_time,
        cet_lock_time,
        fund_output_serial_id,
    )
    .map_err(DLCError::from)?;

    // Convert back to UniFFI types
    Ok(rust_dlc_transactions_to_uniffi(dlc_txs))
}

/// Create spliced DLC transactions
pub fn create_spliced_dlc_transactions(
    outcomes: Vec<Payout>,
    local_params: PartyParams,
    remote_params: PartyParams,
    refund_locktime: u32,
    fee_rate: u64,
    fund_lock_time: u32,
    cet_lock_time: u32,
    fund_output_serial_id: u64,
) -> Result<DlcTransactions, DLCError> {
    // Convert UniFFI types to rust-dlc types
    let rust_local_params = party_params_to_rust(&local_params)?;
    let rust_remote_params = party_params_to_rust(&remote_params)?;

    // Convert outcomes to payouts
    let payouts: Vec<DlcPayout> = outcomes
        .iter()
        .map(|outcome| DlcPayout {
            offer: Amount::from_sat(outcome.offer),
            accept: Amount::from_sat(outcome.accept),
        })
        .collect();

    // Use rust-dlc library to create spliced transactions
    let dlc_txs = dlc::create_spliced_dlc_transactions(
        &rust_local_params,
        &rust_remote_params,
        &payouts,
        refund_locktime,
        fee_rate,
        fund_lock_time,
        cet_lock_time,
        fund_output_serial_id,
    )
    .map_err(DLCError::from)?;

    // Convert back to UniFFI types
    Ok(rust_dlc_transactions_to_uniffi(dlc_txs))
}

/// Create a single CET
pub fn create_cet(
    local_output: TxOutput,
    local_payout_serial_id: u64,
    remote_output: TxOutput,
    remote_payout_serial_id: u64,
    fund_tx_id: String,
    fund_vout: u32,
    lock_time: u32,
) -> Result<Transaction, DLCError> {
    let txid = Txid::from_str(&fund_tx_id)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    let local_btc_output = BtcTxOut {
        value: Amount::from_sat(local_output.value),
        script_pubkey: ScriptBuf::from(local_output.script_pubkey),
    };

    let remote_btc_output = BtcTxOut {
        value: Amount::from_sat(remote_output.value),
        script_pubkey: ScriptBuf::from(remote_output.script_pubkey),
    };

    let fund_tx_input = TxIn {
        previous_output: OutPoint {
            txid,
            vout: fund_vout,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ZERO,
        witness: Witness::new(),
    };

    let btc_tx = dlc::create_cet(
        local_btc_output,
        local_payout_serial_id,
        remote_btc_output,
        remote_payout_serial_id,
        &fund_tx_input,
        lock_time,
    );

    Ok(btc_tx_to_transaction(&btc_tx))
}

/// Create multiple CETs
pub fn create_cets(
    fund_tx_id: String,
    fund_vout: u32,
    local_final_script_pubkey: Vec<u8>,
    remote_final_script_pubkey: Vec<u8>,
    outcomes: Vec<Payout>,
    lock_time: u32,
    local_serial_id: u64,
    remote_serial_id: u64,
) -> Result<Vec<Transaction>, DLCError> {
    let txid = Txid::from_str(&fund_tx_id)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    let fund_tx_input = TxIn {
        previous_output: OutPoint {
            txid,
            vout: fund_vout,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ZERO,
        witness: Witness::new(),
    };

    let local_script = Script::from_bytes(&local_final_script_pubkey);
    let remote_script = Script::from_bytes(&remote_final_script_pubkey);

    let payouts: Vec<DlcPayout> = outcomes
        .iter()
        .map(|outcome| DlcPayout {
            offer: Amount::from_sat(outcome.offer),
            accept: Amount::from_sat(outcome.accept),
        })
        .collect();

    let btc_txs = dlc::create_cets(
        &fund_tx_input,
        local_script,
        local_serial_id,
        remote_script,
        remote_serial_id,
        &payouts,
        lock_time,
    );

    Ok(btc_txs.iter().map(btc_tx_to_transaction).collect())
}

/// Create a refund transaction
pub fn create_refund_transaction(
    local_final_script_pubkey: Vec<u8>,
    remote_final_script_pubkey: Vec<u8>,
    local_amount: u64,
    remote_amount: u64,
    lock_time: u32,
    fund_tx_id: String,
    fund_vout: u32,
) -> Result<Transaction, DLCError> {
    let txid = Txid::from_str(&fund_tx_id)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    let local_output = BtcTxOut {
        value: Amount::from_sat(local_amount),
        script_pubkey: ScriptBuf::from(local_final_script_pubkey),
    };

    let remote_output = BtcTxOut {
        value: Amount::from_sat(remote_amount),
        script_pubkey: ScriptBuf::from(remote_final_script_pubkey),
    };

    let funding_input = TxIn {
        previous_output: OutPoint {
            txid,
            vout: fund_vout,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
        witness: Witness::new(),
    };

    let btc_tx =
        dlc::create_refund_transaction(local_output, remote_output, funding_input, lock_time);

    Ok(btc_tx_to_transaction(&btc_tx))
}

/// Check if a transaction output is dust
pub fn is_dust_output(output: TxOutput) -> bool {
    output.value < DUST_LIMIT
}

/// Get change output and fees for a party
pub fn get_change_output_and_fees(
    params: PartyParams,
    fee_rate: u64,
) -> Result<ChangeOutputAndFees, DLCError> {
    let rust_params = party_params_to_rust(&params)?;
    let total_collateral = Amount::from_sat(params.collateral * 2); // Assume bilateral

    let (change_output, fund_fee, cet_fee) = rust_params
        .get_change_output_and_fees(total_collateral, fee_rate, Amount::ZERO)
        .map_err(DLCError::from)?;

    let uniffi_output = TxOutput {
        value: change_output.value.to_sat(),
        script_pubkey: change_output.script_pubkey.to_bytes(),
    };

    Ok(ChangeOutputAndFees {
        change_output: uniffi_output,
        fund_fee: fund_fee.to_sat(),
        cet_fee: cet_fee.to_sat(),
    })
}

/// Get total input virtual size for fee calculation
pub fn get_total_input_vsize(inputs: Vec<TxInputInfo>) -> u32 {
    // Simplified calculation: P2WPKH inputs are ~148 vbytes each
    inputs.len() as u32 * 148
}

/// Verify a fund transaction signature
pub fn verify_fund_tx_signature(
    fund_tx: Transaction,
    signature: Vec<u8>,
    pubkey: Vec<u8>,
    txid: String,
    vout: u32,
    input_amount: u64,
) -> Result<bool, DLCError> {
    let btc_tx = transaction_to_btc_tx(&fund_tx)?;
    let pk = PublicKey::from_slice(&pubkey).map_err(|_| DLCError::InvalidPublicKey)?;
    let input_txid = Txid::from_str(&txid)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    // Find the input index
    let input_index = btc_tx
        .input
        .iter()
        .position(|input| {
            input.previous_output.txid == input_txid && input.previous_output.vout == vout
        })
        .ok_or(DLCError::InvalidArgument(format!(
            "Input index not found in {}",
            input_txid
        )))?;

    // Create a simple P2WPKH script for verification
    let wpkh = WPubkeyHash::hash(&pk.serialize());
    let script = bitcoin::ScriptBuf::new_p2wpkh(&wpkh);

    // Parse signature
    let sig = if signature.len() == 64 {
        EcdsaSignature::from_compact(&signature).map_err(|_| DLCError::InvalidSignature)?
    } else {
        // Remove sighash type byte and parse DER
        let sig_bytes = &signature[..signature.len() - 1];
        EcdsaSignature::from_der(sig_bytes).map_err(|_| DLCError::InvalidSignature)?
    };

    let secp = Secp256k1::verification_only();
    match dlc::verify_tx_input_sig(
        &secp,
        &sig,
        &btc_tx,
        input_index,
        &script,
        Amount::from_sat(input_amount),
        &pk,
    ) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ============================================================================
// SIGNING AND SIGNATURE FUNCTIONS (using rust-dlc library)
// ============================================================================

/// Get raw signature for a fund transaction input
pub fn get_raw_funding_transaction_input_signature(
    funding_transaction: Transaction,
    privkey: Vec<u8>,
    prev_tx_id: String,
    prev_tx_vout: u32,
    value: u64,
) -> Result<Vec<u8>, DLCError> {
    let btc_tx = transaction_to_btc_tx(&funding_transaction)?;
    let sk = SecretKey::from_slice(&privkey)
        .map_err(|_| DLCError::InvalidArgument("Invalid private key".to_string()))?;
    let prev_txid = Txid::from_str(&prev_tx_id)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    // Find the input index
    let input_index = btc_tx
        .input
        .iter()
        .position(|input| {
            input.previous_output.txid == prev_txid && input.previous_output.vout == prev_tx_vout
        })
        .ok_or(DLCError::InvalidArgument(format!(
            "Input index not found in {}",
            prev_txid
        )))?;

    let secp = get_secp_context();
    // Create P2WPKH script for signing
    let pk = PublicKey::from_secret_key(secp, &sk);
    let wpkh = WPubkeyHash::hash(&pk.serialize());
    let script = bitcoin::ScriptBuf::new_p2wpkh(&wpkh);

    let sig = dlc::util::get_raw_sig_for_tx_input(
        secp,
        &btc_tx,
        input_index,
        &script,
        Amount::from_sat(value),
        &sk,
    )
    .map_err(DLCError::from)?;

    Ok(sig.serialize_compact().to_vec())
}

/// Sign a funding transaction input
pub fn sign_fund_transaction_input(
    fund_transaction: Transaction,
    privkey: Vec<u8>,
    prev_tx_id: String,
    prev_tx_vout: u32,
    value: u64,
) -> Result<Transaction, DLCError> {
    let mut btc_tx = transaction_to_btc_tx(&fund_transaction)?;
    let sk = SecretKey::from_slice(&privkey)
        .map_err(|_| DLCError::InvalidArgument("Invalid private key".to_string()))?;
    let prev_txid = Txid::from_str(&prev_tx_id)
        .map_err(|_| DLCError::InvalidArgument("Invalid transaction id".to_string()))?;

    // Find the input index
    let input_index = btc_tx
        .input
        .iter()
        .position(|input| {
            input.previous_output.txid == prev_txid && input.previous_output.vout == prev_tx_vout
        })
        .ok_or(DLCError::InvalidArgument(format!(
            "Input index not found in {}",
            prev_txid
        )))?;

    let secp = Secp256k1::signing_only();
    dlc::util::sign_p2wpkh_input(
        &secp,
        &sk,
        &mut btc_tx,
        input_index,
        EcdsaSighashType::All,
        Amount::from_sat(value),
    )
    .map_err(DLCError::from)?;

    Ok(btc_tx_to_transaction(&btc_tx))
}

pub fn sign_cet(
    cet: Transaction,
    adaptor_signature: Vec<u8>,
    oracle_signatures: Vec<Vec<u8>>,
    funding_secret_key: Vec<u8>,
    other_pubkey: Vec<u8>,
    funding_script_pubkey: Vec<u8>,
    fund_output_value: u64,
) -> Result<Transaction, DLCError> {
    let mut btc_tx = transaction_to_btc_tx(&cet)?;
    let adaptor_sig = vec_to_ecdsa_adaptor_signature(adaptor_signature)?;
    let oracle_sigs = oracle_signatures
        .iter()
        .map(|sig| vec_to_schnorr_signature(sig))
        .collect::<Result<Vec<_>, _>>()?;
    let funding_sk = SecretKey::from_slice(&funding_secret_key)
        .map_err(|_| DLCError::InvalidArgument("Invalid funding secret key".to_string()))?;
    let other_pk = PublicKey::from_slice(&other_pubkey).map_err(|_| DLCError::InvalidPublicKey)?;
    let funding_script = Script::from_bytes(&funding_script_pubkey);
    let secp = get_secp_context();

    dlc::sign_cet(
        secp,
        &mut btc_tx,
        &adaptor_sig,
        &[oracle_sigs],
        &funding_sk,
        &other_pk,
        &funding_script,
        Amount::from_sat(fund_output_value),
    )
    .map_err(|e| DLCError::Secp256k1Error(e.to_string()))?;

    Ok(btc_tx_to_transaction(&btc_tx))
}

fn vec_to_schnorr_signature(signature: &Vec<u8>) -> Result<SchnorrSignature, DLCError> {
    let sig = SchnorrSignature::from_slice(signature).map_err(|_| DLCError::InvalidSignature)?;
    Ok(sig)
}

fn vec_to_ecdsa_adaptor_signature(signature: Vec<u8>) -> Result<EcdsaAdaptorSignature, DLCError> {
    EcdsaAdaptorSignature::from_slice(&signature).map_err(|_| DLCError::InvalidSignature)
}

pub fn create_cet_adaptor_sigs_from_oracle_info(
    cets: Vec<Transaction>,
    oracle_info: Vec<OracleInfo>,
    funding_secret_key: Vec<u8>,
    funding_script_pubkey: Vec<u8>,
    fund_output_value: u64,
    msgs: Vec<Vec<Vec<u8>>>,
) -> Result<Vec<AdaptorSignature>, DLCError> {
    if msgs.len() != cets.len() {
        return Err(DLCError::InvalidArgument(
            "Messages length must be equal to cets length".to_string(),
        ));
    }

    let cets = cets
        .iter()
        .map(|cet| transaction_to_btc_tx(cet))
        .collect::<Result<Vec<_>, _>>()?;
    let oracle_infos = oracle_info
        .iter()
        .map(|info| {
            let public_key = XOnlyPublicKey::from_slice(&info.public_key)
                .map_err(|_| DLCError::InvalidPublicKey)?;
            let nonces = info
                .nonces
                .iter()
                .map(|nonce| XOnlyPublicKey::from_slice(nonce))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| DLCError::InvalidArgument("Invalid nonce pubkey".to_string()))?;
            Ok(DlcOracleInfo { public_key, nonces })
        })
        .collect::<Result<Vec<_>, DLCError>>()
        .map_err(|_| DLCError::InvalidArgument("Invalid oracle info".to_string()))?;

    let funding_sk = SecretKey::from_slice(&funding_secret_key)
        .map_err(|_| DLCError::InvalidArgument("Invalid funding secret key".to_string()))?;
    let funding_script = Script::from_bytes(&funding_script_pubkey);
    let msgs = msgs
        .iter()
        .map(|msg| {
            msg.iter()
                .map(|m| {
                    Message::from_digest_slice(m)
                        .map_err(|_| DLCError::InvalidArgument("Invalid message".to_string()))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let secp = get_secp_context();

    let adaptor_sigs = dlc::create_cet_adaptor_sigs_from_oracle_info(
        secp,
        &cets,
        &oracle_infos,
        &funding_sk,
        &funding_script,
        Amount::from_sat(fund_output_value),
        &[msgs],
    )
    .map_err(|_| DLCError::InvalidSignature)?;

    let adaptor_sigs = adaptor_sigs
        .iter()
        .map(|sig| AdaptorSignature {
            signature: sig.as_ref().to_vec(),
            proof: Vec::new(),
        })
        .collect::<Vec<_>>();

    Ok(adaptor_sigs)
}

pub fn verify_cet_adaptor_sig_from_oracle_info(
    adaptor_sig: AdaptorSignature,
    cet: Transaction,
    oracle_infos: Vec<OracleInfo>,
    pubkey: Vec<u8>,
    funding_script_pubkey: Vec<u8>,
    total_collateral: u64,
    msgs: Vec<Vec<Vec<u8>>>,
) -> bool {
    let secp = get_secp_context();
    let Ok(btc_tx) = transaction_to_btc_tx(&cet) else {
        return false;
    };
    let Ok(adaptor_sig) = vec_to_ecdsa_adaptor_signature(adaptor_sig.signature) else {
        return false;
    };
    let Ok(oracle_infos) = oracle_infos
        .iter()
        .map(|info| {
            let public_key = XOnlyPublicKey::from_slice(&info.public_key)?;
            let nonces = info
                .nonces
                .iter()
                .map(|nonce| XOnlyPublicKey::from_slice(nonce))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DlcOracleInfo { public_key, nonces })
        })
        .collect::<Result<Vec<_>, dlc::Error>>()
    else {
        return false;
    };
    let Ok(pubkey) = PublicKey::from_slice(&pubkey) else {
        return false;
    };
    let funding_script = Script::from_bytes(&funding_script_pubkey);
    let Ok(msgs) = msgs
        .into_iter()
        .map(|msg| {
            msg.iter()
                .map(|m| Message::from_digest_slice(m).map_err(|_| DLCError::InvalidArgument))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()
    else {
        return false;
    };
    let Ok(adaptor_point) = dlc::get_adaptor_point_from_oracle_info(secp, &oracle_infos, &msgs)
    else {
        return false;
    };
    let Ok(_) = dlc::verify_cet_adaptor_sig_from_point(
        secp,
        &adaptor_sig,
        &btc_tx,
        &adaptor_point,
        &pubkey,
        &funding_script,
        Amount::from_sat(total_collateral),
    ) else {
        return false;
    };

    true
}

pub fn verify_cet_adaptor_sigs_from_oracle_info(
    adaptor_sigs: Vec<AdaptorSignature>,
    cets: Vec<Transaction>,
    oracle_infos: Vec<OracleInfo>,
    pubkey: Vec<u8>,
    funding_script_pubkey: Vec<u8>,
    total_collateral: u64,
    msgs: Vec<Vec<Vec<u8>>>,
) -> bool {
    cets.into_iter()
        .zip(adaptor_sigs.into_iter())
        .all(|(cet, adaptor_sig)| {
            verify_cet_adaptor_sig_from_oracle_info(
                adaptor_sig,
                cet,
                oracle_infos.clone(),
                pubkey.clone(),
                funding_script_pubkey.clone(),
                total_collateral,
                msgs.clone(),
            )
        })
}

/// Create CET adaptor signature from oracle info
pub fn create_cet_adaptor_signature_from_oracle_info(
    cet: Transaction,
    oracle_info: OracleInfo,
    funding_sk: Vec<u8>,
    funding_script_pubkey: Vec<u8>,
    total_collateral: u64,
    msgs: Vec<Vec<u8>>,
) -> Result<AdaptorSignature, DLCError> {
    let btc_tx = transaction_to_btc_tx(&cet)?;
    let sk = SecretKey::from_slice(&funding_sk)
        .map_err(|_| DLCError::InvalidArgument("Invalid funding secret key".to_string()))?;
    let funding_script = Script::from_bytes(&funding_script_pubkey);

    // Convert oracle info
    let oracle_pk = XOnlyPublicKey::from_slice(&oracle_info.public_key)
        .map_err(|_| DLCError::InvalidPublicKey)?;
    let nonces: Result<Vec<_>, _> = oracle_info
        .nonces
        .iter()
        .map(|n| XOnlyPublicKey::from_slice(n))
        .collect();
    let oracle_nonces = nonces.map_err(|_| DLCError::InvalidPublicKey)?;

    let dlc_oracle_info = DlcOracleInfo {
        public_key: oracle_pk,
        nonces: oracle_nonces,
    };

    // Convert messages
    let messages: Result<Vec<_>, _> = msgs
        .iter()
        .map(|msg| Message::from_digest_slice(msg))
        .collect();
    let msg_vec = messages.map_err(|_| DLCError::InvalidArgument("Invalid message".to_string()))?;
    let nested_msgs = vec![msg_vec]; // Wrap in vector for single oracle

    let secp = get_secp_context();
    let adaptor_sig = dlc::create_cet_adaptor_sig_from_oracle_info(
        secp,
        &btc_tx,
        &[dlc_oracle_info],
        &sk,
        funding_script,
        Amount::from_sat(total_collateral),
        &nested_msgs,
    )
    .map_err(DLCError::from)?;

    Ok(AdaptorSignature {
        signature: adaptor_sig.as_ref().to_vec(),
        proof: Vec::new(), // EcdsaAdaptorSignature doesn't expose proof directly
    })
}

pub fn convert_mnemonic_to_seed(
    mnemonic: String,
    passphrase: Option<String>,
) -> Result<Vec<u8>, DLCError> {
    let seed_mnemonic = Mnemonic::parse_in_normalized(Language::English, &mnemonic)
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidMnemonic))?;
    let passphrase = passphrase.unwrap_or("".to_string());
    let seed = seed_mnemonic.to_seed(&passphrase);
    Ok(seed.to_vec())
}

pub fn create_xpriv_from_parent_path(
    xpriv: Vec<u8>,
    base_derivation_path: String,
    network: String,
    path: String,
) -> Result<Vec<u8>, DLCError> {
    if xpriv.len() != 64 {
        return Err(DLCError::KeyError(ExtendedKey::InvalidXpriv));
    }
    let secp = get_secp_context();
    let network = Network::from_str(&network).map_err(|_| DLCError::InvalidNetwork)?;
    let xpriv = Xpriv::new_master(network, &xpriv)
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidXpriv))?;
    // Base path: 84'/0'/0'
    let base_path = DerivationPath::from_str(&base_derivation_path)
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidDerivationPath))?;

    // App path: {0 || 1}/{child_number}
    let app_path = path
        .into_derivation_path()
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidDerivationPath))?;

    let full_path = base_path.extend(app_path);

    let derived_xpriv = xpriv
        .derive_priv(secp, &full_path)
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidXpriv))?;

    Ok(derived_xpriv.encode().to_vec())
}

pub fn get_xpub_from_xpriv(xpriv: Vec<u8>, network: String) -> Result<Vec<u8>, DLCError> {
    let secp = get_secp_context();
    let network = Network::from_str(&network).map_err(|_| DLCError::InvalidNetwork)?;
    let xpriv = Xpriv::new_master(network, &xpriv)
        .map_err(|_| DLCError::KeyError(ExtendedKey::InvalidXpriv))?;
    let xpub = Xpub::from_priv(secp, &xpriv);
    Ok(xpub.encode().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::locktime::absolute::LockTime;
    use secp256k1_zkp::rand::{thread_rng, RngCore};
    use std::str::FromStr;

    /// Create test keys similar to rust-dlc tests
    fn create_test_keys() -> (SecretKey, PublicKey, SecretKey, PublicKey) {
        let secp = Secp256k1::new();
        let offer_sk =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let offer_pk = PublicKey::from_secret_key(&secp, &offer_sk);
        let accept_sk =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000002")
                .unwrap();
        let accept_pk = PublicKey::from_secret_key(&secp, &accept_sk);
        (offer_sk, offer_pk, accept_sk, accept_pk)
    }

    /// Create realistic party params for testing
    fn create_test_party_params(
        input_amount: u64,
        collateral: u64,
        fund_pubkey: Vec<u8>,
        serial_id: u64,
    ) -> PartyParams {
        let mut rng = thread_rng();

        // Create a realistic P2WPKH script
        let mut random_hash = [0u8; 20];
        rng.fill_bytes(&mut random_hash);
        let mut change_script = vec![0x00, 0x14]; // OP_0 + 20 bytes (P2WPKH)
        change_script.extend_from_slice(&random_hash);

        rng.fill_bytes(&mut random_hash);
        let mut payout_script = vec![0x00, 0x14]; // OP_0 + 20 bytes (P2WPKH)
        payout_script.extend_from_slice(&random_hash);

        PartyParams {
            fund_pubkey,
            change_script_pubkey: change_script,
            change_serial_id: serial_id + 1,
            payout_script_pubkey: payout_script,
            payout_serial_id: serial_id + 2,
            inputs: vec![TxInputInfo {
                txid: "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"
                    .to_string(),
                vout: serial_id as u32,
                script_sig: vec![],
                max_witness_length: 108,
                serial_id,
            }],
            input_amount,
            collateral,
            dlc_inputs: vec![],
        }
    }

    #[test]
    fn mnemonic_to_seed_test() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let rust_seed = mnemonic.to_seed_normalized("").to_vec();
        let ffi_seed = convert_mnemonic_to_seed(mnemonic.to_string(), None).unwrap();
        assert_eq!(rust_seed, ffi_seed);
    }

    #[test]
    fn xpriv_to_xpub_test() {
        let mnemonic = Mnemonic::generate(24).unwrap();
        let rust_xpriv =
            Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed_normalized("").to_vec()).unwrap();
        let ffi_xpriv = convert_mnemonic_to_seed(mnemonic.to_string(), None).unwrap();
        let rust_xpub = Xpub::from_priv(get_secp_context(), &rust_xpriv);
        let ffi_xpub = get_xpub_from_xpriv(ffi_xpriv, "bitcoin".to_string()).unwrap();
        assert_eq!(rust_xpub.encode().to_vec(), ffi_xpub);
    }

    #[test]
    fn xpriv_to_path() {
        let base_derivation_path = "84'/0'/0'";
        let app_path = "0/1";
        let network = "bitcoin";
        let secp = get_secp_context();

        let mnemonic = Mnemonic::generate(24).unwrap();
        let rust_xpriv =
            Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed_normalized("")).unwrap();
        let rust_path =
            DerivationPath::from_str(&format!("{}/{}", base_derivation_path, app_path)).unwrap();
        let rust_xpriv = rust_xpriv.derive_priv(&secp, &rust_path).unwrap();

        let ffi_xpriv_bytes = convert_mnemonic_to_seed(mnemonic.to_string(), None).unwrap();
        let ffi_xpub = create_xpriv_from_parent_path(
            ffi_xpriv_bytes,
            base_derivation_path.to_string(),
            network.to_string(),
            app_path.to_string(),
        )
        .unwrap();
        assert_eq!(rust_xpriv.encode().to_vec(), ffi_xpub);
    }

    #[test]
    fn test_create_fund_tx_locking_script_matches_rust_dlc() {
        let (_offer_sk, offer_pk, _accept_sk, accept_pk) = create_test_keys();

        // Test our wrapper
        let wrapper_result = create_fund_tx_locking_script(
            offer_pk.serialize().to_vec(),
            accept_pk.serialize().to_vec(),
        )
        .unwrap();

        // Compare with direct rust-dlc call
        let direct_result = dlc::make_funding_redeemscript(&offer_pk, &accept_pk);

        assert_eq!(wrapper_result, direct_result.to_bytes());
    }

    #[test]
    fn test_get_change_output_and_fees_wrapper() {
        let (_offer_sk, offer_pk, _accept_sk, _accept_pk) = create_test_keys();

        let params = create_test_party_params(
            150_000_000, // 1.5 BTC input
            100_000_000, // 1 BTC collateral
            offer_pk.serialize().to_vec(),
            1,
        );

        let result = get_change_output_and_fees(params.clone(), 4);
        assert!(result.is_ok());

        let change_and_fees = result.unwrap();

        // Verify we get reasonable values
        assert!(change_and_fees.fund_fee > 0);
        assert!(change_and_fees.cet_fee > 0);
        assert!(change_and_fees.change_output.value > 0);

        // Compare with direct rust-dlc call
        let rust_params = party_params_to_rust(&params).unwrap();
        let total_collateral = Amount::from_sat(params.collateral * 2);
        let direct_result = rust_params
            .get_change_output_and_fees(total_collateral, 4, Amount::ZERO)
            .unwrap();

        assert_eq!(change_and_fees.fund_fee, direct_result.1.to_sat());
        assert_eq!(change_and_fees.cet_fee, direct_result.2.to_sat());
        assert_eq!(
            change_and_fees.change_output.value,
            direct_result.0.value.to_sat()
        );
    }

    #[test]
    fn test_create_dlc_transactions_wrapper() {
        let (_offer_sk, offer_pk, _accept_sk, accept_pk) = create_test_keys();

        let offer_params = create_test_party_params(
            1_000_000_000, // 10 BTC input
            100_000_000,   // 1 BTC collateral
            offer_pk.serialize().to_vec(),
            1,
        );

        let accept_params = create_test_party_params(
            1_000_000_000, // 10 BTC input
            100_000_000,   // 1 BTC collateral
            accept_pk.serialize().to_vec(),
            2,
        );

        let outcomes = vec![
            Payout {
                offer: 200_000_000, // 2 BTC to offer
                accept: 0,          // 0 BTC to accept
            },
            Payout {
                offer: 0,            // 0 BTC to offer
                accept: 200_000_000, // 2 BTC to accept
            },
        ];

        let result = create_dlc_transactions(
            outcomes,
            offer_params,
            accept_params,
            100, // refund locktime
            4,   // fee rate
            10,  // fund lock time
            10,  // cet lock time
            0,   // fund output serial id
        );

        assert!(result.is_ok());
        let dlc_txs = result.unwrap();

        // Verify structure
        assert_eq!(dlc_txs.fund.lock_time, 10);
        assert_eq!(dlc_txs.refund.lock_time, 100);
        assert_eq!(dlc_txs.cets.len(), 2);
        assert!(dlc_txs.cets.iter().all(|cet| cet.lock_time == 10));

        // Verify funding transaction has correct structure
        assert_eq!(dlc_txs.fund.inputs.len(), 2); // Two parties contributing
        assert!(dlc_txs.fund.outputs.len() >= 1); // At least funding output

        // Verify CETs have correct structure
        for cet in &dlc_txs.cets {
            assert_eq!(cet.inputs.len(), 1); // Single funding input
            assert!(cet.outputs.len() >= 1); // At least one output (dust may be filtered)
        }

        // Verify refund transaction
        assert_eq!(dlc_txs.refund.inputs.len(), 1); // Single funding input
        assert!(dlc_txs.refund.outputs.len() >= 2); // At least two refund outputs
    }

    #[test]
    fn test_create_cet_wrapper() {
        let local_output = TxOutput {
            value: 100_000_000, // 1 BTC
            script_pubkey: vec![
                0x00, 0x14, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14,
            ],
        };

        let remote_output = TxOutput {
            value: 100_000_000, // 1 BTC
            script_pubkey: vec![
                0x00, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
                0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
            ],
        };

        let result = create_cet(
            local_output,
            1,
            remote_output,
            2,
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            0,
            10,
        );

        assert!(result.is_ok());
        let cet = result.unwrap();

        assert_eq!(cet.lock_time, 10);
        assert_eq!(cet.inputs.len(), 1);
        assert_eq!(cet.outputs.len(), 2);
        assert_eq!(cet.outputs[0].value, 100_000_000);
        assert_eq!(cet.outputs[1].value, 100_000_000);
    }

    #[test]
    fn test_create_refund_transaction_wrapper() {
        let local_script = vec![
            0x00, 0x14, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
            0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14,
        ];
        let remote_script = vec![
            0x00, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
        ];

        let result = create_refund_transaction(
            local_script,
            remote_script,
            100_000_000, // 1 BTC to local
            100_000_000, // 1 BTC to remote
            144,         // locktime (1 day in blocks)
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            0,
        );

        assert!(result.is_ok());
        let refund_tx = result.unwrap();

        assert_eq!(refund_tx.lock_time, 144);
        assert_eq!(refund_tx.inputs.len(), 1);
        assert_eq!(refund_tx.outputs.len(), 2);
        assert_eq!(refund_tx.outputs[0].value, 100_000_000);
        assert_eq!(refund_tx.outputs[1].value, 100_000_000);
    }

    #[test]
    fn test_is_dust_output() {
        let dust_output = TxOutput {
            value: 500, // Below dust limit
            script_pubkey: vec![],
        };

        let non_dust_output = TxOutput {
            value: 5000, // Above dust limit
            script_pubkey: vec![],
        };

        assert!(is_dust_output(dust_output));
        assert!(!is_dust_output(non_dust_output));
    }

    #[test]
    fn test_conversion_functions() {
        let (_offer_sk, offer_pk, _accept_sk, _accept_pk) = create_test_keys();

        // Test party params conversion
        let params =
            create_test_party_params(100_000_000, 50_000_000, offer_pk.serialize().to_vec(), 1);

        let rust_params = party_params_to_rust(&params).unwrap();
        assert_eq!(rust_params.fund_pubkey, offer_pk);
        assert_eq!(rust_params.input_amount, Amount::from_sat(100_000_000));
        assert_eq!(rust_params.collateral, Amount::from_sat(50_000_000));

        // Test TX input conversion
        let tx_input = TxInputInfo {
            txid: "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456".to_string(),
            vout: 0,
            script_sig: vec![],
            max_witness_length: 108,
            serial_id: 1,
        };

        let rust_input = tx_input_info_to_rust(&tx_input).unwrap();
        assert_eq!(rust_input.serial_id, 1);
        assert_eq!(rust_input.max_witness_len, 108);
        assert_eq!(rust_input.outpoint.vout, 0);
    }

    #[test]
    fn test_transaction_bidirectional_conversion() {
        // Create a test Bitcoin transaction
        let btc_tx = BtcTransaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: LockTime::from_consensus(144),
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str(
                        "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456",
                    )
                    .unwrap(),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Witness::new(),
            }],
            output: vec![BtcTxOut {
                value: Amount::from_sat(100_000_000),
                script_pubkey: ScriptBuf::from(vec![0x00, 0x14]),
            }],
        };

        // Convert to UniFFI format and back
        let uniffi_tx = btc_tx_to_transaction(&btc_tx);
        let converted_back = transaction_to_btc_tx(&uniffi_tx).unwrap();

        // Verify they're equivalent
        assert_eq!(btc_tx.version, converted_back.version);
        assert_eq!(btc_tx.lock_time, converted_back.lock_time);
        assert_eq!(btc_tx.input.len(), converted_back.input.len());
        assert_eq!(btc_tx.output.len(), converted_back.output.len());
        assert_eq!(
            btc_tx.input[0].previous_output,
            converted_back.input[0].previous_output
        );
        assert_eq!(btc_tx.output[0].value, converted_back.output[0].value);
    }

    #[test]
    fn test_error_handling_invalid_keys() {
        // Test invalid public key
        let result = create_fund_tx_locking_script(
            vec![0u8; 20], // Invalid key length
            vec![1u8; 33],
        );
        assert!(matches!(result, Err(DLCError::InvalidPublicKey)));

        // Test invalid txid
        let result = create_cet(
            TxOutput {
                value: 1000,
                script_pubkey: vec![],
            },
            1,
            TxOutput {
                value: 1000,
                script_pubkey: vec![],
            },
            2,
            "invalid_txid".to_string(),
            0,
            0,
        );
        assert!(matches!(result, Err(DLCError::InvalidArgument(_))));
    }
}
