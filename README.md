# TreeGen
[![Language: Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Last Commit](https://img.shields.io/github/last-commit/JoeChala/treegen-rs)](https://github.com/JoeChala/treegen-rs/commits/main)
[![License](https://img.shields.io/github/license/JoeChala/treegen-rs)](LICENSE)

A simple CLI tool to generate project file structures from text, templates, or defaults.

---

## Table of Contents
- [Overview](#overview)
- [Getting Started](#getting-started)
- [Usage Examples](#usage-examples)
- [Templates](#templates)
- [License](#license)
---

## Overview

`treegen` lets you define a project’s folder and file structure via:
- Command-line arguments  
- A text file  
- A saved language template (like `--default py` or `--default rs`)

You can preview with `--dry` before creating files.

Example:
```bash
treegen src/core/test.rs .. lib.rs tests/test.rs : ui/f1.rs Cargo.toml
```
creates:
src/
  core/
    test.rs
  lib.rs
  tests/
    test.rs
ui/
  f1.rs
Cargo.toml

---
## Getting Started
### Install
```bash
git clone https://github.com/JoeChala/treegen-rs.git
cd treegen
cargo install --path .

```
---

## Usage Examples
### Dry run (preview structure)
```bash
treegen --default py --dry
```
### From a structure text file
```bash
treegen --from my_structure.txt --output ./myproject
```

### Using templates
```bash
treegen --template rust_lib --output ./lib_project
```
---
## Templates

You can define reusable templates at:
```bash
~/.config/treegen/templates/
```

Each template file describes a structure, for example:
```bash
src/
    main.rs
Cargo.toml
README.md
```

Then use it via:
```bash
treegen --template my_template
```
---
## License

This project is licensed under the MIT License

---
