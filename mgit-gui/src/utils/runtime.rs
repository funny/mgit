use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Global Tokio runtime for background async operations.
/// Using a single runtime prevents resource exhaustion from creating
/// multiple runtimes for each operation.
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Initialize the global Tokio runtime.
/// Must be called before any async operations that use it.
pub fn init_runtime() {
    // Create the runtime if not already initialized
    // This uses OnceLock which handles thread safety
    let _ = TOKIO_RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"));
}

/// Get a reference to the global Tokio runtime.
/// Panics if the runtime hasn't been initialized.
#[track_caller]
pub fn runtime() -> &'static Runtime {
    TOKIO_RUNTIME
        .get()
        .expect("Tokio runtime not initialized. Call init_runtime() first.")
}

/// Spawn an async task on the global runtime and block until completion.
/// This is a convenience wrapper for running async code from sync contexts.
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    runtime().block_on(future)
}

/// Spawn an async task on the global runtime.
#[allow(dead_code)]
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send,
{
    runtime().spawn(future)
}

/// Spawn a blocking sync task on the global runtime's blocking thread pool.
#[allow(dead_code)]
pub fn spawn_blocking<F, T>(f: F) -> tokio::task::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    runtime().spawn_blocking(f)
}
