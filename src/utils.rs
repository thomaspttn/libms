use anyhow::{Context, Result};
use base64::decode;
use flate2::read::ZlibDecoder;
use msnumpress::{decode_linear, decode_slof};
use std::io::Read;

pub fn decode_binary_data(
    encoded: &str,
    compression: Option<&str>,
    precision: &str,
) -> Result<Vec<f32>> {
    let raw_data = decode(encoded).context("Failed to decode Base64")?;
    let decompressed_data = match compression {
        Some("zlib") => {
            let mut decoder = ZlibDecoder::new(&raw_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        }
        Some("MS-Numpress linear") => {
            decode_linear(&raw_data).context("Failed to decode linear")?
        }
        Some("MS-Numpress slof") => decode_slof(&raw_data).context("Failed to decode slof")?,
        _ => raw_data,
    };

    match precision {
        "32-bit float" => Ok(decompressed_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()),
        "64-bit float" => Ok(decompressed_data
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()) as f32)
            .collect()),
        _ => Err(anyhow::anyhow!("Unknown precision: {}", precision)),
    }
}

pub fn get_attr(e: &quick_xml::events::BytesStart, attr_name: &str) -> Result<String> {
    e.attributes()
        .find_map(|a| {
            let a = a.ok()?;
            if a.key.as_ref() == attr_name.as_bytes() {
                Some(String::from_utf8_lossy(&a.value).into_owned())
            } else {
                None
            }
        })
        .context(format!("Missing attribute: {}", attr_name))
}

pub fn get_attr_optional(e: &quick_xml::events::BytesStart, attr_name: &str) -> Option<String> {
    e.attributes().find_map(|a| {
        let a = a.ok()?;
        if a.key.as_ref() == attr_name.as_bytes() {
            Some(String::from_utf8_lossy(&a.value).into_owned())
        } else {
            None
        }
    })
}
