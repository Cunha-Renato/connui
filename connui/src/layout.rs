use crate::{tree::layout::*, types::*};

pub(crate) fn measure(element: &LayoutElementRef, bounds: &Bounds) {
    let (axis, size_op, padding) = {
        let style = element.style.borrow();
        // Clear (also resets the dirty flag).
        element.data_mut().clear();

        (
            style.layout.axis,
            style.size.validate(),
            style.padding.clone(),
        )
    };
    let children = element.link.borrow().children.clone();

    let mut bounds = bounds.width(size_op.width).height(size_op.height);
    let inner_bounds = bounds.inner_bounds(&padding);

    let relative_op = axis.main_cross(&size_op);
    let mut relative_size = axis.main_cross(&bounds.desired(size_op));

    for child in &children {
        measure(child, &inner_bounds);

        let child_style = child.style.borrow();
        if child_style.position.is_pinned() {
            continue;
        }
        let child_data = child.data();

        let relative_child_size = axis.main_cross(&(
            child_data.rect.width() + child_style.margin.horizontal(),
            child_data.rect.height() + child_style.margin.vertical(),
        ));

        if !relative_op.main.is_absolute() {
            relative_size.main += relative_child_size.main;
        }
        if !relative_op.cross.is_absolute() {
            relative_size.cross = relative_size.cross.max(relative_child_size.cross);
        }
    }

    let (width, height) = axis.horizontal_vertical(&relative_size);

    // TODO: There's a bug here. (size + padded_size). Sometimes it doubles the actual size (observed on fill).
    bounds = bounds.padding(&padding);
    let padded_size = bounds.desired(size_op);
    let acc_size = Size::new(width + padded_size.width, height + padded_size.height);

    let mut data = element.data_mut();
    data.rect.size = bounds.clamp(acc_size);
    data.bounds = bounds;
}

pub(crate) fn resolve_children_size(element: &LayoutElementRef) {
    let children = element.link.borrow().children.clone();
    if children.is_empty() {
        return;
    }

    let (axis, relative_size) = {
        let style = element.style.borrow();
        let data = element.data();
        let axis = style.layout.axis;

        let relative_size = axis.main_cross(&Size::new(
            data.rect.width() - style.padding.horizontal(),
            data.rect.height() - style.padding.vertical(),
        ));

        (axis, relative_size)
    };

    let mut budget = relative_size.main;
    for child in &children {
        let child_style = child.style.borrow();
        let mut child_data = child.data_mut();

        let relative_child_margin = axis.main_cross(&(
            child_style.margin.horizontal(),
            child_style.margin.vertical(),
        ));
        let relative_child_size = axis.main_cross_mut(&mut child_data.rect.size);

        if let Sizing::Fill { .. } = axis.cross(&child_style.size.validate()) {
            *relative_child_size.cross = relative_size.cross - relative_child_margin.cross;
        }

        budget -= *relative_child_size.main + relative_child_margin.main;
    }

    if budget > LPixel::new(0.0) {
        resolve_grow(&children, budget, axis);
    } else if budget < LPixel::new(0.0) {
        resolve_shrink(&children, budget, axis);
    }

    for child in &children {
        resolve_children_size(child);
    }
}

