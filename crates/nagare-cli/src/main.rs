use std::env;
use std::process::ExitCode;

mod args;
mod commands;
mod output;

fn main() -> ExitCode {
    match commands::run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
