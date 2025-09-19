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

        // Create a vec of times to sort. doesn't affect instrumentation timing as all the timing
        // information has already been captured.
        let mut times = Vec::new();

        // Consuming a LazyLock is only in nightly, so manually create the vec.
        TIMED.lock().unwrap().iter().for_each(|(key, value)| {
            times.push((key.clone(), value.clone()));
        });

        times.sort_by(|(_, Timed { cycles: a, .. }), (_, Timed { cycles: b, ..})| b.cmp(&a));
        times.iter().for_each(|(key, value)| {
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

        #[derive(Clone)]
        pub struct Timed {
            pub count: usize,
            pub cycles: u64,
        }

        // A function/ block in the timing stack currently being executed
        pub struct Timing {
            // name of the function/ block being timed
            pub name: String,
            // cycle at which the function/ block started
            pub start: u64,
            // total number of cycles spent executing this function/ block
            pub cycles: u64,
        }

        pub struct TimingStack {
            // the stack of functions/ blocks being timed
            pub stack: Vec<Timing>,
        }

        impl TimingStack {
            pub fn new() -> Self {
                TimingStack {
                    stack: vec![],
                }
            }

            pub fn push(&mut self, value: Timing) {
                self.stack.push(value);
            }

            /// Returns the Timing instance its cycle count
            pub fn pop(&mut self, function_end: u64) -> Timing {
                let mut timing = self.stack.pop().expect("pop on an empty timing stack");
                timing.cycles = function_end - timing.start;

                if self.stack.len() > 0 {
                    // Accumulate the time spent in the popped function/ block to its parents.
                    //
                    // Wish there was a cleaner way to do this w/o a loop, but need to mark when
                    // some accumulating value is relevent or not.
                    self.stack.iter_mut().for_each(|parent| {
                        parent.start += timing.cycles;
                    });
                }

                timing
            }
        }

        impl Timing {
            pub fn new(name: &str, start: u64) -> Self {
                Timing {
                    name: name.to_string(),
                    start,
                    cycles: 0,
                }
            }
        }

        impl Timer {
            pub fn new(name: &str) -> Self {
                let timer = Self {
                    name: name.to_string(),
                    start: read_cpu_timer(),
                };

                TIMING_STACK.lock().expect("unable to lock when pushing").push(
                    Timing::new(name, timer.start)
                );

                timer
            }
        }

        impl Drop for Timer {
            fn drop(&mut self) {
                let Timing { name, cycles, .. } = TIMING_STACK
                    .lock()
                    .expect("unable to lock when dropping")
                    .pop(read_cpu_timer());

                unsafe {
                    TIMED
                        .lock()
                        .unwrap()
                        .entry(name.clone())
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

        // initialize the global variables
        pub static TIMING_STACK: LazyLock<Mutex<TimingStack>> = LazyLock::new(|| Mutex::new(TimingStack::new()));
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
