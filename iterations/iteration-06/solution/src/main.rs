use std::collections::HashMap;

use rgit::Error;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let cwd = std::env::current_dir().unwrap();
    let env: HashMap<String, String> = std::env::vars().collect();
    let mut out = std::io::stdout();
    if let Err(error) = rgit::cli::run(&args, &cwd, &env, &mut out) {
        match error {
            Error::Usage(error) => error.exit(),
            error => {
                eprintln!("fatal: {error}");
                std::process::exit(128);
            }
        }
    }
}
