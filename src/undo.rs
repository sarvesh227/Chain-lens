use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct UndoPrevout {
    pub value_sats: u64,
    pub script_pubkey_hex: String,
}

pub fn parse_undo_block(data: &[u8]) -> Result<Vec<Vec<UndoPrevout>>, String> {
    let mut cursor = Cursor::new(data);

    let tx_count = read_varint(&mut cursor)?;

    let mut result = Vec::new();

    for _ in 0..tx_count {
        let input_count = read_varint(&mut cursor)?;
        let mut tx_prevouts = Vec::new();

        for _ in 0..input_count {
            let value = cursor.read_u64::<LittleEndian>()
                .map_err(|_| "Failed to read prevout value")?;

            let script_len = read_varint(&mut cursor)?;
            let mut script = vec![0u8; script_len as usize];

            cursor.read_exact(&mut script)
                .map_err(|_| "Failed to read prevout script")?;

            tx_prevouts.push(UndoPrevout {
                value_sats: value,
                script_pubkey_hex: hex::encode(script),
            });
        }

        result.push(tx_prevouts);
    }

    Ok(result)
}

fn read_varint(cursor: &mut Cursor<&[u8]>) -> Result<u64, String> {
    let prefix = cursor.read_u8().map_err(|_| "varint error")?;

    match prefix {
        0xFD => Ok(cursor.read_u16::<LittleEndian>().unwrap() as u64),
        0xFE => Ok(cursor.read_u32::<LittleEndian>().unwrap() as u64),
        0xFF => Ok(cursor.read_u64::<LittleEndian>().unwrap()),
        _ => Ok(prefix as u64),
    }
}