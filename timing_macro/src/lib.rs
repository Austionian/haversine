extern crate core;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Nothing, Result};
use syn::{ItemFn, Lit, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn time_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_timing(function);
            quote! {
                #[cfg(not(doc))]
                #expanded
                // Keep generated parameter names out of doc builds.
                #[cfg(doc)]
                #input
            }
        }
        Err(parse_error) => {
            let compile_error = parse_error.to_compile_error();
            quote! {
                #compile_error
                #input
            }
        }
    })
}

/// Example use of `#[time_main]`
///
/// ```
/// use timing_macro::{time_block, time_function, time_main};
///
/// #[time_main]
/// fn main() {
///     let ans = fib(6);
///
///     assert_eq!(ans, 13);
///
///     // inside baseball - shows the fib function was timed as a single function
///     // and was executed 25 times.
///     assert_eq!(TIMED.lock().unwrap().len(), 1);
///     assert_eq!(TIMED.lock().unwrap().get("fib").unwrap().count, 25);
///}
///
///
/// #[time_function]
/// fn fib(x: usize) -> usize {
///     if x == 0 || x == 1 {
///         return 1;
///     }
///
///     fib(x - 1) + fib(x - 2)
/// }
#[proc_macro_attribute]
pub fn time_main(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = TokenStream2::from(input);
    TokenStream::from(match parse(args, input.clone()) {
        Ok(function) => {
            let expanded = expand_main(function);
            quote! {
                #[cfg(not(doc))]
                #expanded
                // Keep generated parameter names out of doc builds.
                #[cfg(doc)]
                #input
            }
        }
        Err(parse_error) => {
            let compile_error = parse_error.to_compile_error();
            quote! {
                #compile_error
                #input
            }
        }
    })
}

fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemFn> {
    let function: ItemFn = syn::parse2(input)?;
    let _: Nothing = syn::parse2::<Nothing>(args)?;

    Ok(function)
}

fn expand_main(mut function: ItemFn) -> TokenStream2 {
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use platform_metrics::{read_cpu_timer, read_os_timer, get_os_time_freq};

        let time_start = read_os_timer();
        let cpu_start = read_cpu_timer();

        #(#stmts)*

        let cpu_end = read_cpu_timer();
        let time_end = read_os_timer();

        let total_cpu = cpu_end - cpu_start;
        let total_time = time_end - time_start;

        println!(
            "Total time: {:.4}ms (CPU freq {:.0})",
            total_time as f64 / 1_000.0,
            get_os_time_freq() as f64 * total_cpu as f64 / total_time as f64
        );

        TIMED.lock().unwrap().iter().for_each(|(key, value)| {
            println!(
                "\t{}[{}]: {} ({:.2}%)",
                key,
                value.count,
                value.cycles,
                (value.cycles) as f64 / total_cpu as f64 * 100.0,
            );
        })
    }));

    quote!(
        use std::sync::{LazyLock, Mutex};
        use platform_metrics::read_cpu_timer;
        use std::collections::HashMap;

        #[derive(Clone, Debug)]
        pub struct Timer {
            pub name: String,
            pub start: u64,
        }

        pub struct Timed {
            pub count: usize,
            pub cycles: u64,
    }

        impl Timer {
            pub fn new(name: &str) -> Self {
                let timer = Self {
                    name: name.to_string(),
                    start: read_cpu_timer(),
                };

                TIMING_STACK.lock().unwrap().push((timer.start, name.to_string()));

                timer
            }
        }

        impl Drop for Timer {
            fn drop(&mut self) {
                let function_end = read_cpu_timer();

                let mut lock = TIMING_STACK.lock().unwrap();
                let timer = lock.pop().expect("Pop on an empty vec");
                let mut cycles = function_end - timer.0;

                // Check if there's a parent in the stack
                if lock.len() > 0 {
                    lock.iter_mut().for_each(|parent| {
                        parent.0 += cycles;
                    });
                }

                unsafe {
                    TIMED
                        .lock()
                        .unwrap()
                        .entry(timer.1.clone())
                        .and_modify(|timed| {
                            timed.count += 1;
                            timed.cycles += cycles;
                        })
                        .or_insert(Timed {
                            count: 1,
                            cycles,
                        });
                }
            }
        }

        pub static TIMING_STACK: LazyLock<Mutex<Vec<(u64, String)>>> = LazyLock::new(|| Mutex::new(vec![]));
        pub static TIMED: LazyLock<Mutex<HashMap<String, Timed>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
        #function
    )
}

fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use platform_metrics::read_cpu_timer;
        use timing_macro::time_block;
        use crate::{TIMING_STACK, TIMED};

        let output = {
            time_block!(#name);

            #(#stmts)*
        };

        output
    }));

    quote!(#function)
}

/// Macro to instrumentally time a block of code.
/// Requires that main is marked with `#[time_main]`
///
/// ```ignore
/// let output = {
///     time_block!("block_name");
///
///     // expressions
/// }
/// ```
#[proc_macro]
pub fn time_block(input: TokenStream) -> TokenStream {
    let block_name: Lit = parse_macro_input!(input as Lit);
    quote!(
        use crate::Timer;

        let timer = Timer::new(#block_name);
    )
    .into()
}
