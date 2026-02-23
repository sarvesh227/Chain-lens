use serde::{Deserialize, Serialize};

//
// ==========================
// INPUT FIXTURE MODELS
// ==========================
//

#[derive(Deserialize, Clone)]
pub struct Prevout {
    pub txid: String,
    pub vout: u32,
    pub value_sats: u64,
    pub script_pubkey_hex: String,
}

#[derive(Deserialize, Clone)]
pub struct Fixture {
    pub network: String,
    pub raw_tx: String,
    pub prevouts: Vec<Prevout>,
}

//
// ==========================
// INTERNAL PARSED TX MODEL
// ==========================
//

#[derive(Serialize, Debug, Clone)]
pub struct TxInput {
    pub txid: String,
    pub vout: u32,
    pub sequence: u32,
    pub script_sig_hex: String,
    pub witness: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TxOutput {
    pub value_sats: u64,
    pub script_pubkey_hex: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Transaction {
    pub version: u32,
    pub locktime: u32,

    pub input_count: u64,
    pub output_count: u64,

    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,

    pub total_input_sats: u64,
    pub total_output_sats: u64,
    pub fee_sats: u64,

    pub segwit: bool,
    pub txid: String,
    pub wtxid: Option<String>,

    pub size_bytes: usize,
    pub stripped_size: usize,
    pub weight: u64,
    pub vbytes: u64,
    pub fee_rate_sat_vb: f64,
}

//
// ==========================
// FINAL TX REPORT (WEB + CLI)
// ==========================
//

#[derive(Serialize)]
pub struct TxReport {
    pub ok: bool,
    pub network: String,

    // Identity
    pub txid: String,
    pub wtxid: Option<String>,
    pub segwit: bool,

    // Basic metadata
    pub version: u32,
    pub locktime: u32,
    pub locktime_type: String,
    pub locktime_value: u32,

    // Size & fee
    pub size_bytes: usize,
    pub weight: u64,
    pub vbytes: u64,
    pub total_input_sats: u64,
    pub total_output_sats: u64,
    pub fee_sats: u64,
    pub fee_rate_sat_vb: f64,

    // Behavioral signals
    pub rbf_signaling: bool,

    // SegWit comparison
    pub segwit_savings: Option<SegwitSavings>,

    // Detailed I/O
    pub vin: Vec<VinReport>,
    pub vout: Vec<VoutReport>,

    // Risk / informational
    pub warnings: Vec<Warning>,
}

//
// ==========================
// SEGWIT SAVINGS
// ==========================
//

#[derive(Serialize)]
pub struct SegwitSavings {
    pub witness_bytes: usize,
    pub non_witness_bytes: usize,
    pub total_bytes: usize,
    pub weight_actual: u64,
    pub weight_if_legacy: u64,
    pub savings_pct: f64,
}

//
// ==========================
// VIN / VOUT REPORTS
// ==========================
//

#[derive(Serialize)]
pub struct VinReport {
    pub txid: String,
    pub vout: u32,
    pub sequence: u32,

    pub script_sig_hex: String,
    pub script_asm: String,
    pub witness: Vec<String>,

    pub script_type: String,
    pub address: Option<String>,

    pub prevout: PrevoutReport,
    pub relative_timelock: RelativeTimelock,
}

#[derive(Serialize)]
pub struct VoutReport {
    pub n: u32,
    pub value_sats: u64,

    pub script_pubkey_hex: String,
    pub script_asm: String,

    pub script_type: String,
    pub address: Option<String>,

    pub op_return_data_hex: Option<String>,
    pub op_return_data_utf8: Option<String>,
    pub op_return_protocol: Option<String>,
}

//
// ==========================
// SUPPORT STRUCTS
// ==========================
//

#[derive(Serialize)]
pub struct Warning {
    pub code: String,
}

#[derive(Serialize)]
pub struct RelativeTimelock {
    pub enabled: bool,
    pub r#type: String,
    pub value: u16,
}

#[derive(Serialize)]
pub struct PrevoutReport {
    pub value_sats: u64,
    pub script_pubkey_hex: String,
}

//
// ==========================
// BLOCK MODE STRUCTURES
// ==========================
//

use std::collections::HashMap;

#[derive(Serialize)]
pub struct BlockReport {
    pub ok: bool,
    pub mode: String,

    pub block_header: BlockHeader,

    

    pub tx_count: usize,

    pub coinbase: CoinbaseInfo,
    pub block_stats: BlockStats,

    pub transactions: Vec<BlockTxSummary>,
}

#[derive(Serialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u32,
    pub bits: String,
    pub nonce: u32,

    pub block_hash: String,
    pub merkle_root_valid: bool,
}

#[derive(Serialize)]
pub struct CoinbaseInfo {
    pub bip34_height: u32,
    pub coinbase_script_hex: String,
    pub total_output_sats: u64,
}

#[derive(Serialize)]
pub struct BlockStats {
    pub total_fees_sats: u64,
    pub total_weight: u64,
    pub avg_fee_rate_sat_vb: f64,
    pub script_type_summary: HashMap<String, u64>,
}

#[derive(Serialize)]
pub struct BlockTxSummary {
    pub txid: String,
    pub version: u32,
    pub vin: Vec<String>,
    pub vout: Vec<u64>,
}