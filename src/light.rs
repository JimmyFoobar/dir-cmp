use log::{debug, error, trace};

use std::io;
use std::path::Path;

use crate::{list_files, zip_dir_entries, EitherOrBoth, Filter};

fn compare_dirs_inner(
    left_path: &Path,
    right_path: &Path,
    left_base: &str,
    right_base: &str,
    filter: &Option<Filter>,
) -> io::Result<Vec<EitherOrBoth>> {
    trace!("comparing 2 dirs");

    let mut results: Vec<EitherOrBoth> = Vec::new();
    for dir_entry in zip_dir_entries(
        &left_path.to_path_buf(),
        &right_path.to_path_buf(),
        left_base,
        right_base,
        filter,
    )? {
        match dir_entry {
            EitherOrBoth::Both(left_entry, right_entry) => {
                trace!("left and right dir have the same entry");
                debug!("comparing{:?} vs {:?}", left_entry, right_entry);
                let subtree_results = compare_dirs_inner(
                    left_entry.as_path(),
                    right_entry.as_path(),
                    left_base,
                    right_base,
                    filter,
                )?;
                results.extend(subtree_results);
            }
            EitherOrBoth::Left(left_entry) => {
                trace!("missing entry in right dir");
                if left_entry.is_dir() {
                    let entry_list = list_files(&left_entry);
                    for file_path in entry_list {
                        results.push(EitherOrBoth::Left(file_path));
                    }
                    continue;
                }
                if left_entry.is_file() {
                    results.push(EitherOrBoth::Left(left_entry));
                    continue;
                }
                if left_entry.is_symlink() {
                    //ignore
                    continue;
                }
            }
            EitherOrBoth::Right(right_entry) => {
                trace!("extra entry in right dir");
                if right_entry.is_dir() {
                    let entry_list = list_files(&right_entry);
                    for file_path in entry_list {
                        results.push(EitherOrBoth::Right(file_path));
                    }
                    continue;
                }
                if right_entry.is_file() {
                    results.push(EitherOrBoth::Right(right_entry));
                    continue;
                }
                if right_entry.is_symlink() {
                    //ignore
                    continue;
                }
            }
        }
    }
    Ok(results)
}

pub fn compare_dirs(
    left_path: &Path,
    right_path: &Path,
    filter: Option<Filter>,
) -> io::Result<Vec<EitherOrBoth>> {
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

    compare_dirs_inner(left_path, right_path, left_base, right_base, &filter)
}
