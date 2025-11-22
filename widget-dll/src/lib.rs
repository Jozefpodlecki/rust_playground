

#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct Widget {
    pub id: u32,
    pub value: f64,
}

#[repr(C)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn get_version() -> Version {
    Version { major: 1, minor: 0, patch: 3 }
}

#[unsafe(no_mangle)]
pub extern "C" fn make_widget(id: u32, value: f64) -> Widget {
    Widget { id, value }
}

#[unsafe(no_mangle)]
pub extern "C" fn make_point(x: f32, y: f32) -> Point {
    Point { x, y }
}