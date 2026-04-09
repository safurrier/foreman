use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = foreman::cli::Cli::parse();

    match foreman::cli::run(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
