# Layout Engine Specification — Sizing & Positioning

## 1. Scope

This engine positions and sizes rectangles only. No text metrics, no painting, no
z-order. Input is a tree of `Node`s plus one external "viewport" size fed to the
root. Output is a resolved `(x, y, width, height)` per node, in logical pixels.

Everything below is written to make the system **deterministic**: given the same
tree and the same viewport size, layout must produce the exact same numbers every
time, with no dependence on traversal order, hashmap iteration, or floating point
drift.

---

## 2. Terminology

The algorithm passes several distinct quantities around per node, per axis, and
it's easy to blur them together in prose. This table is the single source of
truth for what each one means — every later section uses these names exactly,
nothing else.

| Term                              | Meaning                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Computed in                                                                | Feeds into                                                                                         |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| **Intrinsic Size**                | The size a node "wants," independent of any parent: content-driven hug for `Fit`, `clamp(max(content hug, initial), min, max)` for `Fill`, `value` for `Absolute`. Context-free — never asks what the parent can offer.                                                                                                                                                                                                                                                                     | §5.1, Pass 1 (bottom-up)                                                   | Starting point for every node's Pass 2 resolution                                                  |
| **Offered Size**                  | The space a parent hands a child along one axis, *before* the child's own spec is applied. The root's Offered Size is the external viewport size.                                                                                                                                                                                                                                                                                                                                           | §5.2, Pass 2 (top-down), once per parent, before recursing into each child | Combined with Intrinsic Size to produce Resolved Size                                              |
| **Resolved Size**                 | The final, authoritative size of a node on one axis, after clamping/growing/shrinking against its Offered Size.                                                                                                                                                                                                                                                                                                                                                                             | §5.2, Pass 2                                                               | Becomes the *Offered Size* for that node's own children; the number layout consumers actually read |
| **Main-Axis Budget**              | The slice of a container's Resolved Size (on its own layout axis) actually available to hand out among its Dynamic children — Resolved Size minus gaps (and padding, once §10.3 is added).                                                                                                                                                                                                                                                                                                  | §5.3, once per container, before the distribution loop                     | Input to Free Space / Deficit                                                                      |
| **Free Space** / **Deficit**      | Main-Axis Budget minus the sum of the Dynamic children's Intrinsic Sizes. Positive → Free Space (room to grow into). Negative → Deficit (amount that must be shrunk out).                                                                                                                                                                                                                                                                                                                   | §5.3, once per container                                                   | Divided into Round Shares across the Growth/Shrink Pool                                            |
| **Growth Pool** / **Shrink Pool** | The subset of Dynamic siblings still eligible to receive or give up space *in the current round* — i.e. haven't yet hit `max` (growing) or `min` (shrinking).                                                                                                                                                                                                                                                                                                                               | §5.3, recomputed every round                                               | Determines who receives the next Round Share                                                       |
| **Round Share**                   | The equal amount of Free Space or Deficit each pool member receives or gives up in one round of the loop.                                                                                                                                                                                                                                                                                                                                                                                   | §5.3                                                                       | Applied to each pool member's Resolved Size; pool is then re-evaluated                             |
| **Cross Offer**                   | The Offered Size on whichever axis is *not* the parent's layout axis. Always equals the parent's full Resolved Size on that axis — never split among siblings (§4.7).                                                                                                                                                                                                                                                                                                                       | §5.2                                                                       | Directly clamps `Fill` / `Fit`-with-`shrink` resolution on the cross axis                          |
| **Base Size**                     | For `Fill` specifically: its Intrinsic Size at the moment the distribution algorithm starts — `clamp(max(content hug, initial), min, max)`. `initial` acts as a *floor*, not the sole input: if `Fill`'s own children need more room than `initial`, their hug wins; `initial` only matters when it exceeds what the children need. Numerically identical to Intrinsic Size — this name is used only when talking about "the number the loop starts adjusting from," to keep §5.3 readable. | Same as Intrinsic Size                                                     | Starting value inside the distribution loop                                                        |

One discipline this buys you: **"size" is never used alone anywhere else in this
document.** If a sentence needs a bare "size," it's a sign the sentence is
ambiguous about which of the five above it means — say the real name instead.

---

## 3. Core Types

