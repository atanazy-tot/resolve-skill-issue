---
kind: shared-reference
name: lang-rust
description: >
  Canonical's Rust best practices, distilled: error and panic discipline,
  functional core with pattern matching, type-system-first design, and the
  structural, naming, and import conventions for production Rust.
tags: [lang-guide]
---
# Rust language guide (Canonical best practices, distilled)

Distilled from <https://canonical.github.io/rust-best-practices/> (14 chapters).
Where this file and the live guide disagree, the guide wins — update this file.

## Contents

- Preconditions
- Error and panic discipline
- Structural discipline
- Pattern matching discipline
- Code discipline
- Function discipline
- Ordering discipline
- Naming discipline
- Import discipline
- Comment discipline
- Unsafe discipline
- Cosmetic discipline

## Preconditions

- All code passes `cargo fmt`, `cargo clippy`, and `cargo clippy --tests` (with
  `--all-features` when features exist). Zero warnings before merging.

## Error and panic discipline

The highest-value chapter. Write the `Error` type first.

- Use a concrete enumerated error type (`#[derive(thiserror::Error)]`). Never
  type-erased errors (`Box<dyn Error>`, `anyhow`) in libraries; they are for
  prototypes only.
- Messages: concise, consistent form, start with a verb — usually "cannot …".
  Lowercase first letter (errors get wrapped); capitals only for acronyms and
  proper names (TCP, NixOS).
- Wrapping: inner error in a field named `source`. Unrecoverable causes in a
  field named `reason: String`. Hide dependency error types behind an
  `Internal(#[from] InternalError)` transparent variant.
- Convert foreign errors into the crate's `Error` at the earliest reasonable
  opportunity — inside the call chain, not after it.
- Never panic on user input. Avoid `.unwrap()`; replace with `?`, `.ok_or` /
  `.ok_or_else` for options, or `if let Ok(x) = …` / `if let Some(x) = x`
  instead of `is_ok`/`is_some` guards. `.expect("precondition")` only to
  document programmer preconditions. `.unwrap()` acceptable in tests (panic
  trace pinpoints the failure) and for trivially-infallible constants.
- Panic messages signal fault: internal bugs start with "internal error:".

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot access {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },

    #[error("invalid {credential}: {reason}")]
    InvalidCredentials { credential: String, reason: String },
}
```

## Structural discipline

- `mod.rs` declares structure only: `pub mod`, `pub(crate) mod`, `mod`,
  `pub use` (most public first; `#[cfg]`-gated items last, in a block). No
  definitions. Prefer `foo/mod.rs` over `foo.rs` + `foo/`.
- Library crates: `Error` in `lib.rs` below the mod/use declarations, with
  `type Result<T> = std::result::Result<T, Error>;` immediately below.
- Binary crates: `Error` in `error.rs`, the `Result` alias in `result.rs`.
- Keep remote/serde shapes out of core types: model foreign APIs with local
  1:1 serde types at the bottom of the function under a `// serde types.`
  marker, then convert into crate-owned types.

## Pattern matching discipline

- Match exhaustively on internal types — let future fields cause compiler
  errors. Destructure structs in `impl Ord`-style code (`let Self { a, b, .. } = self;`)
  rather than field access, so new fields force a decision.
- Name matched bindings after their source field (or its first letter).
- No numeric tuple indexing outside newtype impl blocks — destructure for names.
- Don't pattern-match references to `Copy` types; dereference explicitly (`|x| *x`).
- No destructuring in `fn` signatures; unpack on the function's first line.

## Code discipline

- Use `Self` wherever possible (not to construct associated types — use
  concrete types in implementations).
- Struct population: compute values into `let` bindings first, then populate
  with field-init shorthand, in declaration order, one independent line per
  field. All fields computed or none — never mix inline computation with shorthand.
