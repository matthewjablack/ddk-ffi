#![allow(unused_imports, unused_variables)]
use bitcoin::hashes::Hash;
use bitcoin::{Amount, ScriptBuf, Transaction as BtcTransaction, TxIn, TxOut as BtcTxOut, Txid};
use dlc::{
    self, dlc_input::DlcInputInfo as RustDlcInputInfo, PartyParams as DlcPartyParams,
    Payout as DlcPayout,
};
use secp256k1_zkp::{schnorr::Signature as SchnorrSignature, EcdsaAdaptorSignature};
use secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey};
use std::str::FromStr;

uniffi::include_scaffolding!("ddk_ffi");

pub fn hello_world() -> String {
    "Hello, World from Rust!".to_string()
}

pub fn do_the_dlc() -> String {
    "heyhowareya".to_string()
}

pub fn lygos() -> String {
    "lygos".to_string()
}

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
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Serialization error")]
    SerializationError,
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(String),
    #[error("Miniscript error")]
    MiniscriptError,
    #[error("Network error")]
    NetworkError,
}

impl From<dlc::Error> for DLCError {
    fn from(err: dlc::Error) -> Self {
        match err {
            dlc::Error::Secp256k1(_) => DLCError::Secp256k1Error(err.to_string()),
            dlc::Error::InvalidArgument => DLCError::InvalidArgument,
            dlc::Error::Miniscript(_) => DLCError::MiniscriptError,
            _ => DLCError::InvalidArgument,
        }
    }
}

// UniFFI struct definitions (as defined in UDL)
pub struct Transaction {
    pub version: i32,
    pub lock_time: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub raw_bytes: Vec<u8>,
}

pub struct TxInput {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

pub struct TxInputInfo {
    pub txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub max_witness_length: u32,
    pub serial_id: u64,
}

pub struct DlcOutcome {
    pub local_payout: u64,
    pub remote_payout: u64,
}

pub struct Payout {
    pub offer: u64,
    pub accept: u64,
}

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

pub struct DlcTransactions {
    pub fund: Transaction,
    pub cets: Vec<Transaction>,
    pub refund: Transaction,
    pub funding_script_pubkey: Vec<u8>,
}

pub struct AdaptorSignature {
    pub signature: Vec<u8>,
    pub proof: Vec<u8>,
}

pub struct ChangeOutputAndFees {
    pub change_output: TxOutput,
    pub fund_fee: u64,
    pub cet_fee: u64,
}

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
    let contract_id: [u8; 32] = input
        .contract_id
        .as_slice()
        .try_into()
        .map_err(|_| DLCError::InvalidArgument)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let result = hello_world();
        assert_eq!(result, "Hello, World from Rust!");
    }
}
