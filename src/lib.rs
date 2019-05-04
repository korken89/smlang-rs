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
//! # DSL
//!
//! Please consult the README for the DSL specification.
//!
//! # Examples
//!
//! Please consult the README for examples.
//!
//! # Errors
//!
//! `StateMachine::process_event` will return `Ok(NextState)` if the transition was successful,
//! or `Err(Error::GuardFailed)` if the guard failed, or `Err(Error::InvalidEvent)` if an event
//! which should not come at this stage of the state machine was processed.
//!
//! # Panics
//!
//! There are no `panic!` in this library.
//!
//! # Unsafe
//!
//! There is no use of `unsafe` in this library.
//!
//! # Auto-generated types
//!
//! ```ignore
//! // Auto generated enum of states
//! enum States { ... }
//! ```
//!
//! ```ignore
//! // Auto generated enum of possible events
//! enum Events { ... }
//! ```
//!
//! ```ignore
//! // Auto generated struct which holds the state machine implementation
//! struct StateMachine { ... }
//! ```
//!
//! # State machine generated API
//!
//! ```ignore
//! struct StateMachine {
//!     /// Creates a state machine with the starting state
//!     pub fn new() -> Self;
//!
//!     /// Returns the current state
//!     pub fn state(&self) -> States;
//!
//!     /// Process an event
//!     pub fn process_event(&mut self, event: Events) -> Result<States, Error>;
//! }
//! ```

#![no_std]

pub use smlang_macros::statemachine;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests;
