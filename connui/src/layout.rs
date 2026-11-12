use crate::{tree::layout::*, types::*};

pub(crate) fn measure(element: &LayoutElementRef, bounds: &Bounds) {
    let mut element_borrow_mut = element.borrow_mut();
    element_borrow_mut.clear();

    let axis = element_borrow_mut.style.layout.axis;
    let size_op = element_borrow_mut.style.size.validate();
    let padding = element_borrow_mut.style.padding.clone();

    let mut bounds = bounds.width(size_op.width).height(size_op.height);
    let inner_bounds = bounds.inner_bounds(&padding);

    let relative_op = axis.main_cross(&size_op);
    let mut relative_size = axis.main_cross(&bounds.desired(size_op));

    for child in &element_borrow_mut.children {
        if child.borrow().style.position.is_pinned() {
            measure(child, &Bounds::default());
            continue;
        } else {
            measure(child, &inner_bounds);
        }

        let child_borrow = child.borrow();

        let relative_child_size = axis.main_cross(&(
            child_borrow.rect.width() + child_borrow.style.margin.horizontal(),
            child_borrow.rect.height() + child_borrow.style.margin.vertical(),
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

    element_borrow_mut.rect.size = bounds.clamp(acc_size);
    element_borrow_mut.bounds = bounds;
}

pub(crate) fn resolve_children_size(element: &LayoutElementRef) {
    let element_borrow_mut = element.borrow_mut();

    if element_borrow_mut.children.is_empty() {
        return;
    }

    let axis = element_borrow_mut.style.layout.axis;

    let relative_size = axis.main_cross(&Size::new(
        element_borrow_mut.rect.width() - element_borrow_mut.style.padding.horizontal(),
        element_borrow_mut.rect.height() - element_borrow_mut.style.padding.vertical(),
    ));

    let mut budget = relative_size.main;
    for child in &element_borrow_mut.children {
        let mut child_borrow_mut = child.borrow_mut();

        let relative_child_margin = axis.main_cross(&(
            child_borrow_mut.style.margin.horizontal(),
            child_borrow_mut.style.margin.vertical(),
        ));
        let cross_size_op = axis.cross(&child_borrow_mut.style.size.validate());
        let relative_child_size = axis.main_cross_mut(&mut child_borrow_mut.rect.size);

        if let Sizing::Fill { .. } = cross_size_op {
            *relative_child_size.cross = relative_size.cross - relative_child_margin.cross;
        }

        budget -= *relative_child_size.main + relative_child_margin.main;
    }

    if budget > LPixel::new(0.0) {
        resolve_grow(&element_borrow_mut.children, budget, axis);
    } else if budget < LPixel::new(0.0) {
        resolve_shrink(&element_borrow_mut.children, budget, axis);
    }

    for child in &element_borrow_mut.children {
        resolve_children_size(child);
    }
}

fn resolve_grow(children: &[LayoutElementRef], mut budget: LPixel<f32>, axis: LayoutAxis) {
    let mut growable = children
        .iter()
        .filter(|child| {
            let borrow = child.borrow();

            let main_op = axis.main(&borrow.style.size.validate());
            let child_main_size = axis.main(&borrow.rect.size);
            let child_main_max = axis.main(&borrow.bounds.max);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size < child_main_max
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while budget > LPixel::new(0.0) && !growable.is_empty() {
        let portion = budget / LPixel::new(growable.len() as f32);

        growable.retain(|child| {
            let mut borrow_mut = child.borrow_mut();

            let child_main_max = axis.main(&borrow_mut.bounds.max);
            let child_main_size = axis.main_mut(&mut borrow_mut.rect.size);

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
            let borrow = child.borrow();

            let main_op = axis.main(&borrow.style.size.validate());
            let child_main_size = axis.main(&borrow.rect.size);
            let child_main_min = axis.main(&borrow.bounds.min);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size > child_main_min
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while deficit.abs() > epsilon && !shrinkable.is_empty() {
        let total_size = LPixel::new(
            shrinkable
                .iter()
                .map(|child| axis.main(&child.borrow().rect.size).inner())
                .sum::<f32>(),
        );

        if total_size <= epsilon {
            break;
        }

        let portion = deficit / LPixel::new(shrinkable.len() as f32);
        shrinkable.retain(|child| {
            let mut borrow_mut = child.borrow_mut();

            let child_main_min = axis.main(&borrow_mut.bounds.min);
            let child_main_size = axis.main_mut(&mut borrow_mut.rect.size);

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
    let borrow = element.borrow();

    let (mut cursor, axis, content_w, content_h, h_align, v_align) = {
        let axis = borrow.style.layout.axis;
        let h_align = borrow.style.hor_align;
        let v_align = borrow.style.ver_align;

        let padding = &borrow.style.padding;
        let content_w = borrow.rect.width() - (padding.left + padding.right);
        let content_h = borrow.rect.height() - (padding.top + padding.bottom);

        // Total size of flowing (non-pinned) children along the main axis.
        let main_total: f32 = borrow
            .children
            .iter()
            .filter_map(|child| {
                let child_borrow = child.borrow();

                if !matches!(child_borrow.style.position, Positioning::Dynamic) {
                    return None;
                }
                Some(match axis {
                    LayoutAxis::Horizontal => {
                        child_borrow.rect.width().inner()
                            + child_borrow.style.margin.horizontal().inner()
                    }
                    LayoutAxis::Vertical => {
                        child_borrow.rect.height().inner()
                            + child_borrow.style.margin.vertical().inner()
                    }
                })
            })
            .sum();
        let main_total = LPixel::new(main_total);

        let mut cursor = borrow.rect.position;
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

        (cursor, axis, content_w, content_h, h_align, v_align)
    };

    // Cross-axis origin (before per-child offset).
    let cross_origin = match axis {
        LayoutAxis::Horizontal => cursor.y,
        LayoutAxis::Vertical => cursor.x,
    };

    for child in &borrow.children {
        {
            let mut child_borrow_mut = child.borrow_mut();

            match child_borrow_mut.style.position {
                Positioning::Pinned { position, flags } => {
                    if flags.contains(PinnedFlags::RELATIVE_TO_PARENT) {
                        child_borrow_mut.rect.position.x =
                            position.x.as_float() + borrow.rect.position.x;
                        child_borrow_mut.rect.position.y =
                            position.y.as_float() + borrow.rect.position.y;
                    } else {
                        child_borrow_mut.rect.position = position.map(|lp| lp.as_float())
                    }
                }
                Positioning::Dynamic => {
                    let outer_w =
                        child_borrow_mut.rect.width() + child_borrow_mut.style.margin.horizontal();
                    let outer_h =
                        child_borrow_mut.rect.height() + child_borrow_mut.style.margin.vertical();

                    match axis {
                        LayoutAxis::Horizontal => {
                            // Cross axis is vertical.
                            let cross = align_v_offset(v_align, content_h - outer_h);
                            child_borrow_mut.rect.position.x =
                                cursor.x + child_borrow_mut.style.margin.left;
                            child_borrow_mut.rect.position.y =
                                cross_origin + cross + child_borrow_mut.style.margin.top;
                            cursor.x += outer_w;
                        }
                        LayoutAxis::Vertical => {
                            // Cross axis is horizontal.
                            let cross = align_h_offset(h_align, content_w - outer_w);
                            child_borrow_mut.rect.position.x =
                                cross_origin + cross + child_borrow_mut.style.margin.left;
                            child_borrow_mut.rect.position.y =
                                cursor.y + child_borrow_mut.style.margin.top;
                            cursor.y += outer_h;
                        }
                    }
                }
            }
        }

        resolve_children_position(child);
    }
}
