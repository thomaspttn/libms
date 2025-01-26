use anyhow::Result;
use libms::{parse_mzml, Run};

fn main() -> Result<()> {
    // Example: Load mzML data from a string (you can replace this with file reading)
    let mzml_data = include_str!("../sample01.mzML");
    let run: Run = parse_mzml(mzml_data)?;

    // Output parsed data
    println!("Successfully parsed mzML file!");
    println!("Run ID: {}", run.id);
    println!("Start Time: {}", run.start_time);
    println!("\nFound {} spectra", run.spectra.len());

    for (i, spectrum) in run.spectra.iter().take(2).enumerate() {
        println!("\nSpectrum #{}", i + 1);
        println!("  ID: {}", spectrum.id);
        println!("  Index: {}", spectrum.index);
        println!("  Default Array Length: {}", spectrum.default_array_length);
    }

    Ok(())
}
