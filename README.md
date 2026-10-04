# Shae Morph

[![Build & Test](https://img.shields.io/badge/tests-11%20passed-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

**Shae Morph** is a high-speed domain-specific language engineered in Rust for **structural data transformation and JSON plumbing**.

Instead of writing verbose structs in Go/Rust, handling brittle `undefined` errors in JavaScript, or parsing cryptic `jq` expressions, Shae Morph provides a clean, declarative syntax designed specifically for reshaping API payloads and JSON data pipelines.

> 💡 **Note on Naming**: **Shae Morph** is the specialized data-plumbing DSL. The upcoming general-purpose language for web applications, servers, and systems will be named **Shae**.

---

## Features

- 🛡️ **Zero Null Panics**: Safe path traversal (`.user.meta.age ?? 20`). Missing keys and nulls seamlessly cascade to fallback values without throwing runtime errors.
- 🔁 **Implicit Iteration & Filtering**: Map and filter arrays (`from .items[] where .age >= 18 select { ... }`) without writing manual loops.
- 🧰 **Built-in Transformations**: Fluent method chaining (`.trim().uppercase()`, `.toInt()`, `.split(",")`, `.join(" | ")`).
- 🚰 **Native Unix Piping**: Accepts JSON from `stdin` and writes formatted or compact JSON to `stdout`.
- ⚡ **Zero-Dependency Native Speed**: Compiles to a single 1.3 MB native executable in Rust.

---

## Installation & Build

```bash
# Clone and build
git clone https://github.com/spdedsec/shae-morph.git
cd shae-morph
cargo build --release

# The compiled binary is located at target/release/shae-morph
```

To install directly to your local Cargo path (`~/.cargo/bin`):
```bash
cargo install --path .
```
*(Provides both `shae-morph` and `morph` commands)*

---

## Quick Start

### 1. Define a transformation script (`examples/phase3_showcase.shae`)
```shae
out ActiveContributors = from .payload.events[] 
    where .type == "PushEvent" and .actor.contributions >= 5 and not .actor.bot 
    select {
        contributor: .actor.login.trim().capitalize(),
        badge: "Top Contributor (" + .actor.contributions.toString() + " commits)",
        recent_repo: .repo.name,
        contact: .actor.email ?? "no-public-email"
    }
```

### 2. Run against a JSON payload
```bash
shae-morph run examples/phase3_showcase.shae examples/github_events.json
```

Output:
```json
[
  {
    "contributor": "Spdedsec",
    "badge": "Top Contributor (42 commits)",
    "recent_repo": "spdedsec/shae",
    "contact": "satya@dev.io"
  },
  {
    "contributor": "Alex_coder",
    "badge": "Top Contributor (8 commits)",
    "recent_repo": "spdedsec/shae",
    "contact": "no-public-email"
  }
]
```

### 3. Piping via Stdin
```bash
cat examples/stripe.json | shae-morph run examples/stripe_to_db.shae --compact
```

---

## Running Tests

```bash
cargo test
```

---

## Documentation

- [LANGUAGE.md](LANGUAGE.md) — The Shae Morph Language Constitution and Specification.
