use std::path::PathBuf;

use dir_cmp::{full::compare_dirs, Options};

use clap::Parser;
use log::debug;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to compare
    left: PathBuf,
    /// Path to compare
    right: PathBuf,

    /// compare sub directories recursivly
    #[arg(short, long)]
    recursive: bool,

    /// show identical files
    #[arg(short)]
    show_same: bool,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    //create options without any restrictions
    let diff_options = Options {
        ignore_equal: !cli.show_same,
        ignore_left_only: false,
        ignore_right_only: false,
        filter: None,
        recursive: cli.recursive,
    };

    debug!("used options: {:?}", diff_options);

    let result = compare_dirs(&cli.left, &cli.right, diff_options).unwrap();
    println!("{:?}", result)
}
