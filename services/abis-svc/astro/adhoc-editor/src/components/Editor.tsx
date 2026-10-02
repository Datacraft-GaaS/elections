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
import type { AdhocReportVersion, LintReport } from '../lib/api';
import { api } from '../lib/api';
import LintBadge from './LintBadge';
import PreviewTable from './PreviewTable';

interface Props {
  adhocId: string;
  initialVersion?: AdhocReportVersion;
}

export default function Editor({ adhocId: adhocIdProp, initialVersion }: Props) {
  // When rendered on a static page, the real ID comes from ?id= query param.
  const adhocId =
    adhocIdProp === '__URL__'
      ? new URLSearchParams(typeof window !== 'undefined' ? window.location.search : '').get('id') ?? ''
      : adhocIdProp;

  const [source, setSource] = useState(initialVersion?.source ?? '');
  const [allowedTables, setAllowedTables] = useState(
    (initialVersion?.allowed_tables ?? []).join(', ')
  );
  const [commitMessage, setCommitMessage] = useState('');
  const [lintReport, setLintReport] = useState<LintReport | null>(null);
  const [saving, setSaving] = useState(false);
  const [currentVersionId, setCurrentVersionId] = useState(initialVersion?.version_id);
  const [previewRows, setPreviewRows] = useState<Record<string, unknown>[]>([]);
  const [previewFormat, setPreviewFormat] = useState('json');
  const [sampleLimit, setSampleLimit] = useState(100);
  const [previewing, setPreviewing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSave() {
    setSaving(true);
    setError(null);
    try {
      const tables = allowedTables.split(',').map((t) => t.trim()).filter(Boolean);
      const version = await api.createVersion(adhocId, {
        source,
        source_type: 'sql',
        allowed_tables: tables,
        commit_message: commitMessage || undefined,
      });
      setLintReport(version.lint_report);
      setCurrentVersionId(version.version_id);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handlePreview() {
    if (!currentVersionId) {
      setError('Save a version first.');
      return;
    }
    setPreviewing(true);
    setError(null);
    try {
      const result = await api.runPreview(currentVersionId, {
        format: previewFormat,
        sample_limit: sampleLimit,
      });
      setPreviewRows((result.rows as Record<string, unknown>[]) ?? []);
    } catch (err) {
      setError(String(err));
    } finally {
      setPreviewing(false);
    }
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
      {/* SQL pane */}
      <section aria-label="SQL editor">
        <h3>SQL Source</h3>
        <textarea
          value={source}
          onInput={(e) => setSource((e.target as HTMLTextAreaElement).value)}
          style={{
            width: '100%', height: '280px',
            fontFamily: 'monospace', fontSize: '0.9rem',
            padding: '8px', border: '1px solid #ced4da', borderRadius: '4px',
          }}
          aria-label="SQL query"
          spellcheck={false}
          data-testid="sql-editor"
        />
        <div style={{ marginTop: '8px' }}>
          <label for="allowed-tables-input">Allowed tables (comma-separated):</label>
          <input
            id="allowed-tables-input"
            type="text"
            value={allowedTables}
            onInput={(e) => setAllowedTables((e.target as HTMLInputElement).value)}
            style={{ width: '100%', padding: '6px', margin: '4px 0' }}
          />
          <label for="commit-msg-input">Commit message:</label>
          <input
            id="commit-msg-input"
            type="text"
            value={commitMessage}
            onInput={(e) => setCommitMessage((e.target as HTMLInputElement).value)}
            style={{ width: '100%', padding: '6px', margin: '4px 0' }}
          />
          <button
            onClick={handleSave}
            disabled={saving}
            style={{
              background: '#0d6efd', color: '#fff',
              border: 'none', padding: '8px 16px',
              borderRadius: '4px', cursor: 'pointer', minHeight: '48px',
            }}
            data-testid="save-button"
          >
            {saving ? 'Saving...' : 'Save version'}
          </button>
        </div>

        {error && (
          <div role="alert" style={{ color: '#721c24', background: '#f8d7da', padding: '8px', borderRadius: '4px', marginTop: '8px' }}>
            {error}
          </div>
        )}

        <LintBadge report={lintReport} />
      </section>

      {/* Preview pane */}
      <section aria-label="Preview">
        <h3>Preview</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'flex-end', marginBottom: '8px' }}>
          <div>
            <label for="preview-format">Format:</label>
            <select
              id="preview-format"
              value={previewFormat}
              onChange={(e) => setPreviewFormat((e.target as HTMLSelectElement).value)}
              style={{ padding: '6px', marginLeft: '4px' }}
            >
              <option value="json">JSON</option>
              <option value="markdown">Markdown</option>
              <option value="docx">DOCX</option>
              <option value="csv">CSV</option>
            </select>
          </div>
          <div>
            <label for="sample-limit-input">Limit:</label>
            <input
              id="sample-limit-input"
              type="number"
              value={sampleLimit}
              onInput={(e) => setSampleLimit(Number((e.target as HTMLInputElement).value))}
              min={1}
              max={1000}
              style={{ width: '70px', padding: '6px', marginLeft: '4px' }}
            />
          </div>
          <button
            onClick={handlePreview}
            disabled={previewing || !currentVersionId}
            style={{
              background: '#6c757d', color: '#fff',
              border: 'none', padding: '8px 16px',
              borderRadius: '4px', cursor: 'pointer', minHeight: '48px',
            }}
            data-testid="preview-button"
          >
            {previewing ? 'Running...' : 'Run preview'}
          </button>
        </div>

        <PreviewTable rows={previewRows} format={previewFormat} />
      </section>
    </div>
  );
}
