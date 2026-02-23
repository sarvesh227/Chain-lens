use std::{
    fs::{self, File},
    io::{Cursor, Read, BufReader},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use hex;

use crate::parser::parse_transaction_bytes;
use crate::merkle::{compute_merkle_root, double_sha256};
use crate::models::*;
use crate::script::classify_script;

//
// =====================================================
// CLI ENTRY
// =====================================================
//

pub fn run_block_mode(
    blk_path: &str,
    _rev_path: &str,
    _xor_path: &str,
) -> Result<(), String> {

    let file = File::open(blk_path)
        .map_err(|_| "Failed to read blk file")?;

    let mut reader = BufReader::new(file);

    // Read magic + block size
    let mut header8 = [0u8; 8];
    reader.read_exact(&mut header8)
        .map_err(|_| "Failed to read block header")?;

    let block_size =
        u32::from_le_bytes(header8[4..8].try_into().unwrap());

    // Read only first block payload
    let mut payload = vec![0u8; block_size as usize];
    reader.read_exact(&mut payload)
        .map_err(|_| "Failed to read block data")?;

    // Rebuild expected layout for analyzer
    let mut full_block = Vec::with_capacity(8 + payload.len());
    full_block.extend_from_slice(&header8);
    full_block.extend_from_slice(&payload);

    let report = analyze_block_bytes(&full_block)?;

    let out_dir = Path::new("out");
    if !out_dir.exists() {
        fs::create_dir(out_dir)
            .map_err(|_| "Failed to create out directory")?;
    }

    let filename = std::path::Path::new(blk_path)
    .file_stem()
    .unwrap()
    .to_string_lossy()
    .to_string();

let output_path = format!("out/{}.json", filename);

fs::write(
    output_path,
    serde_json::to_string_pretty(&report).unwrap(),
)
.map_err(|_| "Failed to write block output")?;
Ok(())
}

//
// =====================================================
// BLOCK ANALYZER (FAST VERSION)
// =====================================================
//

pub fn analyze_block_bytes(blk_bytes: &[u8]) -> Result<BlockReport, String> {

    let mut cursor = Cursor::new(blk_bytes);

    // Skip magic
    cursor.read_u32::<LittleEndian>()
        .map_err(|_| "Failed to read magic")?;

    // Read block size
    let block_size = cursor.read_u32::<LittleEndian>()
        .map_err(|_| "Failed to read block size")?;

    // Read block payload
    let mut block = vec![0u8; block_size as usize];
    cursor.read_exact(&mut block)
        .map_err(|_| "Failed to read block data")?;

    let mut block_cursor = Cursor::new(block.as_slice());

    // ---------------- HEADER ----------------

    let mut header = [0u8; 80];
    block_cursor.read_exact(&mut header)
        .map_err(|_| "Failed to read header")?;

    
    

    let mut header_merkle = [0u8; 32];
    header_merkle.copy_from_slice(&header[36..68]);

    let tx_count = read_varint(&mut block_cursor)?;

    // ---------------- PARSE TXS (OPTIMIZED) ----------------

    let mut parsed_txs = Vec::with_capacity(tx_count as usize);
    let mut txid_hashes = Vec::with_capacity(tx_count as usize);

    for _ in 0..tx_count {

        let tx_start = block_cursor.position() as usize;

        // Remaining slice
        let remaining = &block[tx_start..];

        // Parse once to get consumed size
        let (tx, consumed) =
        parse_transaction_bytes(remaining)?;

        block_cursor.set_position((tx_start + consumed) as u64);

        // Collect txid hashes for merkle
        let mut txid_bytes =
            hex::decode(&tx.txid).map_err(|_| "Invalid txid")?;
        txid_bytes.reverse();

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&txid_bytes);
        txid_hashes.push(arr);

        parsed_txs.push(tx);
    }

    // ---------------- MERKLE VALIDATION ----------------

    let computed_merkle =
        compute_merkle_root(txid_hashes);

    if computed_merkle != header_merkle {
        return Err("Merkle root mismatch".into());
    }
    

   

    // ---------------- BUILD REPORT ----------------

let version =
    u32::from_le_bytes(header[0..4].try_into().unwrap());

let mut prev_hash = header[4..36].to_vec();
prev_hash.reverse();
let prev_block_hash = hex::encode(prev_hash);

let mut merkle_bytes = header[36..68].to_vec();
merkle_bytes.reverse();
let merkle_root = hex::encode(&merkle_bytes);

let timestamp =
    u32::from_le_bytes(header[68..72].try_into().unwrap());

let bits = hex::encode(&header[72..76]);

let nonce =
    u32::from_le_bytes(header[76..80].try_into().unwrap());

let mut header_hash = double_sha256(&header);
header_hash.reverse();
let block_hash = hex::encode(header_hash);

// merkle_root_valid already validated above
let merkle_root_valid = true;

use std::collections::HashMap;

let mut total_fees: u64 = 0;
let mut total_weight: u64 = 0;
let mut script_summary: HashMap<String, u64> = HashMap::new();

for (i, tx) in parsed_txs.iter().enumerate() {

    total_weight += tx.weight;

    if i != 0 {
        total_fees = total_fees
            .checked_add(tx.fee_sats)
            .ok_or("Fee overflow")?;
    }

    for out in &tx.outputs {
        let t = classify_script(&out.script_pubkey_hex).0;
        *script_summary.entry(t).or_insert(0) += 1;
    }
}

let avg_fee_rate =
    if total_weight > 0 {
        total_fees as f64 /
            (total_weight as f64 / 4.0)
    } else { 0.0 };

// Coinbase
let coinbase_tx = &parsed_txs[0];

let coinbase_script_hex =
    coinbase_tx.inputs[0].script_sig_hex.clone();

let bip34_height = 0;

// Build transaction summaries
let tx_summaries =
    parsed_txs.iter().map(|tx| BlockTxSummary {
        txid: tx.txid.clone(),
        version: tx.version,

        vin: tx.inputs
            .iter()
            .map(|input| input.txid.clone())
            .collect(),

        vout: tx.outputs
            .iter()
            .map(|output| output.value_sats)
            .collect(),
    }).collect();

Ok(BlockReport {
    ok: true,
    mode: "block".to_string(),

    block_header: BlockHeader {
        version,
        prev_block_hash,
        merkle_root,
        timestamp,
        bits,
        nonce,
        block_hash,
        merkle_root_valid,
    },

    

    tx_count: tx_count as usize,

    coinbase: CoinbaseInfo {
        bip34_height,
        coinbase_script_hex,
        total_output_sats:
            coinbase_tx.total_output_sats,
    },

    block_stats: BlockStats {
        total_fees_sats: total_fees,
        total_weight,
        avg_fee_rate_sat_vb: avg_fee_rate,
        script_type_summary: script_summary,
    },

    transactions: tx_summaries,
})
}

//
// =====================================================
// VARINT READER
// =====================================================
//

fn read_varint(cursor: &mut Cursor<&[u8]>) -> Result<u64, String> {
    let prefix = cursor.read_u8().map_err(|_| "varint error")?;
    match prefix {
        0xFD => Ok(cursor.read_u16::<LittleEndian>().unwrap() as u64),
        0xFE => Ok(cursor.read_u32::<LittleEndian>().unwrap() as u64),
        0xFF => Ok(cursor.read_u64::<LittleEndian>().unwrap()),
        _ => Ok(prefix as u64),
    }
}