# Agent Client Protocol

The Agent Client Protocol (ACP) standardizes communication between _code editors_ (interactive programs for viewing and editing source code) and _coding agents_ (programs that use generative AI to autonomously modify code).

The protocol is still under heavy development, and we aim to mature it as we get confidence in the design by implementing it in various settings.

- The official ACP documentation is available in [agentclientprotocol.com](https://agentclientprotocol.com/) and it's built using [Mintlify](http://mintlify.com/).
- The schema is defined in [acp.rs](./rust/acp.rs), and a TypeScript definition is generated to [acp.ts](./typescript/acp.ts).

## Contributing

ACP is a protocol intended for broad adoption across the ecosystem; we follow a structured process to ensure changes are well-considered.

- Bug Reports: If you notice a bug in the protocol, please file an [issue](new?template=05_bug_report.yml) and we will be in touch.
- Protocol Suggestions: If you'd like to propose additions or changes to the protocol, please start a [discussion](https://github.com/zed-industries/agent-client-protocol/discussions/categories/protocol-suggestions) first. We want to make sure suggestions proposed align well with the project. If accepted, we can have a conversation around how these changes or additions can be implemented. Once that is complete, we can create an issue for pull requests to target.

## Pull Requests

Pull requests should intend to close an existing [issue](https://github.com/zed-industries/agent-client-protocol/issues).
