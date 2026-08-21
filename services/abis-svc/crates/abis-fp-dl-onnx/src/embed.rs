//! DeepPrint-style ONNX embedding.
//!
//! Reference: Engelsma, Cao, Jain, "Learning a Fixed-Length Fingerprint Representation,"
//! IEEE Trans. Pattern Anal. Mach. Intell., 2021.
//!
//! Model loading: set the environment variable `EEL_FP_DL_ONNX_PATH` to the path of
//! the ONNX model file.  If the variable is unset or the file is absent, the embedder
//! falls back to a deterministic SHA-256-based pseudo-embedding and emits a WARN log.

use crate::{DlError, GrayImage, EMBED_DIM};
use sha2::{Digest, Sha256};

/// A 384-dimensional unit-normalised embedding vector.
pub type Embedding = [f32; EMBED_DIM];

/// Target image size for the DeepPrint model (224x224, grayscale).
const MODEL_SIZE: u32 = 224;

/// Resize a GrayImage to `target x target` using bilinear interpolation.
fn resize_bilinear(image: &GrayImage, target: u32) -> Vec<u8> {
    let (sw, sh) = (image.width as f32, image.height as f32);
    let tw = target as f32;
    let th = target as f32;
    let mut out = vec![0u8; (target * target) as usize];
    for dy in 0..target {
        for dx in 0..target {
            let sx = (dx as f32 + 0.5) / tw * sw - 0.5;
            let sy = (dy as f32 + 0.5) / th * sh - 0.5;
            let x0 = sx.floor().max(0.0) as u32;
            let y0 = sy.floor().max(0.0) as u32;
            let x1 = (x0 + 1).min(image.width - 1);
            let y1 = (y0 + 1).min(image.height - 1);
            let wx = sx - sx.floor();
            let wy = sy - sy.floor();
            let p00 = image.pixels[(y0 * image.width + x0) as usize] as f32;
            let p10 = image.pixels[(y0 * image.width + x1) as usize] as f32;
            let p01 = image.pixels[(y1 * image.width + x0) as usize] as f32;
            let p11 = image.pixels[(y1 * image.width + x1) as usize] as f32;
            let v = p00 * (1.0 - wx) * (1.0 - wy)
                + p10 * wx * (1.0 - wy)
                + p01 * (1.0 - wx) * wy
                + p11 * wx * wy;
            out[(dy * target + dx) as usize] = v.round() as u8;
        }
    }
    out
}

/// Normalise a pixel buffer to [-1, 1] (mean 0.5, std 0.5 assumed).
fn normalise(pixels: &[u8]) -> Vec<f32> {
    pixels
        .iter()
        .map(|&p| (p as f32 / 255.0 - 0.5) / 0.5)
        .collect()
}

/// Deterministic pseudo-embedding derived from SHA-256 of the image pixels.
/// Used when the ONNX model is unavailable.
fn hash_embedding(pixels: &[u8]) -> Embedding {
    let mut hasher = Sha256::new();
    hasher.update(pixels);
    // Run the hasher 6 times (6 * 32 bytes = 192 bytes -> 384 f32 values at 0.5 bytes each;
    // we expand each byte to two f32 values in [-1, 1]).
    let mut raw = Vec::with_capacity(EMBED_DIM);
    let mut seed = hasher.finalize().to_vec();
    while raw.len() < EMBED_DIM {
        let mut h2 = Sha256::new();
        h2.update(&seed);
        seed = h2.finalize().to_vec();
        for byte in &seed {
            if raw.len() >= EMBED_DIM {
                break;
            }
            // Map [0,255] -> [-1, 1].
            raw.push(*byte as f32 / 127.5 - 1.0);
        }
    }
    // L2-normalise.
    let norm: f32 = raw.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-8);
    let mut emb = [0.0f32; EMBED_DIM];
    for (i, v) in raw.iter().take(EMBED_DIM).enumerate() {
        emb[i] = v / norm;
    }
    emb
}

/// DeepPrint-style embedder.
pub struct DeepPrintEmbedder {
    model_path: Option<std::path::PathBuf>,
}

impl DeepPrintEmbedder {
    /// Create an embedder that loads its model from `EEL_FP_DL_ONNX_PATH`.
    pub fn from_env() -> Self {
        let model_path = std::env::var("EEL_FP_DL_ONNX_PATH")
            .ok()
            .map(std::path::PathBuf::from);
        Self { model_path }
    }

    /// Create an embedder with an explicit model path.
    pub fn from_path(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            model_path: Some(path.into()),
        }
    }

    /// Preprocess an image: resize to 224x224, normalise to [-1, 1].
    pub fn preprocess(image: &GrayImage) -> Vec<f32> {
        let resized = resize_bilinear(image, MODEL_SIZE);
        normalise(&resized)
    }

    /// Compute a 384-d embedding for `image`.
    ///
    /// If the ONNX model is configured and present, runs it via ONNX Runtime.
    /// Otherwise falls back to a deterministic hash-based embedding and emits WARN.
    pub fn embed(&self, image: &GrayImage) -> Result<Embedding, DlError> {
        let model_present = self
            .model_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false);

        if !model_present {
            tracing::warn!(
                model_path = ?self.model_path,
                "ONNX model absent; using deterministic hash-based pseudo-embedding"
            );
            let resized = resize_bilinear(image, MODEL_SIZE);
            return Ok(hash_embedding(&resized));
        }

        // ONNX Runtime path (requires the `onnx` feature).
        #[cfg(feature = "onnx")]
        {
            self.embed_onnx(image)
        }
        #[cfg(not(feature = "onnx"))]
        {
            tracing::warn!("onnx feature disabled; using hash embedding");
            let resized = resize_bilinear(image, MODEL_SIZE);
            Ok(hash_embedding(&resized))
        }
    }

    #[cfg(feature = "onnx")]
    fn embed_onnx(&self, image: &GrayImage) -> Result<Embedding, DlError> {
        use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};

        let path = self
            .model_path
            .as_ref()
            .ok_or(DlError::ModelNotConfigured)?;
        let env = Environment::builder()
            .with_name("eel-fp-dl")
            .build()
            .map_err(|e| DlError::SessionError(e.to_string()))?;
        let session = SessionBuilder::new(&env)
            .map_err(|e| DlError::SessionError(e.to_string()))?
            .with_model_from_file(path)
            .map_err(|e| DlError::SessionError(e.to_string()))?;

        let input = Self::preprocess(image);
        // Shape: [1, 1, 224, 224] (batch=1, channels=1, H, W).
        let array = ndarray::Array4::from_shape_vec(
            (1, 1, MODEL_SIZE as usize, MODEL_SIZE as usize),
            input,
        )
        .map_err(|e| DlError::SessionError(e.to_string()))?;
        let outputs = session
            .run(vec![Value::from_array(session.allocator(), &array)
                .map_err(|e| DlError::SessionError(e.to_string()))?])
            .map_err(|e| DlError::SessionError(e.to_string()))?;

        let tensor = outputs[0]
            .try_extract::<f32>()
            .map_err(|e| DlError::SessionError(e.to_string()))?;
        let slice = tensor.view();
        let flat: Vec<f32> = slice.iter().cloned().collect();
        if flat.len() != EMBED_DIM {
            return Err(DlError::DimMismatch(flat.len()));
        }
        let mut emb = [0.0f32; EMBED_DIM];
        emb.copy_from_slice(&flat);
        Ok(emb)
    }
}
