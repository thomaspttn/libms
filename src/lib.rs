pub mod models;
pub mod utils;

use anyhow::Result;
use models::{BinaryDataArray, CvParam, Run, Scan, ScanList, ScanWindow, Spectrum};
use quick_xml::events::Event;
use quick_xml::Reader;
use utils::{decode_binary_data, get_attr, get_attr_optional};

/// Parses an mzML string into a Run object
pub fn parse_mzml(xml_data: &str) -> Result<Run> {
    let mut reader = Reader::from_str(xml_data);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut current_run = None;
    let mut spectra = Vec::new();
    let mut current_spectrum = None;
    let mut current_cv_params = Vec::new();
    let mut current_scan_list = None;
    let mut current_scan = None;
    let mut current_scan_window = None;
    let mut current_binary_data_array = None;

    while let Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Start(ref e) => match e.name().as_ref() {
                b"run" => {
                    current_run = Some(Run {
                        id: get_attr(e, "id")?,
                        start_time: get_attr(e, "startTimeStamp")?,
                        spectra: Vec::new(),
                    });
                }
                b"spectrum" => {
                    current_spectrum = Some(Spectrum {
                        id: get_attr(e, "id")?,
                        index: get_attr(e, "index")?.parse()?,
                        default_array_length: get_attr(e, "defaultArrayLength")?.parse()?,
                        cv_params: Vec::new(),
                        scan_list: None,
                        binary_data_arrays: Vec::new(),
                    });
                }
                b"binaryDataArray" => {
                    current_binary_data_array = Some(BinaryDataArray {
                        encoded_length: get_attr(e, "encodedLength")?.parse()?,
                        cv_params: Vec::new(),
                        decoded_data: None,
                    });
                }
                b"binary" => {
                    if let Some(array) = current_binary_data_array.as_mut() {
                        let encoded_data = reader.read_text(e.name(), &mut Vec::new())?;
                        let compression = array.cv_params.iter().find_map(|p| {
                            if p.name.contains("compression") {
                                p.name.clone().into()
                            } else {
                                None
                            }
                        });

                        let precision = array.cv_params.iter().find_map(|p| {
                            if p.name.contains("32-bit") || p.name.contains("64-bit") {
                                p.name.clone().into()
                            } else {
                                None
                            }
                        });

                        array.decoded_data = Some(decode_binary_data(
                            &encoded_data,
                            compression.as_deref(),
                            precision.as_deref().unwrap_or("32-bit float"),
                        )?);
                    }
                }
                _ => {}
            },
            Event::End(ref e) => match e.name().as_ref() {
                b"spectrum" => {
                    if let Some(mut spectrum) = current_spectrum.take() {
                        spectrum.cv_params = current_cv_params.clone();
                        spectra.push(spectrum);
                        current_cv_params.clear();
                    }
                }
                b"binaryDataArray" => {
                    if let Some(array) = current_binary_data_array.take() {
                        if let Some(spectrum) = current_spectrum.as_mut() {
                            spectrum.binary_data_arrays.push(array);
                        }
                    }
                }
                b"run" => {
                    if let Some(run) = current_run.as_mut() {
                        run.spectra = spectra.clone();
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    current_run.ok_or_else(|| anyhow::anyhow!("No <run> element found in the mzML file"))
}
