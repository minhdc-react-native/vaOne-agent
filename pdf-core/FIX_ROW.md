# Dynamic fixed-row layouts

For a table bound to an array (`fieldName: "orderDetails"`), a non-empty
`fixRow.data` describes the lines to render **for each array item**. For example,
the first line can contain a product name spanning all four columns, followed
by quantity, price, discount and amount on the second line.

- Fields in those lines bind to the current array item, not the report root.
- `columns` on the table still defines the header and the underlying column widths.
- `colSpan` defaults to one. A spanning cell uses the sum of the corresponding
  underlying widths; excessive spans are bounded by the remaining columns.
- Missing, empty or whitespace-only `formatString` preserves the original text.
- Each line measures its formatted text at the same inset/width used to draw it.
- All lines of one item form one pagination unit. Headers repeat on subsequent
  pages, and following report elements use the table's actual height.
- An absent or empty `fixRow.data` keeps the ordinary one-line-per-item layout.
- Static fixed-row tables retain their existing root-data binding.

`fixRow.data` supplies the actual lines; `fixRow.row` remains template metadata.
This change supports horizontal `colSpan`, not vertical `rowSpan` in body cells.
As with ordinary body rows, an item group taller than a page cannot be split by
the current paginator.

## Verify and build

From the repository root:

```sh
cargo test -p pdf-core --locked
cargo build -p report-cli --release --locked
```

The native library is `target/release/report_cli.dll` on Windows or
`target/release/libreport_cli.so` on Linux. Build for the host OS/architecture
used by the consuming API. Editing this source alone does not update a deployed
API's native library.

Existing `visibleIf` expressions are unchanged. In particular, a receipt's
total remains hidden if its template ties visibility to a zero tax reduction;
remove that condition from the total element if it should always be shown.
