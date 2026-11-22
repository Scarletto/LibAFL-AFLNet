//! Expansions to `Fuzzer` for support for internal state machines

#[cfg(feature = "std")]
use alloc::borrow::Cow;
use alloc::{string::ToString, vec::Vec};
use core::{fmt::Debug, time::Duration};
#[cfg(feature = "std")]
use core::{hash::Hash, marker::PhantomData};

#[cfg(feature = "std")]
use fastbloom::BloomFilter;
#[cfg(feature = "std")]
use libafl_bolts::impl_serdeany;
use libafl_bolts::{current_time, tuples::MatchName};
#[cfg(feature = "std")]
use serde::Deserialize;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "introspection")]
use crate::monitors::stats::PerfFeature;
#[cfg(feature = "std")]
use crate::monitors::stats::{AggregatorOps, UserStats, UserStatsValue};
use crate::{
    Error, HasFeedback, HasMetadata, ExecutionProcessor, corpus::{Corpus, CorpusId, HasCurrentCorpusId, HasTestcase, Testcase}, events::{
        Event, EventConfig, EventFirer, EventReceiver, EventWithStats, ProgressReporter,
        SendExiting,
    }, executors::{Executor, ExitKind, HasObservers}, feedbacks::Feedback, inputs::{BytesInputConverter, Input, ToBytesInputConverter, ToTargetBytesConverter}, mark_feature_time, observers::ObserversTuple, schedulers::Scheduler, stages::StagesTuple, start_timer, state::{
        HasCorpus, HasCurrentStageId, HasCurrentTestcase, HasExecutions, HasImported,
        HasLastFoundTime, HasLastReportTime, HasSolutions, MaybeHasClientPerfMonitor, Stoppable,
    },
};

/// Holds a feedback, specifically for adding things to the state machine.
pub trait HasSMFeedback : HasFeedback {}

/// The result of a harness execution, pertaining to the state machine
#[derive(Debug, PartialEq, Eq)]
pub enum SMExecuteInputResult {
    /// No special input,
    None,
    /// This input should be propagated in the state machine
    StateMachine,
}

/// Evaluates if an input is interesting, pertaining to the state machine
pub trait SMExecutionProcessor<EM, I, OT, S> : ExecutionProcessor<EM, I, OT, S> {
    /// Check the outcome of the execution, find if it is worth propagating in the state machine
    fn sm_check_results(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &I,
        observers: &OT,
        exit_kind: &ExitKind,
    ) -> Result<SMExecuteInputResult, Error>;

    /// Process `SMExecuteInputResult`. Propagate in state machine or ignore
    fn sm_process_execution(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &I,
        exec_res: &SMExecuteInputResult,
        exit_kind: &ExitKind,
        observers: &OT,
    ) -> Result<(), Error>;

    /// serialize and send event via manager
    fn sm_serialize_and_dispatch(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &I,
        exec_res: &SMExecuteInputResult,
        observers: &OT,
        exit_kind: &ExitKind,
    ) -> Result<(), Error>;

    /// send event via manager
    fn sm_dispatch_event(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &I,
        exec_res: &SMExecuteInputResult,
        obs_buf: Option<Vec<u8>>,
        exit_kind: &ExitKind,
    ) -> Result<(), Error>;

    /// Evaluate if a set of observation channels has an interesting state
    fn sm_evaluate_execution(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &I,
        observers: &OT,
        exit_kind: &ExitKind,
        send_events: bool,
    ) -> Result<SMExecuteInputResult, Error>;
}