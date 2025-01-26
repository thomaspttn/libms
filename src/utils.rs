use anyhow::{Context, Result};
use base64::decode;
use flate2::read::ZlibDecoder;
use numpress::low_level::decode_linear;
use std::io::Read;

pub fn decode_binary_data(
    encoded: &str,
    compression: Option<&str>,
    precision: &str,
) -> Result<Vec<f32>> {
    // Step 1: Base64 decode
    let raw_data = decode(encoded).context("Failed to decode Base64")?;

    // Step 2: Decompress (if needed)
    let decompressed_data = match compression {
        Some("zlib") => {
            let mut decoder = ZlibDecoder::new(&raw_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        }
        Some("MS-Numpress linear") => decode_ms_numpress(&raw_data)?,
        _ => raw_data, // No decompression needed
    };

    // Step 3: Convert to floats
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

/// Decodes MS-Numpress linear-compressed data
fn decode_ms_numpress(data: &[u8]) -> Result<Vec<u8>> {
    // Estimate maximum output size: (data.len() - 8) * 2
    let max_decoded_size = (data.len() - 8) * 2;
    let mut decoded_data = Vec::with_capacity(max_decoded_size);

    // Call unsafe decode_linear function
    let decoded_count = unsafe {
        decode_linear(
            data.as_ptr(),
            data.len(),
            decoded_data.as_mut_ptr() as *mut f64,
        )
    }
    .context("Failed to decode MS-Numpress linear")?;

    // Set the actual length of the decoded vector
    unsafe {
        decoded_data.set_len(decoded_count);
    }

    Ok(decoded_data)
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
