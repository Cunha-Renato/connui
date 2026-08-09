use crate::{renderer::Renderer, types::*};

pub(crate) fn layout<T, R: Renderer>(node: &mut Node<T, R>, viewport: LSize<f32>) {
    resolve_intrinsic(node);
    resolve_main_cross(node, viewport, LayoutAxis::Horizontal);
}

/// Computes the intrinsic size of the node. Unchanged from the previous
/// pass — no bugs found here.
fn resolve_intrinsic<T, R: Renderer>(node: &mut Node<T, R>) {
    for child in &mut node.children {
        resolve_intrinsic(child);
    }

    let axis = node.widget.get_layout().axis;

    let (main_op, cross_op) = axis.pack(&node.widget.get_size().validate());
    let (main_padding, cross_padding) = {
        let padding = node.widget.get_padding();

        axis.pack(&(
            padding.get_horizontal().as_float(),
            padding.get_vertical().as_float(),
        ))
    };

    let mut main_size = main_padding;
    let mut cross_size = cross_padding;

    if !main_op.is_absolute() {
        main_size += node
            .children
            .iter()
            .filter(|c| !c.widget.get_position().is_pinned())
            .fold(LPixel::new(0.0), |total, child| {
                let child_intrinsic = axis.main(&child.intrinsic);
                let child_margin = {
                    let margin = child.widget.get_margin();

                    axis.main(&(
                        margin.get_horizontal().as_float(),
                        margin.get_vertical().as_float(),
                    ))
                };

                child_intrinsic + child_margin + total
            });
    }

    if !cross_op.is_absolute() {
        let children_max = node
            .children
            .iter()
            .filter(|c| !c.widget.get_position().is_pinned())
            .fold(LPixel::new(0.0), |total, child| {
                let child_intrinsic = axis.cross(&child.intrinsic);
                let child_margin = axis.cross(&{
                    let margin = child.widget.get_margin();
                    (
                        margin.get_horizontal().as_float(),
                        margin.get_vertical().as_float(),
                    )
                });

                total.max(child_intrinsic + child_margin)
            });

        cross_size += children_max;
    }

    match main_op {
        SizeOp::Absolute(val) => main_size = val.as_float(),
        SizeOp::Fit { min, max, .. } => main_size = main_size.clamp(min.as_float(), max.as_float()),

        SizeOp::Fill {
            min, max, initial, ..
        } => {
            main_size = main_size
                .max(initial.as_float())
                .clamp(min.as_float(), max.as_float())
        }
    }

    match cross_op {
        SizeOp::Absolute(val) => cross_size = val.as_float(),
        SizeOp::Fit { min, max, .. } => {
            cross_size = cross_size.clamp(min.as_float(), max.as_float())
        }

        SizeOp::Fill {
            min, max, initial, ..
        } => {
            cross_size = cross_size
                .max(initial.as_float())
                .clamp(min.as_float(), max.as_float())
        }
    }

    let (width, height) = axis.unpack(main_size, cross_size);

    node.intrinsic = Size::new(width, height);
}

/// Computes the cross axis size for the node, if it is Fill, or Fit shrink.
///
/// **offered** is the total ammount that the **node** can take.
///
/// **returns** the resolved size for the **node** — callers MUST store this
/// into `node.resolved.size` themselves (via `axis.cross_mut`); this
/// function does not write to the node.
fn resolve_cross_axis<T, R: Renderer>(
    node: &mut Node<T, R>,
    offered: LPixel<f32>,
    axis: LayoutAxis,
) {
    let intrinsic = axis.cross(&node.intrinsic);

    *axis.cross_mut(&mut node.resolved.size) = match axis.cross(&node.widget.get_size()) {
        // Doesn't resize.
        SizeOp::Absolute(val) => val.as_float(),

        // Fit: keep intrinsic size normally; only shrink toward the offer
        // when it doesn't fit AND shrink is explicitly allowed. Never
        // grows past intrinsic (§4.2) — no `Fill`-style stretch here.
        SizeOp::Fit {
            min, max, shrink, ..
        } => {
            if shrink && intrinsic > offered {
                offered.clamp(min.as_float(), max.as_float())
            } else {
                intrinsic
            }
        }

        // Fill: an UNCONDITIONAL plain clamp against the offer (§4.7).
        // `shrink` plays no role in Fill's cross-axis rule, unlike Fit's.
        // This must never fall through to a raw, unclamped `offered` — a
        // Fill node could otherwise end up outside its own [min, max].
        SizeOp::Fill { min, max, .. } => offered.clamp(min.as_float(), max.as_float()),
    };
}

