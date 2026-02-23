use sha2::{Digest, Sha256};
use crate::models::{Transaction, TxInput, TxOutput};

//
// =====================================================
// HASH HELPERS
// =====================================================
//

fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second);
    result
}

fn write_varint(buf: &mut Vec<u8>, value: u64) {
    if value < 0xFD {
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        buf.push(0xFD);
        buf.extend(&(value as u16).to_le_bytes());
    } else if value <= 0xFFFF_FFFF {
        buf.push(0xFE);
        buf.extend(&(value as u32).to_le_bytes());
    } else {
        buf.push(0xFF);
        buf.extend(&value.to_le_bytes());
    }
}

//
// =====================================================
// FAST BYTE-BASED PARSER (USED BY BLOCK MODE)
// =====================================================
//

pub fn parse_transaction_bytes(
    bytes: &[u8],
) -> Result<(Transaction, usize), String> {

    let mut pos: usize = 0;
    let size_bytes = bytes.len();
    let mut segwit = false;
    let mut stripped_tx: Vec<u8> = Vec::new();

    fn read_u32_le(bytes: &[u8], pos: &mut usize) -> Result<u32, String> {
        if *pos + 4 > bytes.len() {
            return Err("Unexpected end while reading u32".into());
        }
        let val = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        Ok(val)
    }

    fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, String> {
        if *pos >= bytes.len() {
            return Err("Unexpected end while reading varint".into());
        }

        let prefix = bytes[*pos];
        *pos += 1;

        match prefix {
            0xFD => {
                let val = u16::from_le_bytes(bytes[*pos..*pos + 2].try_into().unwrap());
                *pos += 2;
                Ok(val as u64)
            }
            0xFE => {
                let val = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
                *pos += 4;
                Ok(val as u64)
            }
            0xFF => {
                let val = u64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
                *pos += 8;
                Ok(val)
            }
            _ => Ok(prefix as u64),
        }
    }

    // Version
    let version = read_u32_le(bytes, &mut pos)?;
    stripped_tx.extend(&version.to_le_bytes());

    // SegWit marker
    if pos + 2 <= bytes.len() && bytes[pos] == 0x00 && bytes[pos + 1] == 0x01 {
        segwit = true;
        pos += 2;
    }

    // Inputs
    let input_count = read_varint(bytes, &mut pos)?;
    write_varint(&mut stripped_tx, input_count);

    let mut inputs = Vec::new();

    for _ in 0..input_count {
        let txid_bytes = &bytes[pos..pos + 32];
        stripped_tx.extend(txid_bytes);
        pos += 32;

        let mut reversed = txid_bytes.to_vec();
        reversed.reverse();
        let txid = hex::encode(reversed);

        let vout = read_u32_le(bytes, &mut pos)?;
        stripped_tx.extend(&vout.to_le_bytes());

        let script_len = read_varint(bytes, &mut pos)?;
        write_varint(&mut stripped_tx, script_len);

        let script_bytes = &bytes[pos..pos + script_len as usize];
        let script_sig_hex = hex::encode(script_bytes);
        stripped_tx.extend(script_bytes);
        pos += script_len as usize;

        let sequence = read_u32_le(bytes, &mut pos)?;
        stripped_tx.extend(&sequence.to_le_bytes());

        inputs.push(TxInput {
            txid,
            vout,
            sequence,
            script_sig_hex,
            witness: vec![],
        });
    }

    // Outputs
    let output_count = read_varint(bytes, &mut pos)?;
    write_varint(&mut stripped_tx, output_count);

    let mut outputs = Vec::new();
    let mut total_output_sats = 0u64;

    for _ in 0..output_count {
        let value = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
        total_output_sats += value;

        stripped_tx.extend(&value.to_le_bytes());
        pos += 8;

        let script_len = read_varint(bytes, &mut pos)?;
        write_varint(&mut stripped_tx, script_len);

        let script_bytes = &bytes[pos..pos + script_len as usize];
        let script_pubkey_hex = hex::encode(script_bytes);

        stripped_tx.extend(script_bytes);
        pos += script_len as usize;

        outputs.push(TxOutput {
            value_sats: value,
            script_pubkey_hex,
        });
    }

    // Witness
    if segwit {
        for i in 0..input_count as usize {
            let stack_count = read_varint(bytes, &mut pos)? as usize;
            for _ in 0..stack_count {
                let item_len = read_varint(bytes, &mut pos)? as usize;
                let item = &bytes[pos..pos + item_len];
                inputs[i].witness.push(hex::encode(item));
                pos += item_len;
            }
        }
    }

    // Locktime
    let locktime = read_u32_le(bytes, &mut pos)?;
    stripped_tx.extend(&locktime.to_le_bytes());

    let stripped_size = stripped_tx.len();

    let wtxid = if segwit {
        let mut w = double_sha256(bytes);
        w.reverse();
        Some(hex::encode(w))
    } else {
        None
    };

    let mut t = double_sha256(&stripped_tx);
    t.reverse();
    let txid = hex::encode(t);

    let weight = if segwit {
        (stripped_size as u64 * 4) + (size_bytes - stripped_size) as u64
    } else {
        size_bytes as u64 * 4
    };

    let vbytes = (weight + 3) / 4;

    Ok((
        Transaction {
            version,
            input_count,
            inputs,
            output_count,
            outputs,
            total_input_sats: 0,
            total_output_sats,
            fee_sats: 0,
            locktime,
            segwit,
            txid,
            wtxid,
            size_bytes,
            stripped_size,
            weight,
            vbytes,
            fee_rate_sat_vb: 0.0,
        },
        pos,
    ))
}

//
// =====================================================
// LEGACY HEX PARSER (FOR FIXTURES)
// =====================================================
//

pub fn parse_transaction(raw_tx: &str)
    -> Result<(Transaction, usize), String>
{
    let bytes = hex::decode(raw_tx)
        .map_err(|_| "Invalid hex in raw_tx")?;

    parse_transaction_bytes(&bytes)
}