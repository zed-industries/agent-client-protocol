# Changelog

## 0.4.5 (2025-10-02)

### Protocol

- No changes

### Typescript

- **Unstable** initial support for model selection.

## 0.4.4 (2025-09-30)

### Protocol

- No changes

### Rust

- Provide default trait implementations for optional capability-based `Agent` and `Client` methods.

### Typescript

- Correctly mark capability-based `Agent` and `Client` methods as optional.

## 0.4.3 (2025-09-25)

### Protocol

- Defined `Resource not found` error type as code `-32002` (same as MCP)

### Rust

- impl `Agent` and `Client` for `Rc<T>` and `Arc<T>` where `T` implements either trait.

## 0.4.2 (2025-09-22)

### Rust

**Unstable** fix missing method for model selection in Rust library.

## 0.4.1 (2025-09-22)

### Protocol

**Unstable** initial support for model selection.

## 0.4.0 (2025-09-17)

### Protocol

No changes.

### Rust Library

- Make `Agent` and `Client` dyn compatible (you'll need to annotate them with `#[async_trait]`) [#97](https://github.com/zed-industries/agent-client-protocol/pull/97)
- `ext_method` and `ext_notification` methods are now more consistent with the other trait methods [#95](https://github.com/zed-industries/agent-client-protocol/pull/95)
  - There are also distinct types for `ExtRequest`, `ExtResponse`, and `ExtNotification`
- Rexport `serde_json::RawValue` for easier use [#95](https://github.com/zed-industries/agent-client-protocol/pull/95)

### Typescript Library

- Use Stream abstraction instead of raw byte streams [#93](https://github.com/zed-industries/agent-client-protocol/pull/93)
  - Makes it easier to use with websockets instead of stdio
- Improve type safety for method map helpers [#94](https://github.com/zed-industries/agent-client-protocol/pull/94)
