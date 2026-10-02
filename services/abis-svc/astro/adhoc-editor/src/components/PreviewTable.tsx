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

interface Props {
  rows: Record<string, unknown>[];
  format: string;
}

export default function PreviewTable({ rows, format }: Props) {
  if (!rows || rows.length === 0) {
    return <p role="status">No preview rows.</p>;
  }

  const headers = Object.keys(rows[0]);

  return (
    <div>
      <p style={{ fontSize: '0.85rem', color: '#6c757d' }}>
        {rows.length} rows (format: {format})
      </p>
      <div style={{ overflowX: 'auto' }}>
        <table
          role="grid"
          style={{ borderCollapse: 'collapse', width: '100%', fontSize: '0.85rem' }}
        >
          <thead>
            <tr>
              {headers.map((h) => (
                <th
                  key={h}
                  scope="col"
                  style={{
                    border: '1px solid #dee2e6',
                    padding: '6px 10px',
                    background: '#f8f9fa',
                    textAlign: 'left',
                  }}
                >
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                {headers.map((h) => (
                  <td key={h} style={{ border: '1px solid #dee2e6', padding: '6px 10px' }}>
                    {String(row[h] ?? '')}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
