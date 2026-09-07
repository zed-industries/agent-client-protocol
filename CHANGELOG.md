# Changelog

## [1.8.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.7.0...v1.8.0) - 2026-09-07

### Added

- *(unstable)* add session notices schema ([#2004](https://github.com/agentclientprotocol/agent-client-protocol/pull/2004))

## [1.7.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.6.0...v1.7.0) - 2026-08-20

### Added

- *(unstable)* add session compaction updates ([#2002](https://github.com/agentclientprotocol/agent-client-protocol/pull/2002))
- *(schema)* stabilize terminal authentication ([#2000](https://github.com/agentclientprotocol/agent-client-protocol/pull/2000))
- *(rust)* make schemars optional ([#1939](https://github.com/agentclientprotocol/agent-client-protocol/pull/1939))
- *(schema)* stabilize elicitation ([#1779](https://github.com/agentclientprotocol/agent-client-protocol/pull/1779))

### Other

- *(unstable-v2)* remove conversion helpers ([#1809](https://github.com/agentclientprotocol/agent-client-protocol/pull/1809))
- *(rfd)* Move terminal auth to preview ([#1796](https://github.com/agentclientprotocol/agent-client-protocol/pull/1796))

## [1.6.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.5.0...v1.6.0) - 2026-07-21

### Added

- *(unstable)* add tool call name ([#1752](https://github.com/agentclientprotocol/agent-client-protocol/pull/1752))
- *(unstable-v2)* add cancelled variant to tool call and plan item ([#1750](https://github.com/agentclientprotocol/agent-client-protocol/pull/1750))

## [1.5.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.4.0...v1.5.0) - 2026-07-20

### Added

- *(unstable-v2)* add v2 semantic string types ([#1742](https://github.com/agentclientprotocol/agent-client-protocol/pull/1742))
- *(unstable-v2)* rename diff patch payload to text ([#1730](https://github.com/agentclientprotocol/agent-client-protocol/pull/1730))
- *(unstable-v2)* add v2 terminal output surface ([#1679](https://github.com/agentclientprotocol/agent-client-protocol/pull/1679))

### Fixed

- *(unstable-v2)* gate v2 auth methods on authMethods ([#1728](https://github.com/agentclientprotocol/agent-client-protocol/pull/1728))
- *(unstable-v2)* Clean up conversion helpers ([#1640](https://github.com/agentclientprotocol/agent-client-protocol/pull/1640))
- *(schema)* remove enum discriminators from invalid schemas ([#1612](https://github.com/agentclientprotocol/agent-client-protocol/pull/1612))

### Other

- *(unstable-v2)* clarify idle session semantics ([#1729](https://github.com/agentclientprotocol/agent-client-protocol/pull/1729))

## [1.4.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.3.0...v1.4.0) - 2026-07-06

### Added

- *(unstable)* Add descriptions to elicitation enum options ([#1397](https://github.com/agentclientprotocol/agent-client-protocol/pull/1397))

## [1.3.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.2.0...v1.3.0) - 2026-07-06

### Added

- *(schema)* Stabilize boolean session config options ([#1604](https://github.com/agentclientprotocol/agent-client-protocol/pull/1604))
- *(unstable-v2)* tighten v2 field validation ([#1587](https://github.com/agentclientprotocol/agent-client-protocol/pull/1587))
- *(unstable-v2)* New diff format ([#1586](https://github.com/agentclientprotocol/agent-client-protocol/pull/1586))
- *(unstable-v2)* Unify session/load and session/resume ([#1584](https://github.com/agentclientprotocol/agent-client-protocol/pull/1584))
- *(unstable-v2)* require more session methods by default ([#1578](https://github.com/agentclientprotocol/agent-client-protocol/pull/1578))
- *(unstable-v2)* More flexible permission requests ([#1577](https://github.com/agentclientprotocol/agent-client-protocol/pull/1577))
- *(unstable-v2)* Align v2 Content types with the latest MCP spec ([#1576](https://github.com/agentclientprotocol/agent-client-protocol/pull/1576))
- *(unstable-v2)* Unify the ID naming conventions across the schema ([#1567](https://github.com/agentclientprotocol/agent-client-protocol/pull/1567))
- *(unstable-v2)* require typed config values ([#1561](https://github.com/agentclientprotocol/agent-client-protocol/pull/1561))

### Fixed

- *(schema)* Reject malformed protocol fields ([#1583](https://github.com/agentclientprotocol/agent-client-protocol/pull/1583))
- *(unstable)* remove URL elicitation error ([#1574](https://github.com/agentclientprotocol/agent-client-protocol/pull/1574))
- *(unstable-v2)* Preserve meta update signals in v2 ([#1573](https://github.com/agentclientprotocol/agent-client-protocol/pull/1573))
- *(unstable-v2)* Continue to make more enums future compatible ([#1571](https://github.com/agentclientprotocol/agent-client-protocol/pull/1571))
- *(unstable-v2)* Make empty MCP arrays optional ([#1570](https://github.com/agentclientprotocol/agent-client-protocol/pull/1570))
- *(unstable-v2)* Make all session lifecycle requests consistent ([#1555](https://github.com/agentclientprotocol/agent-client-protocol/pull/1555))

### Other

- *(schema)* Clean up generated documentation and make wording more consistent ([#1568](https://github.com/agentclientprotocol/agent-client-protocol/pull/1568))
- *(rust)* Box v2 protocol enum variants ([#1562](https://github.com/agentclientprotocol/agent-client-protocol/pull/1562))

## [1.2.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.1.0...v1.2.0) - 2026-06-29

### Added

- *(schema)* Stabilize request cancellation ([#1549](https://github.com/agentclientprotocol/agent-client-protocol/pull/1549))
- *(unstable-v2)* use EnvVariable for terminal auth env ([#1522](https://github.com/agentclientprotocol/agent-client-protocol/pull/1522))
- *(unstable-v2)* Require auth logout support ([#1520](https://github.com/agentclientprotocol/agent-client-protocol/pull/1520))
- *(unstable-v2)* Group v2 auth methods under auth/* ([#1519](https://github.com/agentclientprotocol/agent-client-protocol/pull/1519))
- *(unstable-v2)* require v2 implementation info ([#1517](https://github.com/agentclientprotocol/agent-client-protocol/pull/1517))

### Fixed

- *(unstable-v2)* allow null auth capabilities ([#1539](https://github.com/agentclientprotocol/agent-client-protocol/pull/1539))
- Deserialization leniency part 2 ([#1526](https://github.com/agentclientprotocol/agent-client-protocol/pull/1526))
- *(unstable-v2)* make mcpServers optional in new sessions ([#1523](https://github.com/agentclientprotocol/agent-client-protocol/pull/1523))
- *(rust)* Clean up Protocol version handling ([#1515](https://github.com/agentclientprotocol/agent-client-protocol/pull/1515))

### Other

- *(unstable-v2)* reorder v2 content fields ([#1541](https://github.com/agentclientprotocol/agent-client-protocol/pull/1541))
- *(unstable-v2)* Clean up client schema types ([#1540](https://github.com/agentclientprotocol/agent-client-protocol/pull/1540))
- *(schema)* correct Implementation description ([#1518](https://github.com/agentclientprotocol/agent-client-protocol/pull/1518))

## [1.1.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v1.0.0...v1.1.0) - 2026-06-24

### Added

- *(schema)* Stabilize model_config option category ([#1502](https://github.com/agentclientprotocol/agent-client-protocol/pull/1502))

## [1.0.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.14.0...v1.0.0) - 2026-06-24

### Added

- *(unstable)* Add boolean config option capabilities ([#1490](https://github.com/agentclientprotocol/agent-client-protocol/pull/1490))

## [0.14.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.8...v0.14.0) - 2026-06-18

### Added

- *(rust)* Expose v1 schema types only under v1 module ([#1457](https://github.com/agentclientprotocol/agent-client-protocol/pull/1457))

## [0.13.8](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.7...v0.13.8) - 2026-06-18

### Other

- *(rust)* Move schema crate into workspace subdirectory ([#1456](https://github.com/agentclientprotocol/agent-client-protocol/pull/1456))
- various cleanups ([#1453](https://github.com/agentclientprotocol/agent-client-protocol/pull/1453))
- Clarify schema release versioning docs ([#1443](https://github.com/agentclientprotocol/agent-client-protocol/pull/1443))

## [0.13.7](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.6...v0.13.7) - 2026-06-16

### Added

- *(unstable-v2)* Add streaming tool-call content in v2 ([#1407](https://github.com/agentclientprotocol/agent-client-protocol/pull/1407))
- *(unstable-v2)* Add v2 whole-message session updates ([#1396](https://github.com/agentclientprotocol/agent-client-protocol/pull/1396))
- *(unstable-v2)* Unify v2 tool call updates ([#1390](https://github.com/agentclientprotocol/agent-client-protocol/pull/1390))
- *(unstable-v2)* Require auth method type discriminator in v2 ([#1387](https://github.com/agentclientprotocol/agent-client-protocol/pull/1387))
- *(unstable-v2)* Nest session-scoped capabilities under session ([#1373](https://github.com/agentclientprotocol/agent-client-protocol/pull/1373))

### Fixed

- *(rust)* Preserve JSON object key order ([#1393](https://github.com/agentclientprotocol/agent-client-protocol/pull/1393))
- *(schema)* Add missing _meta fields to protocol schemas ([#1440](https://github.com/agentclientprotocol/agent-client-protocol/pull/1440))
- *(unstable-v2)* Tolerate errors in v1/v2 conversions ([#1420](https://github.com/agentclientprotocol/agent-client-protocol/pull/1420))
- *(unstable-v2)* Clean up old agent auth deserialization ([#1388](https://github.com/agentclientprotocol/agent-client-protocol/pull/1388))

### Other

- Setup separate publishes for JSON Schemas ([#1377](https://github.com/agentclientprotocol/agent-client-protocol/pull/1377))

## [0.13.6](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.5...v0.13.6) - 2026-06-05

### Added

- *(schema)* Stabilize optional message IDs ([#1372](https://github.com/agentclientprotocol/agent-client-protocol/pull/1372))
- *(schema)* Stabilize session usage updates ([#1371](https://github.com/agentclientprotocol/agent-client-protocol/pull/1371))
- *(schema)* Stabilize session/delete ([#1370](https://github.com/agentclientprotocol/agent-client-protocol/pull/1370))
- *(unstable-v2)* Remove MCP SSE transport and make stdio opt-in ([#1368](https://github.com/agentclientprotocol/agent-client-protocol/pull/1368))
- *(unstable-v2)* Clean up capability objects ([#1367](https://github.com/agentclientprotocol/agent-client-protocol/pull/1367))
- *(unstable-v2)* Require message IDs in v2 chunks ([#1352](https://github.com/agentclientprotocol/agent-client-protocol/pull/1352))
- *(unstable-v2)* Adopt plan_update as v2 plan shape ([#1347](https://github.com/agentclientprotocol/agent-client-protocol/pull/1347))
- *(unstable-v2)* Remove v2 client filesystem and terminal surface ([#1346](https://github.com/agentclientprotocol/agent-client-protocol/pull/1346))

### Fixed

- *(unstable)* Fix plan capability key ([#1369](https://github.com/agentclientprotocol/agent-client-protocol/pull/1369))

### Other

- *(rfd)* Add JSON-RPC batch guidance for v2 ([#1348](https://github.com/agentclientprotocol/agent-client-protocol/pull/1348))
- *(rfd)* Split end-turn token usage from session usage ([#1345](https://github.com/agentclientprotocol/agent-client-protocol/pull/1345))
- *(rfd)* Clarify agent-owned message IDs in RFDs ([#1344](https://github.com/agentclientprotocol/agent-client-protocol/pull/1344))
- *(deps)* bump the minor group with 5 updates ([#1341](https://github.com/agentclientprotocol/agent-client-protocol/pull/1341))
- Render feature docs in docs.rs ([#1331](https://github.com/agentclientprotocol/agent-client-protocol/pull/1331))

## [0.13.5](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.4...v0.13.5) - 2026-06-01

### Added

- *(schema)* Experiment annotating lenient deserialize opportunities ([#1328](https://github.com/agentclientprotocol/agent-client-protocol/pull/1328))
- Stabilize additionalDirectories for sessions ([#1327](https://github.com/agentclientprotocol/agent-client-protocol/pull/1327))
- *(unstable)* Remove unstable session model API ([#1325](https://github.com/agentclientprotocol/agent-client-protocol/pull/1325))
- *(unstable-v2)* Remove dedicated session modes and models apis from v2 ([#1324](https://github.com/agentclientprotocol/agent-client-protocol/pull/1324))
- *(unstable)* Add v2 enum extension RFD and fallbacks ([#1304](https://github.com/agentclientprotocol/agent-client-protocol/pull/1304))

### Other

- Move existing protocol docs to v1 ([#1326](https://github.com/agentclientprotocol/agent-client-protocol/pull/1326))
- Add some draft v2 protocol docs and schema ([#1306](https://github.com/agentclientprotocol/agent-client-protocol/pull/1306))

## [0.13.4](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.3...v0.13.4) - 2026-05-27

### Added

- *(unstable)* Add unstable plan operations ([#1299](https://github.com/agentclientprotocol/agent-client-protocol/pull/1299))

## [0.13.3](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.2...v0.13.3) - 2026-05-22

### Added

- Stabilize logout method ([#1273](https://github.com/agentclientprotocol/agent-client-protocol/pull/1273))

### Fixed

- *(unstable)* Rename provider method types to singular ([#1272](https://github.com/agentclientprotocol/agent-client-protocol/pull/1272))

### Other

- *(rfd)* Move additional directories RFD to Preview ([#1276](https://github.com/agentclientprotocol/agent-client-protocol/pull/1276))
- Add schema download note to schema page ([#1269](https://github.com/agentclientprotocol/agent-client-protocol/pull/1269))
- *(deps)* bump num-conv from 0.2.1 to 0.2.2 in the minor group ([#1244](https://github.com/agentclientprotocol/agent-client-protocol/pull/1244))
- Set minimum supported Rust version ([#1232](https://github.com/agentclientprotocol/agent-client-protocol/pull/1232))
- Document ACP versioning semantics ([#1229](https://github.com/agentclientprotocol/agent-client-protocol/pull/1229))

## [0.13.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.1...v0.13.2) - 2026-05-17

### Fixed

- *(unstable)* Update additionalDirectories guidance ([#1227](https://github.com/agentclientprotocol/agent-client-protocol/pull/1227))

## [0.13.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.13.0...v0.13.1) - 2026-05-16

### Added

- *(unstable)* Add unstable session delete support ([#1216](https://github.com/agentclientprotocol/agent-client-protocol/pull/1216))

## [0.13.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.12.2...v0.13.0) - 2026-05-12

### Added

- *(unstable)* Add experimental MCP-over-ACP message types ([#1185](https://github.com/agentclientprotocol/agent-client-protocol/pull/1185))

### Other

- add unstable mcp-over-acp additions to the schema ([#1173](https://github.com/agentclientprotocol/agent-client-protocol/pull/1173))
- *(deps)* bump the minor group with 3 updates ([#1178](https://github.com/agentclientprotocol/agent-client-protocol/pull/1178))
- *(deps)* bump the minor group with 2 updates ([#1121](https://github.com/agentclientprotocol/agent-client-protocol/pull/1121))
- *(unstable)* Start setting up v2 Schema scaffolding for experimentation ([#1099](https://github.com/agentclientprotocol/agent-client-protocol/pull/1099))
- reorganize to v1 module ([#1094](https://github.com/agentclientprotocol/agent-client-protocol/pull/1094))

## [0.12.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.12.1...v0.12.2) - 2026-04-23

### Added

- Stabilize session/close ([#1062](https://github.com/agentclientprotocol/agent-client-protocol/pull/1062))
- Stabilize session/resume ([#1051](https://github.com/agentclientprotocol/agent-client-protocol/pull/1051))

## [0.12.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.12.0...v0.12.1) - 2026-04-21

### Other

- *(unstable)* Remove RequiredNullable dead code ([#1026](https://github.com/agentclientprotocol/agent-client-protocol/pull/1026))
- Optional current provider ([#1021](https://github.com/agentclientprotocol/agent-client-protocol/pull/1021))

## [0.12.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.7...v0.12.0) - 2026-04-17

### Added

- *(rust-only)* Remove unused RPC message schema types (schema.json unchanged) ([#1009](https://github.com/agentclientprotocol/agent-client-protocol/pull/1009))
- *(rust-only)* better tolerate malformed optional fields in deserialization ([#1006](https://github.com/agentclientprotocol/agent-client-protocol/pull/1006))

### Fixed

- *(rpc)* preserve '_' prefix for extension methods and reject empty ext ([#883](https://github.com/agentclientprotocol/agent-client-protocol/pull/883))

## [0.11.7](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.6...v0.11.7) - 2026-04-15

### Added

- *(unstable)* Initial implementation of providers ([#899](https://github.com/agentclientprotocol/agent-client-protocol/pull/899))

### Other

- *(rfd)* Move session/close to Preview ([#970](https://github.com/agentclientprotocol/agent-client-protocol/pull/970))
- *(rfd)* Move session/resume to Preview ([#969](https://github.com/agentclientprotocol/agent-client-protocol/pull/969))

## [0.11.6](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.5...v0.11.6) - 2026-04-14

### Fixed

- *(unstable)* Move elicitation scope into mode variants ([#966](https://github.com/agentclientprotocol/agent-client-protocol/pull/966))

## [0.11.5](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.4...v0.11.5) - 2026-04-09

### Added

- *(unstable)* elicitation for session, tool call, and requests ([#792](https://github.com/agentclientprotocol/agent-client-protocol/pull/792))

## [0.11.4](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.3...v0.11.4) - 2026-03-28

### Added

- *(unstable)* initial unstable nes implementation ([#754](https://github.com/agentclientprotocol/agent-client-protocol/pull/754))
- *(unstable)* initial additional directories implementation ([#838](https://github.com/agentclientprotocol/agent-client-protocol/pull/838))

### Other

- properly interpolate variables in generate.rs error messages ([#862](https://github.com/agentclientprotocol/agent-client-protocol/pull/862))
- add tests for content file ([#850](https://github.com/agentclientprotocol/agent-client-protocol/pull/850))
- Update README.md ([#836](https://github.com/agentclientprotocol/agent-client-protocol/pull/836))

## [0.11.3](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.2...v0.11.3) - 2026-03-18

### Added

- *(unstable)* More robust schema for elicitation types ([#771](https://github.com/agentclientprotocol/agent-client-protocol/pull/771))
- *(unstable)* initial implementation for the logout method capability ([#751](https://github.com/agentclientprotocol/agent-client-protocol/pull/751))
- *(rust-only)* Add meta getter for AuthMethod enum ([#725](https://github.com/agentclientprotocol/agent-client-protocol/pull/725))

### Other

- initial implementation: elicitation ([#769](https://github.com/agentclientprotocol/agent-client-protocol/pull/769))

## [0.11.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.1...v0.11.2) - 2026-03-11

### Fixed

- *(unstable)* Complete session/stop → session/close rename ([#724](https://github.com/agentclientprotocol/agent-client-protocol/pull/724))

### Other

- Update ecosystem docs for new clients and libraries ([#715](https://github.com/agentclientprotocol/agent-client-protocol/pull/715))

## [0.11.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.11.0...v0.11.1) - 2026-03-09

### Added

- *(unstable)* Remove unused auth_methods from Error type ([#708](https://github.com/agentclientprotocol/agent-client-protocol/pull/708))
- Stabilize session/list and session_info_update ([#705](https://github.com/agentclientprotocol/agent-client-protocol/pull/705))
- *(unstable)* Rename unstable session/stop method to session/close ([#701](https://github.com/agentclientprotocol/agent-client-protocol/pull/701))
- *(unstable)* Add config option type for boolean on/off toggles ([#576](https://github.com/agentclientprotocol/agent-client-protocol/pull/576))

### Other

- *(rfd)* Move initial registry RFD to completed ([#706](https://github.com/agentclientprotocol/agent-client-protocol/pull/706))
- *(deps)* bump quote from 1.0.44 to 1.0.45 in the minor group ([#698](https://github.com/agentclientprotocol/agent-client-protocol/pull/698))

## [0.11.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.8...v0.11.0) - 2026-03-04

### Added

- *(unstable)* implementation for unstable session/stop ([#583](https://github.com/agentclientprotocol/agent-client-protocol/pull/583))
- *(unstable)* implement message id rfd ([#581](https://github.com/agentclientprotocol/agent-client-protocol/pull/581))
- *(unstable)* Initial support for various auth methods ([#588](https://github.com/agentclientprotocol/agent-client-protocol/pull/588))

### Fixed

- Align struct naming and documentation ([#637](https://github.com/agentclientprotocol/agent-client-protocol/pull/637))
- remove duplicate word typos across docs and source ([#606](https://github.com/agentclientprotocol/agent-client-protocol/pull/606))
- use impl IntoOption<Meta> for CancelRequestNotification::meta() ([#467](https://github.com/agentclientprotocol/agent-client-protocol/pull/467))
- avoid redundant JSON validation in extension notification decoding ([#459](https://github.com/agentclientprotocol/agent-client-protocol/pull/459))

### Other

- Clean up some builder pattern inconsistencies ([#635](https://github.com/agentclientprotocol/agent-client-protocol/pull/635))
- fix incomplete sentence in KillTerminalCommandRequest doc comment ([#608](https://github.com/agentclientprotocol/agent-client-protocol/pull/608))
- *(deps)* bump the minor group with 2 updates ([#563](https://github.com/agentclientprotocol/agent-client-protocol/pull/563))
- *(deps)* bump strum from 0.27.2 to 0.28.0 ([#564](https://github.com/agentclientprotocol/agent-client-protocol/pull/564))
- *(deps)* bump the minor group with 3 updates ([#518](https://github.com/agentclientprotocol/agent-client-protocol/pull/518))
- *(deps)* bump the minor group with 4 updates ([#480](https://github.com/agentclientprotocol/agent-client-protocol/pull/480))

## [0.10.8](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.7...v0.10.8) - 2026-02-04

### Added

- Stabilize Session Config Options ([#411](https://github.com/agentclientprotocol/agent-client-protocol/pull/411))
- *(unstable)* Add unstable support for session usage ([#454](https://github.com/agentclientprotocol/agent-client-protocol/pull/454))

## [0.10.7](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.6...v0.10.7) - 2026-01-15

### Fixed

- *(schema)* Add missing titles for enum variants in schema ([#384](https://github.com/agentclientprotocol/agent-client-protocol/pull/384))
- *(unstable)* Add missing session capabilities builder method ([#380](https://github.com/agentclientprotocol/agent-client-protocol/pull/380))
- *(unstable)* Add copy to SessionConfigOptionCategory ([#368](https://github.com/agentclientprotocol/agent-client-protocol/pull/368))

### Other

- *(rfd)* Session Config Options to Preview stage ([#378](https://github.com/agentclientprotocol/agent-client-protocol/pull/378))
- *(deps)* bump the minor group with 5 updates ([#375](https://github.com/agentclientprotocol/agent-client-protocol/pull/375))

## [0.10.6](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.5...v0.10.6) - 2026-01-09

### Added

- *(unstable)* Add a category to session config options ([#366](https://github.com/agentclientprotocol/agent-client-protocol/pull/366))
- *(unstable)* Add a request cancelled error constructor ([#347](https://github.com/agentclientprotocol/agent-client-protocol/pull/347))

### Fixed

- *(error)* Add human readable titles for error code variants ([#367](https://github.com/agentclientprotocol/agent-client-protocol/pull/367))

### Other

- *(deps)* bump the minor group with 2 updates ([#362](https://github.com/agentclientprotocol/agent-client-protocol/pull/362))
- *(deps)* bump the minor group across 1 directory with 7 updates ([#358](https://github.com/agentclientprotocol/agent-client-protocol/pull/358))

## [0.10.5](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.4...v0.10.5) - 2025-12-17

### Added

- *(unstable)* Make constructing SessionConfigSelects on the Rust side nicer ([#343](https://github.com/agentclientprotocol/agent-client-protocol/pull/343))

## [0.10.4](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.3...v0.10.4) - 2025-12-16

### Added

- *(unstable)* Draft implementation of session config options ([#339](https://github.com/agentclientprotocol/agent-client-protocol/pull/339))

## [0.10.3](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.2...v0.10.3) - 2025-12-15

### Added

- *(unstable)* add SessionInfoUpdate to SessionUpdate enum ([#334](https://github.com/agentclientprotocol/agent-client-protocol/pull/334))
- *(rust-only)* Introduce MaybeUndefined type to allow for distinguishing between null and undefined ([#337](https://github.com/agentclientprotocol/agent-client-protocol/pull/337))

## [0.10.2](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.1...v0.10.2) - 2025-12-11

### Added

- *(unstable)* add cwd and mcp_servers to session/fork ([#333](https://github.com/agentclientprotocol/agent-client-protocol/pull/333))
- *(unstable)* Draft implementation of session/resume ([#324](https://github.com/agentclientprotocol/agent-client-protocol/pull/324))

## [0.10.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.10.0...v0.10.1) - 2025-12-09

### Added

- *(unstable)* Draft implementation of `$/cancel_request` notification ([#303](https://github.com/agentclientprotocol/agent-client-protocol/pull/303))

### Fixed

- *(schema)* Add title field back ([#321](https://github.com/agentclientprotocol/agent-client-protocol/pull/321))

## [0.10.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.9.1...v0.10.0) - 2025-12-06

This release mostly contains several nice quality-of-life improvements for the Rust version of the schema, as well as an unstable draft implementation of session/fork for people to start trying out.

### Added

- *(rust-only)* More convenient builder method params ([#313](https://github.com/agentclientprotocol/agent-client-protocol/pull/313))
- *(unstable)* Draft implementation of session/fork ([#311](https://github.com/agentclientprotocol/agent-client-protocol/pull/311))
- *(rust-only)*: Provide nicer interface to `ErrorCode` and add them to the docs ([#301](https://github.com/agentclientprotocol/agent-client-protocol/pull/301))

### Fixed

- *(rust)* Make new methods consistent for all id params ([#306](https://github.com/agentclientprotocol/agent-client-protocol/pull/306))

### Other

- Bump the minor group with 2 updates ([#310](https://github.com/agentclientprotocol/agent-client-protocol/pull/310))
- *(rust)* Move to a more typical rust lib setup ([#299](https://github.com/agentclientprotocol/agent-client-protocol/pull/299))

## [0.9.1](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.9.0...v0.9.1) - 2025-12-01

### Fixed

- Remove incorrect discriminator on `McpServer` type ([#292](https://github.com/agentclientprotocol/agent-client-protocol/pull/292))

## [0.9.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.8.0...v0.9.0) - 2025-12-01

This release defines the `_meta` properties in the schema as intended and currently used, which is always an object of key/value pairs, with string keys and arbitrary values.

While this is how everyone is using them, it became clear in code generation that the types weren't quite matching up to the expected usage. This should alleviate some extra checks on the implementer side.

### Added

- [**breaking**] Provide clearer schema for \_meta properties ([#290](https://github.com/agentclientprotocol/agent-client-protocol/pull/290))

## [0.8.0](https://github.com/agentclientprotocol/agent-client-protocol/compare/v0.7.0...v0.8.0) - 2025-11-28

Some follow-up changes from 0.7.0. Most of the changes were in the Rust schema to make things a bit easier to work with.

However, there were some further cleanups to the JSON schema to remove some $ref indirection where possible to have the schema be a bit flatter.

There are also some fixes that were causing issues with code generators related to Extension methods, these now have concrete types in the schema as well.

**Rust**: There are some breaking changes to the `OutgoingMessage` types and other low-level RPC types to make them generate clearer JSON schema representations. Likely these are only used by SDKs, but they moved to tuple enum variants.

Also, rather than having free-floating `V0` and `V1` constants, these are now associated constants on the `ProtocolVersion` type itself.

### Fixed

- Broken doctest and test in CI ([#267](https://github.com/agentclientprotocol/agent-client-protocol/pull/267))

### Other

- Remove some nesting of the JSON schema ([#278](https://github.com/agentclientprotocol/agent-client-protocol/pull/278))
- Easier ids in constructors ([#275](https://github.com/agentclientprotocol/agent-client-protocol/pull/275))
- Exhaustive RPC types ([#272](https://github.com/agentclientprotocol/agent-client-protocol/pull/272))
- Easier `new` methods for ExtRequest + ExtNotification ([#271](https://github.com/agentclientprotocol/agent-client-protocol/pull/271))
- Protocol Version constants ([#270](https://github.com/agentclientprotocol/agent-client-protocol/pull/270))
- Cleanup Rust example from schema docs ([#269](https://github.com/agentclientprotocol/agent-client-protocol/pull/269))
- Introduce helper methods to get the corresponding method name of a ([#268](https://github.com/agentclientprotocol/agent-client-protocol/pull/268))

## 0.7.0 (2025-11-25)

This is a big release as we move towards a v1.0 release of the JSON Schema.

This should be the final form, we just want to go through the motions of upgrading all of the SDKs to verify no further changes are needed.

**NOTE: The Protocol version is already, and remains, `1`. This is just for the JSON Schema itself.** There are no breaking changes to the protocol, we just reworked the schema representation to be more compliant with code generation tooling for the various SDKs.

We also now have two variants of the schema attached to the release:

**Stable**

- schema.json
- meta.json

**Unstable**

- schema.unstable.json
- meta.unstable.json

As we have more [RFD](https://agentclientprotocol.com/rfds/about) implementations in progress, this will allow us to iterate on the schema without requiring SDKs to churn through the changes.

For SDK authors, it is important if you use the unstable version, to make sure the unstable features are behind a flag of some kind with clear direction to your users about the state of these features. But this will also allow teams to start testing the unstable features and provide feedback to the RFD authors.

### Rust

The Rust crate, `agent-client-protocol-schema` has major breaking changes. All exported type are now marked as `#[non_exhaustive]`. Since the schema itself is JSON, and we can introduce new fields and variants in a non-breaking way, we wanted to allow for the same behavior in the Rust library.

All enum variants are also tuple variants now, with their own structs. This made it nicer to represent in the JSON Schema, and also made sure we have `_meta` fields on all variants.

This upgrade will likely come with a lot of compilation errors, but ideally upgrading will be more painless in the future.

## 0.6.3 (2025-10-30)

### Protocol

- Add `discriminator` fields to the schema.json for tagged enums to aid with code generation in language tooling.

## 0.6.2 (2025-10-24)

### Protocol

Fix incorrectly named `_meta` field on `SetSessionModeResponse`

## 0.6.1 (2025-10-24)

### Protocol

- No changes

### Rust

- Make `Implementation` fields public

## 0.6.0 (2025-10-24)

### Protocol

- Add ability for agents and clients to provide information about their implementation https://github.com/agentclientprotocol/agent-client-protocol/pull/192

## 0.5.0 (2025-10-23)

### Protocol

- JSON Schema: More consistent inlining for enum representations to fix issues with code generation in language tooling.
- Provide more schema-level information about JSON-RPC format.
- Provide missing `_meta` fields on certain enum variants.

### Rust

- More consistent enum usage. Enums are always either newtype or struct variants within a single enum, not mixed.

## 0.4.11 (2025-10-20)

### Protocol

- No changes

### Rust

- Make id types easier to create and add `PartialEq` and `Eq` impls for as many types as possible.

## 0.4.10 (2025-10-16)

### Protocol

- No changes

### Rust

- Export `Result` type with a default of `acp::Error`

## 0.4.9 (2025-10-13)

- Fix schema publishing

## 0.4.8 (2025-10-13)

- Fix publishing

## 0.4.7 (2025-10-13)

### Protocol

- Schema uploaded to GitHub releases

### Rust

- SDK has moved to https://github.com/agentclientprotocol/rust-sdk
- Start publishing schema types to crates.io: https://crates.io/crates/agent-client-protocol-schema

## 0.4.6 (2025-10-10)

### Protocol

- No changes

### Rust

- Fix: support all valid JSON-RPC ids (int, string, null)

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

- Make `Agent` and `Client` dyn compatible (you'll need to annotate them with `#[async_trait]`) [#97](https://github.com/agentclientprotocol/agent-client-protocol/pull/97)
- `ext_method` and `ext_notification` methods are now more consistent with the other trait methods [#95](https://github.com/agentclientprotocol/agent-client-protocol/pull/95)
  - There are also distinct types for `ExtRequest`, `ExtResponse`, and `ExtNotification`
- Rexport `serde_json::RawValue` for easier use [#95](https://github.com/agentclientprotocol/agent-client-protocol/pull/95)

### Typescript Library

- Use Stream abstraction instead of raw byte streams [#93](https://github.com/agentclientprotocol/agent-client-protocol/pull/93)
  - Makes it easier to use with websockets instead of stdio
- Improve type safety for method map helpers [#94](https://github.com/agentclientprotocol/agent-client-protocol/pull/94)
