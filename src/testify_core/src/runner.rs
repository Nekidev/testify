use std::{
    cmp::Ordering,
    io::{self, Write},
    panic,
    time::{Duration, Instant},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    CLEANUP, SETUP, TEST_RUNNER_CONFIG, TESTS,
    test::{Test, TestStatus},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct TestifyConfig {
    pub name_filter: Option<String>,
    pub tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub fail_fast: bool,
}

fn flush() {
    io::stdout().flush().unwrap();
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();

    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        let micros = nanos as f64 / 1_000.0;
        format!("{:.0}µs", micros)
    } else if nanos < 1_000_000_000 {
        let millis = nanos as f64 / 1_000_000.0;
        format!("{:.0}ms", millis)
    } else if nanos < 60_000_000_000 {
        let secs = nanos as f64 / 1_000_000_000.0;
        format!("{:.2}s", secs)
    } else {
        let secs_total = nanos as f64 / 1_000_000_000.0;
        let minutes = (secs_total / 60.0).floor();
        let remaining_secs = secs_total % 60.0;
        format!("{:.0}m {:.0}s", minutes, remaining_secs)
    }
}

struct TestGroup {
    tags: Vec<String>,
    test_plans: Vec<TestPlan>,
}

struct TestPlan {
    name: String,
    cases: Vec<Test>,
}

fn organize(tests: Vec<Test>, config: &TestifyConfig, pattern: &glob::Pattern) -> Vec<TestGroup> {
    let mut tests: Vec<Test> = tests
        .iter()
        .filter(|test| {
            for tag in config.tags.iter() {
                if !test.tags.contains(tag) {
                    return false;
                }
            }

            for tag in config.exclude_tags.iter() {
                if test.tags.contains(tag) {
                    return false;
                }
            }

            if !pattern.matches(&test.name) {
                return false;
            }

            true
        })
        .cloned()
        .collect();

    tests.sort_by(|a, b| {
        let cmp = a.tags.cmp(&b.tags);

        if cmp != Ordering::Equal {
            return cmp;
        }

        let cmp = a.name.cmp(&b.name);

        if cmp != Ordering::Equal {
            return cmp;
        }

        a.case.cmp(&b.case)
    });

    let mut result: Vec<TestGroup> = Vec::new();

    for test in tests {
        if let Some(last_group) = result.last_mut() {
            if last_group.tags == test.tags {
                if let Some(last_test) = last_group.test_plans.last_mut() {
                    if last_test.name == test.name {
                        last_test.cases.push(test);
                    } else {
                        last_group.test_plans.push(TestPlan {
                            name: test.name.clone(),
                            cases: vec![test],
                        });
                    }
                } else {
                    panic!("This code shouldn't be running.");
                }
            } else {
                result.push(TestGroup {
                    tags: test.tags.clone(),
                    test_plans: vec![TestPlan {
                        name: test.name.clone(),
                        cases: vec![test],
                    }],
                });
            }
        } else {
            result.push(TestGroup {
                tags: test.tags.clone(),
                test_plans: vec![TestPlan {
                    name: test.name.clone(),
                    cases: vec![test],
                }],
            });
        }
    }

    result
}

/// Executes a function and returns the result together with the time the function took to execute.
fn exec_with_timing<T>(f: fn() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();

    (result, start.elapsed())
}

pub fn run() {
    // TODO: Capture stdout and stderr to prevent polluting the test runner output. Currently, the
    // function used to capture outputs by cargo test is only available on nightly builds of Rust.

    // Initialize the runtime to avoid performance overhead later on.
    #[cfg(feature = "async-tokio")]
    let _ = &*crate::ASYNC_RT;

    println!("✨ Testify! Running tests...\n");
    let mut step = 1;

    if SETUP.lock().unwrap().is_some() {
        print!("{step}. Starting up...");
        flush();
        step += 1;

        if let Some(startup) = SETUP.lock().unwrap().take() {
            startup();
        }

        print!("{}", " Ok.\n".green());
        flush();
    }

    let config: TestifyConfig = serde_json::from_str(&std::env::var(TEST_RUNNER_CONFIG).expect("Testify configuration env var was not found")).expect("Could not parse testify's configuration. Are the versions of testify_core and testify correct?");

    let pattern = match glob::Pattern::new(if let Some(p) = &config.name_filter {
        p
    } else {
        "*"
    }) {
        Ok(pa) => pa,
        Err(_) => {
            eprintln!("The pattern passed to the glob filter was invalid.");
            std::process::exit(1);
        }
    };

    // TODO: Collect panic messages to display them nicely later on.
    panic::set_hook(Box::new(|_info| {}));

    let all_tests = TESTS.lock().unwrap();

    let groups = organize(all_tests.clone(), &config, &pattern);

    let tests_to_run = groups.iter().fold(0, |prev, group| {
        prev + group
            .test_plans
            .iter()
            .fold(0, |gprev, test_plan| gprev + test_plan.cases.len())
    });

    let mut failures = 0;
    let mut successes = 0;

    println!(
        "{step}. Running {} tests {}...",
        tests_to_run,
        format!("({} skipped)", all_tests.len() - tests_to_run).black()
    );
    step += 1;

    let mut test_i = 1;

    'groups_loop: for (group_i, group) in groups.iter().enumerate() {
        let tags_str = group.tags.join(", ");

        println!(
            "{}   {}",
            if group_i == 0 { "" } else { "\n" },
            format!(
                "---- {} ----",
                if group.tags.is_empty() {
                    "No tags"
                } else {
                    &tags_str
                }
            )
            .black()
        );

        for plan in &group.test_plans {
            if plan.cases.len() == 1 {
                print!("   {test_i}. {}...", plan.name);
                flush();

                let (result, duration) = exec_with_timing(plan.cases.first().unwrap().function);

                match result {
                    TestStatus::Passed => {
                        println!(
                            " {} {}",
                            "Ok.".green(),
                            format!("({})", format_duration(duration)).dimmed()
                        );

                        successes += 1;
                    }
                    _ => {
                        print!(" {}", "Failed!".red());
                        failures += 1;

                        if config.fail_fast {
                            print!(" {}", "Aborted.".red());
                            flush();

                            break 'groups_loop;
                        }

                        println!();
                    }
                }
            } else {
                println!("   {test_i}. {}...", plan.name);

                for case in &plan.cases {
                    print!(
                        "      {} {}{}",
                        "Case".black(),
                        case.case.as_deref().unwrap_or("unknown"),
                        "...".dimmed()
                    );
                    flush();

                    let (result, duration) = exec_with_timing(case.function);

                    match result {
                        TestStatus::Passed => {
                            println!(
                                " {} {}",
                                "Ok.".green(),
                                format!("({})", format_duration(duration)).dimmed()
                            );

                            successes += 1;
                        }
                        _ => {
                            print!(" {}", "Failed!".red());
                            failures += 1;

                            if config.fail_fast {
                                print!(" {}", "Aborted.".red());
                                flush();

                                break 'groups_loop;
                            }

                            println!();
                        }
                    }
                }
            }

            test_i += 1;
        }
    }

    if CLEANUP.lock().unwrap().is_some() {
        print!("{}{step}. Cleaning up...", if groups.len() > 1 { "\n" } else { "" });
        flush();
        if let Some(cleanup) = CLEANUP.lock().unwrap().take() {
            cleanup();
        }
        print!("{}", " Ok.\n".green());
        flush();
    }

    println!(
        "\n✅ Finished running tests. {} and {}.",
        format!("{failures} failed").red(),
        format!("{successes} succeeded").green()
    );

    if failures > 0 {
        std::process::exit(1);
    }
}
