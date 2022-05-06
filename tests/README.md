How to run a specified test
```bash
cargo test -- <test name> --nocapture
```
How to debug a specified test
```bash
rust-gdb --args ./target/debug/deps/<test file>-<code> -- <test name> --nocapture
```
Set breakpoint below in order to stop on a panic (including an assert) and have a backtrace
```
(gdb) b rust_panic
```
