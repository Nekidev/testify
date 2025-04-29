use std::sync::Mutex;

pub mod runner;
pub mod test;

pub use runner::run;
pub use test::TestTermination;

pub static TESTS: Mutex<Vec<test::Test>> = Mutex::new(Vec::new());
pub static SETUP: Mutex<Option<fn() -> ()>> = Mutex::new(None);
pub static CLEANUP: Mutex<Option<fn() -> ()>> = Mutex::new(None);

#[cfg(feature = "async-tokio")]
pub static ASYNC_RT: once_cell::sync::Lazy<tokio::runtime::Runtime> = once_cell::sync::Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not initialize the tokio runtime")
});

pub const TEST_RUNNER_TOGGLE_ENV_VAR_NAME: &str = "DO_NOT_MANUALLY_SET_TESTIFY_ARE_TESTS_BEING_RUN";
pub const TEST_RUNNER_CONFIG: &str = "DO_NOT_MANUALLY_SET_TESTIFY_CONFIG";
