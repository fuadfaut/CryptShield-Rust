mod app;
mod commands;
mod core;
mod state;

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--system-helper") {
        std::process::exit(commands::validate_system_helper_invocation(&args[2..]));
    }

    app::run()
}
