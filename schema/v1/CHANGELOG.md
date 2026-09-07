# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.22.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.21.0...schema-v1.22.0) - 2026-09-07

### Added

- *(unstable)* add session notices schema ([#2004](https://github.com/agentclientprotocol/agent-client-protocol/pull/2004))

## [1.21.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.20.0...schema-v1.21.0) - 2026-08-20

### Added

- *(unstable)* add session compaction updates ([#2002](https://github.com/agentclientprotocol/agent-client-protocol/pull/2002))
- *(schema)* stabilize terminal authentication ([#2000](https://github.com/agentclientprotocol/agent-client-protocol/pull/2000))
- *(schema)* stabilize elicitation ([#1779](https://github.com/agentclientprotocol/agent-client-protocol/pull/1779))

### Other

- *(rfd)* Move terminal auth to preview ([#1796](https://github.com/agentclientprotocol/agent-client-protocol/pull/1796))

## [1.20.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.19.1...schema-v1.20.0) - 2026-07-21

### Added

- *(unstable)* add tool call name ([#1752](https://github.com/agentclientprotocol/agent-client-protocol/pull/1752))

## [1.19.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.19.0...schema-v1.19.1) - 2026-07-20

### Fixed

- *(schema)* remove enum discriminators from invalid schemas ([#1612](https://github.com/agentclientprotocol/agent-client-protocol/pull/1612))

## [1.19.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.18.0...schema-v1.19.0) - 2026-07-06

### Added

- *(unstable)* Add descriptions to elicitation enum options ([#1397](https://github.com/agentclientprotocol/agent-client-protocol/pull/1397))

## [1.18.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.17.0...schema-v1.18.0) - 2026-07-06

### Added

- *(schema)* Stabilize boolean session config options ([#1604](https://github.com/agentclientprotocol/agent-client-protocol/pull/1604))
- *(unstable-v2)* Unify the ID naming conventions across the schema ([#1567](https://github.com/agentclientprotocol/agent-client-protocol/pull/1567))

### Fixed

- *(schema)* Reject malformed protocol fields ([#1583](https://github.com/agentclientprotocol/agent-client-protocol/pull/1583))
- *(unstable)* remove URL elicitation error ([#1574](https://github.com/agentclientprotocol/agent-client-protocol/pull/1574))
- *(unstable-v2)* Continue to make more enums future compatible ([#1571](https://github.com/agentclientprotocol/agent-client-protocol/pull/1571))

### Other

- *(schema)* Clean up generated documentation and make wording more consistent ([#1568](https://github.com/agentclientprotocol/agent-client-protocol/pull/1568))

## [1.17.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.16.0...schema-v1.17.0) - 2026-06-29

### Added

- *(schema)* Stabilize request cancellation ([#1549](https://github.com/agentclientprotocol/agent-client-protocol/pull/1549))

### Fixed

- Deserialization leniency part 2 ([#1526](https://github.com/agentclientprotocol/agent-client-protocol/pull/1526))
- *(unstable-v2)* make mcpServers optional in new sessions ([#1523](https://github.com/agentclientprotocol/agent-client-protocol/pull/1523))

### Other

- *(unstable-v2)* Clean up client schema types ([#1540](https://github.com/agentclientprotocol/agent-client-protocol/pull/1540))
- *(schema)* correct Implementation description ([#1518](https://github.com/agentclientprotocol/agent-client-protocol/pull/1518))

## [1.16.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.15.0...schema-v1.16.0) - 2026-06-24

### Added

- *(schema)* Stabilize model_config option category ([#1502](https://github.com/agentclientprotocol/agent-client-protocol/pull/1502))

## [1.15.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.14.0...schema-v1.15.0) - 2026-06-24

### Added

- *(unstable)* Add boolean config option capabilities ([#1490](https://github.com/agentclientprotocol/agent-client-protocol/pull/1490))

## [1.14.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/schema-v1.13.7...schema-v1.14.0) - 2026-06-18

### Added

- Add unstable model config category ([#1455](https://github.com/agentclientprotocol/agent-client-protocol/pull/1455))

### Other

- Clean up missing documentation on various schemas and builder ([#1454](https://github.com/agentclientprotocol/agent-client-protocol/pull/1454))
- various cleanups ([#1453](https://github.com/agentclientprotocol/agent-client-protocol/pull/1453))
- Clarify schema release versioning docs ([#1443](https://github.com/agentclientprotocol/agent-client-protocol/pull/1443))

## [1.13.7](https://github.com/agentclientprotocol/agent-client-protocol/releases/tag/schema-v1.13.7) - 2026-06-16

### Fixed

- *(schema)* Add missing _meta fields to protocol schemas ([#1440](https://github.com/agentclientprotocol/agent-client-protocol/pull/1440))
- *(rust)* Preserve JSON object key order ([#1393](https://github.com/agentclientprotocol/agent-client-protocol/pull/1393))

### Other

- Fix versioning for new schema git releases ([#1382](https://github.com/agentclientprotocol/agent-client-protocol/pull/1382))
- Setup separate publishes for JSON Schemas ([#1377](https://github.com/agentclientprotocol/agent-client-protocol/pull/1377))
