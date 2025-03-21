#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl From<Point> for String {
    fn from(point: Point) -> Self {
        format!("Point({}, {})", point.x, point.y)
    }
}