use super::{GraphFileParseError, QP_FILE_VERSION};
use serde::{Deserialize, Serialize};

pub(crate) const HEADER_LEN: usize = 8;

pub(crate) fn encode_with_header<T: Serialize>(
    magic: [u8; 4],
    version: u32,
    payload: &T,
) -> Result<Vec<u8>, postcard::Error> {
    let body = postcard::to_allocvec(payload)?;
    let mut out = Vec::with_capacity(HEADER_LEN + body.len());
    out.extend_from_slice(&magic);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

pub(crate) fn decode_with_header<T: for<'de> Deserialize<'de>>(
    data: &[u8],
    expected_magic: [u8; 4],
) -> Result<T, GraphFileParseError> {
    if data.len() < HEADER_LEN {
        return Err(GraphFileParseError::TooShort);
    }
    let mut found = [0u8; 4];
    found.copy_from_slice(&data[0..4]);
    if found != expected_magic {
        return Err(GraphFileParseError::InvalidMagic {
            expected: expected_magic,
            found,
        });
    }
    let version = u32::from_le_bytes(data[4..8].try_into().expect("header"));
    if version > QP_FILE_VERSION {
        return Err(GraphFileParseError::UnsupportedVersion {
            found: version,
            max: QP_FILE_VERSION,
        });
    }
    Ok(postcard::from_bytes(&data[HEADER_LEN..])?)
}
