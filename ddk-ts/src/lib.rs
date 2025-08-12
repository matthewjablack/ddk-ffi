#![deny(clippy::all)]

pub mod conversions;
mod types;

use conversions::*;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use types::*;

// Import ddk_ffi crate
extern crate ddk_ffi;

#[napi]
pub fn version() -> String {
  ddk_ffi::version()
}

#[napi]
pub fn create_fund_tx_locking_script(
  local_fund_pubkey: Buffer,
  remote_fund_pubkey: Buffer,
) -> Result<Buffer> {
  let local_pubkey = buffer_to_vec(&local_fund_pubkey);
  let remote_pubkey = buffer_to_vec(&remote_fund_pubkey);

  let result = ddk_ffi::create_fund_tx_locking_script(local_pubkey, remote_pubkey)
    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(vec_to_buffer(result))
}

#[napi]
pub fn create_dlc_transactions(
  outcomes: Vec<DlcOutcome>,
  local_params: PartyParams,
  remote_params: PartyParams,
  refund_locktime: u32,
  fee_rate: BigInt,
  fund_lock_time: u32,
  cet_lock_time: u32,
  fund_output_serial_id: BigInt,
) -> Result<DlcTransactions> {
  let ffi_outcomes: Result<Vec<ddk_ffi::DlcOutcome>> =
    outcomes.into_iter().map(TryInto::try_into).collect();

  let ffi_local_params = local_params.try_into()?;
  let ffi_remote_params = remote_params.try_into()?;

  let result = ddk_ffi::create_dlc_transactions(
    ffi_outcomes?,
    ffi_local_params,
    ffi_remote_params,
    refund_locktime,
    bigint_to_u64(&fee_rate)?,
    fund_lock_time,
    cet_lock_time,
    bigint_to_u64(&fund_output_serial_id)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn create_spliced_dlc_transactions(
  outcomes: Vec<DlcOutcome>,
  local_params: PartyParams,
  remote_params: PartyParams,
  refund_locktime: u32,
  fee_rate: BigInt,
  fund_lock_time: u32,
  cet_lock_time: u32,
  fund_output_serial_id: BigInt,
) -> Result<DlcTransactions> {
  let ffi_outcomes: Result<Vec<ddk_ffi::DlcOutcome>> =
    outcomes.into_iter().map(TryInto::try_into).collect();

  let ffi_local_params = local_params.try_into()?;
  let ffi_remote_params = remote_params.try_into()?;

  let result = ddk_ffi::create_spliced_dlc_transactions(
    ffi_outcomes?,
    ffi_local_params,
    ffi_remote_params,
    refund_locktime,
    bigint_to_u64(&fee_rate)?,
    fund_lock_time,
    cet_lock_time,
    bigint_to_u64(&fund_output_serial_id)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn create_cet(
  local_output: TxOutput,
  local_payout_serial_id: BigInt,
  remote_output: TxOutput,
  remote_payout_serial_id: BigInt,
  fund_tx_id: String,
  fund_vout: u32,
  lock_time: u32,
) -> Result<Transaction> {
  let result = ddk_ffi::create_cet(
    local_output.try_into()?,
    bigint_to_u64(&local_payout_serial_id)?,
    remote_output.try_into()?,
    bigint_to_u64(&remote_payout_serial_id)?,
    fund_tx_id,
    fund_vout,
    lock_time,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn create_cets(
  fund_tx_id: String,
  fund_vout: u32,
  local_final_script_pubkey: Buffer,
  remote_final_script_pubkey: Buffer,
  outcomes: Vec<DlcOutcome>,
  lock_time: u32,
  local_serial_id: BigInt,
  remote_serial_id: BigInt,
) -> Result<Vec<Transaction>> {
  let ffi_outcomes: Result<Vec<ddk_ffi::DlcOutcome>> =
    outcomes.into_iter().map(TryInto::try_into).collect();

  let result = ddk_ffi::create_cets(
    fund_tx_id,
    fund_vout,
    buffer_to_vec(&local_final_script_pubkey),
    buffer_to_vec(&remote_final_script_pubkey),
    ffi_outcomes?,
    lock_time,
    bigint_to_u64(&local_serial_id)?,
    bigint_to_u64(&remote_serial_id)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into_iter().map(Into::into).collect())
}

#[napi]
pub fn create_refund_transaction(
  local_final_script_pubkey: Buffer,
  remote_final_script_pubkey: Buffer,
  local_amount: BigInt,
  remote_amount: BigInt,
  lock_time: u32,
  fund_tx_id: String,
  fund_vout: u32,
) -> Result<Transaction> {
  let result = ddk_ffi::create_refund_transaction(
    buffer_to_vec(&local_final_script_pubkey),
    buffer_to_vec(&remote_final_script_pubkey),
    bigint_to_u64(&local_amount)?,
    bigint_to_u64(&remote_amount)?,
    lock_time,
    fund_tx_id,
    fund_vout,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn is_dust_output(output: TxOutput) -> Result<bool> {
  let ffi_output = output.try_into()?;
  Ok(ddk_ffi::is_dust_output(ffi_output))
}

#[napi]
pub fn get_change_output_and_fees(
  params: PartyParams,
  fee_rate: BigInt,
) -> Result<ChangeOutputAndFees> {
  let result = ddk_ffi::get_change_output_and_fees(params.try_into()?, bigint_to_u64(&fee_rate)?)
    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn get_total_input_vsize(inputs: Vec<TxInputInfo>) -> Result<u32> {
  let ffi_inputs: Result<Vec<ddk_ffi::TxInputInfo>> =
    inputs.into_iter().map(TryInto::try_into).collect();

  Ok(ddk_ffi::get_total_input_vsize(ffi_inputs?))
}

#[napi]
pub fn verify_fund_tx_signature(
  fund_tx: Transaction,
  signature: Buffer,
  pubkey: Buffer,
  txid: String,
  vout: u32,
  input_amount: BigInt,
) -> Result<bool> {
  let result = ddk_ffi::verify_fund_tx_signature(
    fund_tx.try_into()?,
    buffer_to_vec(&signature),
    buffer_to_vec(&pubkey),
    txid,
    vout,
    bigint_to_u64(&input_amount)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result)
}

#[napi]
pub fn get_raw_funding_transaction_input_signature(
  funding_transaction: Transaction,
  privkey: Buffer,
  prev_tx_id: String,
  prev_tx_vout: u32,
  value: BigInt,
) -> Result<Buffer> {
  let result = ddk_ffi::get_raw_funding_transaction_input_signature(
    funding_transaction.try_into()?,
    buffer_to_vec(&privkey),
    prev_tx_id,
    prev_tx_vout,
    bigint_to_u64(&value)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(vec_to_buffer(result))
}

#[napi]
pub fn sign_fund_transaction_input(
  fund_transaction: Transaction,
  privkey: Buffer,
  prev_tx_id: String,
  prev_tx_vout: u32,
  value: BigInt,
) -> Result<Transaction> {
  let result = ddk_ffi::sign_fund_transaction_input(
    fund_transaction.try_into()?,
    buffer_to_vec(&privkey),
    prev_tx_id,
    prev_tx_vout,
    bigint_to_u64(&value)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn sign_cet(
  cet: Transaction,
  adaptor_signature: Buffer,
  oracle_signatures: Vec<Buffer>,
  funding_secret_key: Buffer,
  other_pubkey: Buffer,
  funding_script_pubkey: Buffer,
  fund_output_value: BigInt,
) -> Result<Transaction> {
  let result = ddk_ffi::sign_cet(
    cet.try_into()?,
    buffer_to_vec(&adaptor_signature),
    oracle_signatures.iter().map(buffer_to_vec).collect(),
    buffer_to_vec(&funding_secret_key),
    buffer_to_vec(&other_pubkey),
    buffer_to_vec(&funding_script_pubkey),
    bigint_to_u64(&fund_output_value)?,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn create_cet_adaptor_signature_from_oracle_info(
  cet: Transaction,
  oracle_info: OracleInfo,
  funding_sk: Buffer,
  funding_script_pubkey: Buffer,
  total_collateral: BigInt,
  msgs: Vec<Buffer>,
) -> Result<AdaptorSignature> {
  let ffi_oracle_info = oracle_info.into();
  let ffi_msgs: Vec<Vec<u8>> = msgs.iter().map(buffer_to_vec).collect();

  let result = ddk_ffi::create_cet_adaptor_signature_from_oracle_info(
    cet.try_into()?,
    ffi_oracle_info,
    buffer_to_vec(&funding_sk),
    buffer_to_vec(&funding_script_pubkey),
    bigint_to_u64(&total_collateral)?,
    ffi_msgs,
  )
  .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(result.into())
}

#[napi]
pub fn convert_mnemonic_to_seed(mnemonic: String, passphrase: Option<String>) -> Result<Buffer> {
  let result = ddk_ffi::convert_mnemonic_to_seed(mnemonic, passphrase)
    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(vec_to_buffer(result))
}

#[napi]
pub fn create_xpriv_from_parent_path(
  xpriv: Buffer,
  base_derivation_path: String,
  network: String,
  path: String,
) -> Result<Buffer> {
  let xpriv_bytes = buffer_to_vec(&xpriv);
  let result =
    ddk_ffi::create_xpriv_from_parent_path(xpriv_bytes, base_derivation_path, network, path)
      .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(vec_to_buffer(result))
}

#[napi]
pub fn get_xpub_from_xpriv(xpriv: Buffer, network: String) -> Result<Buffer> {
  let xpriv_bytes = buffer_to_vec(&xpriv);
  let result = ddk_ffi::get_xpub_from_xpriv(xpriv_bytes, network)
    .map_err(|e| Error::from_reason(format!("{:?}", e)))?;

  Ok(vec_to_buffer(result))
}
