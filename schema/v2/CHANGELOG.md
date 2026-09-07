# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0-alpha.4](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.3...schema-v2.0.0-alpha.4) - 2026-09-07

### Added

- *(unstable)* add session notices schema ([#2004](https://github.com/agentclientprotocol/agent-client-protocol/pull/2004))

## [2.0.0-alpha.3](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.2...schema-v2.0.0-alpha.3) - 2026-08-20

### Added

- *(unstable)* add session compaction updates ([#2002](https://github.com/agentclientprotocol/agent-client-protocol/pull/2002))
- *(schema)* stabilize terminal authentication ([#2000](https://github.com/agentclientprotocol/agent-client-protocol/pull/2000))
- *(schema)* stabilize elicitation ([#1779](https://github.com/agentclientprotocol/agent-client-protocol/pull/1779))

### Other

- *(rfd)* Move terminal auth to preview ([#1796](https://github.com/agentclientprotocol/agent-client-protocol/pull/1796))

## [2.0.0-alpha.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.1...schema-v2.0.0-alpha.2) - 2026-07-21

### Added

- *(unstable)* add tool call name ([#1752](https://github.com/agentclientprotocol/agent-client-protocol/pull/1752))
- *(unstable-v2)* add cancelled variant to tool call and plan item ([#1750](https://github.com/agentclientprotocol/agent-client-protocol/pull/1750))

## [2.0.0-alpha.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v2.0.0-alpha.0...schema-v2.0.0-alpha.1) - 2026-07-20

### Added

- *(unstable-v2)* add v2 semantic string types ([#1742](https://github.com/agentclientprotocol/agent-client-protocol/pull/1742))
- *(unstable-v2)* rename diff patch payload to text ([#1730](https://github.com/agentclientprotocol/agent-client-protocol/pull/1730))
- *(unstable-v2)* add v2 terminal output surface ([#1679](https://github.com/agentclientprotocol/agent-client-protocol/pull/1679))

### Fixed

- *(unstable-v2)* gate v2 auth methods on authMethods ([#1728](https://github.com/agentclientprotocol/agent-client-protocol/pull/1728))
- *(schema)* remove enum discriminators from invalid schemas ([#1612](https://github.com/agentclientprotocol/agent-client-protocol/pull/1612))

### Other

- *(unstable-v2)* clarify idle session semantics ([#1729](https://github.com/agentclientprotocol/agent-client-protocol/pull/1729))
- enable schema v2 releases ([#1632](https://github.com/agentclientprotocol/agent-client-protocol/pull/1632))
