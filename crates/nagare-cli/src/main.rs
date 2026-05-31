use std::env;
use std::process::ExitCode;

mod args;
mod commands;
mod output;
mod ui;
mod ui_answer;
mod ui_assets;
mod ui_detail;
mod ui_form;
mod ui_history;
mod ui_html;
mod ui_pages;

fn main() -> ExitCode {
    match commands::run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
