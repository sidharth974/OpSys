use alloc::string::String;
use alloc::vec::Vec;
use super::tensor::DType;

/// GGUF magic number.
pub const GGUF_MAGIC: u32 = 0x46475547; // "GGUF"

/// GGUF tensor data types (subset).
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum GgufDType {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    Q8_0 = 8,
}

impl GgufDType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(GgufDType::F32),
            1 => Some(GgufDType::F16),
            2 => Some(GgufDType::Q4_0),
            3 => Some(GgufDType::Q4_1),
            8 => Some(GgufDType::Q8_0),
            _ => None,
        }
    }

    pub fn to_dtype(&self) -> DType {
        match self {
            GgufDType::F32 => DType::F32,
            GgufDType::F16 => DType::F16,
            GgufDType::Q4_0 => DType::Q4_0,
            GgufDType::Q4_1 => DType::Q4_0, // treat Q4_1 as Q4_0 for now
            GgufDType::Q8_0 => DType::Q8_0,
        }
    }
}

/// Parsed GGUF file header info.
#[derive(Debug)]
pub struct GgufHeader {
    pub version: u32,
    pub tensor_count: u64,
    pub metadata_kv_count: u64,
}

/// Metadata about a tensor in a GGUF file.
#[derive(Debug)]
pub struct GgufTensorInfo {
    pub name: String,
    pub dimensions: Vec<u64>,
    pub dtype: GgufDType,
    pub offset: u64,
}

/// Parse a GGUF file header from raw bytes.
/// Returns the header and tensor info, or None if invalid.
pub fn parse_header(data: &[u8]) -> Option<(GgufHeader, Vec<GgufTensorInfo>)> {
    if data.len() < 24 {
        return None;
    }

    let magic = u32::from_le_bytes(data[0..4].try_into().ok()?);
    if magic != GGUF_MAGIC {
        return None;
    }

    let version = u32::from_le_bytes(data[4..8].try_into().ok()?);
    let tensor_count = u64::from_le_bytes(data[8..16].try_into().ok()?);
    let metadata_kv_count = u64::from_le_bytes(data[16..24].try_into().ok()?);

    let header = GgufHeader {
        version,
        tensor_count,
        metadata_kv_count,
    };

    // In a full implementation, we'd parse metadata KV pairs and tensor info here.
    // For now, return the header with an empty tensor list.
    Some((header, Vec::new()))
}

/// Format model info for display.
pub fn model_summary(header: &GgufHeader) -> String {
    alloc::format!(
        "GGUF v{}, {} tensors, {} metadata entries",
        header.version, header.tensor_count, header.metadata_kv_count
    )
}
