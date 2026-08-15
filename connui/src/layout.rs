use crate::types::*;

pub trait WidgetDesc {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Default::default()
    }

    #[inline]
    fn get_position(&self) -> Position {
        Default::default()
    }

    #[inline]
    fn get_padding(&self) -> LSides<u16> {
        Default::default()
    }

    #[inline]
    fn get_margin(&self) -> LSides<u16> {
        Default::default()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        Default::default()
    }
}

pub trait WidgetLayout: WidgetDesc {
    // TODO: Implement.
    // fn depends_on_children(&self) -> bool;

    fn measure(&mut self, layout_element: &mut LayoutElement, bounds: &Bounds);
    fn resolve(&mut self, layout_element: &mut LayoutElement);
}

#[derive(Default)]
pub struct LayoutElement {
    pub children: Vec<Self>,
    pub bounds: Bounds,
    pub rect: LRect<f32, f32>,
    pub clip: LRect<f32, f32>,
    pub dirty: bool,
}
impl LayoutElement {
    pub(crate) fn new<T, R: crate::renderer::Renderer>(
        element: &crate::widget::Element<T, R>,
    ) -> Self {
        Self {
            children: element.get_children().iter().map(Self::new).collect(),
            bounds: Bounds::default(),
            rect: Rect::default(),
            clip: Rect::default(),
            dirty: false,
        }
    }

    pub(crate) fn clip(&mut self, prev: &LRect<f32, f32>) {
        self.clip = prev.intersection(&self.rect).unwrap_or_default();

        for child in &mut self.children {
            child.clip(&self.clip);
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        let mut dirty = self.dirty;

        for child in &self.children {
            dirty |= child.is_dirty();
        }

        dirty
    }
}
impl std::fmt::Debug for LayoutElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("LayoutElement");

        let mut dbg = binding
            .field("bounds", &self.bounds)
            .field("rect", &self.rect)
            .field("clip", &self.clip)
            .field("dirty", &self.dirty);

        if !self.children.is_empty() {
            dbg = dbg.field("children", &self.children);
        }

        dbg.finish()
    }
}

pub mod default {
    use crate::{renderer::Renderer, widget::Widget};

    use super::*;

    #[macro_export]
    macro_rules! impl_default_widget_layout {
        ($({$($pregen:tt)+})?$widget:ident $({$($gen:tt)+})?) => {
            impl$(<$($pregen)+>)? $crate::layout::WidgetLayout for $widget$(<$($gen)+>)? {
                #[inline]
                fn measure(&mut self, layout_element: &mut $crate::layout::LayoutElement, bounds: &Bounds) {
                    $crate::layout::default::measure(self, layout_element, bounds);
                }

                fn resolve(&mut self, layout_element: &mut $crate::layout::LayoutElement) {
                    $crate::layout::default::resolve_children_size(self, layout_element);
                    $crate::layout::default::resolve_children_position(self, layout_element);

                    for (child_widget, child_layout_element) in self
                        .get_children_mut()
                        .iter_mut()
                        .zip(&mut layout_element.children)
                    {
                        // Just to make sure.
                        child_layout_element.rect.size = child_layout_element
                            .bounds
                            .clamp(child_layout_element.rect.size);

                        child_widget.resolve(child_layout_element);
                    }
                }
            }
        };
    }

    pub fn measure<W, T, R>(widget: &mut W, layout_element: &mut LayoutElement, bounds: &Bounds)
    where
        W: Widget<T, R>,
        R: Renderer,
    {
        let axis = widget.get_layout().axis;
        let size_op = widget.get_size().validate();
        let padding = widget.get_padding();

        // Ensures correct bounds.
        let mut bounds = bounds.width(size_op.width).height(size_op.height);
        let inner_bounds = bounds.inner_bounds(&padding);

        let (main_op, cross_op) = axis.pack(&size_op);
        // Initiates sizes based on desired size within bounds.
        let (mut main_size, mut cross_size) = axis.pack(&bounds.desired(size_op));

        for (child_widget, child_layout_element) in widget
            .get_children_mut()
            .iter_mut()
            .zip(&mut layout_element.children)
        {
            child_widget.measure(child_layout_element, &inner_bounds);

            // For Fit & Fill ops.
            let (main_child_size, cross_child_size) = axis.pack(&child_layout_element.rect.size);

            if !main_op.is_absolute() {
                main_size += main_child_size;
            }

            if !cross_op.is_absolute() {
                cross_size = cross_size.max(cross_child_size);
            }
        }

        let (width, height) = axis.unpack(main_size, cross_size);

        // Padding.
        bounds = bounds.padding(&padding);
        let padded_size = bounds.desired(size_op);
        let acc_size = Size::new(width + padded_size.width, height + padded_size.height);

        layout_element.rect.size = bounds.clamp(acc_size);
        layout_element.bounds = bounds;
    }