```rust
struct LPixel;
struct Point;
struct Size;

pub enum SizeOp {
    Fit    { min: LPixel, max: LPixel, shrink: bool },
    Fill   { min: LPixel, max: LPixel, initial: LPixel, shrink: bool },
    Absolute(LPixel),
}

pub enum Position {
    Dynamic,
    Pinned { point: Point, parent_relative: bool },
}

pub enum LayoutAxis { Horizontal, Vertical }

pub struct Node {
    // --- input: set by whoever builds the tree, never written by layout ---
    width:       SizeOp,
    height:      SizeOp,
    position:    Position,
    layout_axis: LayoutAxis,       // arrangement axis for THIS node's own children
    children:    Vec<Node>,

    // --- output: populated BY layout, empty/undefined before it runs ---
    intrinsic:     Size,           // written once by measure_intrinsic_tree (Pass 1, §5.1)
    resolved:      Size,           // written once by resolve (Pass 2, §5.2)
    origin:        Point,          // parent-relative position, written by resolve
    global_origin: Point,          // root-relative position, written by resolve
}
```

The input/output split matters for the pseudocode in §5.4: every `node.intrinsic[axis]` read there is only ever valid *after* `measure_intrinsic_tree` has visited that node, and every `node.resolved[axis]` read is only valid after `resolve` has visited it. Reading either field before its writing pass has reached that node is a bug, not a matter of style — there is no sensible default to fall back to.

`width` and `height` are **independent** `SizeOp`s — a node can be `Fit` on one
axis and `Fill` on the other. Whether a given axis behaves as "main" or "cross"
depends entirely on the *parent's* `layout_axis`, not on the node itself.

---

## 4. Semantics, precisely

### 4.1 `Absolute(value)`

