# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- `router-access`: blacklist entries can now include an optional `reason` and an `expires_at` timestamp. Expired blacklist entries are treated as not blacklisted.
- `router-registry`: `ContractEntry` includes an optional `deprecation_reason` and the `deprecate()` API accepts an optional reason which is emitted in the `contract_deprecated` event.
- `metrics/alerts.yml`: example Prometheus alerting rules for circuit breaker opens, high failure/error rates, and high request volume.

### Changed
- Documentation: added a top-level `CHANGELOG.md` following Keep a Changelog format.

