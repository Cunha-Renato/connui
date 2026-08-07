use crate::{
    event::{Event, InputState},
    renderer::Renderer,
    state::StateContext,
    widget::Element,
};
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
impl Eq for LPixel<f32> {}
impl Ord for LPixel<f32> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
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
impl Eq for PPixel {}
impl PartialOrd for PPixel {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PPixel {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
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
pub struct Point<T> {
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
pub struct Rect<P, S> {
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
        min: LPixel<u16>,
        max: LPixel<u16>,
        shrink: bool,
    },
    Fill {
        min: LPixel<u16>,
        max: LPixel<u16>,
        portion: LPixel<u16>,
        shrink: bool,
    },
    Absolute(LPixel<u16>),
}
impl SizeOp {
    #[inline]
    pub const fn fit(shrink: bool) -> Self {
        Self::Fit {
            min: LPixel(0),
            max: LPixel(u16::MAX),
            shrink,
        }
    }

    #[inline]
    pub const fn fill(portion: LPixel<u16>, shrink: bool) -> Self {
        Self::Fill {
            min: LPixel(0),
            max: LPixel(u16::MAX),
            portion,
            shrink,
        }
    }

    #[inline]
    pub fn absolute<L: Into<LPixel<u16>>>(value: L) -> Self {
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
impl<T: Into<LPixel<u16>>> From<T> for SizeOp {
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
        position: Point<LPixel<i32>>,
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

// Logical & Physical types.
pub type LSize<T> = Size<LPixel<T>>;
pub type PSize = Size<PPixel>;

pub type LPoint<T> = Point<LPixel<T>>;
pub type PPoint = Point<PPixel>;

pub type LSides<T> = Sides<LPixel<T>>;
pub type PSides = Sides<PPixel>;

pub type LRect<P, S> = Rect<LPixel<P>, LPixel<S>>;
pub type PRect = Rect<PPixel, PPixel>;

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
    pub min: LSize<u16>,
    pub max: LSize<u16>,
}
impl Bounds {
    pub fn width(mut self, width: SizeOp, padding: LPixel<u16>, margin: LPixel<u16>) -> Self {
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

    pub fn height(mut self, height: SizeOp, padding: LPixel<u16>, margin: LPixel<u16>) -> Self {
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
                width: LPixel::default(),
                height: LPixel::default(),
            },
            max: Size {
                width: LPixel::<u16>::MAX,
                height: LPixel::<u16>::MAX,
            },
        }
    }
}

pub struct Node<T, R: Renderer> {
    pub(crate) children: Vec<Node<T, R>>,
    pub(crate) widget: Element<T, R>,

    pub(crate) clip: LRect<i32, u16>,
    pub(crate) bounds: Bounds,
}
impl<T, R: Renderer> Node<T, R> {
    pub fn render(&mut self, renderer: &mut R) {
        todo!()
    }

    #[inline]
    pub(crate) fn layout(&mut self) {}

    pub(crate) fn update(&mut self, ctx: &mut StateContext<R>) -> bool {
        todo!()
    }

    pub(crate) fn clip(&mut self) {
        todo!()
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
        }
    }

    pub(crate) fn is_point_inside(&self, point: Point<LPixel<i32>>) -> bool {
        todo!()
    }

    fn calc_clip(&mut self, parent_clip: &Rect<LPixel<f32>, LPixel<f32>>) {
        todo!()
    }
}
impl<T, R: Renderer> std::fmt::Debug for Node<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("Node");

        let mut dbg = binding
            .field("bounds", &self.bounds)
            .field("clip", &self.clip);

        if !self.children.is_empty() {
            dbg = dbg.field("children", &self.children);
        }

        dbg.finish()
    }
}
