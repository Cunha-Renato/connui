pub enum Response<T> {
    Value(T),
    Callback(Box<dyn FnOnce() -> T>),
}

pub enum Position {
    Dynamic,
    Absolute { x: f32, y: f32 },
    Relative { x: f32, y: f32 },
}

pub enum Size {
    Fit,
    Fill,
    Absolute { width: f32, height: f32 },
}
