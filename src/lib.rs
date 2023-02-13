pub mod full;
pub mod light;

//use log::debug;
use regex::Regex;
use std::fs;
use std::{io, path::PathBuf};

#[derive(Debug)]
pub enum Filter {
    Exclude(Vec<Regex>),
    Include(Vec<Regex>),
}
//returns true if the path should be filtered out
fn apply_filter(path: &str, filter_opt: &Option<Filter>) -> bool {
    if let Some(filter) = filter_opt {
        match filter {
            Filter::Exclude(pattern_list) => {
                for pattern in pattern_list {
                    if pattern.is_match(path) {
                        return true;
                    }
                }
            }
            Filter::Include(pattern_list) => {
                for pattern in pattern_list {
                    if !pattern.is_match(path) {
                        return true;
                    }
                }
            }
        }
    }
    //default if no filter values are provided
    false
}

#[cfg(test)]
mod tests_apply_filter {
    use super::*;

    // fn init() {
    //     let _ = env_logger::builder().is_test(true).try_init();
    // }

    #[test]
    fn empty() {
        let path = ".git/config";
        let filter = Some(Filter::Include(Vec::new()));

        assert!(!apply_filter(path, &filter));
    }

    #[test]
    fn none() {
        let path = ".git/config";
        let filter = None;

        assert!(!apply_filter(path, &filter));
    }

    #[test]
    fn include() {
        let path = "src/main.rs";
        let regex = Regex::new(r".rs").unwrap();
        let filter = Some(Filter::Include(vec![regex]));

        assert!(!apply_filter(path, &filter));
    }

