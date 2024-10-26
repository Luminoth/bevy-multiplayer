#![cfg(feature = "agones")]

use super::Orchestration;

#[derive(Debug)]
pub(crate) struct AgonesOrchestration {}

impl Orchestration for AgonesOrchestration {}
