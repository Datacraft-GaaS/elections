// Supported locale codes for the eel public portal (FR-M11-033, AC-M11-11).
// Extend ui map below when adding translations.

export const LOCALES = ['en', 'sw', 'ki', 'luo', 'kln', 'mer', 'kam', 'som', 'orm', 'msa', 'ar']

export const ui = {
  en: {
    'nav.home': 'Results',
    'nav.transparency': 'Transparency Log',
    'nav.history': 'Historical Snapshots',
    'nav.downloads': 'Bulk Downloads',
    'home.title': 'Election Results',
    'home.last_updated': 'Last updated',
    'home.stale_warning': 'Data may be stale. Last update: {ts}',
    'home.petition_window': 'Petition window closes: {date}',
    'snapshot.merkle_root': 'Merkle root',
    'snapshot.sig': 'Signature',
    'snapshot.download_csv': 'Download CSV',
    'snapshot.download_parquet': 'Download Parquet',
    'proof.inclusion': 'Inclusion proof',
    'proof.consistency': 'Consistency proof',
    'proof.verified': 'Verified',
    'proof.unverified': 'Unverified',
    'rss.title': 'Election Results Feed',
  },
  sw: {
    'nav.home': 'Matokeo',
    'nav.transparency': 'Kumbukumbu ya Uwazi',
    'nav.history': 'Picha za Kihistoria',
    'nav.downloads': 'Pakua Data',
    'home.title': 'Matokeo ya Uchaguzi',
    'home.last_updated': 'Imesasishwa mwisho',
    'home.stale_warning': 'Data inaweza kuwa ya zamani. Sasisha mwisho: {ts}',
    'home.petition_window': 'Muda wa malalamiko unafungwa: {date}',
    'snapshot.merkle_root': 'Mzizi wa Merkle',
    'snapshot.sig': 'Saini',
    'snapshot.download_csv': 'Pakua CSV',
    'snapshot.download_parquet': 'Pakua Parquet',
    'proof.inclusion': 'Uthibitisho wa ujumuishaji',
    'proof.consistency': 'Uthibitisho wa uthabiti',
    'proof.verified': 'Imethibitishwa',
    'proof.unverified': 'Haijathibitishwa',
    'rss.title': 'Matokeo ya Uchaguzi - Chanzo',
  },
}

export function t(locale, key, vars = {}) {
  const dict = ui[locale] ?? ui['en']
  let str = dict[key] ?? ui['en'][key] ?? key
  for (const [k, v] of Object.entries(vars)) {
    str = str.replaceAll(`{${k}}`, v)
  }
  return str
}