    #[test]
    fn exclude() {
        let path = ".git/config";
        let regex = Regex::new(".git").unwrap();
        let filter = Some(Filter::Exclude(vec![regex]));

        assert!(apply_filter(path, &filter));
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EitherOrBoth {
    Both(PathBuf, PathBuf),
    Left(PathBuf),
    Right(PathBuf),
}

fn zip_dir_entries(
    left_dir: &PathBuf,
    right_dir: &PathBuf,
    left_base: &str,
    right_base: &str,
    filter: &Option<Filter>,
) -> io::Result<Vec<EitherOrBoth>> {
    let left_read_dir = fs::read_dir(left_dir)?;
    let right_read_dir = fs::read_dir(right_dir)?;

    let left_entries = left_read_dir
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("some error with left dir");

    let right_entries = right_read_dir
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("some error with right dir");

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    //left_entries.sort();
    //right_entries.sort();
    let mut results: Vec<EitherOrBoth> = Vec::new();

    for left_entry in &left_entries {
        //debug!("left entry: {:?}", left_entry);
        let left_short_path = left_entry.strip_prefix(left_base).unwrap();
        if !apply_filter(left_short_path.to_str().unwrap(), filter) {
            let mut found_match = None;
            for right_entry in &right_entries {
                let right_short_path = right_entry.strip_prefix(right_base).unwrap();
                if left_short_path == right_short_path {
                    found_match = Some(EitherOrBoth::Both(
                        left_entry.to_owned(),
                        right_entry.to_owned(),
                    ));
                }
            }

            match found_match {
                Some(both) => results.push(both),
                None => results.push(EitherOrBoth::Left(left_entry.to_owned())),
            }
        }
    }

    for right_entry in &right_entries {
        let right_short_path = right_entry.strip_prefix(right_base).unwrap();
        if !apply_filter(right_short_path.to_str().unwrap(), filter) {
            let mut found_match = None;
            for left_entry in &left_entries {
                let left_short_path = left_entry.strip_prefix(left_base).unwrap();
                if left_short_path == right_short_path {
                    found_match = Some(());
                }
            }

            if found_match.is_none() {
                results.push(EitherOrBoth::Right(right_entry.to_owned()));
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests_zip_dir_entries {
    use super::*;
    use std::fs;

    fn create_temp_dir() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("compare_zip_dirs_")
            .tempdir()
            .unwrap()
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn emtpy() {
        init();
        let left_dir = create_temp_dir();
        let left_path_buf = left_dir.into_path();
        let left_base = left_path_buf.to_str().unwrap();

        let right_dir = create_temp_dir();
        let right_path_buf = right_dir.into_path();
        let right_base = right_path_buf.to_str().unwrap();

        let result = zip_dir_entries(
            &left_path_buf,
            &right_path_buf,
            left_base,
            right_base,
            &None,
        )
        .unwrap();

        assert_eq!(result, Vec::<EitherOrBoth>::new());
    }

    #[test]
    fn both() {
        init();
        let left_dir = create_temp_dir();
        let left_file = left_dir.path().join("file1");
        fs::write(left_file.as_path(), b"Hello, world!").unwrap();
        let left_path_buf = left_dir.into_path();
        let left_base = left_path_buf.to_str().unwrap();

        let right_dir = create_temp_dir();
        let right_file = right_dir.path().join("file1");
        fs::write(right_file.as_path(), b"Hello, world!").unwrap();
        let right_path_buf = right_dir.into_path();
        let right_base = right_path_buf.to_str().unwrap();

        let result = zip_dir_entries(
            &left_path_buf,
            &right_path_buf,
            left_base,
            right_base,
            &None,
        )
        .unwrap();

        assert_eq!(result, vec![EitherOrBoth::Both(left_file, right_file)]);
    }

    #[test]
    fn both_subdir() {
        init();
        let left_dir = create_temp_dir();
        let left_sub_dir = left_dir.path().join("subdir");
        fs::create_dir(left_sub_dir.as_path()).unwrap();
        let left_file = left_sub_dir.as_path().join("file1");
        fs::write(left_file.as_path(), b"Hello, world!").unwrap();
        let left_base = left_dir.path().to_str().unwrap();

        let right_dir = create_temp_dir();
        let right_sub_dir = right_dir.path().join("subdir");
        fs::create_dir(right_sub_dir.as_path()).unwrap();
        let right_file = right_sub_dir.as_path().join("file1");
        fs::write(right_file.as_path(), b"Hello, world!").unwrap();
        let right_base = right_dir.path().to_str().unwrap();

        let result =
            zip_dir_entries(&left_sub_dir, &right_sub_dir, left_base, right_base, &None).unwrap();

        assert_eq!(result, vec![EitherOrBoth::Both(left_file, right_file)]);
    }
    #[test]
    fn left() {
        init();
        let left_dir = create_temp_dir();
        let left_file = left_dir.path().join("file1");
        fs::write(left_file.as_path(), b"Hello, world!").unwrap();
        let left_path_buf = left_dir.into_path();
        let left_base = left_path_buf.to_str().unwrap();

        let right_dir = create_temp_dir();
        let right_path_buf = right_dir.into_path();
        let right_base = right_path_buf.to_str().unwrap();

        let result = zip_dir_entries(
            &left_path_buf,
            &right_path_buf,
            left_base,
            right_base,
            &None,
        )
        .unwrap();
        assert_eq!(result, vec![EitherOrBoth::Left(left_file)]);
    }
    #[test]
    fn right() {
        init();
        let left_dir = create_temp_dir();
        let left_path_buf = left_dir.into_path();
        let left_base = left_path_buf.to_str().unwrap();

        let right_dir = create_temp_dir();
        let right_file = right_dir.path().join("file1");
        fs::write(right_file.as_path(), b"Hello, world!").unwrap();
        let right_path_buf = right_dir.into_path();
        let right_base = right_path_buf.to_str().unwrap();

        let result = zip_dir_entries(
            &left_path_buf,
            &right_path_buf,
            left_base,
            right_base,
            &None,
        )
        .unwrap();

        assert_eq!(result, vec![EitherOrBoth::Right(right_file)]);
    }
}

fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();

    let read_dir = fs::read_dir(path).unwrap();

    let dir_entries = read_dir
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("some error with left dir");
    for entry in dir_entries {
        if entry.is_dir() {
            //get elements from sub dirs
            let mut subtree_results = list_files(&entry);
            result.append(&mut subtree_results);
            continue;
        }
        if entry.is_file() {
            result.push(entry);
            continue;
        }
        if entry.is_symlink() {
            //ignore
            continue;
        }
    }
    result
}

#[derive(Debug)]
pub struct Options {
    pub ignore_equal: bool,
    pub ignore_left_only: bool,
    pub ignore_right_only: bool,
    pub filter: Option<Filter>,
    pub recursive: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileCompResult {
    Equal,
    Different,
}
fn compare_two_files(left_path: &PathBuf, right_path: &PathBuf) -> io::Result<FileCompResult> {
    let left_file = fs::read(left_path)?;
    let right_file = fs::read(right_path)?;

    if left_file == right_file {
        Ok(FileCompResult::Equal)
    } else {
        Ok(FileCompResult::Different)
    }
}
