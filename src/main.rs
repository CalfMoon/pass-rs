use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Test argument
    #[arg(short, long)]
    test: String,
}

fn main() {
    let args = Args::parse();

    println!("{}", args.test);
}
