use crate::{
    event::{Event, InputState},
    renderer::Renderer,
    state::StateContext,
    widget::Element,
};
use bitflags::bitflags;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub struct LogicalPixel<T = u16>(T);
impl<T> LogicalPixel<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}
impl<T: Copy> LogicalPixel<T> {
    #[inline]
    pub const fn inner(self) -> T {
        self.0
    }
}
impl LogicalPixel<i32> {
    pub const MAX: Self = Self(i32::MAX);

    #[inline]
    pub fn as_unsigned(self) -> LogicalPixel {
        LogicalPixel(self.0.clamp(0, u16::MAX as i32) as u16)
    }

    #[inline]
    pub const fn as_float(self) -> LogicalPixel<f32> {
        LogicalPixel(self.0 as f32)
    }

    #[inline]
    pub const fn as_physical(self) -> PhysicalPixel {
        PhysicalPixel(self.0 as f32)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PhysicalPixel {
        PhysicalPixel(self.0 as f32 * scale_factor)
    }
}
impl LogicalPixel<u16> {
    pub const MAX: Self = Self(u16::MAX);

    #[inline]
    pub const fn as_signed(self) -> LogicalPixel<i32> {
        LogicalPixel(self.0 as i32)
    }

    #[inline]
    pub const fn as_float(self) -> LogicalPixel<f32> {
        LogicalPixel(self.0 as f32)
    }

    #[inline]
    pub const fn as_physical(self) -> PhysicalPixel {
        PhysicalPixel(self.0 as f32)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PhysicalPixel {
        PhysicalPixel(self.0 as f32 * scale_factor)
    }
}
impl LogicalPixel<f32> {
    pub const MAX: Self = Self(f32::MAX);

    #[inline]
    pub const fn as_signed(self) -> LogicalPixel<i32> {
        LogicalPixel(self.0.round() as i32)
    }

    #[inline]
    pub const fn as_unsigned(self) -> LogicalPixel<u16> {
        LogicalPixel(self.0.round().clamp(0.0, u16::MAX as f32) as u16)
    }

    #[inline]
    pub const fn as_physical(self) -> PhysicalPixel {
        PhysicalPixel(self.0)
    }

