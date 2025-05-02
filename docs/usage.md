# rust-autoargs Usage Guide

This document explains how to use the `autoargs` macro for creating functions with named arguments and default values.

## Basic Usage

The `autoargs` macro transforms your function to use a generated argument struct with defaults:

```rust
use rust_autoargs::autoargs;

#[autoargs]
fn example(
    #[default = "String::from(\"default\")"]
    name: String,
    #[default = "42"]
    count: usize,
) -> String {
    format!("Name: {}, Count: {}", name, count)
}
```

You can then call this function in multiple ways:

```rust
// Use all defaults
let result1 = example!();

// Specify some arguments, others use defaults
let result2 = example!(
    name = "custom".to_string(),
    // count uses default of 42
);

// Specify different arguments
let result3 = example!(
    count = 100,
    // name uses default of "default"
);

// Specify all arguments
let result4 = example!(
    name = "everything custom".to_string(),
    count = 200,
);
```

## How It Works

For a function definition like:

```rust
#[autoargs]
fn draw(
    #[default = "foo()"]
    a: A,
    #[default = "bar()"]
    b: B,
    #[default = "baz()"]
    c: C
) -> X { /* function body */ }
```

The macro generates:

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
            c: baz()
        }
    }
}

fn draw(args: DrawArgs) -> X {
    let (a, b, c) = (args.a, args.b, args.c);
    /* original function body */
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
    ($args:expr) => {
        draw($args)
    };
}
```

## Advanced Usage

### Creating Custom Arg Structs

You can create a custom args struct and pass it directly:

```rust
let custom_args = DrawArgs {
    a: custom_a,
    b: DrawArgs::default().b,  // Use default for b
    c: custom_c,
};

// Pass the args struct directly
let result = draw!(custom_args);
```

### Using Default Trait

If a parameter doesn't have a `#[default = "..."]` attribute, it will use the type's `Default` implementation:

```rust
#[autoargs]
fn simple(
    // Uses String::default()
    name: String,
    // Uses Option::<i32>::default()
    value: Option<i32>,
) { /* ... */ }
```

## Best Practices

1. Always use specific types for your parameters that implement the required traits
2. Provide meaningful default values for each parameter
3. Break complex functions into smaller functions with clear argument sets
4. Use descriptive parameter names