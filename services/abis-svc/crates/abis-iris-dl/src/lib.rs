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

//! Deep iris embedding via hash-based deterministic embedder (DeepIrisNet-inspired).
//!
//! Reference: Gangwar, A. & Joshi, A. (2016). DeepIrisNet: Deep iris representation
//! with applications in iris recognition and cross-sensor iris recognition.
//! IEEE ICIP 2016.
#![warn(missing_docs)]

pub mod embedder;
pub mod hash_embedder;

pub use embedder::{EmbedError, Embedding, IrisEmbedder};
