use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;

/// Tensor data types supported by the AI subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    F32,
    F16,
    Q8_0,  // 8-bit quantized: 32 values per block, 1 f16 scale
    Q4_0,  // 4-bit quantized: 32 values per block, 1 f16 scale
}

impl DType {
    /// Bytes per element (for non-quantized types).
    pub fn element_size(&self) -> usize {
        match self {
            DType::F32 => 4,
            DType::F16 => 2,
            DType::Q8_0 => 1, // approximate, actual is 34 bytes per 32 elements
            DType::Q4_0 => 1, // approximate, actual is 18 bytes per 32 elements
        }
    }

    /// Bytes needed to store `n` elements.
    pub fn storage_size(&self, n: usize) -> usize {
        match self {
            DType::F32 => n * 4,
            DType::F16 => n * 2,
            DType::Q8_0 => {
                // Block size: 32 elements = 32 bytes (int8) + 2 bytes (f16 scale) = 34 bytes
                let blocks = (n + 31) / 32;
                blocks * 34
            }
            DType::Q4_0 => {
                // Block size: 32 elements = 16 bytes (4-bit pairs) + 2 bytes (f16 scale) = 18 bytes
                let blocks = (n + 31) / 32;
                blocks * 18
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DType::F32 => "f32",
            DType::F16 => "f16",
            DType::Q8_0 => "q8_0",
            DType::Q4_0 => "q4_0",
        }
    }
}

/// A multi-dimensional tensor.
/// Data is stored in a contiguous, 64-byte aligned buffer.
pub struct Tensor {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: DType,
    /// Raw data buffer (64-byte aligned for SIMD).
    pub data: Vec<u8>,
}

impl Tensor {
    /// Create a new zero-initialized tensor.
    pub fn zeros(name: &str, shape: &[usize], dtype: DType) -> Self {
        let n_elements: usize = shape.iter().product();
        let storage = dtype.storage_size(n_elements);
        let data = alloc::vec![0u8; storage];
        Self {
            name: String::from(name),
            shape: shape.to_vec(),
            dtype,
            data,
        }
    }

    /// Create an F32 tensor from a slice.
    pub fn from_f32(name: &str, shape: &[usize], values: &[f32]) -> Self {
        let n_elements: usize = shape.iter().product();
        assert_eq!(values.len(), n_elements);
        let mut data = alloc::vec![0u8; n_elements * 4];
        for (i, &v) in values.iter().enumerate() {
            let bytes = v.to_le_bytes();
            data[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }
        Self {
            name: String::from(name),
            shape: shape.to_vec(),
            dtype: DType::F32,
            data,
        }
    }

    /// Total number of elements.
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }

    /// Get data as f32 slice (only valid for F32 tensors).
    pub fn as_f32(&self) -> &[f32] {
        assert_eq!(self.dtype, DType::F32);
        let ptr = self.data.as_ptr() as *const f32;
        let len = self.numel();
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }

    /// Get data as mutable f32 slice.
    pub fn as_f32_mut(&mut self) -> &mut [f32] {
        assert_eq!(self.dtype, DType::F32);
        let ptr = self.data.as_mut_ptr() as *mut f32;
        let len = self.numel();
        unsafe { core::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor(\"{}\", {:?}, {})", self.name, self.shape, self.dtype.name())
    }
}