fn resolve_main_axis<T, R: Renderer>(
    children: &mut [Node<T, R>],
    budget: LPixel<f32>,
    axis: LayoutAxis,
) {
    // Seed every child's resolved main-axis size at its baseline —
    // Intrinsic Size for Fit/Fill (= Base Size for Fill, per §5.1), and
    // `value` for Absolute (its intrinsic already equals `value`, so no
    // special case is needed here). This MUST run before the grow/shrink
    // pools are built below, since they read `child.resolved.size` as
    // their starting point — and it must run on every call, so repeated
    // layout() passes stay idempotent instead of compounding on stale data.
    let mut occupied = LPixel::new(0.0);
    for child in children.iter_mut() {
        let baseline = axis.main(&child.intrinsic);
        *axis.main_mut(&mut child.resolved.size) = baseline;

        let child_margin = child.widget.get_margin();
        let main_margin = axis.main(&(
            child_margin.get_horizontal().as_float(),
            child_margin.get_vertical().as_float(),
        ));

        occupied += baseline + main_margin;
    }

    let mut free = budget - occupied;

    // Grow
    if free > LPixel::new(f32::EPSILON) {
        let mut pool = children
            .iter_mut()
            .filter_map(|child| {
                let main_op = axis.main(&child.widget.get_size());
                let main_resolved = axis.main(&child.resolved.size);

                // Should Grow.
                if let SizeOp::Fill { max, .. } = main_op
                    && main_resolved < max.as_float()
                {
                    Some((child, max.as_float()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        while free > LPixel::new(f32::EPSILON) && !pool.is_empty() {
            let share = free / LPixel::new(pool.len() as f32);

            pool.retain_mut(|(child, max)| {
                let main_resolved_mut = axis.main_mut(&mut child.resolved.size);
                let room = (*max - *main_resolved_mut).max(0.0.into());
                let grant = share.min(room);
                free -= grant;
                *main_resolved_mut += grant;

                (*main_resolved_mut - *max).abs() > LPixel::new(f32::EPSILON)
            });
        }
    } else if free < LPixel::new(-f32::EPSILON) {
        let mut deficit = LPixel::new(-free.inner());

        let mut pool = children
            .iter_mut()
            .filter_map(|child| {
                let main_op = axis.main(&child.widget.get_size());
                let main_resolved = axis.main(&child.resolved.size);

                // Should Shrink.
                if let SizeOp::Fit {
                    min, shrink: true, ..
                }
                | SizeOp::Fill {
                    min, shrink: true, ..
                } = main_op
                    && main_resolved > min.as_float()
                {
                    Some((child, min.as_float()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        while deficit > LPixel::new(f32::EPSILON) && !pool.is_empty() {
            let share = deficit / LPixel::new(pool.len() as f32);

            pool.retain_mut(|(child, min)| {
                let main_resolved_mut = axis.main_mut(&mut child.resolved.size);
                let room = (*main_resolved_mut - *min).max(0.0.into());
                let cut = share.min(room);
                deficit -= cut;
                *main_resolved_mut -= cut;

                (*main_resolved_mut - *min).abs() > LPixel::new(f32::EPSILON)
            });
        }
    }
}

fn resolve_main_cross<T, R: Renderer>(
    node: &mut Node<T, R>,
    offered: LSize<f32>,
    axis: LayoutAxis,
) {
    // Margin belongs to `node` itself, as seen by its PARENT — so it's
    // packed in the PARENT's axis space (`axis`, the function parameter).
    let margin = node.widget.get_margin();
    let (main_margin, cross_margin) = axis.pack(&(
        margin.get_horizontal().as_float(),
        margin.get_vertical().as_float(),
    ));

    // Calculating this node's own size, against what its parent offered.
    {
        let (main_offered, cross_offered) = axis.pack(&offered);

        resolve_cross_axis(node, cross_offered - cross_margin, axis);
        resolve_main_axis(std::slice::from_mut(node), main_offered - main_margin, axis);
    }

    // Resolving children — everything below is in THIS node's own axis
    // space (`this_axis`), not the parent's (`axis`). Padding belongs
    // here: it's this node's own children budget, not how this node was
    // offered space by its parent — packing it with `axis` instead of
    // `this_axis` (the previous bug) swaps horizontal/vertical padding
    // whenever a node's own layout_axis differs from its parent's.
    {
        let this_axis = node.widget.get_layout().axis;

        let padding = node.widget.get_padding();
        let (main_padding, cross_padding) = this_axis.pack(&(
            padding.get_horizontal().as_float(),
            padding.get_vertical().as_float(),
        ));

        let (main_resolved, cross_resolved) = this_axis.pack(&node.resolved.size);
        let (main_avail, cross_avail) =
            (main_resolved - main_padding, cross_resolved - cross_padding);

        // Resolveing main & cross axis.
        for child in &mut node.children {
            let child_margin = child.widget.get_margin();
            let child_cross_margin = this_axis.cross(&(
                child_margin.get_horizontal().as_float(),
                child_margin.get_vertical().as_float(),
            ));

            resolve_cross_axis(child, cross_avail - child_cross_margin, this_axis);
        }

        resolve_main_axis(&mut node.children, main_avail, this_axis);
    }
}
