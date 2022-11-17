<div align="center">
  <h1><code>CTL</code></h1>
  
  <p>
    <strong>Compiling toolchain library</strong>
  </p>
</div>

# What it is

<strong>CTL</strong> is a program which parses source code (written in Rust's subset)
and generates an intermediate code from it.

## Introduction to the compilers (from Wikipedia)

A **compiler** is a computer program that translates computer code
written in one programming language (the source language) into another language
(the target language). The name "compiler" is primarily used for programs that
translate source code from a high-level programming language to a lower level
language (e.g. assembly language, object code, or machine code) to create
an executable program.

A compiler is likely to perform some or all of the following operations, often called
**phases**: preprocessing, lexical analysis, parsing, semantic analysis (syntax-directed
translation), conversion of input programs to an **intermediate representation**,
code optimization and code generation.

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

You can explore what source code CTL can handle in the [docs](./docs).

* **Parsing**. CTL can parse a source code written in a subset of Rust. One can get familiar with the
language in `tests/parser_test.rs`
* **Generating IR**. It can generate IR (as a sequence of instructions) from the AST.
