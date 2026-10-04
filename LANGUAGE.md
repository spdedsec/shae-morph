# The Shae Programming Language

> **A modern language for structural data transformation and JSON plumbing.**

---

## 1. Mission
Shae exists to make structural data transformation as natural as breathing. It is engineered for a world where modern systems, microservices, and serverless runtimes communicate through massive, deeply nested payloads.

---

## 2. The Problems Shae Solves

| Existing Approach | The Frustration | The Shae Solution |
| :--- | :--- | :--- |
| **Rust / Go / Java** | Writing 50+ lines of rigid `struct` or `interface` definitions just to pluck 3 fields from a 500-field JSON payload. | **Shape over Name**: You define the output shape you want; unneeded incoming fields are ignored automatically without ceremony. |
| **JavaScript / Python** | Concise property access, but type-blind. Accessing `.user.meta.age` throws `TypeError: Cannot read property of undefined` in production. | **Null is a State, Not a Panic**: Missing intermediate paths resolve safely to `Empty` and fall back cleanly via `??`. Zero runtime crashes. |
| **`jq`** | Dedicated CLI tool, but unreadable syntax (`.users[] \| select(.age > 20) \| {id: .id}`) that turns into write-only line noise. | **Human-Readable Grammar**: Clean, declarative pipeline syntax that visually mirrors the resulting JSON output. |

---

## 3. Core Principles

1. **Shape Over Name**  
   Data is defined by what it looks like, not what it is formally named. There are no nominal classes, inheritance hierarchies, or polymorphic dispatch.

2. **Null is a State, Not an Exception**  
   Missing data is normal in distributed systems. Missing paths or explicit `null`s seamlessly evaluate to `Empty` and cascade to user-defined fallback values (`??`).

3. **Implicit Iteration**  
   When mapping an array of items, Shae handles the iteration loop automatically (`from .items[] select { ... }`). You don't write manual `for` loops for standard projections.

4. **Self-Documenting Pipelines**  
   A Shae transformation file visually resembles the JSON output it produces. Opening a `.shae` file immediately tells you what data comes out.

5. **Tooling & Pipe First**  
   Shae is designed from day one to sit directly in Unix pipes (`curl ... | shae run transform.shae | db-loader`).

---

## 4. Deliberate Non-Goals (What Shae Deliberately Refuses to Be)

To remain fast, lightweight, and focused, Shae will **never** attempt to be:
* **A general-purpose application language**: No GUI frameworks, web servers, or game engines.
* **A systems programming language**: No manual pointer arithmetic, arena allocators, or hardware registers.
* **An object-oriented language**: No classes, prototypes, or inheritance.

---

## 5. Syntax & Semantics Quick Reference

### Safe Object Mapping
```shae
out Profile = {
    first_name: .user.name,
    age: .user.meta.age ?? 20,
    role: .user.role ?? "Member",
    is_active: true
}
```

### Implicit Array Mapping
```shae
out DatabaseRows = from .data.customers[] select {
    user_id: .id,
    full_name: .meta.name ?? "Unknown User",
    contact_email: .meta.email ?? "no-email@customer.io"
}
```

### CLI Usage
```bash
# Run with file input
shae run transform.shae input.json

# Run in Unix pipe (stdin)
cat input.json | shae run transform.shae

# Output compact JSON
cat input.json | shae run transform.shae --compact
```
