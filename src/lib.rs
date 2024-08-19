//! # smlang
//!
//! `smlang` is a procedural macro library creating a state machine language DSL is to facilitate the
//! use of state machines, as they quite fast can become overly complicated to write and get an
//! overview of.
//!
//! # Project dependent documentation
//!
//! When this crate is used in a project the documentation will be auto generated in the
//! **documentation of the project**, this comes from the procedural macro also generating
//! documentation.
//!
#![doc = include_str!("../docs/dsl.md")]
//!
//! # Example
//!
//! Below is an example of the state machine macro in use along with the code that would be
//! generated for this sample to demonstrate how this library is used.
//!
//! ```rust
//! use smlang::statemachine;
//!
//! statemachine! {
//!     name: Sample,
//!     states_attr: #[derive(Debug)],
//!     events_attr: #[derive(Clone, Debug)],
//!     transitions: {
//!         *Init + InitEvent [ guard_init ] / action_init = Ready,
//!     }
//! }
//! ```
//!
//! Results in the following code:
//! ```ignore
//! #[derive(Debug)]
//! enum SampleStates {
//!     Init,
//!     Ready,
//! }
//!
//! #[derive(Clone, Debug)]
//! enum SampleEvents {
//!     InitEvent,
//! }
//!
//! struct SampleStateMachine<C: SampleStateMachineContext> {
//!     // ...
//! }
//!
//! enum SampleError {
//!     InvalidEvent,
//!     GuardFailed,
//!     // ...
//! }
//!
//! impl<C: SampleStateMachineContext> SampleStateMachine<C> {
//!     /// Creates a state machine with the starting state
//!     pub fn new() -> Self { /**/ }
//!
//!     /// Returns the current state
//!     pub fn state(&self) -> States { /**/ }
//!
//!     /// Process an event
//!     ///
//!     /// # Returns
//!     /// `Ok(NextState)` if the transition was successful or `Err()` if the transition failed.
//!     /// guard failed
//!     pub fn process_event(&mut self, event: Events) -> Result<SampleStates, SampleError> {
//! #       Err(SampleError::InvalidEvent);
//!     /**/
//!     }
//! }
//!
//! trait SampleStateMachineContext {
//!     // Called to guard the transition to `Ready`. Returns an `Err` if the guard fails.
//!     fn guard_init(&mut self) -> Result<(), ()>;
//!
//!     // Called when transitioning to `Ready`.
//!     fn action_init(&mut self);
//! }
//! ```
#![no_std]

pub use smlang_macros::statemachine;
