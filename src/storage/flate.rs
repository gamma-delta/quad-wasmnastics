use anyhow::Context;
use base64::URL_SAFE;
use flate2::{
    read::{GzDecoder, GzEncoder},
    Compression,
};

use std::io::{Cursor, Read};

/// Gzip some binary data.
///
/// This uses Best-level compression (because there's only so much space in Localstorage.)
pub fn zip<T: AsRef<[u8]>>(data: T) -> anyhow::Result<Vec<u8>> {
    let mut gz = GzEncoder::new(Cursor::new(data.as_ref()), Compression::best());

    let mut out = Vec::new();
    gz.read_to_end(&mut out).context("When gzipping")?;
    Ok(out)
}

/// Gzip some binary data, then return it as a base64 string.
///
/// This uses `URL_SAFE` base64.
pub fn zip64<T: AsRef<[u8]>>(data: T) -> anyhow::Result<String> {
    Ok(base64::encode_config(zip(data.as_ref())?, URL_SAFE))
}

/// Unzip some binary back into the original bytes.
pub fn unzip<T: AsRef<[u8]>>(zipped: T) -> anyhow::Result<Vec<u8>> {
    let mut gz = GzDecoder::new(Cursor::new(zipped.as_ref()));
    let mut out = Vec::new();
    gz.read_to_end(&mut out).context("When un-gzipping")?;
    Ok(out)
}

/// Decode a base64 string, then un-gzip it back into the original bytes.
///
/// This expects `URL_SAFE` base64.
pub fn unzip64<T: AsRef<str>>(encoded: T) -> anyhow::Result<Vec<u8>> {
    unzip(base64::decode_config(encoded.as_ref(), URL_SAFE).context("When decoding base64")?)
}
