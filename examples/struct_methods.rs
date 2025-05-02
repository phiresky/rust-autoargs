use rust_autoargs::{autoargs, impl_autoargs, default};

// Define a simple struct with methods
struct Canvas {
    name: String,
    width: u32,
    height: u32,
}

// First, generate the method macros
#[impl_autoargs]
impl Canvas {
    fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
        }
    }
    
    // Use autoargs on a method
    #[autoargs]
    fn draw_rectangle(
        &self,
        #[default = "0"]
        x: u32,
        #[default = "0"]
        y: u32,
        #[default = "100"]
        width: u32,
        #[default = "50"]
        height: u32,
        #[default = "\"blue\".to_string()"]
        color: String,
    ) -> String {
        format!(
            "Drawing a {} rectangle at ({}, {}) with size {}x{} on canvas \"{}\" ({}x{})",
            color, x, y, width, height, self.name, self.width, self.height
        )
    }
    
    // Use autoargs on a mutable method
    #[autoargs]
    fn resize(
        &mut self,
        #[default = "800"]
        width: u32,
        #[default = "600"]
        height: u32,
    ) -> String {
        self.width = width;
        self.height = height;
        format!("Resized canvas to {}x{}", width, height)
    }
}

// A different struct with a method that takes a value parameter
struct Shape {
    name: String,
}

#[impl_autoargs]
impl Shape {
    fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
    
    #[autoargs]
    fn scale(
        self, // Takes ownership of self
        #[default = "2.0"]
        factor: f64,
        #[default = "true"]
        uniform: bool,
    ) -> String {
        format!("Scaling {} by factor {} (uniform: {})", self.name, factor, uniform)
    }
}

fn main() {
    // Create a canvas
    let mut canvas = Canvas::new("My Canvas", 1024, 768);
    
    // Call a method with default parameters
    let result1 = draw_rectangle!(&canvas);
    println!("Default: {}", result1);
    
    // Call a method specifying some parameters
    let result2 = draw_rectangle!(&canvas, x = 10, y = 20, color = "red".to_string());
    println!("Custom position and color: {}", result2);
    
    // Call a method with custom args struct
    let args = DrawRectangleArgs {
        x: 30,
        y: 40,
        width: 200,
        height: 100,
        color: "green".to_string(),
    };
    let result3 = draw_rectangle!(&canvas, args);
    println!("Custom args struct: {}", result3);
    
    // Call a mutable method
    let result4 = resize!(&mut canvas);
    println!("Default resize: {}", result4);
    
    // Call a mutable method with custom values
    let result5 = resize!(&mut canvas, width = 1200, height = 900);
    println!("Custom resize: {}", result5);
    
    // Call a method that takes ownership of self
    let shape = Shape::new("Circle");
    let result6 = scale!(shape);
    println!("Default scale: {}", result6);
    
    // Call a method with a single parameter
    let shape2 = Shape::new("Square");
    let result7 = scale!(shape2, factor = 3.5);
    println!("Custom scale: {}", result7);
}