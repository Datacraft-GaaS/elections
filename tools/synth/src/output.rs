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

// Output helpers: open a writer to stdout or a file path.

use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub enum OutputSink {
    Stdout(std::io::Stdout),
    File(BufWriter<File>),
}

impl OutputSink {
    pub fn open(path: Option<&Path>) -> Result<Self> {
        match path {
            None => Ok(Self::Stdout(std::io::stdout())),
            Some(p) => {
                if let Some(parent) = p.parent() {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                let f = File::create(p)?;
                Ok(Self::File(BufWriter::new(f)))
            }
        }
    }
}

impl Write for OutputSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout(s) => s.write(buf),
            Self::File(f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(s) => s.flush(),
            Self::File(f) => f.flush(),
        }
    }
}

/// Emit a "parquet not supported in this build" error.
pub fn parquet_unsupported() -> anyhow::Error {
    anyhow::anyhow!(
        "Parquet output is not supported in this build. \
         Use --format csv or --format sql. \
         Parquet support is planned for Wave 1 (eel-arrow crate)."
    )
}
