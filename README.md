# dir-cmp
This Rust library aims to provide convient functions to compare to two files trees.

## Usage
```rust
    //define options
    let diff_options = Options {
        ignore_left_only: false,
        ignore_right_only: false,
        filter: None,
        ignore_equal: false,
    };

    //get dirs to compare
    let left_dir = Path::new("./foo");
    let right_dir = Path::new("./bar");

    //compare
    let result = compare_dirs(&left_dir, &right_dir, diff_options);
```

### Filter
In order blacklist(`exclude`) or whitelist (`include`) any folder or file names, a filter can be added to the compare options.
A filter consists of a list of regular expressions.

```rust
    // define filter to ignore ".git" directory
    let regex = Regex::new(r"\.git$").unwrap();
    let filter = Filter::Exclude(vec![regex]);
    
    //define options
    let diff_options = Options {
        ignore_left_only: false,
        ignore_right_only: false,
        filter: Some(filter),
        ignore_equal: false,
    };

    //get dirs to compare
    let left_dir = Path::new("./foo");
    let right_dir = Path::new("./bar");

    //compare
    let result = compare_dirs(&left_dir, &right_dir, diff_options);
```

## Full vs Light
The `compare_dirs` function is implemented in two flavors: full and light.
The difference is that `full::compare_dirs` compares also file contents while `light::compare_dirs` only compares names / pathes.