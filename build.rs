extern crate clap;

use std::env;
use clap::Shell;

include!("src/cli.rs");

fn main() {
    let mut app = get_cli();
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh].iter() {
        app.gen_completions("xcolor", *shell, env::var("OUT_DIR").unwrap());
    }
}
