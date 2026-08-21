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
