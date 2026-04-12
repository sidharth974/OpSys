use alloc::string::String;
use alloc::vec::Vec;
use super::tensor::{Tensor, DType};
use super::ops;

/// A simple neural network layer.
#[derive(Debug)]
pub enum Layer {
    Linear { weights: usize, bias: usize, out_features: usize },
    ReLU,
    Softmax,
}

/// A simple inference session for demo purposes.
/// Holds tensors and a layer graph.
pub struct InferenceSession {
    pub name: String,
    pub tensors: Vec<Tensor>,
    pub layers: Vec<Layer>,
}

impl InferenceSession {
    /// Create a demo MLP (Multi-Layer Perceptron) for testing.
    /// Input: [1, in_features] -> Hidden -> ReLU -> Output -> Softmax
    pub fn demo_mlp(in_features: usize, hidden: usize, out_features: usize) -> Self {
        let mut tensors = Vec::new();

        // Layer 1: Linear (in -> hidden)
        let mut w1_vals = alloc::vec![0.0f32; in_features * hidden];
        for i in 0..w1_vals.len() {
            // Xavier-like initialization
            w1_vals[i] = ((i * 7 + 3) % 200) as f32 / 200.0 - 0.5;
        }
        tensors.push(Tensor::from_f32("w1", &[in_features, hidden], &w1_vals));

        let b1_vals = alloc::vec![0.01f32; hidden];
        tensors.push(Tensor::from_f32("b1", &[1, hidden], &b1_vals));

        // Layer 2: Linear (hidden -> out)
        let mut w2_vals = alloc::vec![0.0f32; hidden * out_features];
        for i in 0..w2_vals.len() {
            w2_vals[i] = ((i * 13 + 5) % 200) as f32 / 200.0 - 0.5;
        }
        tensors.push(Tensor::from_f32("w2", &[hidden, out_features], &w2_vals));

        let b2_vals = alloc::vec![0.01f32; out_features];
        tensors.push(Tensor::from_f32("b2", &[1, out_features], &b2_vals));

        let layers = alloc::vec![
            Layer::Linear { weights: 0, bias: 1, out_features: hidden },
            Layer::ReLU,
            Layer::Linear { weights: 2, bias: 3, out_features },
            Layer::Softmax,
        ];

        Self {
            name: String::from("demo-mlp"),
            tensors,
            layers,
        }
    }

    /// Run forward pass on input tensor, return output.
    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut current = Tensor::from_f32(
            "input",
            &input.shape,
            input.as_f32(),
        );

        for layer in &self.layers {
            current = match layer {
                Layer::Linear { weights, bias, out_features } => {
                    let w = &self.tensors[*weights];
                    let b = &self.tensors[*bias];
                    let m = current.shape[0];
                    let mut output = Tensor::zeros("linear_out", &[m, *out_features], DType::F32);
                    ops::matmul(&current, w, &mut output);

                    // Add bias
                    let out_data = output.as_f32_mut();
                    let b_data = b.as_f32();
                    for i in 0..m {
                        for j in 0..*out_features {
                            out_data[i * out_features + j] += b_data[j];
                        }
                    }
                    output
                }
                Layer::ReLU => {
                    ops::relu(&mut current);
                    current
                }
                Layer::Softmax => {
                    ops::softmax(&mut current);
                    current
                }
            };
        }

        current
    }

    /// Get total parameter count.
    pub fn param_count(&self) -> usize {
        self.tensors.iter().map(|t| t.numel()).sum()
    }

    /// Get total memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.tensors.iter().map(|t| t.data.len()).sum()
    }
}
