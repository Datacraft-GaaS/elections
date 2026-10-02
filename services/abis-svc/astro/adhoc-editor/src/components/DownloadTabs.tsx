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
  previewId: string | null;
}

const FORMATS = ['pdf', 'docx', 'markdown', 'csv'];

export default function DownloadTabs({ previewId }: Props) {
  if (!previewId) return null;

  return (
    <nav aria-label="Download formats" style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
      {FORMATS.map((fmt) => (
        <span
          key={fmt}
          style={{
            padding: '6px 12px',
            background: '#e9ecef',
            borderRadius: '4px',
            fontSize: '0.85rem',
            color: '#6c757d',
            cursor: 'default',
          }}
          aria-label={`Download ${fmt.toUpperCase()} (preview ${previewId})`}
          title={`Download as ${fmt.toUpperCase()}`}
        >
          {fmt.toUpperCase()}
        </span>
      ))}
    </nav>
  );
}
