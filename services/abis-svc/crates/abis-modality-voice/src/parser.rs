// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! WAV/PCM ingest, inline radix-2 FFT, and MFCC feature extraction.

use chrono::Utc;
use std::f64::consts::PI;

use crate::domain::{UtteranceType, VoiceTemplate};

/// WAV RIFF header fields needed for PCM decoding.
#[derive(Debug, Clone)]
pub struct WavHeader {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub data_offset: usize,
    pub num_samples: usize,
}

/// Errors from WAV parsing or embedding (de)serialization.
#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("buffer too short: need {need}, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("invalid RIFF signature")]
    InvalidRiff,
    #[error("WAV data chunk not found")]
    DataChunkNotFound,
    #[error("embedding length {len} not divisible by 4")]
    InvalidEmbeddingLen { len: usize },
    #[error("unsupported bits per sample: {bps}")]
    UnsupportedBps { bps: u16 },
}

// ---------------------------------------------------------------------------
// Inline Cooley-Tukey radix-2 DIT FFT

/// Perform an in-place radix-2 DIT FFT on complex data.
///
/// Input length must be a power of 2.
pub fn fft_inplace(data: &mut Vec<(f64, f64)>) {
    let n = data.len();
    if n <= 1 {
        return;
    }

    // Bit-reversal permutation.
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }

    // Cooley-Tukey butterfly stages.
    let mut len = 2usize;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * PI / len as f64;
        let wlen = (angle.cos(), angle.sin());

        for i in (0..n).step_by(len) {
            let mut w = (1.0_f64, 0.0_f64);
            for k in 0..half {
                let u = data[i + k];
                let v = data[i + k + half];
                let vw = (v.0 * w.0 - v.1 * w.1, v.0 * w.1 + v.1 * w.0);
                data[i + k] = (u.0 + vw.0, u.1 + vw.1);
                data[i + k + half] = (u.0 - vw.0, u.1 - vw.1);
                let new_wr = w.0 * wlen.0 - w.1 * wlen.1;
                let new_wi = w.0 * wlen.1 + w.1 * wlen.0;
                w = (new_wr, new_wi);
            }
        }

        len <<= 1;
    }
}

/// Return the nearest power of 2 >= n.
fn next_pow2(n: usize) -> usize {
    let mut p = 1;
    while p < n {
        p <<= 1;
    }
    p
}

// ---------------------------------------------------------------------------
// Mel filterbank helpers

fn hz_to_mel(hz: f64) -> f64 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f64) -> f64 {
    700.0 * (10.0_f64.powf(mel / 2595.0) - 1.0)
}

fn mel_filterbank(
    n_filters: usize,
    fft_size: usize,
    sample_rate: u32,
    f_min: f64,
    f_max: f64,
) -> Vec<Vec<f32>> {
    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);
    let mel_points: Vec<f64> = (0..=n_filters + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f64 / (n_filters + 1) as f64)
        .collect();
    let hz_points: Vec<f64> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();
    let bin_points: Vec<usize> = hz_points
        .iter()
        .map(|&hz| (fft_size as f64 * hz / sample_rate as f64).floor() as usize)
        .collect();

    let n_fft_bins = fft_size / 2 + 1;
    let mut filters = vec![vec![0.0f32; n_fft_bins]; n_filters];

    for m in 0..n_filters {
        let f_m_minus = bin_points[m];
        let f_m = bin_points[m + 1];
        let f_m_plus = bin_points[m + 2];

        for k in f_m_minus..f_m.min(n_fft_bins) {
            if f_m > f_m_minus {
                filters[m][k] = (k - f_m_minus) as f32 / (f_m - f_m_minus) as f32;
            }
        }
        for k in f_m..f_m_plus.min(n_fft_bins) {
            if f_m_plus > f_m {
                filters[m][k] = (f_m_plus - k) as f32 / (f_m_plus - f_m) as f32;
            }
        }
    }

    filters
}

