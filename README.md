# Rust template repository.

An opinionated starting point for rust projects such as

 - systemd services
 - command line tools
 - client programs
 - server programs
 - libraries and daemons


# Logging 

The program will log in 2 places by default:

 - `stdout`
 - `syslog`

There is a simple `-v` `--verbose` flag that can be toggled on/off to increase and decrease the level of the logs.

Enabling verbose mode will simply add `Trace` and `Debug` levels to the default configuration.

| Default Runtime   | +Verbose       |
|-------------------|----------------|
 | Info, Warn, Error | +Trace, +Debug |


# Flags

We prefer flags over environmental variables for runtime configuration.

Flags can be added to the `main.rs` file following the official [clap examples](https://github.com/clap-rs/clap/tree/v2.33.0/examples)


# Clion

I use [clion](https://www.jetbrains.com/clion/) to develop rust. I use a few features: 

### Auto Imports 

This will automatically "fix" my `use` statements in the `2021` edition of Rust.

```
Editor > General > Auto Import > Rust
 [X] Import out-of-scope items on completion.
```

### Auto Formatting 

This will automatically `rustfmt` my code when I save.

```
Languages and Frameworks > Rust > Rustfmt
 [X] Run rustfmt on save
```



