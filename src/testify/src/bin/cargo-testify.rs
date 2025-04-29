use clap::Parser;
use std::process::Command;
use testify_core::runner::TestifyConfig;

#[derive(Parser)]
#[command(
    name = "Testify",
    about = "Run testify's tests for a project.",
    long_about = None
)]
struct CommandArgs {
    #[arg(help = "A glob pattern to filter the tests' names by")]
    test_name: Option<String>,

    #[arg(short, long, help = "Filter tests by tag")]
    tag: Vec<String>,

    #[arg(short, long, help = "Exclude tests with tag")]
    exclude_tag: Vec<String>,

    #[arg(short, long, help = "Stop the tests after the first failure")]
    fail_fast: bool,

    #[arg(
        last = true,
        help = "The arguments to pass to your project's `cargo run`"
    )]
    cargo_args: Vec<String>,
}

fn main() -> Result<(), ()> {
    let mut cli_args = std::env::args();
    cli_args.next();

    let args = CommandArgs::parse_from(cli_args);

    let config = serde_json::to_string(&TestifyConfig {
        name_filter: args.test_name,
        tags: args.tag,
        exclude_tags: args.exclude_tag,
        fail_fast: args.fail_fast,
    })
    .expect("Could not serialize testify configuration.");

    let mut command = Command::new("cargo");
    command.env(testify::TEST_RUNNER_TOGGLE_ENV_VAR_NAME, "true");
    command.arg("run");
    command.args(args.cargo_args);
    command.env(testify::TEST_RUNNER_CONFIG, config);

    if command
        .spawn()
        .expect("Failed to run cargo")
        .wait()
        .expect("Failed to wait for cargo to finish")
        .success()
    {
        Ok(())
    } else {
        Err(())
    }
}