- Prefer `.collect()` over `Foo::from_iter(...)`. Empty vecs via `Vec::new()`.
- Scoped mutability: `let x = { let mut x = X::new(); /* build */ x };` — same
  name inside and out. Replace `let mut` counter loops with iterator chains
  (`.filter(..).count()`).
- No unassigned `let` declarations — return values from blocks (`let m = if … { a } else { b };`,
  `let m = loop { … break v; };`).
- `let foo = compute(); use(&foo);` — not `let foo = &compute();`.
- Shadowing: one level max in nested scopes (`if let Some(x) = x`); type-changing
  shadowing at most once per variable; never shadow types.
- Minimal type annotations: `Vec<_>` on `let` bindings beats turbofish, which
  beats fully-qualified syntax. If a parameter name needs a type suffix to be
  clear, improve the name.
- No explicit `drop`; use a scope. Discard values with `|_| ()`; ignore errors
  visibly via `.ok()`; end `Result<()>` functions with `?` + `Ok(())`.
- Generics: constraints live together — if any need a `where` clause (or the
  angle brackets exceed ~30 chars), move them all into `where`.
- Model objects get constructors or builders, not public fields. Data-transfer
  structs may use public fields (+ `#[non_exhaustive]` when extensible).
- No method calls on closing `}` — bind first (`let value = if …; value.f()`).

## Function discipline

- Unit-returning and never-returning calls end with `;`. Do-nothing match arms
  are `.. => {}`, never `.. => ()`.
- Hide generic parameters: `impl Trait` for single-use type parameters, `'_`
  for elided lifetimes (never omit `'_` entirely). Minimal constraints on
  `impl` blocks.
- Unused parameters in default trait impls: `let _ = param;` in the body.
- Builders: `Type::builder()` returns `TypeBuilder` (no public builder
  constructor); consuming `self` methods; fallible `.build() -> Result<Type>`.

## Ordering discipline

- Files read top-down as an API tour: important items first, helpers below.
  `impl` blocks immediately below their type; inherent impl before trait impls
  (std, then own, then third-party; unsafe variants first within each).
- One `#[derive(...)]`, ordered: `Copy` first, std traits lexicographically,
  then third-party.
- Declarations ordered: `const`, `static`, `let`, `let mut`.
- Struct fields ordered: `pub`, `pub(crate)`, private. Hand-write `Ord` impls
  rather than reorder fields for derive.

## Naming discipline

- UK spelling (Canonical policy). Say what it means; consistent word order
  (`verb_noun` if that's the house pattern); concise; simple correct words;
  one name per concept; no type names in variable names.
- Generic type parameters: single letters. Lifetimes: meaningful short names
  tied to the data (`'cursor`, `'tree`, `'h`) — not `'a`.
- If a good name can't be found, suspect the API, not the thesaurus.

## Import discipline

- No `*` imports and no preludes in production code (`use super::*;` allowed in
  test modules only). No `use Enum::*` — rename locally inside the function if
  needed (`use TaskStatus as Ts;`).
- Three blocks in order: std/core/alloc; third-party; self/super/crate.
- Nested import syntax (`use a::b::{C, D};`), not one path per line.
- `use self::foo::Foo;` when re-exporting from child modules.

## Comment discipline

- First doc sentence: at most two wrapped lines, explains the golden path and
  *when* to use the item, not just what it does.
- Refer to parameters by name with definite articles ("the given `delta`").

## Unsafe discipline

- Minimise `unsafe`; shrink blocks to the smallest possible scope even at the
  cost of extra lines. "Faster" alone never justifies it — only profiling does.
- Every `unsafe` fn/block carries a `// SAFETY:` comment documenting
  preconditions, maintained as code changes.

## Cosmetic discipline

- Blank lines are semantic: they separate strongly-associated blocks. A
  declaration belongs to the block that uses only it — no blank line between.
- Group intradependent code; define capturing closures near their use, or
  promote non-capturing closures to `fn`.
- Hex literals lowercase (`0xab5c…`).
