# Shae Morph (`shae-morph`)

<div align="center">

```
   ____  _                   __  __                 _     
  / ___|| |__   __ _   ___  |  \/  | ___  _ __ _ __ | |__  
  \___ \| '_ \ / _` | / _ \ | |\/| |/ _ \| '__| '_ \| '_ \ 
   ___) | | | | (_| ||  __/ | |  | | (_) | |  | |_) | | | |
  |____/|_| |_|\__,_| \___| |_|  |_|\___/|_|  | .__/|_| |_|
                                              |_|          
```

**Data plumbing without the tears.**  
*A modern, crash-proof, domain-specific programming language engineered in Rust for structural data transformation and JSON reshaping.*

[![CI](https://github.com/spdedsec/shae-morph/actions/workflows/ci.yml/badge.svg)](https://github.com/spdedsec/shae-morph/actions)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/spdedsec/shae-morph)](https://github.com/spdedsec/shae-morph/releases)

</div>

---

> 💡 **Shae vs. Shae Morph**:  
> **Shae Morph** is the specialized data-plumbing DSL for surgical JSON transformation and API pipeline wrangling.  
> **Shae** (coming soon) is the upcoming general-purpose programming language for building web servers (`shae serve`), fullstack applications, and backend systems.

---

## ⚡ The Problem: Why Does Shae Morph Exist?

Modern distributed systems communicate through massive, deeply nested JSON payloads. Today's tools make working with them agonizing:

| Tool / Language | The Frustration | The Shae Morph Answer |
| :--- | :--- | :--- |
| **`jq`** | Cryptic, unreadable "alphabet soup" syntax (`.[] \| select(...) \| { ... }`). Difficult to remember, impossible to maintain. | **Natural, human-readable grammar** inspired by SQL and modern JavaScript. If you know English, you can read it. |
| **JavaScript / Python** | Concise, but type-blind. Accessing `.user.meta.age` throws `TypeError: Cannot read property of undefined` at 3 AM in production. | **Zero null panics**. Missing paths and explicit nulls evaluate safely to `Empty` and cascade to defaults via `??`. |
| **Go / Rust / Java** | High-ceremony structs. You have to write 50+ lines of rigid DTOs just to pluck 3 fields from a 500-field JSON response. | **Shape over Name**. Define only the fields you want. Excess incoming payload is ignored automatically. |
| **JMESPath / JSONata** | Either too underpowered (no string/number casting) or slow and memory-heavy (node/JS runtimes). | **Native Rust speed** (1.3 MB standalone binary) with rich method chaining built right into the language. |

---

## 🚀 Key Features

* 🛡️ **Guaranteed Null Safety**: The `??` operator seamlessly handles missing keys or `null` values without throwing exceptions.
* 🔁 **Implicit Iteration with `where` Filters**: Project and filter lists without manual `for` loops (`from .items[] where .active select { ... }`).
* 🧰 **Fluent Method Chaining**: Built-in methods for strings (`.trim().uppercase()`, `.toInt()`, `.split()`), arrays (`.first()`, `.last()`, `.join()`), and numbers.
* 🚰 **Native Unix Pipes**: Works seamlessly in CLI toolchains (`curl -s ... | shae-morph run script.shae | db-loader`).
* ⚡ **Ultra-Lean Binary**: Zero external dependencies at runtime, starts up in <5ms.

---

## 📦 Quick Installation

### Option 1: Download Pre-compiled Binary
Grab the latest standalone binary for Linux x86_64 from the [GitHub Releases](https://github.com/spdedsec/shae-morph/releases):
```bash
curl -sSL https://github.com/spdedsec/shae-morph/releases/latest/download/shae-morph-v0.1.0-linux-x86_64.tar.gz | tar -xz
sudo mv shae-morph /usr/local/bin/
```

### Option 2: Build from Source (Cargo)
```bash
git clone https://github.com/spdedsec/shae-morph.git
cd shae-morph
cargo install --path .
```
*(Installs both `shae-morph` and the `morph` alias into `~/.cargo/bin`)*

---

## 🛠️ Quick Start

### 1. Simple Reshaping with Safe Fallbacks (`transform.shae`)
```shae
out Profile = {
    full_name: .user.name,
    age: .user.meta.age ?? 20,
    role: .user.role ?? "Member",
    is_active: true
}
```

Run it against a JSON file:
```bash
shae-morph run transform.shae input.json
```

Given:
```json
{
  "user": {
    "name": "Satya"
  }
}
```

Output:
```json
{
  "full_name": "Satya",
  "age": 20.0,
  "role": "Member",
  "is_active": true
}
```
*(Notice `.user.meta.age` and `.user.role` were completely missing, yet the script completed without errors, cleanly applying fallbacks.)*

---

### 2. Filtering & Array Projections with `where`
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

Input:
```json
{
  "payload": {
    "events": [
      {
        "type": "PushEvent",
        "actor": { "login": "spdedsec", "contributions": 42, "bot": false, "email": "satya@dev.io" },
        "repo": { "name": "spdedsec/shae-morph" }
      },
      {
        "type": "PushEvent",
        "actor": { "login": "dependabot", "contributions": 100, "bot": true },
        "repo": { "name": "spdedsec/shae-morph" }
      },
      {
        "type": "PushEvent",
        "actor": { "login": "alex_coder", "contributions": 8, "bot": false },
        "repo": { "name": "spdedsec/shae-morph" }
      }
    ]
  }
}
```

Output:
```json
[
  {
    "contributor": "Spdedsec",
    "badge": "Top Contributor (42 commits)",
    "recent_repo": "spdedsec/shae-morph",
    "contact": "satya@dev.io"
  },
  {
    "contributor": "Alex_coder",
    "badge": "Top Contributor (8 commits)",
    "recent_repo": "spdedsec/shae-morph",
    "contact": "no-public-email"
  }
]
```

---

### 3. Unix Piping (`curl` / `stdin`)
Shae Morph detects pipes automatically:
```bash
# Pipe directly into Shae Morph and produce compact JSON
curl -s https://api.github.com/repos/spdedsec/shae-morph | shae-morph run script.shae --compact
```

---

## 📖 Standard Library & Method Reference

### String Methods
| Method | Description | Example |
| :--- | :--- | :--- |
| `.uppercase()` | Converts to uppercase | `"hello".uppercase() => "HELLO"` |
| `.lowercase()` | Converts to lowercase | `"HELLO".lowercase() => "hello"` |
| `.trim()` | Strips leading & trailing whitespace | `"  hi ".trim() => "hi"` |
| `.capitalize()` | Capitalizes first letter | `"satya".capitalize() => "Satya"` |
| `.len()` | Character count | `"word".len() => 4` |
| `.toInt()` | Parses integer from string | `"42".toInt() => 42` |
| `.toNumber()` | Parses float / number | `"3.14".toNumber() => 3.14` |
| `.split(delimiter)` | Splits string into array | `"a,b,c".split(",") => ["a","b","c"]` |
| `.replace(from, to)`| Replaces substrings | `"hello".replace("l","w") => "hewwo"` |
| `.contains(sub)` | Checks if substring exists | `"hello".contains("ell") => true` |
| `.startsWith(prefix)`| Checks prefix | `"shae".startsWith("sh") => true` |
| `.endsWith(suffix)` | Checks suffix | `"shae".endsWith("ae") => true` |

### Number Methods
| Method | Description | Example |
| :--- | :--- | :--- |
| `.toString()` | Converts number to string | `(42).toString() => "42"` |
| `.toInt()` | Floors to integer | `(3.99).toInt() => 3` |
| `.round()` | Rounds to nearest whole | `(3.5).round() => 4` |
| `.floor()` | Floor value | `(3.9).floor() => 3` |
| `.ceil()` | Ceiling value | `(3.1).ceil() => 4` |
| `.abs()` | Absolute value | `(-10).abs() => 10` |

### Array Methods
| Method | Description | Example |
| :--- | :--- | :--- |
| `.len()` | Item count | `[1, 2, 3].len() => 3` |
| `.first()` | Returns first element | `[1, 2, 3].first() => 1` |
| `.last()` | Returns last element | `[1, 2, 3].last() => 3` |
| `.reverse()` | Reverses array | `[1, 2, 3].reverse() => [3, 2, 1]` |
| `.join(delimiter)` | Joins elements into string | `["a", "b"].join("-") => "a-b"` |

### Object Methods
| Method | Description | Example |
| :--- | :--- | :--- |
| `.keys()` | Returns array of object keys | `{ a: 1, b: 2 }.keys() => ["a", "b"]` |
| `.values()` | Returns array of object values | `{ a: 1, b: 2 }.values() => [1, 2]` |
| `.len()` | Returns number of keys | `{ a: 1, b: 2 }.len() => 2` |

---

## 💡 Pro-Tips & CLI Commands

```bash
# Get a friendly pro-tip from the terminal
shae-morph --tip

