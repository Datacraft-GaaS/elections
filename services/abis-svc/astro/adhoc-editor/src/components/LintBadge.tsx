/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type { LintReport } from '../lib/api';

interface Props {
  report: LintReport | null;
}

export default function LintBadge({ report }: Props) {
  if (!report) return null;

  const score = report.safety_score ?? 0;
  const color = score >= 0.8 ? '#155724' : score >= 0.5 ? '#856404' : '#721c24';
  const bg = score >= 0.8 ? '#d4edda' : score >= 0.5 ? '#fff3cd' : '#f8d7da';

  return (
    <div style={{ marginTop: '12px' }}>
      <span
        aria-label={`Safety score ${score.toFixed(2)}`}
        style={{
          background: bg, color, padding: '4px 12px',
          borderRadius: '16px', fontWeight: 'bold',
        }}
      >
        Score: {score.toFixed(2)}
      </span>
      {report.errors.length > 0 && (
        <ul aria-label="Lint errors" style={{ padding: 0, listStyle: 'none', marginTop: '8px' }}>
          {report.errors.map((e, i) => (
            <li key={i} role="alert" style={{
              background: '#f8d7da', color: '#721c24',
              padding: '8px', borderRadius: '4px', marginBottom: '4px',
            }}>
              <strong>[{e.code}]</strong> {e.message}
            </li>
          ))}
        </ul>
      )}
      {report.warnings.length > 0 && (
        <ul aria-label="Lint warnings" style={{ padding: 0, listStyle: 'none' }}>
          {report.warnings.map((w, i) => (
            <li key={i} style={{
              background: '#fff3cd', color: '#856404',
              padding: '8px', borderRadius: '4px', marginBottom: '4px',
            }}>
              <strong>[{w.code}]</strong> {w.message}
            </li>
          ))}
        </ul>
      )}
      {report.errors.length === 0 && report.warnings.length === 0 && (
        <span style={{ color: '#155724', marginLeft: '8px' }}>No issues.</span>
      )}
    </div>
  );
}
