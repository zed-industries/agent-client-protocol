# Agent Client Protocol

The Agent Client Protocol (ACP) is a protocol that standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

The protocol is still under heavy development, and we aim to standardize it as
we get confidence in the design by implementing it in various settings.

## Details

The schema is defined in [acp.rs](./rust/acp.rs), and a type-script definition is generated to [acp.ts](./typescript/acp.ts).

This repo also contains interoperable implementations of the protocol for both Typescript and Rust.
