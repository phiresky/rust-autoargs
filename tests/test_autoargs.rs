use rust_autoargs::{autoargs, default};

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn default_point() -> Point {
    Point { x: 10, y: 20 }
}

fn default_scale() -> f64 {
    2.5
}

fn default_color() -> String {
    "blue".to_string()
}

#[autoargs]
fn draw_rectangle(
    #[default = "default_point()"]
    position: Point,
    #[default = "default_scale()"]
    scale: f64,
    #[default = "default_color()"]
    color: String,
) -> String {
    format!("Drawing rectangle at {:?} with scale {} and color {}", position, scale, color)
}

#[test]
fn test_all_defaults() {
    let result = draw_rectangle!();
    assert!(result.contains("x: 10, y: 20"));
    assert!(result.contains("scale 2.5"));
    assert!(result.contains("color blue"));
}

#[test]
fn test_custom_position() {
    let custom_pos = Point { x: 5, y: 15 };
    let result = draw_rectangle!(
        position = custom_pos,
    );
    assert!(result.contains("x: 5, y: 15"));
    assert!(result.contains("scale 2.5"));
    assert!(result.contains("color blue"));
}

#[test]
fn test_custom_scale_and_color() {
    let result = draw_rectangle!(
        scale = 3.0,
        color = "red".to_string(),
    );
    assert!(result.contains("x: 10, y: 20"));
    assert!(result.contains("scale 3"));
    assert!(result.contains("color red"));
}

#[test]
fn test_all_custom() {
    let custom_pos = Point { x: 0, y: 0 };
    let result = draw_rectangle!(
        position = custom_pos,
        scale = 1.0,
        color = "green".to_string(),
    );
    assert!(result.contains("x: 0, y: 0"));
    assert!(result.contains("scale 1"));
    assert!(result.contains("color green"));
}