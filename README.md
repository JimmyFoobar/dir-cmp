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

## Full vs Light
The `compare_dirs` function is implemented in two flavors: full and light.
The difference is that `full::compare_dirs` compares also file contents while `light::compare_dirs` only compares names / pathes.