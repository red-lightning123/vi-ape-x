use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path from which the model definition should be loaded.
    #[arg(short, long, default_value = "model")]
    pub model_def_path: PathBuf,
}
