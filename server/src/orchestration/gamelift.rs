#![cfg(feature = "gamelift")]

use super::Orchestration;

#[derive(Debug)]
pub(crate) struct GameLiftOrchestration {}

impl Orchestration for GameLiftOrchestration {}
