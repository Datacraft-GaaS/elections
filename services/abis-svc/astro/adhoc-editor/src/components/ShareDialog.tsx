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

interface Props {
  adhocId: string;
}

export default function ShareDialog({ adhocId: adhocIdProp }: Props) {
  const adhocId =
    adhocIdProp === '__URL__'
      ? new URLSearchParams(typeof window !== 'undefined' ? window.location.search : '').get('id') ?? ''
      : adhocIdProp;
  const [open, setOpen] = useState(false);
  const [subjectId, setSubjectId] = useState('');
  const [permission, setPermission] = useState('view');
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleShare() {
    setError(null);
    try {
      const res = await fetch(`/v1/adhoc/reports/${adhocId}/shares`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ shared_with: subjectId, permission }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      setDone(true);
    } catch (err) {
      setError(String(err));
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        style={{
          background: '#6c757d', color: '#fff',
          border: 'none', padding: '8px 16px',
          borderRadius: '4px', cursor: 'pointer', minHeight: '48px',
        }}
      >
        Share...
      </button>
    );
  }

  return (
    <dialog open aria-label="Share report dialog" style={{ padding: '24px', borderRadius: '8px', border: '1px solid #dee2e6' }}>
      <h3 style={{ marginTop: 0 }}>Share Report</h3>
      {done ? (
        <p role="status" style={{ color: '#155724' }}>Shared successfully.</p>
      ) : (
        <>
          <label for="subject-id">Subject ID (UUID):</label>
          <input
            id="subject-id"
            type="text"
            value={subjectId}
            onInput={(e) => setSubjectId((e.target as HTMLInputElement).value)}
            style={{ width: '100%', padding: '6px', margin: '4px 0 12px' }}
          />
          <label for="permission-select">Permission:</label>
          <select
            id="permission-select"
            value={permission}
            onChange={(e) => setPermission((e.target as HTMLSelectElement).value)}
            style={{ padding: '6px', marginLeft: '8px' }}
          >
            <option value="view">View</option>
            <option value="run">Run</option>
            <option value="edit">Edit</option>
            <option value="admin">Admin</option>
          </select>
          {error && (
            <div role="alert" style={{ color: '#721c24', marginTop: '8px' }}>{error}</div>
          )}
          <div style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
            <button onClick={handleShare} style={{ background: '#0d6efd', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: '4px', cursor: 'pointer', minHeight: '48px' }}>
              Share
            </button>
            <button onClick={() => setOpen(false)} style={{ background: '#6c757d', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: '4px', cursor: 'pointer', minHeight: '48px' }}>
              Cancel
            </button>
          </div>
        </>
      )}
    </dialog>
  );
}
