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

use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

/// Tracing subscriber layer that detects PII markers in log fields and warns.
///
/// `PIIRestricted<T>` and `PIISensitive<T>` implement `Debug` with
/// redaction (`PIIRestricted(***REDACTED***)` / `PIISensitive(***REDACTED***)`),
/// so the actual PII value never enters the log stream. This layer adds a
/// secondary check and emits a warning if a redaction marker appears, indicating
/// a PII-typed value was passed to a log macro (ADR-0016 violation).
pub struct PiiScrubberLayer;

struct FieldScanner {
    pub found_pii: bool,
}

impl tracing::field::Visit for FieldScanner {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let s = format!("{value:?}");
        if s.contains("***REDACTED***") {
            self.found_pii = true;
        }
    }

    fn record_str(&mut self, _field: &tracing::field::Field, value: &str) {
        if value.contains("***REDACTED***") {
            self.found_pii = true;
        }
    }
}

impl<S: Subscriber> Layer<S> for PiiScrubberLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut scanner = FieldScanner { found_pii: false };
        event.record(&mut scanner);
        if scanner.found_pii {
            eprintln!(
                "[eel-scrubber] WARNING: PII-typed value detected in log event at {}:{}",
                event.metadata().file().unwrap_or("?"),
                event.metadata().line().unwrap_or(0)
            );
        }
    }
}
