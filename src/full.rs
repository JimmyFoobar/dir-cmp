use log::{debug, error, trace};
use std::path::Path;
use std::{io, path::PathBuf};

use crate::{
    compare_two_files, list_files, zip_dir_entries, EitherOrBoth, FileCompResult, Options,
};

#[derive(PartialEq, Eq, Debug)]
pub enum DirCmpEntry {
    Both(PathBuf, PathBuf, FileCompResult),
    Left(PathBuf),
    Right(PathBuf),
}

fn compare_dirs_inner(
    left_path: &Path,
    right_path: &Path,
    left_base: &str,
    right_base: &str,
    options: &Options,
) -> io::Result<Vec<DirCmpEntry>> {
    trace!("comparing 2 dirs");

    let mut results: Vec<DirCmpEntry> = Vec::new();
    for dir_entry in zip_dir_entries(
        &left_path.to_path_buf(),
        &right_path.to_path_buf(),
        left_base,
        right_base,
        &options.filter,
    )? {
        match dir_entry {
            EitherOrBoth::Both(left_entry, right_entry) => {
                //handle two files
                if left_entry.is_file() && right_entry.is_file() {
                    let comp_result = compare_two_files(&left_entry, &right_entry)?;
                    if FileCompResult::Equal != comp_result || !options.ignore_equal {
                        results.push(DirCmpEntry::Both(
                            left_entry.to_owned(),
                            right_entry.to_owned(),
                            comp_result,
                        ));
                    }
                }

                //handle two dirs
                if left_entry.is_dir() && right_entry.is_dir() {
                    let subtree_results = compare_dirs_inner(
                        left_entry.as_path(),
                        right_entry.as_path(),
                        left_base,
                        right_base,
                        options,
                    )?;
                    results.extend(subtree_results);
                }

                //ignore symlinks and mismatches
            }
            EitherOrBoth::Left(left_entry) => {
                if !options.ignore_left_only {
                    if left_entry.is_dir() {
                        let entry_list = list_files(&left_entry);
                        for file_path in entry_list {
                            results.push(DirCmpEntry::Left(file_path));
                        }
                        continue;
                    }
                    if left_entry.is_file() {
                        results.push(DirCmpEntry::Left(left_entry));
                        continue;
                    }
                    if left_entry.is_symlink() {
                        //ignore
                        continue;
                    }
                }
            }
            EitherOrBoth::Right(right_entry) => {
                if !options.ignore_right_only {
                    if right_entry.is_dir() {
                        let entry_list = list_files(&right_entry);
                        for file_path in entry_list {
                            results.push(DirCmpEntry::Right(file_path));
                        }
                        continue;
                    }
                    if right_entry.is_file() {
                        results.push(DirCmpEntry::Right(right_entry));
                        continue;
                    }
                    if right_entry.is_symlink() {
                        //ignore
                        continue;
                    }
                }
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests_compare_dirs_inner {
    use super::*;
    use std::fs;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn no_restictions() {
        init_logger();
        //prepare left dir
        let left_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_left_both_equal = left_dir.path().join("both_equal.txt");
        fs::write(file_left_both_equal.as_path(), b"same same").unwrap();
        let file_left_both_diff = left_dir.path().join("both_diff.txt");
        fs::write(file_left_both_diff.as_path(), b"differnt").unwrap();
        let file_left_only = left_dir.path().join("left_only.txt");
        fs::write(file_left_only.as_path(), b"Lefty left").unwrap();

        //prepare right dir
        let right_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_right_both_equal = right_dir.path().join("both_equal.txt");
        fs::write(file_right_both_equal.as_path(), b"same same").unwrap();
        let file_right_both_diff = right_dir.path().join("both_diff.txt");
        fs::write(file_right_both_diff.as_path(), b"more different").unwrap();
        let file_right_only = right_dir.path().join("right_only.txt");
        fs::write(file_right_only.as_path(), b"Righty right").unwrap();

        //create options without any restrictions
        let diff_options = Options {
            ignore_left_only: false,
            ignore_right_only: false,
            filter: None,
            ignore_equal: false,
        };

        let expected: Vec<DirCmpEntry> = vec![
            DirCmpEntry::Left(file_left_only.as_path().to_path_buf()),
            DirCmpEntry::Both(
                file_left_both_diff.as_path().to_path_buf(),
                file_right_both_diff.as_path().to_path_buf(),
                FileCompResult::Different,
            ),
            DirCmpEntry::Both(
                file_left_both_equal.as_path().to_path_buf(),
                file_right_both_equal.as_path().to_path_buf(),
                FileCompResult::Equal,
            ),
            DirCmpEntry::Right(file_right_only.as_path().to_path_buf()),
        ];

        //compare
        let result = compare_dirs(left_dir.path(), right_dir.path(), diff_options).unwrap();

        assert_eq!(result, expected);
    }
    #[test]
    fn ignore_equal() {
        init_logger();
        //prepare left dir
        let left_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_left_both_equal = left_dir.path().join("both_equal.txt");
        fs::write(file_left_both_equal.as_path(), b"same same").unwrap();
        let file_left_both_diff = left_dir.path().join("both_diff.txt");
        fs::write(file_left_both_diff.as_path(), b"differnt").unwrap();
        let file_left_only = left_dir.path().join("left_only.txt");
        fs::write(file_left_only.as_path(), b"Lefty left").unwrap();

        //prepare right dir
        let right_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_right_both_equal = right_dir.path().join("both_equal.txt");
        fs::write(file_right_both_equal.as_path(), b"same same").unwrap();
        let file_right_both_diff = right_dir.path().join("both_diff.txt");
        fs::write(file_right_both_diff.as_path(), b"more different").unwrap();
        let file_right_only = right_dir.path().join("right_only.txt");
        fs::write(file_right_only.as_path(), b"Righty right").unwrap();

        //create options without any restrictions
        let diff_options = Options {
            ignore_equal: true,
            ignore_left_only: false,
            ignore_right_only: false,
            filter: None,
        };

        let expected: Vec<DirCmpEntry> = vec![
            DirCmpEntry::Left(file_left_only.as_path().to_path_buf()),
            DirCmpEntry::Both(
                file_left_both_diff.as_path().to_path_buf(),
                file_right_both_diff.as_path().to_path_buf(),
                FileCompResult::Different,
            ),
            DirCmpEntry::Right(file_right_only.as_path().to_path_buf()),
        ];

        //compare
        let result = compare_dirs(left_dir.path(), right_dir.path(), diff_options).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn ignore_left_only() {
        init_logger();
        //prepare left dir
        let left_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_left_both_equal = left_dir.path().join("both_equal.txt");
        fs::write(file_left_both_equal.as_path(), b"same same").unwrap();
        let file_left_both_diff = left_dir.path().join("both_diff.txt");
        fs::write(file_left_both_diff.as_path(), b"differnt").unwrap();
        let file_left_only = left_dir.path().join("left_only.txt");
        fs::write(file_left_only.as_path(), b"Lefty left").unwrap();

        //prepare right dir
        let right_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_right_both_equal = right_dir.path().join("both_equal.txt");
        fs::write(file_right_both_equal.as_path(), b"same same").unwrap();
        let file_right_both_diff = right_dir.path().join("both_diff.txt");
        fs::write(file_right_both_diff.as_path(), b"more different").unwrap();
        let file_right_only = right_dir.path().join("right_only.txt");
        fs::write(file_right_only.as_path(), b"Righty right").unwrap();

        //create options without any restrictions
        let diff_options = Options {
            ignore_equal: false,
            ignore_left_only: true,
            ignore_right_only: false,
            filter: None,
        };

        let expected: Vec<DirCmpEntry> = vec![
            DirCmpEntry::Both(
                file_left_both_diff.as_path().to_path_buf(),
                file_right_both_diff.as_path().to_path_buf(),
                FileCompResult::Different,
            ),
            DirCmpEntry::Both(
                file_left_both_equal.as_path().to_path_buf(),
                file_right_both_equal.as_path().to_path_buf(),
                FileCompResult::Equal,
            ),
            DirCmpEntry::Right(file_right_only.as_path().to_path_buf()),
        ];

        //compare
        let result = compare_dirs(left_dir.path(), right_dir.path(), diff_options).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn ignore_right_only() {
        init_logger();
        //prepare left dir
        let left_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_left_both_equal = left_dir.path().join("both_equal.txt");
        fs::write(file_left_both_equal.as_path(), b"same same").unwrap();
        let file_left_both_diff = left_dir.path().join("both_diff.txt");
        fs::write(file_left_both_diff.as_path(), b"differnt").unwrap();
        let file_left_only = left_dir.path().join("left_only.txt");
        fs::write(file_left_only.as_path(), b"Lefty left").unwrap();

        //prepare right dir
        let right_dir = tempfile::Builder::new().tempdir().unwrap();
        let file_right_both_equal = right_dir.path().join("both_equal.txt");
        fs::write(file_right_both_equal.as_path(), b"same same").unwrap();
        let file_right_both_diff = right_dir.path().join("both_diff.txt");
        fs::write(file_right_both_diff.as_path(), b"more different").unwrap();
        let file_right_only = right_dir.path().join("right_only.txt");
        fs::write(file_right_only.as_path(), b"Righty right").unwrap();

        //create options without any restrictions
        let diff_options = Options {
            ignore_equal: false,
            ignore_left_only: false,
            ignore_right_only: true,
            filter: None,
            
        };

        let expected: Vec<DirCmpEntry> = vec![
            DirCmpEntry::Left(file_left_only.as_path().to_path_buf()),
            DirCmpEntry::Both(
                file_left_both_diff.as_path().to_path_buf(),
                file_right_both_diff.as_path().to_path_buf(),
                FileCompResult::Different,
            ),
            DirCmpEntry::Both(
                file_left_both_equal.as_path().to_path_buf(),
                file_right_both_equal.as_path().to_path_buf(),
                FileCompResult::Equal,
            ),
        ];

        //compare
        let result = compare_dirs(left_dir.path(), right_dir.path(), diff_options).unwrap();

        assert_eq!(result, expected);
    }
}


pub fn compare_dirs(
    left_path: &Path,
    right_path: &Path,
    options: Options,
) -> io::Result<Vec<DirCmpEntry>> {
    debug!(
        "starting to compare for {:?} vs {:?}",
        left_path, right_path
    );

    if !left_path.exists() {
        error!("The left path does not exists!");
        panic!();
    }

    if !left_path.is_dir() {
        error!("The left path is not a directory!");
        panic!();
    }

    if !right_path.exists() {
        error!("The right path does not exists!");
        panic!();
    }

    if !right_path.is_dir() {
        error!("The right path is not a directory!");
        panic!();
    }

    let left_base = left_path.to_str().unwrap();
    let right_base = right_path.to_str().unwrap();

    compare_dirs_inner(left_path, right_path, left_base, right_base, &options)
}
