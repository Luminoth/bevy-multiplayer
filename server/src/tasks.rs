use std::future::Future;

use bevy_tokio_tasks::{MainThreadContext, TokioTasksRuntime};
use tokio::task::JoinHandle;

// TODO: on_success / on_failure should be systems
// either run with ctx.world.run_system_once() or through observers
pub fn spawn_task<Output, Task, Spawnable, S, F>(
    runtime: &mut TokioTasksRuntime,
    task: Spawnable,
    on_success: S,
    on_failure: F,
) -> JoinHandle<()>
where
    Output: Send + 'static,
    Task: Future<Output = anyhow::Result<Output>> + Send + 'static,
    Spawnable: FnOnce() -> Task + Send + 'static,
    S: FnOnce(MainThreadContext, Output) + Send + 'static,
    F: FnOnce(MainThreadContext, anyhow::Error) + Send + 'static,
{
    runtime.spawn_background_task(move |mut ctx| async move {
        let result = task().await;
        ctx.run_on_main_thread(move |ctx| match result {
            Ok(output) => on_success(ctx, output),
            Err(err) => on_failure(ctx, err),
        })
        .await;
    })
}