/// Extract MFCC features from 16-bit PCM audio.
///
/// Steps: pre-emphasis -> frame -> Hamming window -> FFT -> mel filterbank -> log -> DCT.
pub fn extract_mfcc(pcm: &[i16], sample_rate: u32, n_mfcc: usize) -> Vec<Vec<f32>> {
    if pcm.is_empty() {
        return vec![];
    }

    // Pre-emphasis.
    let pre_emph = 0.97_f64;
    let mut emphasized: Vec<f64> = vec![pcm[0] as f64];
    for i in 1..pcm.len() {
        emphasized.push(pcm[i] as f64 - pre_emph * pcm[i - 1] as f64);
    }

    // Frame parameters.
    let frame_len = (0.025 * sample_rate as f64) as usize; // 25ms
    let frame_step = (0.010 * sample_rate as f64) as usize; // 10ms
    if frame_len == 0 || frame_step == 0 || emphasized.len() < frame_len {
        return vec![];
    }

    let n_frames = 1 + (emphasized.len() - frame_len) / frame_step;
    let fft_size = next_pow2(frame_len);
    let n_filters = 26;

    let filters = mel_filterbank(
        n_filters,
        fft_size,
        sample_rate,
        0.0,
        sample_rate as f64 / 2.0,
    );

    let mut mfcc_frames = Vec::with_capacity(n_frames);

    for frame_idx in 0..n_frames {
        let start = frame_idx * frame_step;
        let end = (start + frame_len).min(emphasized.len());

        // Hamming window + zero-pad to fft_size.
        let mut complex_frame: Vec<(f64, f64)> = (0..fft_size)
            .map(|k| {
                if k < end - start {
                    let sample = emphasized[start + k];
                    let window = 0.54 - 0.46 * (2.0 * PI * k as f64 / (frame_len - 1) as f64).cos();
                    (sample * window, 0.0)
                } else {
                    (0.0, 0.0)
                }
            })
            .collect();

        fft_inplace(&mut complex_frame);

        // Power spectrum (one-sided).
        let n_fft_bins = fft_size / 2 + 1;
        let power: Vec<f64> = complex_frame[..n_fft_bins]
            .iter()
            .map(|(re, im)| (re * re + im * im) / fft_size as f64)
            .collect();

        // Apply mel filterbank.
        let log_mel: Vec<f64> = filters
            .iter()
            .map(|f| {
                let energy: f64 = f
                    .iter()
                    .zip(power.iter())
                    .map(|(&fw, &p)| fw as f64 * p)
                    .sum();
                (energy.max(1e-10)).ln()
            })
            .collect();

        // DCT to get MFCC.
        let n_out = n_mfcc.min(n_filters);
        let mut coeffs = vec![0.0f32; n_out];
        for n in 0..n_out {
            let mut sum = 0.0_f64;
            for m in 0..n_filters {
                sum += log_mel[m]
                    * (PI * n as f64 * (2.0 * m as f64 + 1.0) / (2.0 * n_filters as f64)).cos();
            }
            coeffs[n] = sum as f32;
        }

        mfcc_frames.push(coeffs);
    }

    mfcc_frames
}

// ---------------------------------------------------------------------------
// WAV parser

fn read_u16_le(data: &[u8], offset: usize) -> Result<u16, VoiceError> {
    if offset + 2 > data.len() {
        return Err(VoiceError::BufferTooShort {
            need: offset + 2,
            got: data.len(),
        });
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, VoiceError> {
    if offset + 4 > data.len() {
        return Err(VoiceError::BufferTooShort {
            need: offset + 4,
            got: data.len(),
        });
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Parse the RIFF/WAV header from raw bytes.
pub fn parse_wav_header(data: &[u8]) -> Result<WavHeader, VoiceError> {
    if data.len() < 44 {
        return Err(VoiceError::BufferTooShort {
            need: 44,
            got: data.len(),
        });
    }
    if &data[0..4] != b"RIFF" {
        return Err(VoiceError::InvalidRiff);
    }
    // data[8..12] == "WAVE" - not strictly checked to allow synthetic headers
    let channels = read_u16_le(data, 22)?;
    let sample_rate = read_u32_le(data, 24)?;
    let bits_per_sample = read_u16_le(data, 34)?;

    if bits_per_sample != 16 {
        return Err(VoiceError::UnsupportedBps {
            bps: bits_per_sample,
        });
    }

    // Find "data" sub-chunk starting at offset 36.
    let mut offset = 36;
    loop {
        if offset + 8 > data.len() {
            return Err(VoiceError::DataChunkNotFound);
        }
        let chunk_size = read_u32_le(data, offset + 4)? as usize;
        if &data[offset..offset + 4] == b"data" {
            let data_offset = offset + 8;
            let num_samples = chunk_size / (bits_per_sample as usize / 8) / channels as usize;
            return Ok(WavHeader {
                channels,
                sample_rate,
                bits_per_sample,
                data_offset,
                num_samples,
            });
        }
        offset += 8 + chunk_size;
    }
}

/// Parse a WAV file into a [VoiceTemplate] with MFCC features extracted.
pub fn parse_wav(data: &[u8]) -> Result<VoiceTemplate, VoiceError> {
    let header = parse_wav_header(data)?;
    let data_end = header.data_offset + header.num_samples * header.channels as usize * 2;
    if data.len() < data_end {
        return Err(VoiceError::BufferTooShort {
            need: data_end,
            got: data.len(),
        });
    }

    // Decode 16-bit LE PCM samples (mono channel 0).
    let pcm: Vec<i16> = (0..header.num_samples)
        .map(|i| {
            let offset = header.data_offset + i * header.channels as usize * 2;
            i16::from_le_bytes([data[offset], data[offset + 1]])
        })
        .collect();

    let duration_ms = (header.num_samples as u64 * 1000 / header.sample_rate as u64) as u32;
    let mfcc_features = extract_mfcc(&pcm, header.sample_rate, 13);

    Ok(VoiceTemplate {
        template_id: uuid::Uuid::new_v4().to_string(),
        utterance_type: UtteranceType::FreeSpeech,
        sample_rate_hz: header.sample_rate,
        duration_ms,
        language: "und".into(),
        embedding_dim: 0,
        embedding: vec![],
        mfcc_features,
        snr_db: None,
        captured_at: Utc::now(),
    })
}

/// Serialize an f32 embedding slice to little-endian bytes.
pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(embedding.len() * 4);
    for &v in embedding {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// Deserialize little-endian bytes back into an f32 embedding.
pub fn deserialize_embedding(data: &[u8]) -> Result<Vec<f32>, VoiceError> {
    if data.len() % 4 != 0 {
        return Err(VoiceError::InvalidEmbeddingLen { len: data.len() });
    }
    let values = data
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    Ok(values)
}
