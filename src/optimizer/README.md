# Optimizer

This is both source language and target machine independent part.

# Examples of generated IR from source code

Although parsing is not a phase of the optimizer, I will show source code here
for a greater clarity.

## Empty function

```rust
fn main() {}
```

generated to:

```
Function main:

BB 0: preds: [] succs: []
 0 ReturnVoid
```

BB 0 is a basic block with ID 0, it has no predecessors and no successors and
it contains one instruction with ID 0 and opcode ReturnVoid.

## Conditional branch

```rust
fn main(p: i64) -> i64 {
    if (p > 0) {
        return p;
    } else {
        return p + 1;
    }
}
```

generated to:

```
Function main:

BB 0: preds: [] succs: [1, 2]
%0 = Parameter
%1 = Constant 0
 2 Branch %0 > %1

BB 1: preds: [0] succs: [3]
 3 Return %0
 4 Jump

BB 2: preds: [0] succs: [3]
%5 = Constant 1
%6 = Add %0, %5
 7 Return %6
 9 Jump

BB 3: preds: [1, 2] succs: []
 8 ReturnVoid
```

BB 0 has a Branch instruction as a terminator and 2 successors: BB 1 and BB 2.
If a basic block has 2 successors then **first is a true successor** and **second is a false successor**.

## Infinite loop

```rust
fn main() {
    while (true) {}
}
```

generated to:

```
Function main:

BB 0: preds: [0] succs: [0]
 0 Jump

BB 1: preds: [] succs: []
 1 ReturnVoid
```

## Break

```rust
fn main() {
    while (true) {
        break;
    }
}
```

generated to:

```
Function main:

BB 0: preds: [1] succs: [2]
 0 Jump

BB 1: preds: [] succs: [0]
 1 Jump

BB 2: preds: [0] succs: []
 2 ReturnVoid
```

## Continue

```rust
fn main() {
    let mut a: i64 = 0;
    while (a < 9) {
        a = a + 1;
        if (a == 23) {
            continue;
        }
    }
}
```

generated to:

```
Function main:

BB 0: preds: [] succs: [1]
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
 16 Jump

BB 1: preds: [0, 3, 4] succs: [2, 5]
%3 = Load %0
%4 = Constant 9
 5 Branch %3 < %4

BB 2: preds: [1] succs: [3, 4]
%6 = Load %0
%7 = Constant 1
%8 = Add %6, %7
 9 Store %8 at %0
%10 = Load %0
%11 = Constant 23
 12 Branch %10 == %11

BB 3: preds: [2] succs: [1]
 13 Jump

BB 4: preds: [2] succs: [1]
 14 Jump

BB 5: preds: [1] succs: []
 15 ReturnVoid
```
