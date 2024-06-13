use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// A constant epsilon value to be sent to the actors (Useful for evaluation).
    ///
    /// If unset, chooses actor epsilons according to the formula used by Ape-X (Useful for training)
    #[arg(short, long, required = false)]
    pub eps_constant: Option<f64>,
}
