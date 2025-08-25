# Agent Client Protocol

The Agent Client Protocol (ACP) standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

The protocol is still under heavy development, and we aim to mature it as we get confidence in the design by implementing it in various settings.

The official ACP documentation is available in [agentclientprotocol.com](https://agentclientprotocol.com/) and it's built using [Mintlify](http://mintlify.com/).

## Details

The schema is defined in [acp.rs](./rust/acp.rs), and a TypeScript definition is generated to [acp.ts](./typescript/acp.ts).
