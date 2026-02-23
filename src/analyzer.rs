use std::collections::HashMap;

use crate::models::{
    Fixture,
    TxReport,
    SegwitSavings,
    VinReport,
    VoutReport,
    Warning,
    PrevoutReport,
    RelativeTimelock,
    BlockReport,
};

use crate::parser::parse_transaction;
use crate::script::{classify_script, parse_op_return};
use crate::block::analyze_block_bytes;

//
// =====================================================
// TRANSACTION ANALYSIS
// =====================================================
//

pub fn analyze(fixture: Fixture) -> Result<TxReport, String> {

    let (tx, _) = parse_transaction(&fixture.raw_tx)?;

    // -------------------------
    // Build prevout map
    // -------------------------

    let mut prevout_map: HashMap<(String, u32), (u64, String)> = HashMap::new();

    for p in &fixture.prevouts {
        let key = (p.txid.clone(), p.vout);

        if prevout_map.contains_key(&key) {
            return Err("Duplicate prevout detected".into());
        }

        prevout_map.insert(key, (p.value_sats, p.script_pubkey_hex.clone()));
    }

    // -------------------------
    // INPUT SUM
    // -------------------------

    let mut total_input_sats: u64 = 0;

    for input in &tx.inputs {
        let key = (input.txid.clone(), input.vout);

        let (value, _) = prevout_map
            .get(&key)
            .ok_or_else(|| format!(
                "Missing prevout for input {}:{}",
                input.txid, input.vout
            ))?;

        total_input_sats = total_input_sats
            .checked_add(*value)
            .ok_or("Input overflow")?;
    }

    if total_input_sats < tx.total_output_sats {
        return Err("Negative fee detected".into());
    }

    let fee_sats = total_input_sats - tx.total_output_sats;

    let fee_rate_sat_vb =
        ((fee_sats as f64 / tx.vbytes as f64) * 100.0).round() / 100.0;

    // -------------------------
    // RBF
    // -------------------------

    let rbf_signaling =
        tx.inputs.iter().any(|i| i.sequence < 0xFFFFFFFE);

    // -------------------------
    // LOCKTIME
    // -------------------------

    let locktime_type = if tx.locktime == 0 {
        "none".to_string()
    } else if tx.locktime < 500_000_000 {
        "block_height".to_string()
    } else {
        "timestamp".to_string()
    };

    // -------------------------
    // SEGWIT SAVINGS
    // -------------------------

    let weight_if_legacy = tx.size_bytes as u64 * 4;

    let savings_pct = if tx.segwit && weight_if_legacy >= tx.weight {
        ((weight_if_legacy - tx.weight) as f64
            / weight_if_legacy as f64) * 100.0
    } else {
        0.0
    };

    let witness_bytes = if tx.segwit {
        tx.size_bytes - tx.stripped_size
    } else {
        0
    };

    let non_witness_bytes = tx.stripped_size;

    // -------------------------
    // WARNINGS
    // -------------------------

    let mut warnings: Vec<Warning> = Vec::new();

    if rbf_signaling {
        warnings.push(Warning {
            code: "RBF_SIGNALING".to_string(),
        });
    }

    // -------------------------
    // VIN
    // -------------------------

    let vin: Vec<VinReport> = tx.inputs.iter().map(|input| {

    let (value, prev_script) =
        prevout_map.get(&(input.txid.clone(), input.vout)).unwrap();

    let mut stype;
    let mut address = None;

    // 1️⃣ If witness exists → native segwit
    if !input.witness.is_empty() {

        // Check if prevout was wrapped in P2SH
        if prev_script.starts_with("a914") {
            // Nested segwit
            if input.witness.len() == 2 {
                stype = "p2sh-p2wpkh".to_string();
            } else {
                stype = "p2sh-p2wsh".to_string();
            }
        } else {
            // Native segwit
            let (inner, addr) = classify_script(prev_script);
            stype = inner;
            address = addr;
        }

    } else {

        // 2️⃣ Legacy spend → classify using scriptSig itself
        let (inner, addr) = classify_script(&input.script_sig_hex);
        stype = inner;
        address = addr;
    }

    VinReport {
        txid: input.txid.clone(),
        vout: input.vout,
        sequence: input.sequence,
        script_sig_hex: input.script_sig_hex.clone(),
        script_asm: "".into(),
        witness: input.witness.clone(),
        script_type: stype,
        address,
        prevout: PrevoutReport {
            value_sats: *value,
            script_pubkey_hex: prev_script.clone(),
        },
        relative_timelock: RelativeTimelock {
            enabled: false,
            r#type: "blocks".into(),
            value: 0,
        },
    }

}).collect();

    // -------------------------
    // VOUT
    // -------------------------

    let vout: Vec<VoutReport> = tx.outputs
        .iter()
        .enumerate()
        .map(|(i, output)| {

            let (stype, address) =
                classify_script(&output.script_pubkey_hex);

            let (op_hex, op_utf8, op_proto) =
                if stype == "op_return" {
                    parse_op_return(&output.script_pubkey_hex)
                } else {
                    (String::new(), None, String::new())
                };

            if output.value_sats < 546 && stype != "op_return" {
                warnings.push(Warning { code: "DUST_OUTPUT".into() });
            }

            VoutReport {
                n: i as u32,
                value_sats: output.value_sats,
                script_pubkey_hex: output.script_pubkey_hex.clone(),
                script_asm: "".into(),
                script_type: stype.clone(),
                address,
                op_return_data_hex:
                    if stype == "op_return" { Some(op_hex) } else { None },
                op_return_data_utf8: op_utf8,
                op_return_protocol:
                    if stype == "op_return" { Some(op_proto) } else { None },
            }
        })
        .collect();

    Ok(TxReport {
        ok: true,
        network: fixture.network,
        segwit: tx.segwit,
        txid: tx.txid,
        wtxid: tx.wtxid,
        version: tx.version,
        locktime: tx.locktime,
        size_bytes: tx.size_bytes,
        weight: tx.weight,
        vbytes: tx.vbytes,
        total_input_sats,
        total_output_sats: tx.total_output_sats,
        fee_sats,
        fee_rate_sat_vb,
        rbf_signaling,
        locktime_type,
        locktime_value: tx.locktime,
        segwit_savings: if tx.segwit {
            Some(SegwitSavings {
                witness_bytes,
                non_witness_bytes,
                total_bytes: tx.size_bytes,
                weight_actual: tx.weight,
                weight_if_legacy,
                savings_pct,
            })
        } else {
            None
        },
        vin,
        vout,
        warnings,
    })
}

//
// =====================================================
// BLOCK ANALYZER
// =====================================================
//

pub fn analyze_block(bytes: &[u8]) -> Result<BlockReport, String> {
    analyze_block_bytes(bytes)
}