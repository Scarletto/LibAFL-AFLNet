//! A state machine maintains information relevant during a state-aware fuzzing process.
use core::{fmt::Debug, hash::Hash};

use crate::Error;
use serde::{Deserialize, Serialize};

mod fuzzer;
mod feedback;
mod observer;

/// Trait for elements offering an internal state machine
pub trait HasStateMachine<I> {
    /// The associated type implementing [`StateMachine`]
    type StateMachine: StateMachine<I>;

    /// The internal state machine 
    fn state_machine(&self) -> &Self::StateMachine;
    /// The internal state machine (mutable)
    fn state_machine_mut(&mut self) -> &mut Self::StateMachine;
}

/// An abstraction for the index that identifies a state in the state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct StateId(pub usize);

/// A struct holding the basic information of a step in a path in a state machine.
#[derive(Debug)]
pub struct SMStep<'a, I> {
    src: StateId,
    input: &'a I,
    dst: StateId
}

/// An iterator for walking a given path of inputs in a state machine.
#[derive(Debug)]
pub struct SMWalk<'a, It, SM> {
    state_machine: &'a SM,
    cur: Option<StateId>,
    path: It
}

impl<'a, I: 'a, It, SM> Iterator for SMWalk<'_, It, SM> 
where
    It: Iterator<Item = &'a I>,
    SM: StateMachine<I>
{
    type Item = SMStep<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        let src = self.cur?;
        let input = self.path.next()?;
        self.state_machine
            .step(src, input)
            .and_then(|dst| { 
                self.cur = Some(dst);
                Some(SMStep {src, input, dst})
            })
    }
}

/// A trait for a basic state machine used as metadata during the fuzzing run.
/// Takes an Input type `I` for stepping transitions.
/// 
/// Assumes each state-input pair appears at most once; this should be enforced!
pub trait StateMachine<I> : Sized {
    /// Returns the ID of the root state of the state machine.
    fn root(&self) -> StateId;

    /// Steps a transition with the given input from target state. Returns the destination state's ID.
    fn step(&self, src_id: StateId, input: &I) -> Option<StateId>;

    /// Walks the given path from target state. Returns a `SMWalk` iterator of the path in the state machine.
    fn walk<'a, It>(&self, src: StateId, path: It) -> SMWalk<'_, It::IntoIter, Self>
    where 
        I: 'a,
        It: IntoIterator<Item = &'a I>,
    {
        SMWalk { state_machine: self, cur: Some(src), path: path.into_iter() }
    }
}

/// An extension trait for updating a state machine during the fuzzing run.
/// Takes a State input type `SV` and a Transition input type `TV`.
pub trait SMBuilder<SV, TV> {
    /// Adds a state to the machine. Returns that state's ID.
    fn add_state(&mut self, state: SV) -> Result<StateId, Error>;
    
    /// Adds a new transition between two target states.
    fn add_transition(&mut self, src_id: StateId, dst_id: StateId, trans: TV) -> Result<(), Error>;
}

/// An extension trait for accessing a view of a state machine state during the fuzzing run.
/// Takes a State view type `SV`.
pub trait SMStateView<SV> {
    /// Returns a reference to the view for target state.
    fn get_state(&self, id: StateId) -> Option<&SV>;
}

/// An extension trait for accessing a view of a state machine transition during the fuzzing run.
/// Takes a Transition view type `TV` and an input type `I`.
pub trait SMTransitionView<I, TV> {
    /// Returns a reference to the view for target transition.
    /// For paired state ID and input, use `get_transition_by_pair` instead.
    fn get_transition(&self, src_id: StateId, input: &I) -> Option<&TV>;

    /// Returns a reference to the view for target transition.
    /// For separate state ID and input, use `get_transition` instead.
    fn get_transition_by_pair(&self, (src_id, input): (StateId, &I)) -> Option<&TV> {
        self.get_transition(src_id, input)
    }
}