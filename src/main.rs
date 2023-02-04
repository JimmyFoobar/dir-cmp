use std::{fs, path::{Path, PathBuf}};

use dir_cmp::{full::compare_dirs, Options};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    left: PathBuf,
    right:PathBuf,
}

fn main() {
    let cli = Cli::parse();

    //create options without any restrictions
    let diff_options = Options {
        ignore_left_only: false,
        ignore_right_only: false,
        filter: None,
        ignore_equal: false,
    };

    let result = compare_dirs(&cli.left, &cli.right, diff_options).unwrap();
    println!("{:?}", result)
}
