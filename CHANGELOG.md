# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2024-12-05

### Added

- `SimpleBid` type based on Strings for identifiers and unsigned integers for
  quantities.

### Changed

- Use recursive backtracking to speed up computation on some problem types.

## [0.1.0] - 2024-05-03

### Added

- VCG auction calculation for bids that implement the `Bid` trait.
- Tests covering simple cases, floating point, and usage of the `secrecy`
  crate.
