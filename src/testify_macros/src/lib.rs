//! This library provides a set of utilities for testing bunnybook (and other projects) with an
//! extended API.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ExprArray, ItemFn, LitStr, parse_macro_input};

/// Wraps your program's main function and adds the necessary code to run the tests.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let fn_name = item.sig.ident.to_string();
    let fn_block = &item.block;

    if fn_name != "main" {
        return quote! {
            compile_error!(stringify!(#[testify::main] can only be used on the main function, but it was used on #fn_name.));
        }.into();
    }

    quote! {
        fn main() {
            if std::env::var(testify::TEST_RUNNER_TOGGLE_ENV_VAR_NAME).is_ok() {
                testify::run();
            } else #fn_block
        }
    }
    .into()
}

/// Marks a function as a test function.
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let fn_name = &item.sig.ident;
    let fn_args_count = item.sig.inputs.len();
    let fn_block = &item.block;
    let fn_return_type = &item.sig.output;

    if fn_args_count != 0 {
        return quote! {
            compile_error!(stringify!(#[testify::test] can only be used on functions with no arguments, but it was used on #fn_name which has #fn_args_count arguments.));
        }.into();
    }

    let is_async = item.sig.asyncness.is_some();

    if is_async && !cfg!(feature = "async-tokio") {
        return quote! {
            compile_error!("This function is async but the `async-tokio` feature is not enabled. Enable it to use async tests.");
        }.into();
    }

    let mut should_panic = false;
    let mut should_fail = false;
    let mut name: Option<String> = None;
    let mut case: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();

    let test_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            name = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("case") {
            case = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("tags") {
            let array = meta.value()?.parse::<ExprArray>()?;

            for item in array.elems {
                if let syn::Expr::Lit(lit) = item {
                    if let syn::Lit::Str(lit_str) = lit.lit {
                        tags.push(lit_str.value());
                    } else {
                        return Err(meta.error("Expected string literal."));
                    }
                } else {
                    return Err(meta.error("Expected string literal."));
                }
            }

            Ok(())
        } else if meta.path.is_ident("should_panic") {
            should_panic = true;
            Ok(())
        } else if meta.path.is_ident("should_fail") {
            should_fail = true;
            Ok(())
        } else {
            Err(meta.error(
                "Allowed attributes are `name`, `case`, `tags`, `should_panic`, and `should_fail`.",
            ))
        }
    });

    parse_macro_input!(attr with test_parser);

    if should_fail && should_panic {
        return quote! {
            compile_error!("You cannot set both `should_panic` and `should_fail`.");
        }
        .into();
    }

    let case_tokens = if let Some(case_str) = case {
        quote! { Some(#case_str.to_string()) }
    } else {
        quote! { None }
    };

    let name_tokens = if let Some(name_str) = name {
        quote! { #name_str.to_string() }
    } else {
        quote! { stringify!(#fn_name).to_string() }
    };

    let registration_fn_name =
        syn::Ident::new(&format!("__testify_register_{fn_name}"), fn_name.span());

    let test_fn = if is_async {
        quote! {
            #[doc(hidden)]
            fn __testify_test_fn() -> impl testify::test::TestTermination {
                #[inline(always)]
                fn __testify_inner() #fn_return_type {
                    let __testify_result = testify::ASYNC_RT.block_on(async {
                        #fn_block
                    });
                    __testify_result
                }
                __testify_inner()
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            fn __testify_test_fn() -> impl testify::test::TestTermination {
                #[inline(always)]
                fn __testify_inner() #fn_return_type #fn_block
                __testify_inner()
            }
        }
    };

    quote! {
        fn #fn_name() -> testify::test::TestStatus {
            use std::panic;
            use testify::test::{TestStatus, TestTermination};

            let __testify_result = panic::catch_unwind(|| {
                // The test is recreated so that the compiler can infer the return type.
                #test_fn
                __testify_test_fn()
                // termination_bound(test_fn())
            });

            match __testify_result {
                Err(e) => {
                    if #should_panic {
                        return TestStatus::Passed;
                    } else {
                        return TestStatus::Panicked;
                    }
                },
                // testify::utils::termination_to_test_result(r, #should_fail)
                Ok(r) => {
                    let success = r.success();

                    if #should_panic {
                        return TestStatus::NotPanicked;
                    }

                    if #should_fail {
                        if success { TestStatus::NotFailed } else { TestStatus::Passed }
                    } else {
                        if success { TestStatus::Passed } else { TestStatus::Failed }
                    }
                },
            }
        }

        #[doc(hidden)]
        #[testify::ctor::ctor(
            crate_path = testify::ctor
        )]
        fn #registration_fn_name() {
            use testify::{TESTS, test::Test};

            let mut tests = TESTS.lock().unwrap();

            tests.push(Test {
                name: #name_tokens,
                case: #case_tokens,
                tags: vec![#(#tags.to_string()),*],
                function: #fn_name,
            });
        }
    }
    .into()
}

