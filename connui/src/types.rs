use crate::{
    event::{Event, InputState},
    renderer::Renderer,
    state::StateContext,
    widget::Element,
};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);
impl Id {
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
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}
impl<T> Point<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
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
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}
impl Size<SizeOp> {
    pub(crate) fn as_f32(&self) -> Size<f32> {
        Size {
            width: match self.width {
                SizeOp::Absolute(width) => width as f32,
                SizeOp::Grow { min, max, shrink } => {
                    if shrink {
                        max as f32
                    } else {
                        min as f32
                    }
                }
                _ => 0.0,
            },
            height: match self.height {
                SizeOp::Absolute(height) => height as f32,
                SizeOp::Grow { min, max, shrink } => {
                    if shrink {
                        max as f32
                    } else {
                        min as f32
                    }
                }
                _ => 0.0,
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect<P = f32, S = f32> {
    pub position: Point<P>,
    pub size: Size<S>,
}
impl Rect<f32, f32> {
    pub const fn intersects(&self, other: &Self) -> bool {
        self.position.x < other.position.x + other.size.width
            && self.position.x + self.size.width > other.position.x
            && self.position.y < other.position.y + other.size.height
            && self.position.y + self.size.height > other.position.y
    }

    pub const fn intersection(&self, other: &Self) -> Option<Self> {
        let x1 = self.position.x.max(other.position.x);
        let y1 = self.position.y.max(other.position.y);
        let x2 = (self.position.x + self.size.width).min(other.position.x + other.size.width);
        let y2 = (self.position.y + self.size.height).min(other.position.y + other.size.height);

        if x2 <= x1 || y2 <= y1 {
            return None;
        }

        Some(Self {
            position: Point { x: x1, y: y1 },
            size: Size {
                width: x2 - x1,
                height: y2 - y1,
            },
        })
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
    Fit { min: u16, max: u16, shrink: bool },
    Fill { min: u16, max: u16, shrink: bool },
    Grow { min: u16, max: u16, shrink: bool },
    Absolute(u16),
}
impl SizeOp {
    #[inline]
    pub const fn fit(shrink: bool) -> Self {
        Self::Fit {
            min: 0,
            max: u16::MAX,
            shrink,
        }
    }

    #[inline]
    pub const fn fill(shrink: bool) -> Self {
        Self::Fill {
            min: 0,
            max: u16::MAX,
            shrink,
        }
    }

    #[inline]
    pub const fn grow(shrink: bool) -> Self {
        Self::Grow {
            min: 0,
            max: u16::MAX,
            shrink,
        }
    }

    #[inline]
    pub const fn absolute(value: u16) -> Self {
        Self::Absolute(value)
    }

    #[inline]
    pub(crate) const fn is_dynamic(&self) -> bool {
        matches!(
            self,
            SizeOp::Fit { .. } | SizeOp::Fill { .. } | SizeOp::Grow { .. }
        )
    }

    #[inline]
    pub(crate) const fn shrinkable(&self) -> bool {
        matches!(
            self,
            SizeOp::Fit { shrink: true, .. }
                | SizeOp::Fill { shrink: true, .. }
                | SizeOp::Grow { shrink: true, .. }
        )
    }
}
impl Default for SizeOp {
    #[inline]
    fn default() -> Self {
        Self::fit(true)
    }
}
impl From<u16> for SizeOp {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Absolute(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Position {
    #[default]
    Dynamic,
    Pinned {
        position: Point<i16>,
        parent_relative: bool,
    },
}
impl Position {
    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Position::Dynamic)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Sides<T = f32> {
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
        self.left(value);
        self.right(value);
        self
    }

    #[inline]
    pub fn y(self, value: T) -> Self
    where
        T: Copy,
    {
        self.top(value);
        self.bottom(value);
        self
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
    pub min: Size,
    pub max: Size,
}
impl Bounds {
    pub fn width(mut self, width: SizeOp, padding: u16, margin: u16) -> Self {
        match width {
            SizeOp::Fit { min, max, .. }
            | SizeOp::Fill { min, max, .. }
            | SizeOp::Grow { min, max, .. } => {
                self.min.width = min as f32;
                self.max.width = (self.max.width - padding as f32 - margin as f32)
                    .min(max as f32)
                    .max(min as f32);
            }
            SizeOp::Absolute(width) => {
                let new_width = width as f32;

                self.min.width = new_width;
                self.max.width = new_width;
            }
        }

        self
    }

    pub fn height(mut self, height: SizeOp, padding: u16, margin: u16) -> Self {
        match height {
            SizeOp::Fit { min, max, .. }
            | SizeOp::Fill { min, max, .. }
            | SizeOp::Grow { min, max, .. } => {
                self.min.height = min as f32;
                self.max.height = (self.max.height - padding as f32 - margin as f32)
                    .min(max as f32)
                    .max(min as f32);
            }
            SizeOp::Absolute(height) => {
                let new_height = height as f32;

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
                width: 0.0,
                height: 0.0,
            },
            max: Size {
                width: f32::INFINITY,
                height: f32::INFINITY,
            },
        }
    }
}

pub struct Node<T, R: Renderer> {
    pub(crate) children: Vec<Node<T, R>>,
    pub(crate) widget: Element<T, R>,
    pub(crate) bounds: Bounds,
    pub(crate) position: Point,
    pub(crate) size: Size,
}
impl<T, R: Renderer> Node<T, R> {
    pub fn render(&mut self, renderer: &mut R) {
        let padding = self.widget.get_padding();
        let rect = Rect::new_pos_size(self.position, self.size);
        let scissor = Rect::new(
            rect.x() + padding.left as f32 * renderer.scale_factor(),
            rect.y() + padding.top as f32 * renderer.scale_factor(),
            rect.width() - padding.get_horizontal() as f32 * renderer.scale_factor(),
            rect.height() - padding.get_vertical() as f32 * renderer.scale_factor(),
        );
        self.widget
            .render(rect, scissor, renderer, &mut self.children);
    }

    #[inline]
    pub(crate) fn layout(&mut self) {
        crate::layout::layout(self);
    }

    pub(crate) fn update(&mut self, ctx: &mut StateContext<R>) -> bool {
        let position = Point {
            x: self.position.x as i16,
            y: self.position.y as i16,
        };
        let size = Size {
            width: self.size.width as u16,
            height: self.size.height as u16,
        };
        let rect = Rect::new_pos_size(position, size);

        let mut invalid_layout = false;

        invalid_layout |= self.widget.update(rect, ctx);
        for child in &mut self.children {
            invalid_layout |= child.update(ctx);
        }

        invalid_layout
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
        bounds: &Bounds,
        padding: Sides<u16>,
    ) -> Self {
        if let Some(ctx) = ctx {
            element.init(ctx);
        }

        let margin = element.get_margin();
        let child_padding = element.get_padding();

        let mut bounds = bounds
            .width(
                element.get_size().width,
                padding.get_horizontal(),
                margin.get_horizontal(),
            )
            .height(
                element.get_size().height,
                padding.get_vertical(),
                margin.get_vertical(),
            );

        let children: Vec<Self> = element
            .get_children()
            .into_iter()
            .map(|c| Self::from_element(c, ctx, &bounds, child_padding))
            .collect();

        // Don't let this element's min size shrink below the largest child.
        // TODO: Maybe not calc this if not shrink.
        // FIXME: This has a bug!
        for child in &children {
            if child.widget.get_position().is_dynamic() {
                let child_margin = child.widget.get_margin();
                let child_min_w = child.bounds.min.width + child_margin.get_horizontal() as f32;
                let child_min_h = child.bounds.min.height + child_margin.get_vertical() as f32;

                bounds.min.width = bounds.min.width.max(child_min_w).min(bounds.max.width);
                bounds.min.height = bounds.min.height.max(child_min_h).min(bounds.max.height);
            }
        }

        Self {
            children,
            widget: element,
            bounds,
            position: Default::default(),
            size: Default::default(),
        }
    }

    pub(crate) fn is_point_inside(&self, point: Point<i16>) -> bool {
        let point = Point {
            x: point.x as f32,
            y: point.y as f32,
        };

        let lower_bound = Point {
            x: self.position.x + self.size.width,
            y: self.position.y + self.size.height,
        };

        self.position.x <= point.x
            && self.position.y <= point.y
            && lower_bound.x >= point.x
            && lower_bound.y >= point.y
    }
}
impl<T, R: Renderer> std::fmt::Debug for Node<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("position", &self.position)
            .field("size", &self.size)
            .field("bounds", &self.bounds)
            .field("children", &self.children)
            .finish()
    }
}
