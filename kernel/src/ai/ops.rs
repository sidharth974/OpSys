use super::tensor::Tensor;

/// Vector dot product (f32).
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut sum = 0.0f32;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// Matrix multiplication: C = A @ B
/// A is [M x K], B is [K x N], C is [M x N].
/// All tensors are F32, row-major.
pub fn matmul(a: &Tensor, b: &Tensor, c: &mut Tensor) {
    assert_eq!(a.shape.len(), 2);
    assert_eq!(b.shape.len(), 2);
    let m = a.shape[0];
    let k = a.shape[1];
    let n = b.shape[1];
    assert_eq!(b.shape[0], k);
    assert_eq!(c.shape, &[m, n]);

    let a_data = a.as_f32();
    let b_data = b.as_f32();
    let c_data = c.as_f32_mut();

    // Naive matmul with loop tiling for cache friendliness.
    // Tile size chosen for L1 cache (~32 KiB).
    const TILE: usize = 32;

    // Zero output
    for v in c_data.iter_mut() {
        *v = 0.0;
    }

    // Tiled matmul: iterate over tiles of K dimension
    let mut kk = 0;
    while kk < k {
        let k_end = (kk + TILE).min(k);
        for i in 0..m {
            for j in 0..n {
                let mut sum = c_data[i * n + j];
                for p in kk..k_end {
                    sum += a_data[i * k + p] * b_data[p * n + j];
                }
                c_data[i * n + j] = sum;
            }
        }
        kk += TILE;
    }
}

/// Element-wise ReLU activation: max(0, x).
pub fn relu(tensor: &mut Tensor) {
    let data = tensor.as_f32_mut();
    for v in data.iter_mut() {
        if *v < 0.0 {
            *v = 0.0;
        }
    }
}

/// Softmax over the last dimension.
/// For a 1D tensor, this is just softmax over all elements.
pub fn softmax(tensor: &mut Tensor) {
    let data = tensor.as_f32_mut();
    let n = data.len();
    if n == 0 {
        return;
    }

    // Find max for numerical stability
    let mut max_val = data[0];
    for &v in data.iter() {
        if v > max_val {
            max_val = v;
        }
    }

    // exp(x - max) and sum
    let mut sum = 0.0f32;
    for v in data.iter_mut() {
        *v = libm::expf(*v - max_val);
        sum += *v;
    }

    // Normalize
    if sum > 0.0 {
        for v in data.iter_mut() {
            *v /= sum;
        }
    }
}

/// Simple benchmark: multiply two NxN matrices and return elapsed ticks.
pub fn bench_matmul(n: usize) -> (u64, f64) {
    use crate::arch::x86_64::idt::TICKS;
    use core::sync::atomic::Ordering;

    // Create random-ish matrices (use deterministic pattern)
    let mut a_vals = alloc::vec![0.0f32; n * n];
    let mut b_vals = alloc::vec![0.0f32; n * n];
    for i in 0..n * n {
        a_vals[i] = ((i * 7 + 13) % 100) as f32 / 100.0;
        b_vals[i] = ((i * 11 + 7) % 100) as f32 / 100.0;
    }

    let a = Tensor::from_f32("bench_a", &[n, n], &a_vals);
    let b = Tensor::from_f32("bench_b", &[n, n], &b_vals);
    let mut c = Tensor::zeros("bench_c", &[n, n], super::tensor::DType::F32);

    let start = TICKS.load(Ordering::Relaxed);
    matmul(&a, &b, &mut c);
    let end = TICKS.load(Ordering::Relaxed);

    let elapsed = end - start;
    // FLOPs for matrix multiply: 2 * N^3
    let flops = 2.0 * (n as f64) * (n as f64) * (n as f64);
    // Ticks are ~1ms each (PIT default ~1000 Hz, actually ~18.2 Hz for PIT)
    // With PIT at 18.2 Hz, each tick ≈ 55ms
    let elapsed_ms = elapsed as f64 * 55.0; // approximate
    let gflops = if elapsed_ms > 0.0 {
        flops / (elapsed_ms / 1000.0) / 1e9
    } else {
        0.0
    };

    (elapsed, gflops)
}
