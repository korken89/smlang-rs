# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).


## [Unreleased]

### Fixed

* API documentation should now be correctly generated in a project

### Added

### Changes

* Most derives on `States`, `Events` (`Copy`, `Clone`, `Debug`) and trait bounds on
`StateMachineContext` are removed.
* All returns of state are now by reference
* Guards now take self my mutable reference, this to allow for context modifications. Quite common
when receiving the same event N times can be accepted as a transition. Before one would have to have
a long list of states to go through.
* Most function are made `#[inline]`

## [v0.2.2]

### Added

* Lifetime support added to guards and actions

## [v0.2.1]

### Added

* Basic lifetime support for event data

## v0.2.0

### Added

* Support for generating a graphviz file over the state machine
* Support for data in events
* Support for data in states
* Change log added


[Unreleased]: https://github.com/korken89/smlang-rs/compare/v0.2.2...master
[v0.2.2]: https://github.com/korken89/smlang-rs/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/korken89/smlang-rs/compare/v0.2.0...v0.2.1
