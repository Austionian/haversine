mod parser;

use clap::Parser;
pub use parser::parse;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// input json file path
    #[arg(short, long, require_equals(false))]
    pub json_path: String,

    /// answer file path
    #[arg(short, long, require_equals(false))]
    pub answer_path: String,
}
