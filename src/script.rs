use sha2::{Digest, Sha256};
use bs58;
use bech32::{ToBase32, Variant, u5};

//
// ----------------------------
// Script Classification
// ----------------------------
//

pub fn classify_script(script_hex: &str) -> (String, Option<String>) {

    // ---------------- P2PKH ----------------
    if script_hex.starts_with("76a914")
        && script_hex.ends_with("88ac")
        && script_hex.len() == 50
    {
        let hash160 = &script_hex[6..46];
        let address = base58check("00", hash160);
        return ("p2pkh".into(), Some(address));
    }

    // ---------------- P2SH ----------------
    if script_hex.starts_with("a914")
        && script_hex.ends_with("87")
        && script_hex.len() == 46
    {
        let hash160 = &script_hex[4..44];
        let address = base58check("05", hash160);
        return ("p2sh".into(), Some(address));
    }

    // ---------------- P2WPKH ----------------
    if script_hex.starts_with("0014") && script_hex.len() == 44 {
        let program = &script_hex[4..];
        let address = bech32_address("bc", 0, program);
        return ("p2wpkh".into(), Some(address));
    }

    // ---------------- P2WSH ----------------
    if script_hex.starts_with("0020") && script_hex.len() == 68 {
        let program = &script_hex[4..];
        let address = bech32_address("bc", 0, program);
        return ("p2wsh".into(), Some(address));
    }

    // ---------------- P2TR (Taproot) ----------------
    if script_hex.starts_with("5120") && script_hex.len() == 68 {
        let program = &script_hex[4..];
        let address = bech32_address("bc", 1, program);
        return ("p2tr".into(), Some(address));
    }

    // ---------------- OP_RETURN ----------------
    if script_hex.starts_with("6a") {
        return ("op_return".into(), None);
    }

    ("unknown".into(), None)
}

//
// ----------------------------
// OP_RETURN Parsing
// ----------------------------
//

pub fn parse_op_return(script_hex: &str) -> (String, Option<String>, String) {
    let bytes = match hex::decode(script_hex) {
        Ok(b) => b,
        Err(_) => return ("".into(), None, "unknown".into()),
    };

    if bytes.len() <= 1 {
        return ("".into(), None, "unknown".into());
    }

    let mut pos = 1; // skip OP_RETURN (0x6a)
    let mut data = Vec::new();

    while pos < bytes.len() {
        let opcode = bytes[pos];
        pos += 1;

        let push_len = match opcode {
            0x01..=0x4b => opcode as usize,

            0x4c => {
                if pos >= bytes.len() { break; }
                let len = bytes[pos] as usize;
                pos += 1;
                len
            }

            0x4d => {
                if pos + 1 >= bytes.len() { break; }
                let len = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
                pos += 2;
                len
            }

            0x4e => {
                if pos + 3 >= bytes.len() { break; }
                let len = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                pos += 4;
                len
            }

            _ => break,
        };

        if pos + push_len > bytes.len() {
            break;
        }

        data.extend(&bytes[pos..pos + push_len]);
        pos += push_len;
    }

    let hex_data = hex::encode(&data);

    let utf8 = match String::from_utf8(data.clone()) {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    let protocol = if hex_data.starts_with("6f6d6e69") {
        "omni"
    } else if hex_data.starts_with("0109f91102") {
        "opentimestamps"
    } else {
        "unknown"
    };

    (hex_data, utf8, protocol.into())
}

//
// ----------------------------
// Base58Check (P2PKH / P2SH)
// ----------------------------
//

fn base58check(version_hex: &str, hash160_hex: &str) -> String {
    let mut payload = hex::decode(version_hex).unwrap();
    payload.extend(hex::decode(hash160_hex).unwrap());

    let checksum = Sha256::digest(&Sha256::digest(&payload));
    payload.extend(&checksum[0..4]);

    bs58::encode(payload).into_string()
}

//
// ----------------------------
// Bech32 Address (SegWit)
// ----------------------------
//

fn bech32_address(hrp: &str, version: u8, program_hex: &str) -> String {
    let program = hex::decode(program_hex).unwrap();

    let mut data = vec![u5::try_from_u8(version).unwrap()];
    data.extend(program.to_base32());

    bech32::encode(hrp, data, Variant::Bech32).unwrap()
}