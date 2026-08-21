# PDF Layout Constants

All values are in millimetres on A4 Landscape (297 x 210 mm).

| Constant          | Value (mm) | Purpose                          |
|-------------------|-----------|----------------------------------|
| PAGE_WIDTH        | 297.0     | A4 landscape width               |
| PAGE_HEIGHT       | 210.0     | A4 landscape height              |
| MARGIN_LEFT       | 10.0      | Left margin for all elements     |
| MARGIN_RIGHT      | 10.0      | Right margin                     |
| HEADER_Y          | 196.0     | Title baseline Y coordinate      |
| HEADER_SUBTITLE_Y | 190.0     | Subtitle / window Y coordinate   |
| TABLE_HEADER_Y    | 183.0     | Column header row Y              |
| TABLE_BODY_START  | 178.0     | First data row Y                 |
| ROW_HEIGHT        | 4.0       | Vertical spacing between rows    |
| ROWS_PER_PAGE     | 40        | Rows before a page break         |
| FOOTER_Y          | 5.0       | Footer text baseline Y           |
| FONT_TITLE        | 14.0 pt   | Report title                     |
| FONT_HEADER       | 8.0 pt    | Column header text               |
| FONT_BODY         | 7.0 pt    | Data row text                    |
| FONT_FOOTER       | 7.0 pt    | Footer text                      |
| COL_CELL_TRUNC    | 20 chars  | Max characters per cell          |

## Fonts

Built-in fonts only (no embedding required): `Helvetica` (body) and `HelveticaBold` (headers, title).

## Pagination

Pages are added dynamically via `PdfDocument::add_page()`. The footer on every page
includes: `Page N | {slug} | tlog: {leaf_index or "pending"}`.
