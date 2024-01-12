use std::env;
use std::process;

use crep::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        if err == "Help page" {
            process::exit(0);
        }
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = crep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
