pub enum Response<T> {
    Value(T),
    Callback(Box<dyn FnOnce() -> T>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Dynamic,
    Absolute { x: f32, y: f32 },
    Relative { x: f32, y: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeOp {
    Fit,
    Fill,
    Absolute { width: f32, height: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: SizeOp,
    pub height: SizeOp,
}