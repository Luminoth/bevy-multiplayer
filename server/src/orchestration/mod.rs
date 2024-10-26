#![allow(dead_code)]

#[cfg(all(feature = "agones", feature = "gamelift"))]
compile_error!("multiple orchestrator backends cannot be enabled at the same time");

mod agones;
mod gamelift;

pub trait Orchestration {}

pub fn create() -> Box<dyn Orchestration> {
    #[cfg(feature = "agones")]
    return Box::new(agones::AgonesOrchestration {});

    #[cfg(feature = "gamelift")]
    return Box::new(agones::GameLiftOrchestration {});
}
