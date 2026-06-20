pub mod cache;
pub mod db;
pub mod metadata;
pub mod scanner;
pub mod thumbnail;

/// A generic progress reporting interface for background scanning and thumbnailing operations.
pub trait ProgressReporter: Send + Sync + Clone + 'static {
    fn report(&self, phase: &str, current: usize, total: usize, current_file: &str);
}

/// A fallback progress reporter that performs no operations.
#[derive(Clone, Debug)]
pub struct NoopProgressReporter;

impl ProgressReporter for NoopProgressReporter {
    fn report(&self, _phase: &str, _current: usize, _total: usize, _current_file: &str) {}
}
