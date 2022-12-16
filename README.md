<div align="center">
  <h1><code>CTL</code></h1>
  
  <p>
    <strong>Compiling toolchain library</strong>
  </p>
</div>

# What it is

<strong>CTL</strong> is a program which parses source code (written in Rust's subset)
and generates an intermediate code from it.

![CTL diagram](/assets/general_diagram.png)

# Build and test

CTL can be <strong>built</strong> by the command below:
```sh
cargo build
```

And one can run all the <strong>tests</strong> by the command below:
```sh
cargo test -- --test-threads 1
```

Option `-- --test-threads 1` is used in order to prevent heisenbug (it shoots in the tests on optimizer).

# Features

* **Parsing**. CTL can parse a source code written in a subset of Rust. One can get familiar with the
language in `tests/parser_test.rs`
* **Generating IR**. It can generate IR from the AST.
