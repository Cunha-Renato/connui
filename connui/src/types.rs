use bitflags::bitflags;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub struct LPixel<T>(T);
impl<T> LPixel<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}
impl<T: Copy> LPixel<T> {
    #[inline]
    pub const fn inner(self) -> T {
        self.0
    }
}
impl LPixel<i32> {
    pub const MAX: Self = Self(i32::MAX);

    #[inline]
    pub fn as_unsigned(self) -> LPixel<u16> {
        LPixel(self.0.clamp(0, u16::MAX as i32) as u16)
    }

    #[inline]
    pub const fn as_float(self) -> LPixel<f32> {
        LPixel(self.0 as f32)
    }

    #[inline]
    pub const fn as_physical(self) -> PPixel {
        PPixel(self.0 as f32)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PPixel {
        PPixel(self.0 as f32 * scale_factor)
    }
}
impl LPixel<u16> {
    pub const MAX: Self = Self(u16::MAX);

    #[inline]
    pub const fn as_signed(self) -> LPixel<i32> {
        LPixel(self.0 as i32)
    }

    #[inline]
    pub const fn as_float(self) -> LPixel<f32> {
        LPixel(self.0 as f32)
    }

    #[inline]
    pub const fn as_physical(self) -> PPixel {
        PPixel(self.0 as f32)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PPixel {
        PPixel(self.0 as f32 * scale_factor)
    }
}
impl LPixel<f32> {
    pub const MAX: Self = Self(f32::MAX);

    #[inline]
    pub const fn as_signed(self) -> LPixel<i32> {
        LPixel(self.0.round() as i32)
    }

    #[inline]
    pub const fn as_unsigned(self) -> LPixel<u16> {
        LPixel(self.0.round().clamp(0.0, u16::MAX as f32) as u16)
    }

    #[inline]
    pub const fn as_physical(self) -> PPixel {
        PPixel(self.0)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PPixel {
        PPixel(self.0 * scale_factor)
    }

    #[inline]
    pub const fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    #[inline]
    pub const fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }
}
impl Eq for LPixel<i32> {}
impl Ord for LPixel<i32> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl Eq for LPixel<u16> {}
impl Ord for LPixel<u16> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: std::ops::Add<Output = T>> std::ops::Add for LPixel<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<T: std::ops::AddAssign> std::ops::AddAssign for LPixel<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl<T: std::ops::Sub<Output = T>> std::ops::Sub for LPixel<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl<T: std::ops::SubAssign> std::ops::SubAssign for LPixel<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl<T: std::ops::Mul<Output = T>> std::ops::Mul for LPixel<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl<T: std::ops::MulAssign> std::ops::MulAssign for LPixel<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl<T: std::ops::Div<Output = T>> std::ops::Div for LPixel<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl<T: std::ops::DivAssign> std::ops::DivAssign for LPixel<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}
impl From<u16> for LPixel<u16> {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<i32> for LPixel<i32> {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<f32> for LPixel<f32> {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PPixel(f32);
impl PPixel {
    pub const MAX: Self = Self(f32::MAX);

    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn inner(self) -> f32 {
        self.0
    }

