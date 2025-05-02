use rust_autoargs::{autoargs, impl_autoargs, default};

// A test struct to demonstrate method usage
struct Calculator {
    base_value: i32,
}

impl Calculator {
    fn new(base_value: i32) -> Self {
        Self { base_value }
    }
}

// Generate the method macros for Calculator
#[impl_autoargs]
impl Calculator {
    #[autoargs]
    fn add(
        &self,
        #[default = "0"]
        a: i32,
        #[default = "0"]
        b: i32,
    ) -> i32 {
        self.base_value + a + b
    }
    
    #[autoargs]
    fn multiply(
        &self,
        #[default = "1"]
        factor: i32,
    ) -> i32 {
        self.base_value * factor
    }
    
    #[autoargs]
    fn update(
        &mut self,
        #[default = "10"]
        new_value: i32,
    ) -> i32 {
        self.base_value = new_value;
        self.base_value
    }
}

#[test]
fn test_method_with_defaults() {
    let calc = Calculator::new(5);
    assert_eq!(add!(&calc), 5); // 5 + 0 + 0 = 5
    assert_eq!(multiply!(&calc), 5); // 5 * 1 = 5
}

#[test]
fn test_method_with_custom_args() {
    let calc = Calculator::new(5);
    assert_eq!(add!(&calc, a = 10, b = 15), 30); // 5 + 10 + 15 = 30
    assert_eq!(multiply!(&calc, factor = 4), 20); // 5 * 4 = 20
}

#[test]
fn test_method_with_args_struct() {
    let calc = Calculator::new(5);
    let add_args = AddArgs { a: 7, b: 8 };
    assert_eq!(add!(&calc, add_args), 20); // 5 + 7 + 8 = 20
    
    let mult_args = MultiplyArgs { factor: 6 };
    assert_eq!(multiply!(&calc, mult_args), 30); // 5 * 6 = 30
}

#[test]
fn test_mutable_method() {
    let mut calc = Calculator::new(5);
    assert_eq!(update!(&mut calc), 10); // Updates to default of 10
    assert_eq!(update!(&mut calc, new_value = 20), 20); // Updates to 20
}

// A struct that tests methods with different self types
struct SelfTester {
    value: String,
}

impl SelfTester {
    fn new(value: &str) -> Self {
        Self { value: value.to_string() }
    }
}

// Generate method macros for SelfTester
#[impl_autoargs]
impl SelfTester {
    #[autoargs]
    fn ref_method(
        &self,
        #[default = "\"default\".to_string()"]
        suffix: String,
    ) -> String {
        format!("{}-{}", self.value, suffix)
    }
    
    #[autoargs]
    fn mut_ref_method(
        &mut self,
        #[default = "\"suffix\".to_string()"]
        append: String,
    ) -> String {
        self.value.push_str(&append);
        self.value.clone()
    }
    
    #[autoargs]
    fn consume_method(
        self,
        #[default = "false"]
        uppercase: bool,
    ) -> String {
        if uppercase {
            self.value.to_uppercase()
        } else {
            self.value
        }
    }
}

#[test]
fn test_different_self_types() {
    let mut tester = SelfTester::new("test");
    
    // Test &self method
    assert_eq!(ref_method!(&tester), "test-default");
    assert_eq!(ref_method!(&tester, suffix = "custom".to_string()), "test-custom");
    
    // Test &mut self method
    assert_eq!(mut_ref_method!(&mut tester), "testsuffix");
    assert_eq!(mut_ref_method!(&mut tester, append = "123".to_string()), "testsuffix123");
    
    // Test self (owned) method
    let tester2 = SelfTester::new("value");
    assert_eq!(consume_method!(tester2), "value"); // default with false
    
    let tester3 = SelfTester::new("value");
    assert_eq!(consume_method!(tester3, uppercase = true), "VALUE");
}