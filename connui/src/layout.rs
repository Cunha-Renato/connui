use crate::{tree::layout::*, types::*};

pub(crate) fn measure(
    element_key: LayoutElementKey,
    tree: &mut LayoutElementTree,
    bounds: &Bounds,
) {
    let (axis, size_op, padding, child_count) = {
        let element = &mut tree[element_key];
        // Clear.
        element.bounds = Bounds::default();
        element.rect = Rect::default();
        element.clip = Rect::default();

        (
            element.style.layout.axis,
            element.style.size.validate(),
            element.style.padding.clone(),
            element.children.len(),
        )
    };

    let mut bounds = bounds.width(size_op.width).height(size_op.height);
    let inner_bounds = bounds.inner_bounds(&padding);

    let relative_op = axis.main_cross(&size_op);
    let mut relative_size = axis.main_cross(&bounds.desired(size_op));

    for child_index in 0..child_count {
        let child_key = tree[element_key].children[child_index];
        measure(child_key, tree, &inner_bounds);

        let child = &tree[child_key];
        if child.style.position.is_pinned() {
            continue;
        }

        let relative_child_size = axis.main_cross(&(
            child.rect.width() + child.style.margin.horizontal(),
            child.rect.height() + child.style.margin.vertical(),
        ));

        if !relative_op.main.is_absolute() {
            relative_size.main += relative_child_size.main;
        }
        if !relative_op.cross.is_absolute() {
            relative_size.cross = relative_size.cross.max(relative_child_size.cross);
        }
    }

    let (width, height) = axis.horizontal_vertical(&relative_size);

    bounds = bounds.padding(&padding);
    let padded_size = bounds.desired(size_op);
    let acc_size = Size::new(width + padded_size.width, height + padded_size.height);

    let element = &mut tree[element_key];
    element.rect.size = bounds.clamp(acc_size);
    element.bounds = bounds;
}

pub(crate) fn resolve_children_size(element_key: LayoutElementKey, tree: &mut LayoutElementTree) {
    let (axis, relative_size, child_count) = {
        let element = &tree[element_key];

        if element.children.is_empty() {
            return;
        }

        let axis = element.style.layout.axis;

        let relative_size = axis.main_cross(&Size::new(
            element.rect.width() - element.style.padding.horizontal(),
            element.rect.height() - element.style.padding.vertical(),
        ));

        (axis, relative_size, element.children.len())
    };

    let mut budget = relative_size.main;
    for child_index in 0..child_count {
        let child_key = tree[element_key].children[child_index];
        let child = &mut tree[child_key];

        let relative_child_margin = axis.main_cross(&(
            child.style.margin.horizontal(),
            child.style.margin.vertical(),
        ));
        let relative_child_size = axis.main_cross_mut(&mut child.rect.size);

        if let Sizing::Fill { .. } = axis.cross(&child.style.size.validate()) {
            *relative_child_size.cross = relative_size.cross - relative_child_margin.cross;
        }

        budget -= *relative_child_size.main + relative_child_margin.main;
    }

    if budget > LPixel::new(0.0) {
        resolve_grow(element_key, tree, budget, axis);
    } else if budget < LPixel::new(0.0) {
        resolve_shrink(element_key, tree, budget, axis);
    }

    for child_index in 0..child_count {
        let child_key = tree[element_key].children[child_index];
        resolve_children_size(child_key, tree);
    }
}

fn resolve_grow(
    element_key: LayoutElementKey,
    tree: &mut LayoutElementTree,
    mut budget: LPixel<f32>,
    axis: LayoutAxis,
) {
    let mut growable = (0..tree[element_key].children.len())
        .map(|child_index| tree[element_key].children[child_index])
        .filter(|child_key| {
            let child = &tree[*child_key];

            let main_op = axis.main(&child.style.size.validate());
            let child_main_size = axis.main(&child.rect.size);
            let child_main_max = axis.main(&child.bounds.max);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size < child_main_max
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while budget > LPixel::new(0.0) && !growable.is_empty() {
        let portion = budget / LPixel::new(growable.len() as f32);

        growable.retain(|child_key| {
            let child = &mut tree[*child_key];
            let child_main_size = axis.main_mut(&mut child.rect.size);
            let child_main_max = axis.main(&child.bounds.max);

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

fn resolve_shrink(
    element_key: LayoutElementKey,
    tree: &mut LayoutElementTree,
    mut deficit: LPixel<f32>,
    axis: LayoutAxis,
) {
    let mut shrinkable = (0..tree[element_key].children.len())
        .map(|child_index| tree[element_key].children[child_index])
        .filter(|child_key| {
            let child = &tree[*child_key];

            let main_op = axis.main(&child.style.size.validate());
            let child_main_size = axis.main(&child.rect.size);
            let child_main_min = axis.main(&child.bounds.min);

            matches!(main_op, Sizing::Fill { .. }) && child_main_size > child_main_min
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while deficit.abs() > epsilon && !shrinkable.is_empty() {
        let total_size = LPixel::new(
            shrinkable
                .iter()
                .map(|child_key| axis.main(&tree[*child_key].rect.size).inner())
                .sum::<f32>(),
        );

        if total_size <= epsilon {
            break;
        }

        let portion = deficit / LPixel::new(shrinkable.len() as f32);
        shrinkable.retain(|child_key| {
            let child = &mut tree[*child_key];
            let child_main_size = axis.main_mut(&mut child.rect.size);
            let child_main_min = axis.main(&child.bounds.min);

            let child_main_pct = *child_main_size / total_size;
            let portion = portion * child_main_pct;
            let used = (child_main_min - *child_main_size).max(portion);

            *child_main_size += used;
            deficit -= used;

            (child_main_min - *child_main_size).abs() > epsilon
        });
    }
}

pub(crate) fn resolve_children_position(
    element_key: LayoutElementKey,
    tree: &mut LayoutElementTree,
) {
    let (mut cursor, axis, element_position, child_count) = {
        let element = &tree[element_key];
        let axis = element.style.layout.axis;

        let mut cursor = element.rect.position;
        cursor.x += element.style.padding.left;
        cursor.y += element.style.padding.top;

        (cursor, axis, element.rect.position, element.children.len())
    };

    for child_index in 0..child_count {
        let child_key = tree[element_key].children[child_index];
        {
            let child = &mut tree[child_key];

            match child.style.position {
                Positioning::Pinned { position, flags } => {
                    if flags.contains(PinnedFlags::RELATIVE_TO_PARENT) {
                        child.rect.position.x += position.x.as_float() + element_position.x;
                        child.rect.position.y += position.y.as_float() + element_position.y;
                    } else {
                        child.rect.position = position.map(|lp| lp.as_float())
                    }
                }
                Positioning::Dynamic => {
                    child.rect.position.x += cursor.x + child.style.margin.left;
                    child.rect.position.y += cursor.y + child.style.margin.top;

                    match axis {
                        LayoutAxis::Horizontal => {
                            cursor.x += child.rect.width() + child.style.margin.horizontal()
                        }
                        LayoutAxis::Vertical => {
                            cursor.y += child.rect.height() + child.style.margin.vertical()
                        }
                    }
                }
            }
        }

        resolve_children_position(child_key, tree);
    }
}
