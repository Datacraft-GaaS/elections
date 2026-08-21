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
