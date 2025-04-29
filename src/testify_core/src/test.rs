use std::fmt::Debug;

pub enum TestStatus {
    Passed,
    Panicked,

    // The test was expected to panic, but it did not.
    NotPanicked,
    Failed,

    // The test was expected to pass, but it failed.
    NotFailed,
}

pub type TestFn = fn() -> TestStatus;

#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub case: Option<String>,
    pub tags: Vec<String>,
    pub function: TestFn,
}

pub trait TestTermination {
    fn success(&self) -> bool;
}

impl TestTermination for () {
    fn success(&self) -> bool {
        true
    }
}

impl<T: TestTermination, E> TestTermination for Result<T, E> {
    fn success(&self) -> bool {
        match self {
            Ok(r) => r.success(),
            Err(_) => false,
        }
    }
}

impl<T: TestTermination> TestTermination for Option<T> {
    fn success(&self) -> bool {
        match self {
            Some(r) => r.success(),
            None => false
        }
    }
}