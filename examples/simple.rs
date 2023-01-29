use std::fs;

use dir_cmp::full::{compare_dirs, Options};

fn main() {
    //prepare left dir
    let left_dir = tempfile::Builder::new().tempdir().unwrap();
    let file_both = left_dir.path().join("both.txt");
    fs::write(file_both.as_path(), b"Hello, world!").unwrap();
    let file_left_only = left_dir.path().join("left_only.txt");
    fs::write(file_left_only.as_path(), b"Hello, world!").unwrap();

    //prepare right dir
    let right_dir = tempfile::Builder::new().tempdir().unwrap();
    let file_both = left_dir.path().join("both.txt");
    fs::write(file_both.as_path(), b"Hello, world!").unwrap();
    let file_right_only = left_dir.path().join("right_only.txt");
    fs::write(file_right_only.as_path(), b"Hello, world!").unwrap();

    //create options without any restrictions
    let diff_options = Options {
        ignore_left_only: false,
        ignore_right_only: false,
        filter: None,
        ignore_equal: false,
    };

    //compare
    // --> [DirCompEntry::Both(<left_path>/both.txt, <right_path>/both.txt, FileCompResult::Equal),
    //      DirCompEntry::Both(<left_path>/left_only.txt),
    //      DirCompEntry::Both(<right_path>/right_only.txt)]
    let result = compare_dirs(left_dir.path(), right_dir.path(), diff_options).unwrap();
    println!("{:?}", result)
}
