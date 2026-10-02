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
  onQuery: (sql: string) => void;
}

export default function QueryBuilderComponent({ onQuery }: Props) {
  const [tables, setTables] = useState('');
  const [columns, setColumns] = useState('*');
  const [where, setWhere] = useState('');
  const [limit, setLimit] = useState(100);

  function buildSql() {
    const tableList = tables.split(',').map((t) => t.trim()).filter(Boolean).join(', ');
    const cols = columns.trim() || '*';
    let sql = `SELECT ${cols} FROM ${tableList}`;
    if (where.trim()) sql += ` WHERE ${where.trim()}`;
    sql += ` LIMIT ${limit}`;
    onQuery(sql);
  }

  return (
    <div style={{ border: '1px solid #dee2e6', borderRadius: '8px', padding: '16px', marginTop: '8px' }}>
      <h4 style={{ marginTop: 0 }}>Query Builder</h4>
      <div style={{ display: 'grid', gap: '8px' }}>
        <label>
          Tables:
          <input
            type="text"
            value={tables}
            onInput={(e) => setTables((e.target as HTMLInputElement).value)}
            placeholder="eel_voters, eel_polling_stations"
            style={{ width: '100%', padding: '6px', marginTop: '2px' }}
          />
        </label>
        <label>
          Columns:
          <input
            type="text"
            value={columns}
            onInput={(e) => setColumns((e.target as HTMLInputElement).value)}
            placeholder="*"
            style={{ width: '100%', padding: '6px', marginTop: '2px' }}
          />
        </label>
        <label>
          WHERE clause:
          <input
            type="text"
            value={where}
            onInput={(e) => setWhere((e.target as HTMLInputElement).value)}
            placeholder="archived_at IS NULL"
            style={{ width: '100%', padding: '6px', marginTop: '2px' }}
          />
        </label>
        <label>
          Limit:
          <input
            type="number"
            value={limit}
            onInput={(e) => setLimit(Number((e.target as HTMLInputElement).value))}
            min={1}
            max={10000}
            style={{ width: '100px', padding: '6px', marginTop: '2px' }}
          />
        </label>
        <button
          onClick={buildSql}
          style={{
            background: '#0d6efd', color: '#fff',
            border: 'none', padding: '8px 16px',
            borderRadius: '4px', cursor: 'pointer',
          }}
        >
          Build SQL
        </button>
      </div>
    </div>
  );
}
