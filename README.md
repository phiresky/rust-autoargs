# rust-autoargs

A Rust procedural macro for generating argument structs with default values, allowing for named arguments and partial argument specification.

## Overview

When you annotate a function with `#[autoargs]`, the macro:

1. Generates a struct to hold the function's arguments
2. Implements `Default` for that struct, using specified default expressions
3. Creates a macro that lets you call the function with named arguments

## Example

```rust
use rust_autoargs::autoargs;

struct A(String);
struct B(u32);
struct C(bool);

fn foo() -> A { A("default_a".to_string()) }
fn bar() -> B { B(42) }
fn baz() -> C { C(true) }

#[autoargs]
fn draw(
    #[default = "foo()"]
    a: A,
    #[default = "bar()"]
    b: B, 
    #[default = "baz()"]
    c: C,
) -> String {
    format!("Drawing: a={}, b={}, c={}", a.0, b.0, c.0)
}

// Call with named arguments (unspecified arguments use defaults)
draw!(
    a = A("custom_a".to_string()),
    b = B(100),
    // c is set to its default
)
```

This generates:

```rust
struct DrawArgs {
    pub a: A,
    pub b: B,
    pub c: C,
}

impl Default for DrawArgs {
    fn default() -> Self {
        Self {
            a: foo(),
            b: bar(),
            c: baz(),
        }
    }
}

macro_rules! draw {
    () => {
        draw(DrawArgs::default())
    };
    ($(#[allow(non_snake_case)] $name:ident = $value:expr),* $(,)?) => {
        {
            let mut args = DrawArgs::default();
            $(
                args.$name = $value;
            )*
            draw(args)
        }
    };
}
```

## Features

- Named arguments with Rust-like syntax
- Default values for arguments via attributes
- Skip any argument to use its default value
- Generates proper structs and macros with correct visibility

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
rust-autoargs = "0.1.0"
```

## License

MIT