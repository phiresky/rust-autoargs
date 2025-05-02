use rust_autoargs::autoargs;

// Define some example types
struct A(String);
struct B(u32);
struct C(bool);

// Helper functions to create default values
fn foo() -> A {
    A("default_a".to_string())
}

fn bar() -> B {
    B(42)
}

fn baz() -> C {
    C(true)
}

// Function with autoargs attribute
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

fn main() {
    // Call with no arguments (uses all defaults)
    let result1 = draw!();
    println!("All defaults: {}", result1);
    
    // Call with some arguments (others use defaults)
    let custom_a = A("custom_a".to_string());
    let custom_b = B(100);
    let result2 = draw!(
        a = custom_a,
        b = custom_b,
        // c is set to its default because it is not mentioned
    );
    println!("Custom a and b: {}", result2);
    
    // Call with different arguments
    let custom_c = C(false);
    let result3 = draw!(
        c = custom_c,
        // a and b are set to defaults
    );
    println!("Custom c: {}", result3);
}