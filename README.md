# Shae

[![Build & Test](https://img.shields.io/badge/tests-11%20passed-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

**Shae** is a domain-specific programming language engineered in Rust for **structural data transformation and JSON plumbing**.

Instead of writing verbose structs in Go/Rust, handling brittle `undefined` errors in JavaScript, or parsing cryptic `jq` expressions, Shae provides a clean, declarative syntax designed specifically for reshaping API payloads.

---

## Features

- 🛡️ **Zero Null Panics**: Safe path traversal (`.user.meta.age ?? 20`). Missing keys and nulls seamlessly cascade to fallback values without throwing runtime errors.
- 🔁 **Implicit Iteration**: Map arrays of objects (`from .items[] select { ... }`) without writing manual `for` loops.
- 🚰 **Native Unix Piping**: Accepts JSON from `stdin` and writes formatted or compact JSON to `stdout`.
- ⚡ **Native Performance**: Compiles to a single zero-dependency native executable in Rust.

---

## Installation & Build

```bash
# Clone and build
git clone https://github.com/spdedsec/shae.git
cd shae
cargo build --release

# The compiled binary is located at target/release/shae
```

To install directly to your local Cargo path (`~/.cargo/bin`):
```bash
cargo install --path .
```

---

## Quick Start

### 1. Define a transformation script (`examples/basic.shae`)
```shae
out Profile = {
    first_name: .user.name,
    age: .user.meta.age ?? 20,
    role: .user.role ?? "Member",
    is_active: true
}
```

### 2. Run against a JSON payload
```bash
shae run examples/basic.shae examples/input.json
```

Output:
```json
{
  "first_name": "Satya",
  "age": 20.0,
  "role": "Member",
  "is_active": true
}
```

### 3. Piping via Stdin
```bash
cat examples/stripe.json | shae run examples/stripe_to_db.shae --compact
```

---

## Running Tests

```bash
cargo test
```

Includes:
- Lexer unit tests (tokens, escape characters, numbers)
- Parser unit tests (operator precedence, object and array parsing)
- Evaluator unit tests (safe path traversal, fallbacks)
- End-to-end integration tests (`tests/integration.rs`)

---

## Documentation

- [LANGUAGE.md](LANGUAGE.md) — The Shae Language Constitution and Specification.
