# solx Compiler Documentation

This directory contains an **mdBook** project for **solx** documentation.
This README will guide you on how to build and test the book locally.

## Prerequisites

Before you begin, ensure you have the **mdbook** installed:

```bash
cargo install mdbook
```

## Build the Book

To build the book, use the following command:

```bash
cd docs  # if you're not in this directory already
mdbook build
```

This command generates the static HTML files in the `book/` directory. You can open `book/index.html` in your browser to view the generated book.

## Serve the Book Locally

For easier development and to preview changes as you edit, you can serve the book locally with:

```bash
mdbook serve
```

This will start a local web server and open the book in your default browser. Any changes made to the markdown files will automatically reload the book.

By default, the book will be accessible at: `http://localhost:3000`.

## Testing the Book

To ensure your book is correctly built and formatted, you can use the built-in **mdBook** linter by running:

```bash
mdbook test
```

This will check for common issues such as broken links or missing files.

## Directory Structure

- `src/`: This directory contains all the markdown files for the chapters.
- `book/`: This is the output directory where the built HTML files are generated.
- `book.toml`: Configuration file for the **mdBook**.

## Deployment

The book is automatically deployed each time a commit is pushed to the **main** branch or a new release tag is created.
The generated HTML files are hosted on GitHub Pages and are accessible at:
* https://nomicfoundation.github.io/solx/latest/ for the latest documentation.
* https://nomicfoundation.github.io/solx/vX.Y.Z/ for the specific version documentation.
