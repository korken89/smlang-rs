# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).


## [Unreleased]

## [v0.4.1]

### Fixed

### Added

- Initial state can be specified at runtime.

### Changes

## [v0.4.0]

### Added

- Introduce a new named syntax, supporting `guard_error`, `transitions` and `temporary_context`
- The ability to define custom guard errors

## [v0.3.5]

### Fixed

- No longer needed to define each action, the same action can now be reused.

### Added

## [v0.3.4]

### Fixed

### Added

- Added syntax and support for a temporary context which is propagated from `process_event`. This
  allows for usage in systems where the normal context cannot take ownership, such as when having a
  reference which is only valid during the invocation of the state machine. For an example of this
  feature see `examples/guard_action_syntax_with_temporary_context.rs`.

### Changes

## [v0.3.3]

### Fixed

- Now compatible with `#![deny(missing_docs)]`.

## [v0.3.2]

### Fixed

- Having states with data associated, but no action to set this data, caused arcane errors. This is now fixed.

### Added

- Destination state may now have a type associated with it

### Changes

## [v0.3.1]

### Changes

* Better documentation and examples
* Graphviz diagrams only generated if feature is enabled

## [v0.3.0]

### Fixed

* API documentation should now be correctly generated in a project

### Changes

* [breaking] Most derives on `States`, `Events` (`Copy`, `Clone`, `Debug`) and trait bounds on
`StateMachineContext` are removed.
* [breaking] All returns of state are now by reference
* [breaking] Guards now take self my mutable reference, this to allow for context modifications. Quite common
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


[Unreleased]: https://github.com/korken89/smlang-rs/compare/v0.4.1...master
[v0.4.1]: https://github.com/korken89/smlang-rs/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/korken89/smlang-rs/compare/v0.3.5...v0.4.0
[v0.3.5]: https://github.com/korken89/smlang-rs/compare/v0.3.4...v0.3.5
[v0.3.4]: https://github.com/korken89/smlang-rs/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/korken89/smlang-rs/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/korken89/smlang-rs/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/korken89/smlang-rs/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/korken89/smlang-rs/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/korken89/smlang-rs/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/korken89/smlang-rs/compare/v0.2.0...v0.2.1