# Output compact (minified) JSON instead of formatted
shae-morph run script.shae input.json --compact

# Show help & options
shae-morph --help
```

---

## 🏗️ Architecture Under the Hood

Shae Morph is built with a clean, classic compiler architecture:

```
Source (.shae) ──> [ Lexer ] ──> [ Pratt Parser ] ──> [ AST ] ──> [ Evaluator ] ──> Output JSON
                                                                        ▲
Input JSON Payload ─────────────────────────────────────────────────────┘
```

1. **`src/token.rs` & `src/lexer.rs`**: Tokenizer with accurate line/column span tracking and friendly error hints.
2. **`src/ast.rs` & `src/parser.rs`**: Recursive descent parser combined with Pratt precedence climbing for binary operators and method calls.
3. **`src/eval.rs`**: Tree-walking runtime with safe path resolution (`ShaeValue::Empty`) and standard method dispatch.
4. **`src/main.rs`**: CLI runner with terminal stream detection and piping support.

---

## 🧪 Running Tests

```bash
cargo test
```

Includes 10 comprehensive unit and integration test suites:
- Lexer tokenization & escapes
- Parser precedence & `where` clauses
- Evaluator safe path traversal & fallbacks
- String, number, and array method chaining
- Array projections and filtering

---

## 📜 Constitution

For the official language manifesto, design rationale, and non-goals, read [LANGUAGE.md](LANGUAGE.md).

---

## 📄 License

Distributed under the [MIT License](LICENSE). Copyright (c) 2026 Satya Prakash (spdedsec).
