extern crate core;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Nothing, Result};
use syn::{parse_macro_input, parse_quote, ItemFn, Lit};

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

        TIMED_FUNCTIONS.lock().unwrap().iter().for_each(|func| {
            println!(
                "\t{}: {} ({:.2}%)",
                func.1,
                func.0,
                (func.0) as f64 / total_cpu as f64 * 100.0,
            );
        })
    }));

    quote!(
        use std::sync::{LazyLock, Mutex};
        use platform_metrics::read_cpu_timer;

        pub struct Timer {
            pub name: String,
            pub start: u64,
        }

        impl Timer {
            pub fn new(name: &str) -> Self {
                Self {
                    name: name.to_string(),
                    start: read_cpu_timer(),
                }
            }
        }

        impl Drop for Timer {
            fn drop(&mut self) {
                let function_end = read_cpu_timer();

                unsafe {
                    TIMED_FUNCTIONS
                        .lock()
                        .unwrap()
                        .push((function_end - self.start, self.name.to_string()));
                }
            }
        }

        pub static TIMED_FUNCTIONS: LazyLock<Mutex<Vec<(u64, String)>>> = LazyLock::new(|| Mutex::new(vec![]));
        #function
    )
}

fn expand_timing(mut function: ItemFn) -> TokenStream2 {
    let name = function.sig.ident.clone().to_string();
    let stmts = function.block.stmts;
    function.block = Box::new(parse_quote!({
        use platform_metrics::read_cpu_timer;
        use crate::TIMED_FUNCTIONS;

        let function_start = read_cpu_timer();

        let output = {
            #(#stmts)*
        };

        let function_end = read_cpu_timer();

        unsafe {
            TIMED_FUNCTIONS.lock().unwrap().push((function_end - function_start, #name.to_string()));
        }

        output
    }));

    quote!(#function)
}

/// Macro to instrumentally time an expression or block of code.
/// Requires that main is marked with `#[time_main]`
///
/// Must be inputted as a tuple.
///
/// time_block!(("block_name", let a = 5))
#[proc_macro]
pub fn time_block(input: TokenStream) -> TokenStream {
    let block_name: Lit = parse_macro_input!(input as Lit);
    quote!(
        use crate::Timer;

        let timer = Timer::new(#block_name);
    )
    .into()
}