    #[inline]
    pub const fn to_physical(self, scale_factor: f32) -> PhysicalPixel {
        PhysicalPixel(self.0 * scale_factor)
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
impl Eq for LogicalPixel<i32> {}
impl Ord for LogicalPixel<i32> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl Eq for LogicalPixel<u16> {}
impl Ord for LogicalPixel<u16> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl Eq for LogicalPixel<f32> {}
impl Ord for LogicalPixel<f32> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl<T: std::ops::Add<Output = T>> std::ops::Add for LogicalPixel<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<T: std::ops::AddAssign> std::ops::AddAssign for LogicalPixel<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl<T: std::ops::Sub<Output = T>> std::ops::Sub for LogicalPixel<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl<T: std::ops::SubAssign> std::ops::SubAssign for LogicalPixel<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl<T: std::ops::Mul<Output = T>> std::ops::Mul for LogicalPixel<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl<T: std::ops::MulAssign> std::ops::MulAssign for LogicalPixel<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl<T: std::ops::Div<Output = T>> std::ops::Div for LogicalPixel<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl<T: std::ops::DivAssign> std::ops::DivAssign for LogicalPixel<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}
impl From<u16> for LogicalPixel<u16> {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<i32> for LogicalPixel<i32> {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<f32> for LogicalPixel<f32> {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct PhysicalPixel(f32);
impl PhysicalPixel {
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
impl Eq for PhysicalPixel {}
impl PartialOrd for PhysicalPixel {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PhysicalPixel {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl std::ops::Add for PhysicalPixel {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl std::ops::AddAssign for PhysicalPixel {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl std::ops::Sub for PhysicalPixel {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl std::ops::SubAssign for PhysicalPixel {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
impl std::ops::Mul for PhysicalPixel {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}
impl std::ops::MulAssign for PhysicalPixel {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}
impl std::ops::Div for PhysicalPixel {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl std::ops::DivAssign for PhysicalPixel {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}
impl From<f32> for PhysicalPixel {
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color([u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Self::from_hex(0);
    pub const WHITE: Self = Self::from_hex(0xffffffff);
    pub const BLACK: Self = Self::from_hex(0x000000ff);
    pub const RED: Self = Self::from_hex(0xff0000ff);
    pub const GREEN: Self = Self::from_hex(0x00ff00ff);
    pub const BLUE: Self = Self::from_hex(0x0000ffff);

    pub const YELLOW: Self = Self::from_hex(Self::RED.into_hex() | Self::GREEN.into_hex());
    pub const MAGENTA: Self = Self::from_hex(Self::RED.into_hex() | Self::BLUE.into_hex());
    pub const CYAN: Self = Self::from_hex(Self::GREEN.into_hex() | Self::BLUE.into_hex());

    #[inline]
    pub const fn from_hex(hex: u32) -> Self {
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
        Self::from_hex(value)
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
pub struct Point<T = PhysicalPixel> {
    pub x: T,
    pub y: T,
}
impl<T> Point<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
impl<T: Copy> Point<T> {
    #[inline]
    pub fn map<F: Fn(T) -> U, U>(&self, f: F) -> Point<U> {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }
}
impl<T: Copy> Packable<T> for Point<T> {
    #[inline]
    fn hor_ver(&self) -> (T, T) {
        (self.x, self.y)
    }

    #[inline]
    fn hor_ver_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.x, &mut self.y)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Size<T = PhysicalPixel> {
    pub width: T,
    pub height: T,
}
impl Size<SizeOp> {
    pub(crate) fn as_logical(&self) -> Size<LogicalPixel> {
        Size {
            width: match self.width {
                SizeOp::Absolute(width) => width,
                SizeOp::Fill {
                    min, max, portion, ..
                } => portion.clamp(min, max),
                SizeOp::Fit { min, max, .. } => min.min(max),
            },
            height: match self.height {
                SizeOp::Absolute(height) => height,
                SizeOp::Fill {
                    min, max, portion, ..
                } => portion.clamp(min, max),
                SizeOp::Fit { min, max, .. } => min.min(max),
            },
        }
    }
}
impl<T> Size<T> {
    #[inline]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
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
    fn hor_ver(&self) -> (T, T) {
        (self.width, self.height)
    }

    #[inline]
    fn hor_ver_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.width, &mut self.height)
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Rect<P = PhysicalPixel, S = PhysicalPixel> {
    pub position: Point<P>,
    pub size: Size<S>,
}
impl<T> Rect<T, T>
where
    T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + Ord + Copy,
{
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    /// `None` means that **self** is outside **other** and vice versa.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let left = self.x().max(other.x());
        let top = self.y().max(other.y());
        let right = (self.x() + self.width()).min(other.x() + other.width());
        let bottom = (self.y() + self.height()).min(other.y() + other.height());

        if left < right && top < bottom {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }
}
impl<P, S> Rect<P, S> {
    #[inline]
    pub const fn new(x: P, y: P, width: S, height: S) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    #[inline]
    pub const fn new_pos_size(position: Point<P>, size: Size<S>) -> Self {
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
pub enum SizeOp {
    Fit {
        min: LogicalPixel,
        max: LogicalPixel,
        shrink: bool,
    },
    Fill {
        min: LogicalPixel,
        max: LogicalPixel,
        portion: LogicalPixel,
        shrink: bool,
    },
    Absolute(LogicalPixel),
}
impl SizeOp {
    #[inline]
    pub const fn fit(shrink: bool) -> Self {
        Self::Fit {
            min: LogicalPixel(0),
            max: LogicalPixel(u16::MAX),
            shrink,
        }
    }

    #[inline]
    pub const fn fill(portion: LogicalPixel, shrink: bool) -> Self {
        Self::Fill {
            min: LogicalPixel(0),
            max: LogicalPixel(u16::MAX),
            portion,
            shrink,
        }
    }

    #[inline]
    pub fn absolute<L: Into<LogicalPixel>>(value: L) -> Self {
        Self::Absolute(value.into())
    }

    #[inline]
    pub(crate) const fn is_dynamic(&self) -> bool {
        matches!(self, SizeOp::Fit { .. } | SizeOp::Fill { .. })
    }

    #[inline]
    pub(crate) const fn shrinkable(&self) -> bool {
        matches!(
            self,
            SizeOp::Fit { shrink: true, .. } | SizeOp::Fill { shrink: true, .. }
        )
    }
}
impl Default for SizeOp {
    #[inline]
    fn default() -> Self {
        Self::fit(true)
    }
}
impl<T: Into<LogicalPixel<u16>>> From<T> for SizeOp {
    #[inline]
    fn from(value: T) -> Self {
        Self::Absolute(value.into())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Position {
    #[default]
    Dynamic,
    Pinned {
        position: Point<LogicalPixel<i32>>,
        parent_relative: bool,
    },
}
impl Position {
    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Position::Dynamic)
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Sides<T = PhysicalPixel> {
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
    pub fn get_horizontal(&self) -> T {
        self.left + self.right
    }

    #[inline]
    pub fn get_vertical(&self) -> T {
        self.top + self.bottom
    }
}
impl<T: std::ops::Add<Output = T> + Copy> Packable<T> for Sides<T> {
    #[inline]
    fn hor_ver(&self) -> (T, T) {
        (self.get_horizontal(), self.get_vertical())
    }

    fn hor_ver_mut(&mut self) -> (&mut T, &mut T) {
        panic!("This should not be called");
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Layout {
    pub axis: LayoutAxis,
    pub flags: LayoutFlags,
}
impl Layout {
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

    #[inline]
    pub const fn flags(mut self, flags: LayoutFlags) -> Self {
        self.flags = flags;
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
    #[inline]
    pub(crate) fn along<T>(self, value: &dyn Packable<T>) -> T {
        self.pack(value).0
    }

    #[inline]
    pub(crate) fn along_mut<T>(self, value: &mut dyn Packable<T>) -> &mut T {
        self.pack_mut(value).0
    }

    #[inline]
    pub(crate) fn across<T>(self, value: &dyn Packable<T>) -> T {
        self.pack(value).1
    }

    #[inline]
    pub(crate) fn across_mut<T>(self, value: &mut dyn Packable<T>) -> &mut T {
        self.pack_mut(value).1
    }

    #[inline]
    pub(crate) fn pack<T>(self, value: &dyn Packable<T>) -> (T, T) {
        let (hor, ver) = value.hor_ver();

        match self {
            LayoutAxis::Horizontal => (hor, ver),
            LayoutAxis::Vertical => (ver, hor),
        }
    }

    #[inline]
    pub(crate) fn pack_mut<T>(self, value: &mut dyn Packable<T>) -> (&mut T, &mut T) {
        let (hor, ver) = value.hor_ver_mut();

        match self {
            LayoutAxis::Horizontal => (hor, ver),
            LayoutAxis::Vertical => (ver, hor),
        }
    }
}

bitflags! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct LayoutFlags: u8 {
        /// Children will wrap if possible.
        const WRAP = 0b1;
        /// This will be on top of every other [`Widget`].
        const OVERLAY = 0b10;
        /// Parent will ignore this widget for positioning & size calculations.
        const PARENT_IGNORE = 0b100;
    }
}
impl Default for LayoutFlags {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

pub struct Response<T> {
    pub response: Option<T>,
    pub consume: bool,
}
impl<T> Default for Response<T> {
    #[inline]
    fn default() -> Self {
        Self {
            response: Default::default(),
            consume: Default::default(),
        }
    }
}

// INTERNAL
pub(crate) trait Packable<T> {
    fn hor_ver(&self) -> (T, T);
    fn hor_ver_mut(&mut self) -> (&mut T, &mut T);
}
impl<T: Copy> Packable<T> for (T, T) {
    #[inline]
    fn hor_ver(&self) -> (T, T) {
        *self
    }

    #[inline]
    fn hor_ver_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.0, &mut self.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Bounds {
    pub min: Size<LogicalPixel<u16>>,
    pub max: Size<LogicalPixel<u16>>,
}
impl Bounds {
    pub fn width(mut self, width: SizeOp, padding: LogicalPixel, margin: LogicalPixel) -> Self {
        match width {
            SizeOp::Fit { min, max, .. } | SizeOp::Fill { min, max, .. } => {
                self.min.width = min;
                self.max.width =
                    (self.max.width.as_signed() - padding.as_signed() - margin.as_signed())
                        .min(max.as_signed())
                        .max(min.as_signed())
                        .as_unsigned();
            }
            SizeOp::Absolute(width) => {
                let new_width = width;

                self.min.width = new_width;
                self.max.width = new_width;
            }
        }

        self
    }

    pub fn height(mut self, height: SizeOp, padding: LogicalPixel, margin: LogicalPixel) -> Self {
        match height {
            SizeOp::Fit { min, max, .. } | SizeOp::Fill { min, max, .. } => {
                self.min.height = min;
                self.max.height =
                    (self.max.height.as_signed() - padding.as_signed() - margin.as_signed())
                        .min(max.as_signed())
                        .max(min.as_signed())
                        .as_unsigned();
            }
            SizeOp::Absolute(height) => {
                let new_height = height;

                self.min.height = new_height;
                self.max.height = new_height;
            }
        }

        self
    }
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Size {
                width: LogicalPixel::default(),
                height: LogicalPixel::default(),
            },
            max: Size {
                width: LogicalPixel::<u16>::MAX,
                height: LogicalPixel::<u16>::MAX,
            },
        }
    }
}

pub struct Node<T, R: Renderer> {
    pub(crate) children: Vec<Node<T, R>>,
    pub(crate) widget: Element<T, R>,
    pub(crate) clip: Rect<LogicalPixel<f32>, LogicalPixel<f32>>,
    pub(crate) bounds: Bounds,
    pub(crate) position: Point<LogicalPixel<f32>>,
    pub(crate) size: Size<LogicalPixel<f32>>,
}
impl<T, R: Renderer> Node<T, R> {
    pub fn render(&mut self, renderer: &mut R) {
        let scale_factor = renderer.scale_factor();

        let padding = self
            .widget
            .get_padding()
            .map(|lp| lp.to_physical(scale_factor));

        let rect = Rect::new_pos_size(
            self.position.map(|lp| lp.to_physical(scale_factor)),
            self.size.map(|lp| lp.to_physical(scale_factor)),
        );

        let s_left = (rect.x() + padding.left).inner().floor().max(0.0) as u32;
        let s_top = (rect.y() + padding.top).inner().floor().max(0.0) as u32;
        let s_right = (rect.x() + rect.width() - padding.right)
            .inner()
            .ceil()
            .max(0.0) as u32;
        let s_bottom = (rect.y() + rect.height() - padding.bottom)
            .inner()
            .ceil()
            .max(0.0) as u32;

        let scissor = Rect::new(
            s_left,
            s_top,
            s_right.saturating_sub(s_left),
            s_bottom.saturating_sub(s_top),
        );

        self.widget
            .render(&rect, &scissor, renderer, &mut self.children);
    }

    #[inline]
    pub(crate) fn layout(&mut self) {
        crate::layout::layout(self);
        self.calc_clip(&crate::layout::compute_content_rect(self));
    }

    pub(crate) fn update(&mut self, ctx: &mut StateContext<R>) -> bool {
        let rect = Rect::new_pos_size(
            self.position.map(|lp| lp.as_signed()),
            self.size.map(|lp| lp.as_unsigned()),
        );

        let clip = Rect::new_pos_size(
            self.clip.position.map(|lp| lp.as_signed()),
            self.clip.size.map(|lp| lp.as_unsigned()),
        );

        let mut invalid_layout = false;

        invalid_layout |= self.widget.update(rect, clip, ctx);
        for child in &mut self.children {
            invalid_layout |= child.update(ctx);
        }

        invalid_layout
    }

    pub(crate) fn clip(&mut self) {
        let children = std::mem::take(&mut self.children);
        self.children.reserve(children.len());

        for child in children {
            let child_rect = Rect {
                position: Point {
                    x: child.position.x,
                    y: child.position.y,
                },
                size: Size {
                    width: child.size.width,
                    height: child.size.height,
                },
            };

            if child_rect.intersects(&self.clip) {
                self.children.push(child);
            }
        }
    }

    /// Returns [`true`] if the event is consumed.
    pub(crate) fn event(
        &mut self,
        prev_state: &InputState,
        curr_state: &InputState,
        responses: &mut Vec<T>,
    ) -> bool {
        let events = Event::generate(prev_state, curr_state, self);

        if events.is_empty() {
            return false;
        }

        // Event was consumed by some of the children.
        if self
            .children
            .iter_mut()
            .any(|child| child.event(prev_state, curr_state, responses))
        {
            return true;
        }

        // If none was consumed we are free to receive the event.
        events.into_iter().any(|e| {
            let response = self.widget.on_event(e);

            if let Some(response) = response.response {
                responses.push(response);
            }

            response.consume
        })
    }

    pub(crate) fn from_element(
        mut element: Element<T, R>,
        ctx: &mut Option<&mut StateContext<R>>,
    ) -> Self {
        if let Some(ctx) = ctx {
            element.init(ctx);
        }

        let children: Vec<Self> = element
            .get_children()
            .into_iter()
            .map(|c| Self::from_element(c, ctx))
            .collect();

        Self {
            children,
            widget: element,
            clip: Rect::default(),
            bounds: Default::default(),
            position: Default::default(),
            size: Default::default(),
        }
    }

    pub(crate) fn is_point_inside(&self, point: Point<LogicalPixel<i32>>) -> bool {
        let point = point.map(|lp| lp.as_float());
        let lower_bound = Point {
            x: self.position.x + self.size.width,
            y: self.position.y + self.size.height,
        };

        self.position.x <= point.x
            && self.position.y <= point.y
            && lower_bound.x >= point.x
            && lower_bound.y >= point.y
    }

    fn calc_clip(&mut self, parent_clip: &Rect<LogicalPixel<f32>, LogicalPixel<f32>>) {
        let content_rect = crate::layout::compute_content_rect(self);

        let clip_rect = parent_clip
            .intersection(&content_rect)
            .unwrap_or(content_rect);

        self.clip = clip_rect;
        for child in &mut self.children {
            child.calc_clip(&self.clip);
        }
    }
}
impl<T, R: Renderer> std::fmt::Debug for Node<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("Node");

        let mut dbg = binding
            .field("position", &self.position)
            .field("size", &self.size)
            .field("bounds", &self.bounds)
            .field("clip", &self.clip);

        if !self.children.is_empty() {
            dbg = dbg.field("children", &self.children);
        }

        dbg.finish()
    }
}