fn resolve_grow(children: &[LayoutElementRef], mut budget: LPixel<f32>, axis: LayoutAxis) {
    let mut growable = children
        .iter()
        .filter(|child| {
            let style = child.style.borrow();
            let data = child.data();

            let main_op = axis.main(&style.size.validate());
            let child_main_size = axis.main(&data.rect.size);
            let child_main_max = axis.main(&data.bounds.max);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size < child_main_max
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while budget > LPixel::new(0.0) && !growable.is_empty() {
        let portion = budget / LPixel::new(growable.len() as f32);

        growable.retain(|child| {
            let mut guard = child.data_mut();
            let data = &mut *guard; // plain &mut so fields can be borrowed disjointly
            let child_main_size = axis.main_mut(&mut data.rect.size);
            let child_main_max = axis.main(&data.bounds.max);

            let used = (child_main_max - *child_main_size).min(portion);

            *child_main_size += used;
            budget -= used;

            (child_main_max - *child_main_size).abs() > epsilon
        });

        if budget <= epsilon {
            break;
        }
    }
}

fn resolve_shrink(children: &[LayoutElementRef], mut deficit: LPixel<f32>, axis: LayoutAxis) {
    let mut shrinkable = children
        .iter()
        .filter(|child| {
            let style = child.style.borrow();
            let data = child.data();

            let main_op = axis.main(&style.size.validate());
            let child_main_size = axis.main(&data.rect.size);
            let child_main_min = axis.main(&data.bounds.min);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size > child_main_min
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while deficit.abs() > epsilon && !shrinkable.is_empty() {
        let total_size = LPixel::new(
            shrinkable
                .iter()
                .map(|child| axis.main(&child.data().rect.size).inner())
                .sum::<f32>(),
        );

        if total_size <= epsilon {
            break;
        }

        let portion = deficit / LPixel::new(shrinkable.len() as f32);
        shrinkable.retain(|child| {
            let mut guard = child.data_mut();
            let data = &mut *guard;
            let child_main_size = axis.main_mut(&mut data.rect.size);
            let child_main_min = axis.main(&data.bounds.min);

            let child_main_pct = *child_main_size / total_size;
            let portion = portion * child_main_pct;
            let used = (child_main_min - *child_main_size).max(portion);

            *child_main_size += used;
            deficit -= used;

            (child_main_min - *child_main_size).abs() > epsilon
        });
    }
}

#[inline]
fn align_h_offset(align: HorAlign, free: LPixel<f32>) -> LPixel<f32> {
    let free = free.max(LPixel::new(0.0)); // never push content outside the box on overflow
    match align {
        HorAlign::Left => LPixel::new(0.0),
        HorAlign::Middle => free * LPixel::new(0.5),
        HorAlign::Right => free,
    }
}

#[inline]
fn align_v_offset(align: VerAlign, free: LPixel<f32>) -> LPixel<f32> {
    let free = free.max(LPixel::new(0.0));
    match align {
        VerAlign::Top => LPixel::new(0.0),
        VerAlign::Middle => free * LPixel::new(0.5),
        VerAlign::Bottom => free,
    }
}

pub(crate) fn resolve_children_position(element: &LayoutElementRef) {
    let children = element.link.borrow().children.clone();

    let (mut cursor, axis, element_position, content_w, content_h, h_align, v_align) = {
        let style = element.style.borrow();
        let data = element.data();
        let axis = style.layout.axis;
        let h_align = style.hor_align;
        let v_align = style.ver_align;

        let padding = &style.padding;
        let content_w = data.rect.width() - (padding.left + padding.right);
        let content_h = data.rect.height() - (padding.top + padding.bottom);

        // Total size of flowing (non-pinned) children along the main axis.
        let main_total: f32 = children
            .iter()
            .filter_map(|c| {
                let cs = c.style.borrow();
                if !matches!(cs.position, Positioning::Dynamic) {
                    return None;
                }
                let cd = c.data();
                Some(match axis {
                    LayoutAxis::Horizontal => {
                        cd.rect.width().inner() + cs.margin.horizontal().inner()
                    }
                    LayoutAxis::Vertical => cd.rect.height().inner() + cs.margin.vertical().inner(),
                })
            })
            .sum();
        let main_total = LPixel::new(main_total);

        let mut cursor = data.rect.position;
        cursor.x += padding.left;
        cursor.y += padding.top;

        // Main axis: shift the starting cursor by the free space.
        match axis {
            LayoutAxis::Horizontal => {
                cursor.x += align_h_offset(h_align, content_w - main_total);
            }
            LayoutAxis::Vertical => {
                cursor.y += align_v_offset(v_align, content_h - main_total);
            }
        }

        (
            cursor,
            axis,
            data.rect.position,
            content_w,
            content_h,
            h_align,
            v_align,
        )
    };

    // Cross-axis origin (before per-child offset).
    let cross_origin = match axis {
        LayoutAxis::Horizontal => cursor.y,
        LayoutAxis::Vertical => cursor.x,
    };

    for child in &children {
        {
            let child_style = child.style.borrow();
            let mut child_data = child.data_mut();

            match child_style.position {
                Positioning::Pinned { position, flags } => {
                    if flags.contains(PinnedFlags::RELATIVE_TO_PARENT) {
                        child_data.rect.position.x = position.x.as_float() + element_position.x;
                        child_data.rect.position.y = position.y.as_float() + element_position.y;
                    } else {
                        child_data.rect.position = position.map(|lp| lp.as_float())
                    }
                }
                Positioning::Dynamic => {
                    let margin = &child_style.margin;
                    let outer_w = child_data.rect.width() + margin.horizontal();
                    let outer_h = child_data.rect.height() + margin.vertical();

                    match axis {
                        LayoutAxis::Horizontal => {
                            // Cross axis is vertical.
                            let cross = align_v_offset(v_align, content_h - outer_h);
                            child_data.rect.position.x = cursor.x + margin.left;
                            child_data.rect.position.y = cross_origin + cross + margin.top;
                            cursor.x += outer_w;
                        }
                        LayoutAxis::Vertical => {
                            // Cross axis is horizontal.
                            let cross = align_h_offset(h_align, content_w - outer_w);
                            child_data.rect.position.x = cross_origin + cross + margin.left;
                            child_data.rect.position.y = cursor.y + margin.top;
                            cursor.y += outer_h;
                        }
                    }
                }
            }
        } // style/data guards dropped before recursing

        resolve_children_position(child);
    }
}
