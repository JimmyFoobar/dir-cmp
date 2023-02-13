# dir-cmp
This Rust library aims to provide convient functions to compare to two file trees.

## Library

### Usage
```rust
    //define options
    let diff_options = Options {
        ignore_left_only: false,
        ignore_right_only: false,
        filter: None,
        ignore_equal: false,
        recursive: true,
    };

    //get dirs to compare
    let left_dir = Path::new("./foo");
    let right_dir = Path::new("./bar");

    //compare
    let result = compare_dirs(&left_dir, &right_dir, diff_options);
```

#### Filter
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
        recursive: true,
    };

    //get dirs to compare
    let left_dir = Path::new("./foo");
    let right_dir = Path::new("./bar");

    //compare
    let result = compare_dirs(&left_dir, &right_dir, diff_options);
```

### Full vs Light
The `compare_dirs` function is implemented in two flavors: full and light.
The difference is that `full::compare_dirs` compares also file contents while `light::compare_dirs` only compares names / pathes.

## Cli
The lib can be tested using the Cli provided in this repo. It was inspired by `diff`, but only covers the basic functionality.

### Usage
```bash
    dir-cmp -h
    dir-cmp -r <LEFT> <RIGHT>
```

## Performance
To evaluate the speed of this library, we can compare it against `diff`.

One simple approach is to simply compare this repo against itself:

```
    time diff -r . . 
        // 0.01s user 0.19s system 99% cpu 0.209 total
    time dir-cmp -r . . 
        // 0.26s user 0.46s system 75% cpu 0.945 total
```
The  result clearly shows that there is a lot of room for improvement.