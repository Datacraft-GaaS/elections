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

import { useState } from 'preact/hooks';
import { api } from '../lib/api';

interface Props {
  versionId: string;
}

export default function PublishWizard({ versionId }: Props) {
  const [formats, setFormats] = useState<string[]>(['docx', 'markdown', 'csv']);
  const [schedule, setSchedule] = useState('');
  const [publishing, setPublishing] = useState(false);
  const [definitionId, setDefinitionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function toggleFormat(f: string) {
    setFormats((prev) =>
      prev.includes(f) ? prev.filter((x) => x !== f) : [...prev, f]
    );
  }

  async function handlePublish() {
    setPublishing(true);
    setError(null);
    try {
      const result = await api.publishVersion(versionId, {
        formats,
        schedule: schedule || undefined,
      });
      setDefinitionId(result.definition_id ?? null);
    } catch (err) {
      setError(String(err));
    } finally {
      setPublishing(false);
    }
  }

  if (definitionId) {
    return (
      <div role="status" style={{ background: '#d4edda', color: '#155724', padding: '16px', borderRadius: '8px' }}>
        Published. Definition ID: <code>{definitionId}</code>
      </div>
    );
  }

  return (
    <div style={{ maxWidth: '480px' }}>
      <h3>Publish Report</h3>

      <fieldset style={{ border: '1px solid #dee2e6', padding: '16px', borderRadius: '8px', marginBottom: '16px' }}>
        <legend>Output formats</legend>
        {['pdf', 'docx', 'markdown', 'csv', 'json'].map((f) => (
          <label key={f} style={{ display: 'block', marginBottom: '4px' }}>
            <input
              type="checkbox"
              checked={formats.includes(f)}
              onChange={() => toggleFormat(f)}
              style={{ marginRight: '8px' }}
            />
            {f.toUpperCase()}
          </label>
        ))}
      </fieldset>

      <div style={{ marginBottom: '16px' }}>
        <label for="cron-input">Schedule (cron expression, optional):</label>
        <input
          id="cron-input"
          type="text"
          value={schedule}
          onInput={(e) => setSchedule((e.target as HTMLInputElement).value)}
          placeholder="0 8 * * 1"
          style={{ width: '200px', padding: '6px', marginLeft: '8px' }}
        />
      </div>

      {error && (
        <div role="alert" style={{ color: '#721c24', background: '#f8d7da', padding: '8px', borderRadius: '4px', marginBottom: '8px' }}>
          {error}
        </div>
      )}

      <button
        onClick={handlePublish}
        disabled={publishing || formats.length === 0}
        style={{
          background: '#0d6efd', color: '#fff',
          border: 'none', padding: '8px 16px',
          borderRadius: '4px', cursor: 'pointer', minHeight: '48px',
        }}
      >
        {publishing ? 'Publishing...' : 'Publish'}
      </button>
    </div>
  );
}
