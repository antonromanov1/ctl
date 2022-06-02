# CTL language of source (input data) code and first IR

CTL's parser gets text of source code. This source is (or almost is) a valid
Rust source code.

## Examples

### Function declaration

1)

```rust
fn main() {}
```

generated to no instructions.

2) Function can have **parameters**:

```rust
fn foo(p0: i64, p1: i64) {}
```

generated to:

```
0. v0 = Parameter
1. v1 = Parameter
```

This IR means v0 and v1 are **Parameter**'s. The instruction numbers are on the left.

3) and have a returning type:

```rust
fn foo() -> i64 {
    return 0;
}
```

generated to:

```
0. MoveImm v0, 0
1. Return v0
```

Instruction **MoveImm** v0, 0 writes immediate operand 0 to variable v0. **Return** v0 returns value from v0.

### Local variable declaration

```rust
fn main() {
    let mut a: i64 = 0;
}
```

generated to:

```
0. MoveImm v1, 0
1. Move v0, v1
2. ReturnVoid
```

Instruction **Move** v0, v1 reads data from v1 and writes it to destination v0.

### Features

* Now only one **i64** type is supported.
* Local variable declarations allowed to be only in the top-level block,
not in inner scopes
* Every local variable should be **mutable** and **initialized**.

### Arithmetic operations

Assigns and arithmetic operations below are supported:
* Add
* Sub
* Mul
* Div
* Mod

Example:

```rust
fn main() {
    let mut num: i64 = 0;
    num = (1 + 2) * 3 - 4 / 5 % 6;
}
```

generated to:

```
0. MoveImm v1, 0
1. Move v0, v1
2. MoveImm v2, 1
3. MoveImm v3, 2
4. v4 = Add(v2, v3)
5. MoveImm v5, 3
6. v6 = Mul(v4, v5)
7. MoveImm v7, 4
8. MoveImm v8, 5
9. v9 = Div(v7, v8)
10. MoveImm v10, 6
11. v11 = Mod(v9, v10)
12. v12 = Sub(v6, v11)
13. Move v0, v12
14. ReturnVoid
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
0. MoveImm v0, 0
1. MoveImm v1, 0
2. IfFalse v0 == v1, goto 3
3. ReturnVoid
```

Instruction **IfFalse v0 == v1, goto 3** compares values read from v0 and v1 and if these are not equal,
then control switches to instruction 3.

2)

```rust
fn main() {
    if (0 == 0) {} else {}
}
```

generated to:

```
0. MoveImm v0, 0
1. MoveImm v1, 0
2. IfFalse v0 == v1, goto 4
3. Goto 4
4. ReturnVoid
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
0. Goto 2
1. Goto 0
2. ReturnVoid
```

2)

```rust
fn main(p: i64) {
    while (p == 0) {}
}
```

generated to:

```
0. v0 = Parameter
1. MoveImm v1, 0
2. IfFalse v0 == v1, goto 4
3. Goto 2
4. ReturnVoid
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

4)

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
0. MoveImm v1, 0
1. Move v0, v1

2. MoveImm v2, 9

// Loop
3. IfFalse v0 < v2, goto 11
4. MoveImm v3, 1
5. v4 = Add(v0, v3)
6. Move v0, v4

7. MoveImm v5, 23

// Conditional branch with `continue`
8. IfFalse v0 == v5, goto 10
9. Goto 3

10. Goto 3

11. ReturnVoid
```

### Calls

1)

```rust
fn main() {
    let mut num: i64 = 0;
    let mut other: i64 = 0;

    print(num, other, 1 + 1);
}
```

generated to:

```
0. MoveImm v1, 0
1. Move v0, v1
2. MoveImm v3, 0
3. Move v2, v3
4. MoveImm v4, 1
5. MoveImm v5, 1
6. v6 = Add(v4, v5)
7. v7 = Call print, args: v0, v2, v6
8. ReturnVoid
```

Instruction **Call** print , args: v0, v2, v6 is a call of function print with arguments v0, v2, v6.

2)

```rust
fn main() {
    let mut num: i64 = 0;
    num = calc() + 1;
}
```

generated to:

```
0. MoveImm v1, 0
1. Move v0, v1
2. v2 = Call calc, args:
3. MoveImm v3, 1
4. v4 = Add(v2, v3)
5. Move v0, v4
6. ReturnVoid
```
