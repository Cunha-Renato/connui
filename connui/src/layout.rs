use crate::{renderer::Renderer, types::*};

pub(crate) fn layout<T, R: Renderer>(node: &mut Node<T, R>) {
    first_pass(node, &Bounds::default());
    second_pass(node);
    resolve_position(node);
}

fn first_pass<T, R: Renderer>(node: &mut Node<T, R>, bounds: &Bounds) {
    let axis = node.widget.get_layout().axis;
    let size_op = node.widget.get_size().validate();

    // Ensures correct bounds.
    let bounds = bounds.width(size_op.width).height(size_op.height);

    let (main_op, cross_op) = axis.pack(&size_op);
    // Initiates sizes based on bounds.min.
    let (mut main_size, mut cross_size) = axis.pack(&bounds.min);

    for child in &mut node.children {
        first_pass(child, &bounds);

        // For Fit & Fill ops.
        let (main_child_size, cross_child_size) = axis.pack(&child.rect.size);

        if !main_op.is_absolute() {
            main_size += main_child_size;
        }

        if !cross_op.is_absolute() {
            cross_size = cross_size.max(cross_child_size);
        }
    }

    let (width, height) = axis.unpack(main_size, cross_size);
    node.rect.size = bounds.clamp(Size::new(width, height));
    node.bounds = bounds;
}

/// This pass is top - down serves to resize children that can take more space or are overflowing (dynamic).
fn second_pass<T, R: Renderer>(node: &mut Node<T, R>) {
    // Early return.
    if node.children.is_empty() {
        return;
    }

    let axis = node.widget.get_layout().axis;

    let (main_size, cross_size) = axis.pack(&node.rect.size);

    let budget = node.children.iter_mut().fold(main_size, |total, child| {
        let (child_main, child_cross) = axis.pack_mut(&mut child.rect.size);

        // Resizing Fill children.
        if let SizeOp::Fill { .. } = axis.cross(&child.widget.get_size()) {
            *child_cross = (*child_cross).max(cross_size);
        }

        total - *child_main
    });

    // Node has space to give.
    if budget > LPixel::new(0.0) {
        resolve_grow(node, budget, axis);
    } else if budget < LPixel::new(0.0) {
        resolve_shrink(node, budget, axis);
    }

    for child in &mut node.children {
        child.rect.size = child.bounds.clamp(child.rect.size);

        second_pass(child);
    }
}

// Always main axis.
fn resolve_grow<T, R: Renderer>(node: &mut Node<T, R>, mut budget: LPixel<f32>, axis: LayoutAxis) {
    let mut growable = node
        .children
        .iter_mut()
        .filter_map(|child| {
            let main_op = axis.main(&child.widget.get_size());
            let main_size = axis.main(&child.rect.size);
            let main_max_bounds = axis.main(&child.bounds.max);

            (matches!(main_op, SizeOp::Fill { .. }) && main_size < main_max_bounds).then_some(child)
        })
        .collect::<Vec<_>>();

    let epsilon = LPixel::new(f32::EPSILON);
    while budget > LPixel::new(0.0) && !growable.is_empty() {
        let portion = budget / LPixel::new(growable.len() as f32);

        growable.retain_mut(|child| {
            let main_size = axis.main_mut(&mut child.rect.size);
            let main_max = axis.main(&child.bounds.max);

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

fn resolve_shrink<T, R: Renderer>(node: &mut Node<T, R>, deficit: LPixel<f32>, axis: LayoutAxis) {
    todo!()
}

// Top - down positioning.
fn resolve_position<T, R: Renderer>(node: &mut Node<T, R>) {
    let axis = node.widget.get_layout().axis;

    let mut main_cursor = axis.main(&node.rect.position);

    for child in &mut node.children {
        let main_size = axis.main(&child.rect.size);
        let main_pos = axis.main_mut(&mut child.rect.position);

        *main_pos = main_cursor;
        main_cursor += main_size;

        resolve_position(child);
    }
}
