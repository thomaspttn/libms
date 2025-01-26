#[derive(Debug, Clone)]
pub struct Run {
    pub id: String,
    pub start_time: String,
    pub spectra: Vec<Spectrum>,
}

#[derive(Debug, Clone)]
pub struct Spectrum {
    pub id: String,
    pub index: usize,
    pub default_array_length: usize,
    pub cv_params: Vec<CvParam>,
    pub scan_list: Option<ScanList>,
    pub binary_data_arrays: Vec<BinaryDataArray>,
}

#[derive(Debug, Clone)]
pub struct CvParam {
    pub cv_ref: String,
    pub accession: String,
    pub name: String,
    pub value: Option<String>,
    pub unit_name: Option<String>,
    pub unit_accession: Option<String>,
    pub unit_cv_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanList {
    pub count: usize,
    pub cv_params: Vec<CvParam>,
    pub scans: Vec<Scan>,
}

#[derive(Debug, Clone)]
pub struct Scan {
    pub cv_params: Vec<CvParam>,
    pub scan_windows: Vec<ScanWindow>,
}

#[derive(Debug, Clone)]
pub struct ScanWindow {
    pub cv_params: Vec<CvParam>,
}

#[derive(Debug, Clone)]
pub struct BinaryDataArray {
    pub encoded_length: usize,
    pub cv_params: Vec<CvParam>,
    pub decoded_data: Option<Vec<f32>>,
}