- Resolved Size = `value`, unconditionally. Never reads Offered Size, never
  reads `shrink` (there is no shrink flag — it doesn't participate).
- **Can overflow its parent.** This is intentional, not a bug: the engine does
  not clip. If you don't want overflow, don't use `Absolute` inside a
  space-constrained parent.

### 4.2 `Fit { min, max, shrink }`

- Defines an Intrinsic Size: for a leaf, Intrinsic Size = `min` (there's
  nothing to hug — `Fit` was designed for containers with children). For a
  container, Intrinsic Size = the extent needed to hug its children (§5.1)
  clamped to `[min, max]`.
- **Never grows** past its Intrinsic Size even if Offered Size is larger —
  that's `Fill`'s job. `Fit` does not mean "as big as possible within
  bounds," it means "as big as content requires, bounded."
- `shrink`: if Offered Size is smaller than this node's Intrinsic Size,
  `shrink = true` permits the engine to compress its Resolved Size down —
  **never below its own `min`**. `shrink = false` means Resolved Size stays
  at the Intrinsic Size and may overflow the offer instead.
- **Validation:** `max >= min`, clamped at construction time.

### 4.3 `Fill { min, max, initial, shrink }`

- Same bounding/shrink semantics as `Fit`, plus: if Free Space exists in the
  parent's Main-Axis Budget, `Fill` nodes are the ones that absorb it,
  growing their Resolved Size from their Base Size up to `max`.
- **Base Size is `clamp(max(content hug, initial), min, max)`** — the same
  children-hugging computation `Fit` uses (§5.1), floored by `initial`.
  `initial` is not itself the Base Size; it only wins when it's *larger*
  than what the node's own children need. A `Fill` container therefore
  never starts out smaller than its content requires, only ever at least
  as big as `initial`.
- `initial` is a flex-basis-style floor, in logical pixels. Neither it nor
  the resulting Base Size is the same thing as Resolved Size — Base Size is
  only the starting point before growth or shrink is applied (§5.3).
- **Growth only happens along the parent's `layout_axis`.** See §5.3 and
  §8.4 for why, and for cross-axis behavior.
- **Validation:** `max >= min`, and `initial` is clamped into `[min, max]`
  at construction time (this bounds `initial` itself — the final Base Size
  is clamped again after factoring in the content hug, since the hug can
  exceed `max` on its own).

### 4.4 `Dynamic`

- Node is placed by the engine according to the parent's `layout_axis`, in
  child order, packed with no overlap (subject to §8.2's alignment gap).
- Participates in the parent's Main-Axis Budget (i.e. its Intrinsic Size
  counts toward what other Dynamic siblings can get).

### 4.5 `Pinned { point, parent_relative }`

- **Removed from flow.** A Pinned child does not consume Main-Axis Budget
  and is invisible to its Dynamic siblings' layout math — exactly like
  `position: absolute` in CSS.
- Still goes through normal size resolution (`Fit`/`Fill`/`Absolute` all
  still apply) — Pinned only overrides *position*, not sizing. For a Pinned
  node, its Offered Size on each axis is the parent's full Resolved Size on
  that axis (as if it were the sole child), not a shared budget.
- `point` is the **top-left corner** of the node's Resolved Size bounding box.
- `parent_relative: true` → `point` is offset from the parent's resolved
  origin (post-layout).
- `parent_relative: false` → `point` is in **root/global coordinate space**.

### 4.6 `LayoutAxis`

- A property of a container, governing how *its own* Dynamic children flow.
  `Horizontal` = children laid left-to-right, main axis = width, cross axis =
  height. `Vertical` = top-to-bottom, main axis = height, cross axis = width.

### 4.7 Cross axis is never shared

**A container never has more than one child's worth of claim on the cross
axis at a time.** Unlike the main axis — where every Dynamic sibling draws
from one shared Main-Axis Budget (§5.3) — each child, independently and
without regard for its siblings, receives the parent's full Resolved Size as
its Cross Offer. There is no competition, no distribution loop, and no
"leftover after other children" concept on the cross axis at all.

Practically, this means:

- `Fill` on the cross axis is a **plain clamp**: Resolved Size =
  `clamp(Cross Offer, min, max)`, full stop. No iteration, no weighting, no
  interaction with sibling cross sizes.
- `Fit`'s `shrink` on the cross axis is likewise a **plain clamp**, not the
  iterative shrink loop from §5.3: if Intrinsic Size > Cross Offer and
  `shrink == true`, Resolved Size = `clamp(Cross Offer, min, Intrinsic Size)`.
  If `shrink == false`, Resolved Size stays at Intrinsic Size and may
  overflow.
- The §5.3 grow/shrink distribution algorithm applies **only** to the main
  axis. The cross axis never needs it, because there's nothing to
  distribute among — every child gets the same full offer.

---

## 5. The Resolution Algorithm

Two passes. This ordering is what guarantees the whole thing terminates
without cycles — see §9 for why.

### 5.1 Pass 1 — Intrinsic Measurement (bottom-up, post-order)

For every node, compute its **Intrinsic Size** per axis, context-free (no
knowledge of Offered Size):

| Spec          | Intrinsic Size on axis A                                                                                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Absolute(v)` | `v`                                                                                                                                                                      |
| `Fit`         | container: hug children (main axis = sum of children's Intrinsic Sizes + gaps; cross axis = max of children's Intrinsic Sizes), then clamp to `[min, max]`. Leaf: `min`. |
| `Fill`        | same hug computation as `Fit`, then `clamp(max(hug, initial), min, max)` — i.e. its Base Size. Leaf (no children, hug = 0): reduces to `clamp(initial, min, max)`.       |

Because this pass never asks "what's my Offered Size," there is no circular
dependency between a `Fit` parent and its children — the parent's Intrinsic
Size is a pure function of the children's already-computed Intrinsic Sizes.

### 5.2 Pass 2 — Constraint Resolution & Positioning (top-down, pre-order)

Each node receives an **Offered Size** per axis from its parent (root
receives the external viewport size on both axes). Then:

1. **Resolve size**, combining Intrinsic Size (from Pass 1) with Offered
   Size:
   - `Absolute`: ignore Offered Size, Resolved Size = `value`.
   - `Fit`: Resolved Size = Intrinsic Size, unless Intrinsic Size exceeds
     Offered Size *and* `shrink == true`, in which case Resolved Size =
     `clamp(Offered Size, min, Intrinsic Size)`.
   - `Fill` on the **cross** axis (Cross Offer applies — §4.7), or when the
     node is Pinned (solo offer, §4.5): Resolved Size =
     `clamp(Offered Size, min, max)`. If the parent itself is `Fit` on this
     axis (no real Cross Offer beyond what the children already determine),
     Resolved Size = Base Size — it cannot grow (§8.4).
   - `Fill` on the **main** axis: Resolved Size comes from the distribution
     algorithm, §5.3, run once per container over all its Dynamic children
     together — never per child in isolation.
2. **Position** the node: `Dynamic` children are placed sequentially along
   the main axis (packed from the start, see §8.2 on alignment) and at the
   cross-axis start of the parent's content box, unless alignment (§8.2) is
   added. `Pinned` children are placed per §4.5 as a second, independent
   sub-pass over the same child list — pinned nodes don't shift Dynamic
   siblings and vice versa.
3. Recurse into children, handing each its Resolved Size as their Offered
   Size for the next level down.

### 5.3 Main-axis distribution algorithm (grow/shrink)

Run once per container, over its Dynamic children only:

```text
intrinsic[i] = Intrinsic Size of child i (Pass 1)         // = Base Size for Fill
budget       = Main-Axis Budget = container's Resolved Size on this axis,
               minus gaps (and padding, once added)
total        = sum(intrinsic) + gaps
free         = budget - total          // Free Space if positive, Deficit if negative

if free > 0:
    pool = Fill children with resolved < max          // Growth Pool
    while free > 0 and pool is not empty:
        share = free / len(pool)                      // Round Share
        for child in pool:
            room  = child.max - child.resolved
            grant = min(share, room)
            child.resolved += grant
            free -= grant
            if child.resolved == child.max: remove child from pool
        # remainder a maxed-out child couldn't absorb rolls into the next
        # round, re-divided among whoever's still in the pool

elif free < 0:
    deficit = -free
    pool    = children (Fit or Fill) with shrink == true and resolved > min  // Shrink Pool
    while deficit > 0 and pool is not empty:
        share = deficit / len(pool)                   // Round Share
        for child in pool:
            room = child.resolved - child.min
            cut  = min(share, room)
            child.resolved -= cut
            deficit -= cut
            if child.resolved == child.min: remove child from pool
    # if deficit remains here, every shrinkable child is pinned at its min
    # and there still isn't enough room — allow overflow, do not crash,
    # do not go below any child's min.
```

This is the standard flexbox grow/shrink algorithm. It's iterative-but-bounded
(terminates in at most `n` rounds per container) and fully deterministic given
a fixed child order.

**This is the definitive rule, not a default:** Free Space and Deficit are
each split into an **equal Round Share** across the current Growth/Shrink
Pool. `initial` (i.e. Base Size) is only ever a starting point — it has no
role as a weight. Concretely, at each round:

- every pool member receives the *same* Round Share, not a share proportional
  to its Base Size or its current Resolved Size;
- the moment a child hits its `max` (growing) or its `min` (shrinking), it
  drops out of the pool for the rest of this call;
- whatever a child *couldn't* absorb (Round Share exceeded its remaining
  room) rolls back into Free Space / Deficit and is re-divided equally among
  whoever's left in the pool, next round;
- this repeats until the budget is exhausted or the pool is empty.

### 5.4 Pseudocode: the two passes end-to-end

Everything above composes into three functions. Two axis-resolution
primitives — one for "this node gets a single concrete offer and nothing
else competes for it" (root, Pinned children, and any Dynamic child's cross
axis), one for "these siblings share a budget" (§5.3, main axis only) — plus
one orchestrating `resolve` that walks the tree.

```text
// ---------- Pass 1 : bottom-up, context-free ----------
// Two functions, deliberately separate:
//   - measure_intrinsic_tree walks EVERY node in the tree exactly once,
//     children before parent, and STORES the result on the node.
//   - compute_intrinsic is a pure formula for one node's one axis; it
//     never recurses — it only reads child.intrinsic[axis], which is
//     guaranteed already-populated because of the traversal order above.
// Splitting these is what makes Pass 1 O(nodes): each node's formula
// runs exactly once, not once per read.

fn measure_intrinsic_tree(node: Node):
    for child in node.children:                 // post-order: children first,
        measure_intrinsic_tree(child)            // regardless of this node's own spec
    node.intrinsic.width  = compute_intrinsic(node, Width)
    node.intrinsic.height = compute_intrinsic(node, Height)

fn compute_intrinsic(node: Node, axis: Axis) -> LPixel:
    spec = node.size_op(axis)
    flow_children = [c for c in node.children if c.position != Pinned]

    // Shared by Fit and Fill: how much this node's own children need.
    // 0 when there are no flow children (leaves, or all-Pinned containers).
    hug = 0
    if not flow_children.is_empty():
        if axis == node.layout_axis:                            // main axis: hug = sum
            hug = total(c.intrinsic[axis] for c in flow_children)
            hug += gap * (len(flow_children) - 1)
        else:                                                     // cross axis: hug = max
            hug = max(c.intrinsic[axis] for c in flow_children)

    match spec:
        Absolute(v) ->
            return v

        Fit { min, max, .. } ->
            return clamp(hug, min, max)
            // when hug == 0 (leaf), this reduces to `min` — the leaf floor,
            // no special case needed: clamp(0, min, max) == min whenever
            // 0 <= min <= max, which validation (§7) already guarantees.

        Fill { min, max, initial, .. } ->
            return clamp(max(hug, initial), min, max)             // Base Size
            // `initial` is a floor, not the whole story: a Fill container
            // never starts smaller than its own children need (§4.3).
            // When hug == 0 (leaf), this reduces to clamp(initial, min, max),
            // matching the simple leaf-Fill case.


// ---------- Pass 2, primitive A : solo-offer resolution ----------
// Used for: root (both axes), Pinned children (both axes),
// and any Dynamic child's CROSS axis (never its main axis).

fn resolve_solo_axis(node: Node, axis: Axis, offered: LPixel) -> LPixel:
    intrinsic = node.intrinsic[axis]                            // plain lookup — Pass 1
                                                                  // already stored this,
                                                                  // nothing computed here
    spec = node.size_op(axis)

    match spec:
        Absolute(v) ->
            return v

        Fit { min, max, shrink } ->
            if intrinsic <= offered or not shrink:
                return intrinsic
            return clamp(offered, min, intrinsic)                // shrink toward offer

        Fill { min, max, .. } ->
            return clamp(offered, min, max)                      // plain stretch/clamp
            // Note: no special case is needed for "parent is Fit on this
            // axis" (§4.7/§8.4) — `offered` there just already equals
            // whatever the Fit parent hugged to, which naturally reduces
            // to this child's Base Size only when this child was the one
            // determining that hug. The formula is uniform either way.


// ---------- Pass 2, primitive B : shared-budget resolution ----------
// This IS the §5.3 algorithm. Sets resolved main-axis size for every
// Dynamic child of one container in one call.

fn distribute_main_axis(dynamic_children: [Node], main_axis: Axis, budget: LPixel) -> [LPixel]:
    resolved = [c.intrinsic[main_axis] for c in dynamic_children]   // plain lookup, not a call
    // ... grow-into-Free-Space / shrink-out-of-Deficit loop, exactly as in §5.3 ...
    return resolved


// ---------- Pass 2, orchestrator : top-down, produces positions too ----------

fn resolve(node: Node, offered: Size, origin: Point, global_origin: Point):
    node.resolved.width  = resolve_solo_axis(node, Width,  offered.width)   // overwritten
    node.resolved.height = resolve_solo_axis(node, Height, offered.height)  // below if
                                                                             // this call came
                                                                             // from a Dynamic
                                                                             // main-axis slot
    node.origin        = origin
    node.global_origin = global_origin

    main_axis  = node.layout_axis
    cross_axis = opposite(main_axis)

    dynamic = [c for c in node.children if c.position == Dynamic]
    pinned  = [c for c in node.children if c.position == Pinned]

    // --- shared budget across this container's Dynamic children ---
    budget = node.resolved[main_axis] - gap * max(len(dynamic) - 1, 0)      // Main-Axis Budget
    main_sizes = distribute_main_axis(dynamic, main_axis, budget)

    // --- place + recurse Dynamic children ---
    cursor = 0
    for i, child in enumerate(dynamic):
        cross_offer = node.resolved[cross_axis]                            // Cross Offer, §4.7
        child_offered      = size_on(main_axis, main_sizes[i], cross_axis, cross_offer)
        child_origin        = origin        + along(main_axis, cursor)
        child_global_origin = global_origin + along(main_axis, cursor)
        resolve(child, child_offered, child_origin, child_global_origin)
        // child's main-axis Resolved Size was fixed by distribute_main_axis
        // above; resolve_solo_axis only fills in its CROSS axis this time.
        cursor += main_sizes[i] + gap

    // --- place + recurse Pinned children (independent sub-pass, §4.5) ---
    for child in pinned:
        child_offered = node.resolved                                      // solo, full box
        child_origin = child.parent_relative
            ? origin + child.point
            : child.point                                                  // already global
        resolve(child, child_offered, child_origin, /* recompute global */ child_origin)


// ---------- Entry point ----------

fn layout(root: Node, viewport: Size) -> void:
    measure_intrinsic_tree(root)                                            // one full
                                                                              // post-order pass,
                                                                              // populates every
                                                                              // node's .intrinsic
    resolve(root, offered = viewport, origin = (0, 0), global_origin = (0, 0))
```

Two things worth calling out about this composition:

- `resolve_solo_axis` is deliberately the *only* place `Fit`/`Fill`/`Absolute`
  semantics are implemented. `distribute_main_axis` doesn't reimplement them —
  it just reads the same stored `.intrinsic` value and then nudges the result
  within `[min, max]` per §5.3. There is exactly one place each spec's rules
  live, not two.
- Writing this out end-to-end also shows that the "falls back to Base Size
  when the parent is `Fit` on this axis" language in §4.3/§6/§8.4 isn't a
  separate rule to implement — it's just what `clamp(offered, min, max)`
  produces automatically in that situation. Nothing to special-case.

---

## 6. Interaction Matrix

| Parent axis role   | Node spec  | Behavior                                                                                                                                                                                                |
| ------------------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Main axis, Dynamic | `Absolute` | Fixed Resolved Size; contributes to Main-Axis Budget as-is; can force overflow.                                                                                                                         |
| Main axis, Dynamic | `Fit`      | Resolved Size = Intrinsic Size; shrinks only if `shrink=true` and the budget is tight.                                                                                                                  |
| Main axis, Dynamic | `Fill`     | Resolved Size starts at Base Size (content hug floored by `initial`, §4.3); grows into Free Space; shrinks under Deficit if `shrink=true`.                                                              |
| Cross axis         | `Absolute` | Fixed; ignores Cross Offer entirely.                                                                                                                                                                    |
| Cross axis         | `Fit`      | Hugs own children (Intrinsic Size); shrinks only under `shrink=true` if Cross Offer is smaller.                                                                                                         |
| Cross axis         | `Fill`     | Resolved Size = `clamp(Cross Offer, min, max)` — **unless** the parent is itself `Fit` on that axis, in which case there's no real Cross Offer to stretch to and Resolved Size falls back to Base Size. |
| Any axis, Pinned   | anything   | Sizing spec still applies, but Offered Size = parent's full content box (solo), not a shared budget. Position is overridden per §4.5 and does not affect Dynamic siblings' flow math.                   |

---

## 7. Validation & Normalization Rules

Enforce these at construction time (smart constructors returning `Result`, or
just clamp — pick one and be consistent):

1. `Fit`/`Fill`: `min >= 0` and `max >= min`.
2. `Fill`: `initial` clamped (or validated) into `[min, max]`.
3. `Absolute(v)`: `v >= 0`.
4. Root node must be given a concrete external size for both axes before
   layout starts — this is fed into Pass 2 as its Offered Size, as if an
   invisible ancestor had `Absolute`-sized the root to the viewport. State
   this explicitly somewhere, because otherwise "what does the root's own
   `SizeOp` even mean" is a real question people will ask.

---

## 8. Resolved Ambiguities & Open Items

### 8.1 What exactly is `initial`? — Resolved

`initial` reads, in the original prose, like it could be doing two jobs:
starting size and growth weight. The growth-weight question is settled:
growth and shrink are both distributed as an equal **Round Share** across
the eligible pool, per round, per §5.3 — never proportionally to `initial`
or to a child's current Resolved Size.

Separately, `initial`'s exact relationship to Base Size was refined in §4.3:
it's a **floor**, not the Base Size outright — `Fill`'s Base Size is
`clamp(max(content hug, initial), min, max)`, so `initial` only wins when
it's bigger than what the node's own children need. Either way, a larger
`initial` only ever changes the starting point of the distribution loop —
it never buys a bigger share of Free Space once that loop runs.

### 8.2 No alignment concept

Nothing in the current spec says where a `Dynamic` child sits on the cross
axis if its Resolved Size is smaller than its Cross Offer, or where Free
Space goes if a container has no `Fill` children to absorb it. The only
implicit answer right now is "packed at the start, cross-axis start-aligned"
— a valid deterministic default, but it means centering anything requires
either a `Fill` spacer trick or falling back to `Pinned`. Suggested addition,
per container:

```rust
pub enum MainAlign  { Start, Center, End, SpaceBetween, SpaceAround }
pub enum CrossAlign { Start, Center, End, Stretch }
```

defaulting to `Start`/`Start` so nothing changes for people who don't set it.

### 8.3 No `gap` or `padding`

`Fit`'s "hug children" definition (§5.1) already assumes a gap value between
children, and Main-Axis Budget (§2) already assumes padding will eventually
be subtracted from Resolved Size. Neither is in `Node` yet. Padding is a
small, mechanical addition (subtract from Offered Size before it becomes
Main-Axis Budget; add back into a `Fit` container's own Intrinsic Size), but
it needs to be in the struct for the spec to be complete.

### 8.4 `Fill` inside a `Fit` parent cannot grow

Not a bug, but a sharp edge worth documenting loudly: a `Fit`-sized container
has no Free Space by definition (it shrinks to exactly match its children's
Intrinsic Sizes), so a `Fill` child inside it never has Free Space to grow
into — its Resolved Size just stays at its Base Size. Since Base Size is
now `clamp(max(content hug, initial), min, max)` (§4.3), this makes a `Fill`
child behave *almost* identically to a `Fit` sibling with the same
`min`/`max`/`shrink` — the only difference is the `initial` floor: if
`initial` exceeds what the `Fill` node's own children need, it still forces
extra room even though there's nothing to grow into. Users will file the
no-growth behavior itself as a bug unless it's called out in your public
docs, not just this internal spec.

### 8.5 Rounding / sub-pixel determinism

§5.3's `share = free / len(pool)` will produce fractional pixels in the
general case. For byte-for-byte determinism you need one fixed rounding rule
(e.g. truncate everything, then dump the accumulated remainder onto the last
pool member in iteration order) so that `sum(children.resolved) + gaps`
exactly equals the Main-Axis Budget every time, with no ±1px drift depending
on float rounding. Pick the rule once, put it in code as a single helper,
don't let it happen ad hoc at each call site.

---

## 9. Why This Terminates (no cycles)

The two-pass split is the whole trick:

- **Pass 1** is a pure bottom-up fold: a node's Intrinsic Size is a function
  of its children's Intrinsic Sizes only — never of any ancestor's Offered
  Size. A `Fit` parent's Intrinsic Size is therefore never "waiting on" a
  number that itself depends on the parent.
- **Pass 2** is a pure top-down fold: a node's Resolved Size is a function of
  its own Intrinsic Size (already known from Pass 1) and its Offered Size
  (already known, because the parent was resolved first). No node's Pass 2
  computation ever depends on a sibling's or descendant's Pass 2 result
  except through the single, bounded, iterative loop in §5.3 (which touches
  only direct siblings, not the whole tree, and terminates in at most `n`
  rounds).

So: no matter how deep or tangled the tree, layout is `O(nodes)` for Pass 1
plus `O(nodes)` for Pass 2 (with a small constant-factor loop per container
for grow/shrink), and always produces the same output for the same input.

---

## 10. Summary of Recommended Additions

- `gap: LPixel` and optionally `padding: Edges` per container (§8.3).
- `MainAlign` / `CrossAlign` per container, defaulting to `Start` (§8.2).
- A single documented rounding rule for fractional pixel distribution (§8.5)
  — this is the one thing left to pin down before implementing, since §5.3/
  §8.1 (equal Round Share growth and shrink; `initial` as a floor on Base
  Size, not the whole of it) are settled.
- An explicit statement of root bootstrapping (§7, rule 4).