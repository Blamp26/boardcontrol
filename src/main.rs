mod backend;
mod board;
mod cli;
mod error;
mod linux;
mod nct;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
