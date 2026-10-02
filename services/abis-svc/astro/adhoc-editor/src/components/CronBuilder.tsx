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
  value: string;
  onChange: (value: string) => void;
}

const PRESETS = [
  { label: 'Daily at 8am', value: '0 8 * * *' },
  { label: 'Weekly Monday 8am', value: '0 8 * * 1' },
  { label: 'Monthly 1st 8am', value: '0 8 1 * *' },
];

export default function CronBuilder({ value, onChange }: Props) {
  return (
    <div>
      <label for="cron-builder-input">Cron expression:</label>
      <input
        id="cron-builder-input"
        type="text"
        value={value}
        onInput={(e) => onChange((e.target as HTMLInputElement).value)}
        placeholder="0 8 * * 1"
        style={{ width: '200px', padding: '6px', marginLeft: '8px' }}
      />
      <div style={{ marginTop: '8px' }}>
        <span style={{ fontSize: '0.8rem', color: '#6c757d' }}>Presets: </span>
        {PRESETS.map((p) => (
          <button
            key={p.value}
            onClick={() => onChange(p.value)}
            style={{
              background: 'none', border: '1px solid #ced4da',
              borderRadius: '4px', padding: '2px 8px', cursor: 'pointer',
              fontSize: '0.8rem', marginLeft: '4px',
            }}
          >
            {p.label}
          </button>
        ))}
      </div>
    </div>
  );
}
