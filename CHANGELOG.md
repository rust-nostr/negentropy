# Changelog

<!-- All notable changes to this project will be documented in this file. -->

<!-- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), -->
<!-- and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). -->

<!-- Template

## Unreleased

### Breaking changes

### Changed

### Added

### Fixed

### Removed

### Deprecated

### Performance

### Security

-->

## 0.5.2 - 2026/10/15

### Security

- Reject an ID list whose count exceeds the IDs left in the message, rather than reserving memory for it (https://github.com/nostrdevkit/negentropy/pull/14)

## 0.5.1 - 2026/08/16

### Performance

- Remove the per-item heap allocation from `Accumulator::add` (https://github.com/nostrdevkit/negentropy/pull/12)
- Avoid the sort scratch allocation in `NegentropyStorageVector::seal` (https://github.com/nostrdevkit/negentropy/pull/13)
