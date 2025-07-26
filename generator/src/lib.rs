mod generator;

use clap::Parser;
pub use generator::Generator;
use std::fmt::Display;

#[derive(Debug, Default, clap::ValueEnum, Clone)]
pub enum Type {
    Uniform,
    #[default]
    Cluster,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cluster => write!(f, "cluster"),
            Self::Uniform => write!(f, "uniform"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Uniform or cluster
    #[arg(
        short,
        long,
        num_args(0..=1),
        require_equals(false),
        default_value_t = Type::Cluster
    )]
    pub ty: Type,

    /// Whether to dump the memory
    #[arg(short, long, require_equals(false))]
    pub number: u64,
}