    /// This pass is top - down serves to resize children that can take more space or are overflowing (dynamic).
    pub fn resolve_children_size<W, T, R>(widget: &mut W, layout_element: &mut LayoutElement)
    where
        W: Widget<T, R>,
        R: Renderer,
    {
        // Early return.
        if layout_element.children.is_empty() {
            return;
        }

        let axis = widget.get_layout().axis;
        let padding = widget.get_padding().map(|lp| lp.as_float());

        let (main_size, cross_size) = axis.pack(&Size::new(
            layout_element.rect.width() - padding.get_horizontal(),
            layout_element.rect.height() - padding.get_vertical(),
        ));

        let budget = widget
            .get_children()
            .iter()
            .zip(&mut layout_element.children)
            .fold(main_size, |total, (child_widget, child_layout_element)| {
                let (child_main, child_cross) = axis.pack_mut(&mut child_layout_element.rect.size);

                // Resizing cross Fill children.
                if let SizeOp::Fill { .. } = axis.cross(&child_widget.get_size().validate()) {
                    *child_cross = (*child_cross).max(cross_size);
                }

                // Acc main_occupied size.
                total - *child_main
            });

        // Node has space to give.
        if budget > LPixel::new(0.0) {
            resolve_grow(widget, layout_element, budget, axis);
        } else if budget < LPixel::new(0.0) {
            resolve_shrink(widget, layout_element, budget, axis);
        }
    }

    // Always main axis.
    pub fn resolve_grow<W, T, R>(
        widget: &mut W,
        layout_element: &mut LayoutElement,
        mut budget: LPixel<f32>,
        axis: LayoutAxis,
    ) where
        W: Widget<T, R>,
        R: Renderer,
    {
        // Collects every child that is allowed to grow.
        let mut growable = widget
            .get_children_mut()
            .iter_mut()
            .zip(&mut layout_element.children)
            .filter_map(|(child_widget, child_layout_element)| {
                let main_op = axis.main(&child_widget.get_size().validate());
                let main_size = axis.main(&child_layout_element.rect.size);
                let main_max_bounds = axis.main(&child_layout_element.bounds.max);

                (matches!(main_op, SizeOp::Fill { .. }) && main_size < main_max_bounds)
                    .then_some(child_layout_element)
            })
            .collect::<Vec<_>>();

        // Distribution of the parent's available space.
        let epsilon = LPixel::new(f32::EPSILON);
        while budget > LPixel::new(0.0) && !growable.is_empty() {
            let portion = budget / LPixel::new(growable.len() as f32);

            growable.retain_mut(|child_layout_element| {
                let main_size = axis.main_mut(&mut child_layout_element.rect.size);
                let main_max = axis.main(&child_layout_element.bounds.max);

                let used = (main_max - *main_size).min(portion);

                *main_size += used;
                budget -= used;

                (main_max - *main_size).abs() > epsilon
            });

            if budget <= epsilon {
                break;
            }
        }
    }

    pub fn resolve_shrink<W, T, R>(
        widget: &mut W,
        layout_element: &mut LayoutElement,
        mut deficit: LPixel<f32>,
        axis: LayoutAxis,
    ) where
        W: Widget<T, R>,
        R: Renderer,
    {
        // Collects every child that needs to shrink.
        let mut shrinkable = widget
            .get_children_mut()
            .iter_mut()
            .zip(&mut layout_element.children)
            .filter_map(|(child_widget, child_layout_element)| {
                let main_op = axis.main(&child_widget.get_size().validate());
                let main_size = axis.main(&child_layout_element.rect.size);
                let main_min_bounds = axis.main(&child_layout_element.bounds.min);

                (matches!(main_op, SizeOp::Fill { .. }) && main_size > main_min_bounds)
                    .then_some(child_layout_element)
            })
            .collect::<Vec<_>>();

        // Shrinking the children that are taking too much space.
        let epsilon = LPixel::new(f32::EPSILON);
        while deficit.abs() > epsilon && !shrinkable.is_empty() {
            let total_size = LPixel::new(
                shrinkable
                    .iter()
                    .map(|c| axis.main(&c.rect.size).inner())
                    .sum::<f32>(),
            );

            if total_size <= epsilon {
                break;
            }

            let portion = deficit / LPixel::new(shrinkable.len() as f32);
            shrinkable.retain_mut(|child_layout_element| {
                let main_size = axis.main_mut(&mut child_layout_element.rect.size);
                let main_min = axis.main(&child_layout_element.bounds.min);

                let main_pct = *main_size / total_size;
                let portion = portion * main_pct;
                let used = (main_min - *main_size).max(portion);

                *main_size += used;
                deficit -= used;

                (main_min - *main_size).abs() > epsilon
            });
        }
    }

    // Top - down positioning.
    pub fn resolve_children_position<W, T, R>(widget: &mut W, layout_element: &mut LayoutElement)
    where
        W: Widget<T, R>,
        R: Renderer,
    {
        let axis = widget.get_layout().axis;
        let padding = widget.get_padding().map(|lp| lp.as_float());

        let mut cursor = layout_element.rect.position;
        cursor.x += padding.left;
        cursor.y += padding.top;

        for child_layout_element in &mut layout_element.children {
            child_layout_element.rect.position.x += cursor.x;
            child_layout_element.rect.position.y += cursor.y;

            match axis {
                LayoutAxis::Horizontal => cursor.x += child_layout_element.rect.width(),
                LayoutAxis::Vertical => cursor.y += child_layout_element.rect.height(),
            }
        }
    }
}
