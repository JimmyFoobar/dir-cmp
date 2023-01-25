use log::{debug, error, trace};
use std::path::Path;
use std::{io, path::PathBuf};

use crate::{compare_two_files, list_files, zip_dir_entries, EitherOrBoth, FileCompResult, Filter};

pub struct Options {
    pub ignore_left_only: bool,
    pub ignore_right_only: bool,
    pub ignore_equal: bool,
    pub filter: Option<Filter>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum DirCompEntry {
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
) -> io::Result<Vec<DirCompEntry>> {
    trace!("comparing 2 dirs");

    let mut results: Vec<DirCompEntry> = Vec::new();
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
                        results.push(DirCompEntry::Both(
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
                        &options,
                    )?;
                    results.extend(subtree_results);
                }

                //ignore symlinks and mismatches s
            }
            EitherOrBoth::Left(left_entry) => {
                if !options.ignore_left_only{
                    if left_entry.is_dir() {
                        let entry_list = list_files(&left_entry);
                        for file_path in entry_list {
                            results.push(DirCompEntry::Left(file_path));
                        }
                        continue;
                    }
                    if left_entry.is_file() {
                        results.push(DirCompEntry::Left(left_entry));
                        continue;
                    }
                    if left_entry.is_symlink() {
                        //ignore
                        continue;
                    }
                }
            }
            EitherOrBoth::Right(right_entry) => {
                if !options.ignore_right_only{
                    if right_entry.is_dir() {
                        let entry_list = list_files(&right_entry);
                        for file_path in entry_list {
                            results.push(DirCompEntry::Right(file_path));
                        }
                        continue;
                    }
                    if right_entry.is_file() {
                        results.push(DirCompEntry::Right(right_entry));
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

pub fn compare_dirs(
    left_path: &Path,
    right_path: &Path,
    options: Options,
) -> io::Result<Vec<DirCompEntry>> {
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
