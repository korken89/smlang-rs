# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- Add support for async guards and actions
- Add name to statemachine and make dot output stable and unique ([issue-62](https://github.com/korken89/smlang-rs/pull/62))
- Add derive macros to states and events ([issue-62](https://github.com/korken89/smlang-rs/pull/62))
- Add hooks to `StateMachineContext` for logging events, guards, actions, and state changes
- Add support multiple guarded transitions for a triggering event
- Add support for guard boolean expressions in the state machine declaration

### Fixed

- Fixes multiple issues with lifetimes ([issue-57](https://github.com/korken89/smlang-rs/issues/57), [issue-58](https://github.com/korken89/smlang-rs/pull/58))

### Changed

- [breaking] Actions now take owned values
- [breaking] `state()` now returns a `Result`
- `StateMachine::new` and `StateMachine::new_with_state` are now const functions
- Fixed clippy warnings
- [breaking] Changed guard functions return type from Result<(),()> to bool

## [v0.6.0] - 2022-11-02

### Fixed

- Updated the link checker in the Github actions to use [lychee](https://github.com/lycheeverse/lychee).

### Added

- Starting state can now contain data ([issue-34](https://github.com/korken89/smlang-rs/issues/34))
- Allow explicit input states before wildcard input state([issue-47](https://github.com/korken89/smlang-rs/pull/47)

### Changed
- Custom guard error types are now specified as a type of the `StateMachineContext` to allow for
  more versatile types.

## [v0.5.1]

### Fixed
* [#36](https://github.com/korken89/smlang-rs/issues/36) Attempts to use actions and guards with
  inconsistent input, event, and output state data will be flagged as compiler errors.

### Added

## [v0.5.0]

### Added

- Changelog enforcer added to CI
- State data now supports lifetimes ([issue-26](https://github.com/korken89/smlang-rs/issues/26))
- New example [dominos.rs](https://github.com/korken89/smlang-rs/blob/master/examples/dominos.rs) illustrating a method of event propagation ([issue-17](https://github.com/korken89/smlang-rs/issues/17))
- Input states support pattern matching and wildcards ([issue-29](https://github.com/korken89/smlang-rs/issues/29))

### Fixed
- PartialEq for States and Events based on discriminant only ([issue-21](https://github.com/korken89/smlang-rs/issues/21))
- Updated the CI badges ([issue-30](https://github.com/korken89/smlang-rs/issues/30))

## [v0.4.2]

### Fixed

### Added

- Initial state can be specified at runtime.

### Changes

## [v0.4.1] -- YANKED

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

[Unreleased]: https://github.com/korken89/smlang-rs/compare/v0.6.0...master
[v0.6.0]: https://github.com/korken89/smlang-rs/compare/v0.5.1...v0.6.0
[v0.5.1]: https://github.com/korken89/smlang-rs/compare/v0.5.0...v0.5.1
[v0.5.0]: https://github.com/korken89/smlang-rs/compare/v0.4.2...v0.5.0
[v0.4.2]: https://github.com/korken89/smlang-rs/compare/v0.4.1...v0.4.2
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
