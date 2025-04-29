# Testify

**Testify** is a flexible, async-friendly test framework for Rust projects. It extends the built-in test system with powerful features like lifecycle hooks, test grouping, filtering, and case management—without sacrificing simplicity.

---

## ✨ Features

- **(almost) Drop-in test support**  
  Just use `#[testify::test]` and you're up and running—sync or async.

- **Lifecycle hooks**  
  Add `#[testify::setup]` and `#[testify::cleanup]` functions to run code before and after tests. No global state hacks required.

- **Test cases**  
  Define multiple named scenarios under one test with `#[testify::case(name = "...")]`.

- **Async-first**  
  With the `async-tokio` feature, you can write async tests and hooks without boilerplate.

- **Flexible filtering**  
  Run tests selectively by name (glob matching) or custom tags using `cargo testify`.

---

## 🔜 Roadmap

These features are on the way:

- **Better test structuring**  
  Less repetition when grouping tests or applying shared hooks.

- **Parallel test execution**  
  Speed up test runs by executing them concurrently—at the group and case level.

- **Shared test state**  
  Clean, isolated ways to pass state between setup, tests, and cleanup logic.

- **Group-level hooks**  
  Setup and cleanup for specific test groups, without nesting headaches.

---

## 🛠️ TODO

- [ ] Reduce boilerplate in nested/grouped test structures  
- [ ] Strip test-related code from release builds  
- [ ] Add a clean, configurable logging layer for test output  
- [ ] Support parallel execution with controlled ordering  
- [ ] Improve error reporting (test names, stack traces, failure context)

---

## 🚀 Why Use Testify?

If you're tired of boilerplate test harnesses, rigid execution order, or workarounds for async testing—**Testify** gives you the control and flexibility you're missing from Rust's default test system, while keeping tests expressive and simple.

---

## 📦 Usage

```rust
#[testify::main]
fn main() {}

#[testify::test]
fn my_test() {
    assert!(true);
}
```

For the full documentation, check [docs.rs/testify-rs](https://docs.rs/testify-rs).

## 🧪 Status

Early stage but production-experiment-ready. Try it, break it, file issues, and help shape where it goes.

## 📬 Contributions Welcome

Suggestions, bug reports, and PRs are all appreciated—especially from real-world use cases.