# doc_search

Search for text patterns inside .odt, .docx, and .doc files.

## Installation
```bash
cargo install doc_search
```

## Usage
```bash
# Search for "budget" in all docs under ./reports
doc_search budget -p ./reports

# Show matching paragraph numbers
doc_search budget -p ./reports -v 2

# Show matching paragraphs with content
doc_search budget -p ./reports -v 3

# Limit search to current direct children of reports
doc_search budget -p ./reports -d 1

# Search file directly
doc_search budget -p ./reports/June.odt -d 0 # (explicit)
doc_search budget -p ./reports/June.odt # (implicit)
```

Also accepts regex to search with
```bash
doc_search ^[a-z].* -p ./reports
```

## Verbosity Levels

- `1` (default): File paths only
- `2`: File paths + paragraph numbers
- `3`: File paths + paragraph numbers + content