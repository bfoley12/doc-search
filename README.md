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
```

Also accepts regex to search with
```bash
doc_search 
```

## Verbosity Levels

- `1` (default): File paths only
- `2`: File paths + paragraph numbers
- `3`: File paths + paragraph numbers + content