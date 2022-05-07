<div align="center">
  <h1><code>CTL</code></h1>
  
  <p>
    <strong>Compiling toolchain library</strong>
  </p>
</div>

## Build and test

CTL can be <strong>built</strong> by the command below:
```sh
cargo build
```

And one can run all the <strong>tests</strong> by the command below:
```sh
cargo test
```

## Features

You can explore what source code CTL can handle in the [docs](./docs/source_code_first_ir.md).

* **Parsing**. CTL can parse a source code written in almost a subset of Rust. One can get familiar with the language in `tests/parser_test.rs`
* **Generating IR**. It can generate IR (as a sequence of instructions) from the AST. This IR is called *first IR*.
