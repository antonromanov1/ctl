# CTL language of source (input data) code and IR

Now CTL consists of the unit tests and also can be compiled in an executable binary which gets **source code** and prints **IR** in a text format.
This source is (or almost is) a valid **Rust** source code.

![CTL diagram](/assets/general_diagram.png)

## Intermediate representation (IR)

On this step I have linear code which represents a sequence of instructions.

### Instructions:

* **Constant**, **Parameter**
* **Alloc**, **Store** and **Load**
* Binary instructions: **Add**, **Sub**, **Mul**, **Div**, **Mod**, **Shl**, **Shr** (add, subtract, multiply, divide, modulo, shift left, shift right)
* Negate **Neg**
* Control flow instructions: **IfFalse**, **Goto**, **Return**, **ReturnVoid**
* **Call**

Every instruction (except Store and control flow instructions) produces a **value**. Instruction **Alloc** allocates a local variable and produces a
pointer to it. **Store** gets an input operand and writes it to the variable pointed by next operand. **Load** reads a value from the local variable which
is pointed by the operand.

**Constant** can be found after its use in this linear code, it is not a big problem.

## Examples

### Function declaration

1)

```rust
fn main() {}
```

generated to:

```
 0 ReturnVoid
```

2) Function can have **constant parameters**:

```rust
fn foo(p0: i64, p1: i64) {}
```

generated to:

```
%0 = Parameter
%1 = Parameter
 2 ReturnVoid
```

This IR means %0 and %1 are **Parameter** instructions.

3) and have a returning type:

```rust
fn foo() -> i64 {
    return 0;
}
```

generated to:

```
%0 = Constant 0
 1 Return %0
```

Instruction **Constant** 0 produces value %0. Instruction **Return** with id 1 gets value %0 and returns it.

### Local variable declaration

```rust
fn main() {
    let mut a: i64 = 0;
}
```

generated to:

```
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
 3 ReturnVoid
```


### Features

* Now only one **i64** type is supported.
* Local variable declarations allowed to be only in the top-level block,
not in inner scopes
* Every local variable should be **mutable** and **initialized**.

### Arithmetic operations

Example:

```rust
fn main() {
    let mut num: i64 = 0;
    num = (1 + 2) * 3 - 4 / 5 % 6;
}
```

generated to:

```
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
%3 = Constant 1
%4 = Constant 2
%5 = Add %3, %4
%6 = Constant 3
%7 = Mul %5, %6
%8 = Constant 4
%9 = Constant 5
%10 = Div %8, %9
%11 = Constant 6
%12 = Mod %10, %11
%13 = Sub %7, %12
 14 Store %13 at %0
 15 ReturnVoid
```

### Conditional branches

1)

```rust
fn main() {
    if (0 == 0) {}
}
```

generated to:

```
%0 = Constant 0
 1 IfFalse %0 == %0, goto 2
 2 ReturnVoid
```

Instruction **IfFalse %0 == %0, goto 2** compares values read from %0 and %0 and if the condition turns to false,
then control switches to instruction 2.

2)

```rust
fn main() {
    if (0 == 0) {} else {}
}
```

generated to:

```
%0 = Constant 0
 1 IfFalse %0 == %0, goto 3
 2 Goto 3
 3 ReturnVoid
```

Instruction **Goto** is an unconditional branch.

### Cycles

1)

```rust
fn main() {
    while (true) {
        break;
    }
}
```

generated to:

```
 0 Goto 2
 1 Goto 0
 2 ReturnVoid
```

`0 Goto 2` corresponds to `break`
`1 Goto 0` is loops end

2)

```rust
fn main(p: i64) {
    while (p == 0) {}
}
```

generated to:

```
%0 = Parameter
%1 = Constant 0
 2 IfFalse %0 == %1, goto 4
 3 Goto 2
 4 ReturnVoid
```

3)

```rust
fn main() {
    while (true) {}
}
```

generated to:

```
0. Goto 0
1. ReturnVoid
```

4) Handling **continue** statement:

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
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
%3 = Load %0
%4 = Constant 9
 5 IfFalse %3 < %4, goto 15
%6 = Load %0
%7 = Constant 1
%8 = Add %6, %7
 9 Store %8 at %0
%10 = Load %0
%11 = Constant 23
 12 IfFalse %10 == %11, goto 14
 13 Goto 3
 14 Goto 3
 15 ReturnVoid
```

### Calls

1) Independent **Call**:

```rust
fn foo(p1: i64, p2: i64, p3: i64) {}

fn main() {
    let mut num: i64 = 0;
    let mut other: i64 = 0;

    foo(num, other, 1 + 1);
}
```

generated to:

```
Function foo, 4 instructions:
%0 = Parameter
%1 = Parameter
%2 = Parameter
 3 ReturnVoid

Function main, 11 instructions:
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
%3 = Alloc
 4 Store %1 at %3
%5 = Load %0
%6 = Load %3
%7 = Constant 1
%8 = Add %7, %7
%9 = Call foo, args: %5, %6, %8
 10 ReturnVoid
```

2) **Call** as a part of an expression:

```rust
fn calc() -> i64 {
    return 0;
}

fn main() {
    let mut num: i64 = 0;
    num = calc() + 1;
}
```

generated to:

```
Function calc, 2 instructions:
%0 = Constant 0
 1 Return %0

Function main, 8 instructions:
%0 = Alloc
%1 = Constant 0
 2 Store %1 at %0
%3 = Call calc, args: 
%4 = Constant 1
%5 = Add %3, %4
 6 Store %5 at %0
 7 ReturnVoid
```
