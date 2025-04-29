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

// #[testify::group(tags = ["one", "two", "three"], in_order)]
// mod tests {
//     #[testify::setup]
//     async fn setup() {}

//     #[testify::test(in_order, name = "My Test")]
//     mod my_test {
//         #[testify::setup]
//         async fn setup() {}

//         #[testify::case(name = "Success")]
//         async fn success() {}

//         #[testify::case(name = "Failure", should_fail)]
//         async fn failure() {}

//         #[testify::cleanup]
//         async fn cleanup() {}
//     }

//     #[testify::test(should_panic)]
//     async fn my_test() {}

//     #[testify::cleanup]
//     async fn cleanup() {}
// }

#[testify::cleanup]
async fn cleanup() {}
