//! This library provides an extended test suite that allows better and easier organization and
//! management of your tests. It has support for async tests with tokio out of the box.
//! 
//! # Setup
//! 
//! To set up the testify test suite, you'll need to add `testify` to your project's dependencies,
//! and to install `testify` via `cargo install testify`. The last one will only set up the
//! `cargo testify` command, which you'll use to run your tests from now on.
//! 
//! ## Features
//! 
//! These are the features you can enable in your project:
//! - `async-tokio`: Enable support for async tests using tokio as the runtime.
//! 
//! # Usage
//! 
//! To set up the tests runner, wrap your `main()` function with `#[testify::main]`. This'll expand
//! to (roughly)
//! ```
//! fn main() {
//!     if std::env::var("DO_NOT_MANUALLY_SET_TESTIFY_ARE_TESTS_BEING_RUN").is_ok() {
//!         testify::run();
//!     } else {
//!         /* YOUR CODE */
//!     }
//! }
//! ```
//! 
//! > *This means that the testing code will be built into your binary from now onwards.
//! > Suggestions and PRs are welcome to solve this issue.*
//! 
//! After wrapping your main function with testify's main macro, you're ready to go. In case you
//! already have any tests set up in your project, replace `#[test]` with `#[testify::test]` and
//! that will be enough for your code to run in most cases.
//! 
//! ## The `#[testify::test]` macro
//! 
//! As you've seen in the previous section, replacing `#[test]` with `#[testify::test]` should be
//! enough to get you started in most cases. Testify's test macro supports a wider set of features
//! than the default one.
//! 
//! ### Test Metadata
//! 
//! You can organize your tests better by passing some keyword arguments to the test macro (all
//! optional):
//! - `name`: A string literal, which allows you to rename the test function to something prettier
//!     to be outputted in the console when running the tests.
//! - `case`: A string literal, it allows you to specify different cases of the same unit being
//!     tested.
//! - `tags`: An array of string literals, it allows you to tag your tests for easier filtering
//!     when running your tests with `cargo testify`, opposed to rust's default test suite with its
//!     substring filtering.
//! - `should_panic`: As the name says, passing this argument to the test macro will make the test
//!     execution being expected to panic, and failing if it does not.
//! - `should_fail`: Similar to `should_panic`, but for the return types of the test function. In
//!     this case, `TestTermination.success()` will be expected to return `false`.
//! 
//! #### Example
//! 
//! ```
//! // All of the arguments passed to the macro are optional.
//! #[testify::test(
//!     name = "Register User",
//!     case = "Weak Password",
//!     tags = ["api", "auth"],
//!     should_fail
//! )]
//! fn my_test() -> Result<(), String> {
//!     /* RUN YOUR CODE */
//!     Err("The password was too weak.".into())
//! }
//! ```
//! 
//! ### Async Support
//! 
//! Tests support async functions out of the box with the `async-tokio` feature. It's as easy as
//! making your test async for it to run in a tokio runtime.
//! 
//! ```
//! #[testify::test]
//! async fn my_async_test() {
//!     /* RUN YOUR CODE */
//! }
//! ```
//! 
//! ### The `TestTermination` Trait
//! 
//! All your tests' return type must implement `TestTermination`. It's a simple trait that only has
//! one method, `success() -> bool`, which returns whether the test has failed or not. There are
//! some provided default implementations, but you're free to implement yours if the default
//! options do not fit your use case.
//! 
//! #### Default Implementations
//! 
//! The trait is implemented by default for:
//! 
//! - `Result<T: TestTermination, E>`: This'll fail in case of an error, otherwise run `.success()`
//!     for the returned value and return it.
//! - `Option<T: TestTermination>`: This'll fail if `None`, otherwise run `.success()` for the
//!     returned value and return it.
//! - `()`: This will always return true.
//! 
//! #### Example
//! 
//! ```
//! use testify::TestTermination;
//! 
//! // This is how the trait is implemented for this type internally.
//! impl<T: TestTermination, E> TestTermination for Result<T, E> {
//!     fn success(&self) -> bool {
//!         match self {
//!             Ok(inner) => inner.success(),
//!             Err(_) => false
//!         }
//!     }
//! }
//! ```
//! 
//! ## The `#[testify::setup]` and `#[testify::cleanup]` Macros
//! 
//! These two macros allow you to set up the test environment before the execution of the tests,
//! and to clean it up after the tests have passed.
//! 
//! ### Example
//! 
//! ```
//! #[testify::main]
//! fn main() {}
//! 
//! #[testify::test]
//! async fn test_db() {
//!     /* CODE THAT REQUIRES A DB */
//! }
//! 
//! #[testify::setup]
//! async fn setup() {
//!     backup_dev_db_and_setup_test_one().await;
//! }
//! 
//! #[testify::cleanup]
//! async fn cleanup() {
//!     destroy_test_db_and_restore_dev_db_backup().await;
//! }
//! ```
//! 
//! There's no need to have both a setup and a cleanup function either. You may use them
//! individually. Both `setup` and `cleanup` functions support both sync and async (with the
//! `async-tokio` feature enabled).
//! 
//! ## Using `cargo testify`
//! 
//! Tests are run using the testify command `cargo testify`. It's a command line tool that allows
//! you to configure the way in which your tests are run. In case you haven't installed it yet, run
//! `cargo install testify` to set it up.
//! 
//! ```
//! $ cargo testify --help
//! ```
//! 
//! ### Filtering by Name
//! 
//! The default `cargo test` command allows you to filter your tests by a substring of the test's
//! name. Testify goes a bit further by allowing you to use glob pattern matching to filter by
//! name.
//! 
//! ```
//! $ cargo testify hello*
//! ```
//! 
//! ### Filtering by Tag
//! 
//! You can also filter by the tags you've set in your tests by passing the `--tag` argument to the
//! `cargo testify` command.
//! 
//! ```
//! // Both --tag and -t do the same
//! $ cargo testify --tag auth -t api
//! ```
//! 
//! You can also exclude tags by passing the `--exclude-tag` argument:
//! 
//! ```
//! $ cargo testify --exclude-tag db
//! // -e for the shortcut
//! ```
//! 
//! ### Fast Failing
//! 
//! If you only care about whether all tests pass or not, you can pass the `--fail-fast` argument.
//! This'll stop testing on the first test that fails. You'll see a `Failed! Aborted.` next to the
//! failing test, in case there's any.

#[doc(hidden)]
pub use testify_core::*;

pub use testify_macros::*;

#[doc(hidden)]
pub use ctor;