/// Runs the test environment setup before the execution of the tests.
#[proc_macro_attribute]
pub fn setup(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let fn_name = &item.sig.ident;
    let fn_block = &item.block;

    let is_async = item.sig.asyncness.is_some();

    if is_async && !cfg!(feature = "async-tokio") {
        return quote! {
            compile_error!("This function is async but the `async-tokio` feature is not enabled. Enable it to use async tests.");
        }.into();
    }

    let setup_runner_fn = if is_async {
        quote! {
            #[doc(hidden)]
            fn __testify_async_setup_runner() {
                testify::ASYNC_RT.block_on(async {
                    #fn_block
                });
            }

            #[doc(hidden)]
            #[testify::ctor::ctor(
                crate_path = testify::ctor
            )]
            fn __testify_register_setup() {
                use testify::SETUP;

                let mut __testify_setup = SETUP.lock().unwrap();

                *__testify_setup = Some(__testify_async_setup_runner);
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            #[testify::ctor::ctor(
                crate_path = testify::ctor
            )]
            fn __testify_register_setup() {
                use testify::SETUP;

                let mut __testify_setup = SETUP.lock().unwrap();

                *__testify_setup = Some(#fn_name);
            }
        }
    };

    quote! {
        fn #fn_name() #fn_block

        #setup_runner_fn
    }
    .into()
}

/// Runs the test environment cleanup after the execution of the tests.
#[proc_macro_attribute]
pub fn cleanup(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let fn_name = &item.sig.ident;
    let fn_block = &item.block;

    let is_async = item.sig.asyncness.is_some();

    if is_async && !cfg!(feature = "async-tokio") {
        return quote! {
            compile_error!("This function is async but the `async-tokio` feature is not enabled. Enable it to use async tests.");
        }.into();
    }

    let cleanup_runner_fn = if is_async {
        quote! {
            #[doc(hidden)]
            fn __testify_async_cleanup_runner() {
                testify::ASYNC_RT.block_on(async {
                    #fn_block
                });
            }

            #[doc(hidden)]
            #[testify::ctor::ctor(
                crate_path = testify::ctor
            )]
            fn __testify_register_cleanup() {
                use testify::CLEANUP;

                let mut __testify_cleanup = CLEANUP.lock().unwrap();

                *__testify_cleanup = Some(__testify_async_cleanup_runner);
            }
        }
    } else {
        quote! {
            #[testify::ctor::ctor(
                crate_path = testify::ctor
            )]
            fn __testify_register_cleanup() {
                use testify::CLEANUP;

                let mut __testify_cleanup = CLEANUP.lock().unwrap();

                *__testify_cleanup = Some(#fn_name);
            }
        }
    };

    quote! {
        fn #fn_name() #fn_block

        #cleanup_runner_fn
    }
    .into()
}