    #[inline]
    pub const fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    #[inline]
    pub const fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }
}
impl std::ops::Add for PPixel {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl std::ops::AddAssign for PPixel {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl std::ops::Sub for PPixel {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl std::ops::SubAssign for PPixel {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl std::ops::Mul for PPixel {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl std::ops::MulAssign for PPixel {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl std::ops::Div for PPixel {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl std::ops::DivAssign for PPixel {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}
impl From<f32> for PPixel {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);
impl Id {
    // TODO: Change function name.
    pub fn new<T: std::hash::Hash>(value: T) -> Self {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color([u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Self::from_hex_rgba(0);
    pub const WHITE: Self = Self::from_hex_rgba(0xffffffff);
    pub const BLACK: Self = Self::from_hex_rgba(0x000000ff);
    pub const RED: Self = Self::from_hex_rgba(0xff0000ff);
    pub const GREEN: Self = Self::from_hex_rgba(0x00ff00ff);
    pub const BLUE: Self = Self::from_hex_rgba(0x0000ffff);

    pub const YELLOW: Self = Self::from_hex_rgba(Self::RED.into_hex() | Self::GREEN.into_hex());
    pub const MAGENTA: Self = Self::from_hex_rgba(Self::RED.into_hex() | Self::BLUE.into_hex());
    pub const CYAN: Self = Self::from_hex_rgba(Self::GREEN.into_hex() | Self::BLUE.into_hex());

    #[inline]
    pub const fn from_hex_rgb(hex: u32) -> Self {
        Self::from_hex_rgba((hex << 8) | 0x000000ff)
    }

    #[inline]
    pub const fn from_hex_rgba(hex: u32) -> Self {
        Self(hex.to_be_bytes())
    }

    #[inline]
    pub const fn into_hex(self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    #[inline]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    #[inline]
    pub const fn into_bytes(self) -> [u8; 4] {
        self.0
    }

    #[inline]
    pub const fn from_f32(value: [f32; 4]) -> Self {
        Self::from_bytes([
            (value[0] * 255.0).max(0.0) as u8,
            (value[1] * 255.0).max(0.0) as u8,
            (value[2] * 255.0).max(0.0) as u8,
            (value[3] * 255.0).max(0.0) as u8,
        ])
    }

    #[inline]
    pub const fn into_f32(self) -> [f32; 4] {
        let bytes = self.into_bytes();

        [
            bytes[0] as f32 / 255.0,
            bytes[1] as f32 / 255.0,
            bytes[2] as f32 / 255.0,
            bytes[3] as f32 / 255.0,
        ]
    }
}
impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::WHITE
    }
}
impl From<[u8; 4]> for Color {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        Self::from_bytes(value)
    }
}
impl From<Color> for [u8; 4] {
    #[inline]
    fn from(value: Color) -> Self {
        value.into_bytes()
    }
}
impl From<u32> for Color {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_hex_rgba(value)
    }
}
impl From<Color> for u32 {
    #[inline]
    fn from(value: Color) -> Self {
        value.into_hex()
    }
}
impl From<[f32; 4]> for Color {
    #[inline]
    fn from(value: [f32; 4]) -> Self {
        Self::from_f32(value)
    }
}
impl From<Color> for [f32; 4] {
    #[inline]
    fn from(value: Color) -> Self {
        value.into_f32()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position<T> {
    pub x: T,
    pub y: T,
}
impl<T> Position<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
impl<T> Position<T> {
    #[inline]
    pub fn map<F: Fn(T) -> U, U>(self, f: F) -> Position<U> {
        Position {
            x: f(self.x),
            y: f(self.y),
        }
    }
}
impl<T: Copy> Packable<T> for Position<T> {
    #[inline]
    fn horizontal_vertical(&self) -> (T, T) {
        (self.x, self.y)
    }

    #[inline]
    fn horizontal_vertical_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.x, &mut self.y)
    }
}
impl<T> From<(T, T)> for Position<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
impl<T: Copy> From<[T; 2]> for Position<T> {
    fn from(value: [T; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}
impl<T> Size<T> {
    #[inline]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}
impl Size<Sizing> {
    pub fn validate(mut self) -> Self {
        self.width.validate();
        self.height.validate();
        self
    }
}
impl<T: Copy> Size<T> {
    #[inline]
    pub fn map<F: Fn(T) -> U, U>(&self, f: F) -> Size<U> {
        Size {
            width: f(self.width),
            height: f(self.height),
        }
    }
}
impl<T: Copy> Packable<T> for Size<T> {
    #[inline]
    fn horizontal_vertical(&self) -> (T, T) {
        (self.width, self.height)
    }

    #[inline]
    fn horizontal_vertical_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.width, &mut self.height)
    }
}
impl<T> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}
impl<T: Copy> From<[T; 2]> for Size<T> {
    fn from(value: [T; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RelativeValue<T> {
    pub main: T,
    pub cross: T,
}
impl<T> RelativeValue<T> {
    #[inline]
    pub const fn new(main: T, cross: T) -> Self {
        Self { main, cross }
    }
}
impl RelativeValue<Sizing> {
    pub fn validate(mut self) -> Self {
        self.main.validate();
        self.cross.validate();
        self
    }
}
impl<T> RelativeValue<T> {
    #[inline]
    pub fn map<F: Fn(T) -> U, U>(self, f: F) -> RelativeValue<U> {
        RelativeValue {
            main: f(self.main),
            cross: f(self.cross),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Rect<P, S> {
    pub position: Position<P>,
    pub size: Size<S>,
}
impl<T> Rect<T, T>
where
    T: Copy,
{
    pub fn map<F, U>(&self, f: F) -> Rect<U, U>
    where
        F: Fn(T) -> U,
    {
        Rect {
            position: self.position.map(&f),
            size: self.size.map(f),
        }
    }
}
impl<T> Rect<T, T>
where
    T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + PartialOrd + Copy,
{
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    /// Returns `None` when the rectangles do not overlap.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let left = if self.x() > other.x() {
            self.x()
        } else {
            other.x()
        };
        let top = if self.y() > other.y() {
            self.y()
        } else {
            other.y()
        };
        let self_right = self.x() + self.width();
        let other_right = other.x() + other.width();
        let right = if self_right < other_right {
            self_right
        } else {
            other_right
        };
        let self_bottom = self.y() + self.height();
        let other_bottom = other.y() + other.height();
        let bottom = if self_bottom < other_bottom {
            self_bottom
        } else {
            other_bottom
        };

        (left < right && top < bottom).then_some(Rect::new(left, top, right - left, bottom - top))
    }
}
impl<T> Rect<T, T>
where
    T: Copy + std::ops::Add<Output = T> + PartialOrd,
{
    pub fn contains(&self, x: T, y: T) -> bool {
        x >= self.x()
            && x < self.x() + self.width()
            && y >= self.y()
            && y < self.y() + self.height()
    }
}
impl<P, S> Rect<P, S> {
    #[inline]
    pub const fn new(x: P, y: P, width: S, height: S) -> Self {
        Self {
            position: Position::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[inline]
    pub const fn from_position_and_size(position: Position<P>, size: Size<S>) -> Self {
        Self { position, size }
    }
}
impl<P, S> Rect<P, S>
where
    P: Copy,
    S: Copy,
{
    #[inline]
    pub const fn x(&self) -> P {
        self.position.x
    }

    #[inline]
    pub const fn y(&self) -> P {
        self.position.y
    }

    #[inline]
    pub const fn width(&self) -> S {
        self.size.width
    }

    #[inline]
    pub const fn height(&self) -> S {
        self.size.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}
impl<T> MinMax<T> {
    pub const fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}
impl<T: Default> Default for MinMax<T> {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}
impl<T: Into<R>, R> From<(T, T)> for MinMax<R> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0.into(), value.1.into())
    }
}
impl<T: Clone + Into<R>, R> From<[T; 2]> for MinMax<R> {
    fn from(value: [T; 2]) -> Self {
        Self::new(value[0].clone().into(), value[1].clone().into())
    }
}
impl<T: Into<R>, R> From<std::ops::RangeInclusive<T>> for MinMax<R> {
    fn from(value: std::ops::RangeInclusive<T>) -> Self {
        let (min, max) = value.into_inner();
        Self::new(min.into(), max.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Sizing {
    Absolute(LPixel<u16>),
    Fit(MinMax<LPixel<u16>>),
    Fill {
        min_max: MinMax<LPixel<u16>>,
        initial: LPixel<u16>,
    },
}
impl Sizing {
    pub fn fit(min_max: impl Into<MinMax<LPixel<u16>>>) -> Self {
        Self::Fit(min_max.into())
    }

    #[inline]
    pub const fn fit_default() -> Self {
        Self::Fit(MinMax::new(LPixel(0), LPixel(u16::MAX)))
    }

    pub fn fill(min_max: impl Into<MinMax<LPixel<u16>>>, initial: impl Into<LPixel<u16>>) -> Self {
        Self::Fill {
            min_max: min_max.into(),
            initial: initial.into(),
        }
    }

    #[inline]
    pub const fn fill_default() -> Self {
        Self::Fill {
            min_max: MinMax::new(LPixel(0), LPixel(u16::MAX)),
            initial: LPixel(0),
        }
    }

    #[inline]
    pub fn absolute(value: impl Into<LPixel<u16>>) -> Self {
        Self::Absolute(value.into())
    }

    #[inline]
    /// Resolves min <= max. Where min has priority.
    ///
    /// Resolves min <= initial <= max.
    pub fn validate(&mut self) {
        match self {
            Sizing::Fit(MinMax { min, max }) => *max = *max.max(min),
            Sizing::Fill {
                min_max: MinMax { min, max },
                initial,
                ..
            } => {
                *max = *max.max(min);
                *initial = *initial.clamp(min, max);
            }
            _ => {}
        }
    }

    #[inline]
    pub const fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute(_))
    }
}
impl Default for Sizing {
    #[inline]
    fn default() -> Self {
        Self::fit_default()
    }
}
impl<T: Into<LPixel<u16>>> From<T> for Sizing {
    #[inline]
    fn from(value: T) -> Self {
        Self::Absolute(value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Positioning {
    Dynamic,
    Pinned {
        position: LPosition<i32>,
        flags: PinnedFlags,
    },
}
impl Positioning {
    #[inline]
    pub fn is_pinned(&self) -> bool {
        matches!(self, Positioning::Pinned { .. })
    }
}
impl Default for Positioning {
    #[inline]
    fn default() -> Self {
        Self::Dynamic
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct PinnedFlags: u8 {
        const OVERLAY = 0b1;
        /// If the position is relative to parent's origin.
        const RELATIVE_TO_PARENT = 0b10;
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Sides<T> {
    pub top: T,
    pub bottom: T,
    pub left: T,
    pub right: T,
}
impl<T> Sides<T> {
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            top: value,
            bottom: value,
            right: value,
            left: value,
        }
    }

    #[inline]
    pub fn top(mut self, value: T) -> Self {
        self.top = value;
        self
    }

    #[inline]
    pub fn bottom(mut self, value: T) -> Self {
        self.bottom = value;
        self
    }

    #[inline]
    pub fn left(mut self, value: T) -> Self {
        self.left = value;
        self
    }

    #[inline]
    pub fn right(mut self, value: T) -> Self {
        self.right = value;
        self
    }

    #[inline]
    pub fn x(self, value: T) -> Self
    where
        T: Copy,
    {
        self.left(value).right(value)
    }

    #[inline]
    pub fn y(self, value: T) -> Self
    where
        T: Copy,
    {
        self.top(value).bottom(value)
    }
}
impl<T: Copy> Sides<T> {
    #[inline]
    pub fn map<F: Fn(T) -> U, U>(&self, f: F) -> Sides<U> {
        Sides {
            top: f(self.top),
            bottom: f(self.bottom),
            left: f(self.left),
            right: f(self.right),
        }
    }
}
impl<T: std::ops::Add<Output = T> + Copy> Sides<T> {
    #[inline]
    pub fn horizontal(&self) -> T {
        self.left + self.right
    }

    #[inline]
    pub fn vertical(&self) -> T {
        self.top + self.bottom
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Layout {
    pub axis: LayoutAxis,
}
impl Layout {
    pub const fn new() -> Self {
        Self {
            axis: LayoutAxis::new(),
        }
    }

    #[inline]
    pub const fn horizontal(mut self) -> Self {
        self.axis = LayoutAxis::Horizontal;
        self
    }

    #[inline]
    pub const fn vertical(mut self) -> Self {
        self.axis = LayoutAxis::Vertical;
        self
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LayoutAxis {
    #[default]
    Horizontal,
    Vertical,
}
impl LayoutAxis {
    pub const fn new() -> Self {
        Self::Horizontal
    }

    #[inline]
    pub fn main<T>(self, value: &dyn Packable<T>) -> T {
        self.main_cross(value).main
    }

    #[inline]
    pub fn main_mut<T>(self, value: &mut dyn Packable<T>) -> &mut T {
        self.main_cross_mut(value).main
    }

    #[inline]
    pub fn cross<T>(self, value: &dyn Packable<T>) -> T {
        self.main_cross(value).cross
    }

    #[inline]
    pub fn cross_mut<T>(self, value: &mut dyn Packable<T>) -> &mut T {
        self.main_cross_mut(value).cross
    }

    #[inline]
    pub fn main_cross<T>(self, value: &dyn Packable<T>) -> RelativeValue<T> {
        let (horizontal, vertical) = value.horizontal_vertical();

        match self {
            LayoutAxis::Horizontal => RelativeValue::new(horizontal, vertical),
            LayoutAxis::Vertical => RelativeValue::new(vertical, horizontal),
        }
    }

    #[inline]
    pub fn main_cross_mut<T>(self, value: &mut dyn Packable<T>) -> RelativeValue<&mut T> {
        let (horizontal, vertical) = value.horizontal_vertical_mut();

        match self {
            LayoutAxis::Horizontal => RelativeValue::new(horizontal, vertical),
            LayoutAxis::Vertical => RelativeValue::new(vertical, horizontal),
        }
    }

    #[inline]
    /// Returns `(horizontal, vertical)`.
    pub const fn horizontal_vertical<T: Copy>(self, relative: &RelativeValue<T>) -> (T, T) {
        match self {
            LayoutAxis::Horizontal => (relative.main, relative.cross),
            LayoutAxis::Vertical => (relative.cross, relative.main),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Response<T> {
    None,
    ConsumedEmpty,
    Consumed(T),
}
impl<T> Response<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self::Consumed(value)
    }

    #[inline]
    pub const fn consumed(&self) -> bool {
        matches!(self, Self::ConsumedEmpty | Self::Consumed(_))
    }

    #[inline]
    pub(crate) fn take(self) -> Option<T> {
        if let Self::Consumed(val) = self {
            Some(val)
        } else {
            None
        }
    }
}
impl<T> Default for Response<T> {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}
impl<T> From<T> for Response<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

pub trait Packable<T> {
    fn horizontal_vertical(&self) -> (T, T);
    fn horizontal_vertical_mut(&mut self) -> (&mut T, &mut T);
}
impl<T: Copy> Packable<T> for (T, T) {
    #[inline]
    fn horizontal_vertical(&self) -> (T, T) {
        *self
    }

    #[inline]
    fn horizontal_vertical_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.0, &mut self.1)
    }
}

// Logical & Physical types.
pub type LSize<T> = Size<LPixel<T>>;
pub type PSize = Size<PPixel>;

pub type LPosition<T> = Position<LPixel<T>>;
pub type PPosition = Position<PPixel>;

pub type LSides<T> = Sides<LPixel<T>>;
pub type PSides = Sides<PPixel>;

pub type LRect<P, S> = Rect<LPixel<P>, LPixel<S>>;
pub type PRect = Rect<PPixel, PPixel>;

// INTERNAL
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min: LSize<f32>,
    pub max: LSize<f32>,
}
impl Bounds {
    #[inline]
    pub fn desired(&self, size: Size<Sizing>) -> LSize<f32> {
        let width = match size.width {
            Sizing::Absolute(_) | Sizing::Fit { .. } => self.min.width,
            Sizing::Fill { initial, .. } => {
                initial.as_float().clamp(self.min.width, self.max.width)
            }
        };

        let height = match size.height {
            Sizing::Absolute(_) | Sizing::Fit { .. } => self.min.height,
            Sizing::Fill { initial, .. } => {
                initial.as_float().clamp(self.min.height, self.max.height)
            }
        };

        Size::new(width, height)
    }

    #[inline]
    pub fn clamp(&self, size: LSize<f32>) -> LSize<f32> {
        Size::new(
            size.width.clamp(self.min.width, self.max.width),
            size.height.clamp(self.min.height, self.max.height),
        )
    }

    pub fn width(&self, width: Sizing) -> Self {
        let (min_w, max_w) = match width {
            Sizing::Absolute(val) => (val.as_float(), val.as_float()),
            Sizing::Fit(MinMax { min, max })
            | Sizing::Fill {
                min_max: MinMax { min, max },
                ..
            } => (
                min.as_float(),
                max.as_float().min(self.max.width).max(min.as_float()),
            ),
        };

        Self {
            min: Size::new(min_w, self.min.height),
            max: Size::new(max_w, self.max.height),
        }
    }

    pub fn height(&self, height: Sizing) -> Self {
        let (min_h, max_h) = match height {
            Sizing::Absolute(val) => (val.as_float(), val.as_float()),
            Sizing::Fit(MinMax { min, max })
            | Sizing::Fill {
                min_max: MinMax { min, max },
                ..
            } => (
                min.as_float(),
                max.as_float().min(self.max.height).max(min.as_float()),
            ),
        };

        Self {
            min: Size::new(self.min.width, min_h),
            max: Size::new(self.max.width, max_h),
        }
    }

    /// Returns [`Bounds`] that has padding merged into min.
    pub fn padding(&self, padding: &LSides<f32>) -> Self {
        let horizontal = padding.horizontal();
        let vertical = padding.vertical();

        let min = Size::new(
            self.min.width.max(horizontal),
            self.min.height.max(vertical),
        );
        let max_width = self.min.width.max(self.max.width);
        let max_height = self.min.height.max(self.max.height);

        Self {
            min,
            max: Size::new(max_width, max_height),
        }
    }

    /// Returns [`Bounds`] that has min = 0.0 & max reduced by padding.
    pub fn inner_bounds(&self, padding: &LSides<f32>) -> Self {
        let horizontal = padding.horizontal();
        let vertical = padding.vertical();

        let new_max = Size::new(self.max.width - horizontal, self.max.height - vertical)
            .map(|size| size.max(0.0.into()));

        Self {
            min: Default::default(),
            max: new_max,
        }
    }
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Size {
                width: LPixel::default(),
                height: LPixel::default(),
            },
            max: Size {
                width: f32::MAX.into(),
                height: f32::MAX.into(),
            },
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! has_sizing {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub fn width(self, width: impl Into<$crate::types::Sizing>) -> Self {
                    self.width_const(width.into())
                }

                #[inline]
                pub const fn width_const(mut self, width: $crate::types::Sizing) -> Self {
                    self.$($member)+.width = width;
                    self
                }

                #[inline]
                pub fn height(self, height: impl Into<$crate::types::Sizing>) -> Self {
                    self.height_const(height.into())
                }

                #[inline]
                pub const fn height_const(mut self, height: $crate::types::Sizing) -> Self {
                    self.$($member)+.height = height;
                    self
                }

                #[inline]
                pub fn size<S, Sg>(self, size: S) -> Self
                where
                    S: Into<$crate::types::Size<Sg>>,
                    Sg: Into<$crate::types::Sizing> + Copy,
                {
                    self.size_const(size.into().map(|s| s.into()))
                }

                #[inline]
                pub const fn size_const(mut self, size: $crate::types::Size<$crate::types::Sizing>) -> Self {
                    self.$($member)+ = size;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_padding {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub fn padding_left(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_left_const(value.into())
                }

                #[inline]
                pub fn padding_left_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.left = value;
                    self
                }

                #[inline]
                pub fn padding_right(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_right_const(value.into())
                }

                #[inline]
                pub fn padding_right_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.right = value;
                    self
                }

                #[inline]
                pub fn padding_top(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_top_const(value.into())
                }

                #[inline]
                pub fn padding_top_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.top = value;
                    self
                }

                #[inline]
                pub fn padding_bottom(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_bottom_const(value.into())
                }

                #[inline]
                pub fn padding_bottom_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.bottom = value;
                    self
                }

                #[inline]
                pub fn padding_x(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_x_const(value.into())
                }

                #[inline]
                pub fn padding_x_const(self, value: $crate::types::LPixel<u16>) -> Self {
                    self.padding_left_const(value).padding_right_const(value)
                }

                #[inline]
                pub fn padding_y(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.padding_y_const(value.into())
                }

                #[inline]
                pub fn padding_y_const(self, value: $crate::types::LPixel<u16>) -> Self {
                    self.padding_top_const(value).padding_bottom_const(value)
                }

                #[inline]
                pub fn padding<Px>(self, padding: Sides<Px>) -> Self
                where Px: Into<$crate::types::LPixel<u16>> + Copy
                {
                    self.padding_const(padding.map(|p| p.into()))
                }

                #[inline]
                pub const fn padding_const(mut self, padding: Sides<$crate::types::LPixel<u16>>) -> Self {
                    self.$($member)+ = padding;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_margin {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub fn margin_left(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_left_const(value.into())
                }

                #[inline]
                pub fn margin_left_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.left = value;
                    self
                }

                #[inline]
                pub fn margin_right(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_right_const(value.into())
                }

                #[inline]
                pub fn margin_right_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.right = value;
                    self
                }

                #[inline]
                pub fn margin_top(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_top_const(value.into())
                }

                #[inline]
                pub fn margin_top_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.top = value;
                    self
                }

                #[inline]
                pub fn margin_bottom(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_bottom_const(value.into())
                }

                #[inline]
                pub fn margin_bottom_const(mut self, value: $crate::types::LPixel<u16>) -> Self {
                    self.$($member)+.bottom = value;
                    self
                }

                #[inline]
                pub fn margin_x(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_x_const(value.into())
                }

                #[inline]
                pub fn margin_x_const(self, value: $crate::types::LPixel<u16>) -> Self {
                    self.margin_left_const(value).margin_right_const(value)
                }

                #[inline]
                pub fn margin_y(self, value: impl Into<$crate::types::LPixel<u16>>) -> Self {
                    self.margin_y_const(value.into())
                }

                #[inline]
                pub fn margin_y_const(self, value: $crate::types::LPixel<u16>) -> Self {
                    self.margin_top_const(value).margin_bottom_const(value)
                }

                #[inline]
                pub fn margin<Px>(self, margin: Sides<Px>) -> Self
                where Px: Into<$crate::types::LPixel<u16>> + Copy
                {
                    self.margin_const(margin.map(|p| p.into()))
                }

                #[inline]
                pub const fn margin_const(mut self, margin: Sides<$crate::types::LPixel<u16>>) -> Self {
                    self.$($member)+ = margin;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_color {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub fn color(mut self, color: impl Into<$crate::types::Color>) -> Self {
                    self.color_const(color.into())
                }

                #[inline]
                pub const fn color_const(mut self, color: $crate::types::Color) -> Self {
                    self.$($member)+ = color;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_positioning {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub const fn dynamic(mut self) -> Self {
                    self.$($member)+ = $crate::types::Positioning::Dynamic;
                    self
                }

                #[inline]
                pub fn pinned<P, Px>(self, position: P, flags: PinnedFlags) -> Self
                where P: Into<$crate::types::Position<Px>>,
                      Px: Into<$crate::types::LPixel<i32>> + Copy
                {
                    self.pinned_const(
                        position.into().map(|p| p.into()),
                        flags,
                    )
                }

                #[inline]
                pub const fn pinned_const(mut self, position: $crate::types::Position<$crate::types::LPixel<i32>>, flags: PinnedFlags) -> Self {
                    self.$($member)+ = $crate::types::Positioning::Pinned {
                        position,
                        flags
                    };
                    self
                }

                #[inline]
                pub const fn positioning(mut self, positioning: Positioning) -> Self {
                    self.$($member)+ = positioning;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_layout {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                #[inline]
                pub const fn horizontal(mut self) -> Self {
                    self.$($member)+ = $crate::types::Layout {
                        axis: $crate::types::LayoutAxis::Horizontal
                    };
                    self
                }

                #[inline]
                pub const fn vertical(mut self) -> Self {
                    self.$($member)+ = $crate::types::Layout {
                        axis: $crate::types::LayoutAxis::Vertical
                    };
                    self
                }

                #[inline]
                pub const fn layout(mut self, layout: $crate::types::Layout) -> Self {
                    self.$($member)+ = layout;
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_children {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? => {$($children_generics:tt)+} with {$($member:tt)+}) => {
            impl$(<$($generics)+>)? $typename$(<$($type_generics)+>)? {
                pub fn children(mut self, children: Vec<$crate::tree::Widget<$($children_generics)+>>) -> Self {
                    self.$($member)+ = children;
                    self
                }

                pub fn children_iter<I>(mut self, iter: I) -> Self
                    where I: IntoIterator<Item = $crate::tree::Widget<$($children_generics)+>>
                {
                    self.$($member)+.clear();
                    self.children_extend(iter)
                }

                pub fn children_extend<I>(mut self, iter: I) -> Self
                    where I: IntoIterator<Item = $crate::tree::Widget<$($children_generics)+>>
                {
                    self.$($member)+.extend(iter.into_iter());
                    self
                }
            }
        };
    }

    #[macro_export]
    macro_rules! has_style {
        ($({$($generics:tt)+})? $typename:ident $({$($type_generics:tt)+})? with {$($member:tt)+}) => {
            $crate::has_sizing!($({$($generics)+})? $typename $({$($type_generics)+})? with {$($member)+.size});
            $crate::has_padding!($({$($generics)+})? $typename $({$($type_generics)+})? with {$($member)+.padding});
            $crate::has_margin!($({$($generics)+})? $typename $({$($type_generics)+})? with {$($member)+.margin});
            $crate::has_positioning!($({$($generics)+})? $typename $({$($type_generics)+})? with {$($member)+.position});
            $crate::has_layout!($({$($generics)+})? $typename $({$($type_generics)+})? with {$($member)+.layout});
        };
    }
}
