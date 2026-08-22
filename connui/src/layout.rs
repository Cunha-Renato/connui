use crate::{tree::layout::*, types::*};

pub fn measure(element_key: LayoutElementKey, tree: &mut LayoutElementTree, bounds: &Bounds) {
    let (axis, size_op, padding, children) = {
        let element = &tree[element_key];
        (
            element.style.layout.axis,
            element.style.size.validate(),
            element.style.padding.clone(),
            element.children.clone(),
        )
    };

    let mut bounds = bounds.width(size_op.width).height(size_op.height);
    let inner_bounds = bounds.inner_bounds(&padding);

    let (main_op, cross_op) = axis.pack(&size_op);
    let (mut main_size, mut cross_size) = axis.pack(&bounds.desired(size_op));

    for child_key in children {
        measure(child_key, tree, &inner_bounds);

        let child = &tree[child_key];
        if child.style.position.is_pinned() {
            continue;
        }

        let (child_main_size, child_cross_size) = axis.pack(&(
            child.rect.width() + child.style.margin.get_horizontal(),
            child.rect.height() + child.style.margin.get_vertical(),
        ));

        if !main_op.is_absolute() {
            main_size += child_main_size;
        }
        if !cross_op.is_absolute() {
            cross_size = cross_size.max(child_cross_size);
        }
    }

    let (width, height) = axis.unpack(main_size, cross_size);

    bounds = bounds.padding(&padding);
    let padded_size = bounds.desired(size_op);
    let acc_size = Size::new(width + padded_size.width, height + padded_size.height);

    let element = &mut tree[element_key];
    element.rect.size = bounds.clamp(acc_size);
    element.bounds = bounds;
}

pub fn resolve_children_size(element_key: LayoutElementKey, tree: &mut LayoutElementTree) {
    let (axis, main_size, cross_size, children) = {
        let element = &tree[element_key];

        if element.children.is_empty() {
            return;
        }

        let axis = element.style.layout.axis;

        let (main_size, cross_size) = axis.pack(&Size::new(
            element.rect.width() - element.style.padding.get_horizontal(),
            element.rect.height() - element.style.padding.get_vertical(),
        ));

        (axis, main_size, cross_size, element.children.clone())
    };

    let budget = children.iter().fold(main_size, |total, child_key| {
        let child = &mut tree[*child_key];

        let (child_margin_main, child_margin_cross) = axis.pack(&(
            child.style.margin.get_horizontal(),
            child.style.margin.get_vertical(),
        ));
        let (child_main_size, child_cross_size) = axis.pack_mut(&mut child.rect.size);

        if let SizeOp::Fill { .. } = axis.cross(&child.style.size.validate()) {
            *child_cross_size = cross_size - child_margin_cross;
        }

        total - *child_main_size - child_margin_main
    });

    if budget > LPixel::new(0.0) {
        resolve_grow(element_key, tree, budget, axis);
    } else if budget < LPixel::new(0.0) {
        resolve_shrink(element_key, tree, budget, axis);
    }
    
    for child_key in tree[element_key].children.clone() {
        resolve_children_size(child_key, tree);
    }
}

fn resolve_grow(
    element_key: LayoutElementKey,
    tree: &mut LayoutElementTree,
    mut budget: LPixel<f32>,
    axis: LayoutAxis,
) {
    let children = tree[element_key].children.clone();

    let mut growable = children
        .into_iter()
        .filter(|child_key| {
            let child = &tree[*child_key];

            let main_op = axis.main(&child.style.size.validate());
            let child_main_size = axis.main(&child.rect.size);
            let child_main_max = axis.main(&child.bounds.max);

            matches!(main_op, SizeOp::Fill { .. }) && child_main_size < child_main_max
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
    let children = tree[element_key].children.clone();

    let mut shrinkable = children
        .into_iter()
        .filter(|child_key| {
            let child = &tree[*child_key];

            let main_op = axis.main(&child.style.size.validate());
            let child_main_size = axis.main(&child.rect.size);
            let child_main_min = axis.main(&child.bounds.min);

            matches!(main_op, SizeOp::Fill { .. }) && child_main_size > child_main_min
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

pub fn resolve_children_position(element_key: LayoutElementKey, tree: &mut LayoutElementTree) {
    let (axis, mut cursor, children) = {
        let element = &tree[element_key];
        let axis = element.style.layout.axis;

        let mut cursor = element.rect.position;
        cursor.x += element.style.padding.left;
        cursor.y += element.style.padding.top;

        (axis, cursor, element.children.clone())
    };

    for child_key in children {
        {
            let child = &mut tree[child_key];

            match child.style.position {
                Position::Pinned {
                    position,
                    parent_relative: false,
                } => child.rect.position = position.map(|lp| lp.as_float()),
                Position::Pinned {
                    position,
                    parent_relative: true,
                } => {
                    child.rect.position.x += position.x.as_float() + child.rect.position.x;
                    child.rect.position.y += position.y.as_float() + child.rect.position.y;
                }
                Position::Dynamic => {
                    child.rect.position.x += cursor.x + child.style.margin.left;
                    child.rect.position.y += cursor.y + child.style.margin.top;

                    match axis {
                        LayoutAxis::Horizontal => {
                            cursor.x += child.rect.width() + child.style.margin.get_horizontal()
                        }
                        LayoutAxis::Vertical => {
                            cursor.y += child.rect.height() + child.style.margin.get_vertical()
                        }
                    }
                }
            }
        }

        resolve_children_position(child_key, tree);
    }
}
