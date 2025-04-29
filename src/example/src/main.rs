#[testify::main]
#[tokio::main]
async fn main() {
    println!("HELLO WORLD!");
}

#[testify::test()]
fn test_example() {
    panic!("HELLO WORLD!");
}

#[testify::test(
    name = "Hello world!",
    case = "success",
    tags = ["tag1", "tag2"],
)]
fn test_hello_world_success() -> Result<(), String> {
    Ok(())
}

#[testify::test(
    name = "Hello world!",
    case = "failure",
    tags = ["tag1", "tag2"],
    should_fail
)]
fn test_hello_world_failure() -> Result<(), String> {
    Err(String::from("This didn't work!"))
}

#[testify::setup]
async fn setup() {}

#[testify::cleanup]
async fn cleanup() {}
