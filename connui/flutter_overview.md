# How Flutter's Layout Engine Works

A deep dive into how Flutter goes from widget declarations to pixels on screen,
with emphasis on the layout algorithm itself.

---

## 1. The Three Trees

Flutter maintains three parallel tree structures. Understanding why there are
three (not one) is the key to understanding everything downstream.

### 1.1 Widget Tree
Widgets are immutable, lightweight configuration objects. A widget describes
*what* the UI should look like at a point in time, not *how* to render it.
Every time `build()` runs, a brand new widget tree is created. Widgets are
cheap to allocate and discard by design.

### 1.2 Element Tree
Elements are the mutable, long-lived "instantiation" of a widget at a
particular position in the tree. An Element:
- Holds a reference to its current configuring Widget.
- Holds a reference to its parent Element.
- Manages its own lifecycle (created, active, inactive, defunct).
- Is responsible for reconciling ("diffing") a new Widget against the old one
  it holds, deciding whether to update, replace, or move the underlying
  RenderObject.

The Element tree persists across rebuilds; only the widget references inside
elements get swapped out. This is what makes Flutter's rebuild model cheap:
rebuilding is mostly just re-running `build()` methods and diffing, not
recreating expensive render state.

Element subtypes matter for layout structurally:
- `ComponentElement` (e.g. StatelessElement, StatefulElement) — no direct
  render object of its own; delegates to child elements.
- `RenderObjectElement` — owns exactly one RenderObject and is the bridge
  between the Element tree and the Render tree.

### 1.3 Render Object Tree (the layout/paint tree)
This is where actual layout, painting, and hit-testing happen. RenderObjects
are heavier, mutable objects that persist across frames when possible. Only
`RenderObjectElement`s create/own them — meaning the Render tree is a
*sparser* version of the Element tree (stateless composition widgets like
`Padding`'s parent wrapper widgets, `Builder`, etc. don't necessarily each
get a render object; but many layout widgets like `Padding`, `Center`,
`Row` do).

Each RenderObject knows:
- Its parent RenderObject.
- Its list of children (varies by subclass: none, one, or many).
- Its `parentData` — an opaque slot the *parent* uses to stash
  child-specific layout info (offset, flex factor, etc.) without the child
  needing to know about it.
- Whether it is a relayout boundary (see §4).
- Its constraints (last constraints handed down by parent).
- Its geometry (size, for box protocol).

The separation exists because:
- Widgets are cheap and functional — good for describing UI declaratively.
- Elements provide identity and enable efficient tree diffing/reconciliation.
- RenderObjects are expensive/stateful — good for retaining layout caches
  across frames and doing the actual geometry math.

---

## 2. From `build()` to a Render Tree: The Reconciliation Step

Before layout can happen, the tree must be reconciled:

1. Root widget's `build()` is invoked (recursively for its children) to
   produce a fresh Widget subtree.
2. For each Element, `updateChild(oldElement, newWidget)` compares the new
   widget to the widget currently configuring that element:
   - If `newWidget == null`: deactivate/unmount the element.
   - If `Widget.canUpdate(oldWidget, newWidget)` is true (same `runtimeType`
     and same `Key`): reuse the existing Element, call
     `element.update(newWidget)`. This is the cheap path — the RenderObject
     is *not* recreated, only reconfigured.
   - Otherwise: unmount the old element/render object subtree, `inflate` a
     brand-new Element (and RenderObject, if it's a `RenderObjectElement`)
     for the new widget.
3. If the element is a `RenderObjectElement` and it was *updated* (not
   recreated), it calls something like `updateRenderObject`, which pushes the
   new configuration values (e.g. new padding amount, new color) onto the
   *existing* RenderObject instance. Because the instance persists, whatever
   cached layout state it has (e.g. "am I already dirty?") also persists.
4. Any RenderObject whose configuration changed in a way that affects
   geometry calls `markNeedsLayout()` on itself. Changes that only affect
   drawing call `markNeedsPaint()` instead. This distinction is central to
   performance: a config change should invalidate the *minimum* set of work
   necessary.

This means layout doesn't operate on widgets at all — by the time layout
runs, we are 100% inside the persistent RenderObject tree.

---

## 3. The Pipeline: One Frame, In Order

Each frame, Flutter's `PipelineOwner` drives a strict ordered pipeline. This
ordering is itself part of the "engine" and matters a lot for correctness:

1. **Animate / build** — microtasks, animation ticker callbacks, and
   `State.build()` calls run, producing the new widget tree and reconciling
   it into the element/render tree as described above. This is where nodes
   get marked dirty (`markNeedsLayout`, `markNeedsPaint`,
   `markNeedsCompositingBitsUpdate`).
2. **Layout** — `PipelineOwner.flushLayout()`. Walks the *dirty layout list*
   and lays out every node that needs it, in an order guaranteeing parents
   are laid out before/around children as appropriate (details in §5-§7).
3. **Compositing bits update** — `flushCompositingBits()`. Propagates
   whether a subtree needs its own compositing `Layer` based on children
   (e.g. any child needing a layer forces ancestors to know they have a
   layer-owning descendant).
4. **Paint** — `flushPaint()`. Walks the dirty paint list and calls `paint()`
   on each dirty RenderObject, recording drawing operations into `Canvas`es
   that get attached to `Layer`s.
5. **Compositing** — the tree of `Layer`s produced during paint is composited
   by the engine (Skia/Impeller) into an actual bitmap and handed to the GPU.
6. **Semantics** — accessibility tree update, run after paint since it can
   depend on final geometry.

Layout strictly precedes paint every frame; nothing paints with stale
geometry (baring explicit opt-outs like `RepaintBoundary` interactions,
which only affect *paint* isolation, not layout ordering).

---

## 4. Relayout Boundaries — the Core Performance Idea

Naively, marking any RenderObject dirty would force laying out the entire
tree from the root, every time. Flutter avoids this with the concept of a
**relayout boundary**.

A RenderObject becomes its own relayout boundary if any of these hold:
- Its `sizedByParent` is true (its size depends *only* on the incoming
  constraints, not on children) — meaning a parent constraint change can't
  possibly ripple into a size change caused by this node's *content*.
- The constraints passed to it are "tight" (a single possible size — both
  min and max equal in each dimension), because then no matter what the
  child does internally, the parent already knows this node's size and does
  not need to re-examine it.
- It has no parent (it's the root).
- The parent does not depend on this child's size at all (parent ignores the
  child's returned size when computing its own, e.g. certain
  `Viewport`/sliver contexts, or `Overlay`-like painters).

When `markNeedsLayout()` is called on a node:
- If the node itself is a relayout boundary, it's added directly to
  `PipelineOwner._nodesNeedingLayout` and dirtying **stops propagating
  upward**. The parent is *not* marked dirty, because the parent's own
  layout output can't possibly change as a result — it already gave this
  node fixed/self-contained constraints.
- If the node is *not* a relayout boundary, the dirty bit walks up to
  `parent.markNeedsLayout()` recursively until it hits a node that is a
  relayout boundary (or the root). That boundary node is what actually gets
  queued for re-layout; layout then proceeds downward from there as normal
  (see §6), potentially re-laying-out large parts of the subtree beneath it,
  but never anything above it.

This is why, e.g., resizing text inside a `SizedBox` with tight dimensions
only relays out inside that box — its parent already knows the box's exact
size and doesn't care what's inside.

`_nodesNeedingLayout` is flushed in a single pass sorted by **depth**
(shallowest first). This guarantees that if laying out a shallow boundary
happens to also clean up (via normal recursive descent) a deeper node that
was independently marked dirty, the deeper node is skipped when the list
reaches it (it's no longer dirty) — avoiding duplicate work.

---

## 5. The Constraint Model (Box Protocol)

Most of the render tree uses the **Box protocol** (`RenderBox`). Its
governing rule, and the single most important idea in the whole system:

> **Constraints go down the tree. Sizes go up the tree. The parent sets the
> position of the child (not the child).**

This is a strict, one-way, single-pass-per-node data flow (see §6 for
exceptions/multi-pass cases):

- A `BoxConstraints` object carries four numbers: `minWidth`, `maxWidth`,
  `minHeight`, `maxHeight`. Values can be `0` up to `double.infinity`.
- A parent calls `child.layout(constraints, parentUsesSize: ...)`.
- The child *must* choose a `Size` that satisfies
  `constraints.isSatisfiedBy(size)` — i.e. within the given min/max box. It
  cannot ignore the constraints it was given.
- The child computes its size using **only** the constraints it was given
  and (recursively) the sizes of *its own* children — never anything about
  its parent's other siblings or global state.
- The child does **not** know or decide its own position. Position
  (`offset`) is decided entirely by the parent after layout, and stored in
  the child's `parentData` (specifically `BoxParentData.offset` or a
  subclass like `FlexParentData` carrying extra info like `flex`).
- Special constraint shapes:
  - *Tight constraints*: `minWidth == maxWidth` and `minHeight == maxHeight`.
    The child has no freedom; it must be exactly that size. (Also what makes
    a node a relayout boundary, as above.)
  - *Loose constraints*: `min == 0`, only a `max` is meaningful. The child
    may be any size up to the max.
  - *Unbounded*: `max == double.infinity`. Common inside scrollables along
    the scroll axis. Many widgets (`Row`, `Column`) will assert/crash if
    given unbounded constraints along their main axis unless wrapped
    appropriately, because "size to fill available space" is undefined when
    there's no available space bound.

### 5.1 `parentUsesSize`
When a parent calls `child.layout(constraints, parentUsesSize: false)`
(the default is `false`; must be explicitly opted into `true`), it's telling
the framework: "I will not read `child.size` after this call to compute my
own size." This has real structural consequence:
- If `parentUsesSize` is false AND the constraints given are tight, then a
  later change to the child's *internal* content cannot possibly change the
  parent's size, satisfying the relayout-boundary condition from §4 — the
  child can be marked as its own relayout boundary safely.
- If `parentUsesSize` is true, the parent's size calculation depends on the
  child, so a change inside the child must be allowed to propagate upward
  through the parent's layout too — no boundary is established there.

### 5.2 `sizedByParent`
A RenderObject can override `sizedByParent = true` to declare that its
`Size` is a pure function of the incoming `BoxConstraints` alone — it
performs that computation in `performResize()`, separately and *before*
`performLayout()` (where children are laid out). This lets the framework
know the node's size (and thus whether it's a relayout boundary) without
needing to touch children at all, which is important because...

---

## 6. The Layout Algorithm, Step by Step

When `PipelineOwner.flushLayout()` processes a dirty node (a relayout
boundary), for that node and recursively downward:

1. **`layout(constraints, parentUsesSize)` is called** on the node (usually
   invoked by its parent during the parent's own `performLayout`, or by the
   pipeline owner directly for root/boundary nodes being flushed).
2. **Short-circuit check**: if the node is not marked dirty (`_needsLayout ==
   false`) AND the incoming constraints are identical to the constraints it
   was laid out with last time, `layout()` returns immediately without doing
   any work — the previous `size` is still valid. This constraints-equality
   memoization is the second major performance mechanism after relayout
   boundaries.
3. If not short-circuited, the node stores the new constraints
   (`this.constraints = constraints`) and:
   - If `sizedByParent` is true: call `performResize()` now, which sets
     `size` from `constraints` alone.
   - Call `performLayout()`. This is the method each RenderObject subclass
     overrides with its specific algorithm. Inside it, the node typically:
     a. Computes constraints to hand to each of its own children
        (derived from its *own* incoming constraints plus its layout
        policy — e.g. a `Padding` subtracts padding from max/min before
        passing down; a `Center` loosens constraints; a `Row` divides
        available main-axis space among flex children).
     b. Calls `child.layout(childConstraints, parentUsesSize: ...)` for
        each child (this recurses into step 1 for each child — layout is
        depth-first).
     c. Reads back `child.size` for any child where `parentUsesSize` was
        true, and uses those sizes to decide its own `size` (if not
        `sizedByParent`) and to compute each child's `offset`, which it
        writes into `child.parentData`.
     d. Sets `size` (if not already set via `sizedByParent`) such that it
        satisfies the constraints passed into *this* node.
   - Clears the dirty flag (`_needsLayout = false`).
4. **Assertions in debug mode** verify the node actually produced a size
   satisfying the given constraints, and that layout for this node didn't
   somehow depend on things outside the allowed protocol (e.g. asserting a
   parent didn't read `size` without declaring `parentUsesSize: true`).

Because step 3b recurses fully before step 3c reads sizes back, the overall
traversal is a **single depth-first pass**: constraints flow down through
the recursive call stack, and sizes flow back up through return values as
the stack unwinds. Painting offsets, by contrast, are computed only on the
way back up (in 3c), which is why a parent can position children only after
all of them have already computed their own size.

### 6.1 Why this is *usually* single-pass, and when it isn't
The protocol as described is a single top-down-then-bottom-up pass — O(n) in
the number of nodes touched. However, some layouts inherently need
information about a child *before* deciding what constraints to give that
same child, or need to compare multiple children before finalizing any one
of them. Flutter handles this via a secondary, deliberately-limited
mechanism: **intrinsic sizing** (see §7), rather than by making the box
protocol itself multi-pass. Widgets that need "measure everything, then
decide" behavior (e.g. `IntrinsicWidth`, custom widgets that align baseline
across siblings) pay for it by explicitly invoking intrinsic queries.

---

## 7. Intrinsic Dimensions — the Escape Hatch

Sometimes a parent needs to know something about a child's "natural" size
*before* it can decide the constraints to actually lay that child out with —
e.g. "make every column in this row as wide as the widest column's
intrinsic width." For this, `RenderBox` exposes a separate, parallel query
API, distinct from `layout()`:

- `getMinIntrinsicWidth(height)`
- `getMaxIntrinsicWidth(height)`
- `getMinIntrinsicHeight(width)`
- `getMaxIntrinsicHeight(width)`

These ask, "if you were given this one dimension, what's the
smallest/largest you'd want to be in the other dimension, ignoring your
actual imposed constraints?" Each RenderObject implements these based on its
own children's intrinsic queries recursively.

Crucially, calling these functions **does not** perform a real layout pass
and does **not** set `size` or clear dirty flags — it's a read-only,
side-effect-free computation (though it can be expensive, since it may
recurse through the whole relevant subtree, and it is *not* cached / memoized
the way `layout()` is). Because of this, intrinsic queries are:
- Deliberately kept out of the hot/default path — most widgets never call
  them.
- Documented as being O(N) or worse and warned against overuse (e.g.
  wrapping a large scrolling list in `IntrinsicHeight` is a classic
  performance foot-gun) since every parent layout can re-trigger the full
  intrinsic recursion.
- Used by a small set of widgets explicitly designed around them
  (`IntrinsicHeight`, `IntrinsicWidth`, `Table` column sizing in some modes,
  baseline-alignment logic).

This is the closest Flutter's core layout gets to "multi-pass," and it's
opt-in per-widget rather than a property of the general algorithm.

---

## 8. Other Layout Protocols

The Box protocol (constraints in, size out, offset set by parent) is a
specific instance of a more general pattern; Flutter has at least one other
first-class protocol:

### 8.1 Sliver Protocol (scrolling)
Used inside `Viewport`/`CustomScrollView` contexts (`RenderSliver`). Instead
of `BoxConstraints`/`Size`, it uses `SliverConstraints`/`SliverGeometry`.
Key differences:
- `SliverConstraints` include not just available extent but also scroll
  position/offset info: `scrollOffset` (how far the sliver has already
  scrolled past), `remainingPaintExtent`, `overlap`, `precedingScrollExtent`,
  axis direction/growth direction, and cache extent (for building content
  slightly outside the visible viewport ahead of time).
- `SliverGeometry` (the "size" analog) includes `scrollExtent` (the sliver's
  total logical length along the scroll axis), `paintExtent` (how much of it
  is actually visible/painted right now), `layoutExtent` (how much space it
  consumes for the purpose of laying out subsequent siblings),
  `maxPaintExtent`, `hitTestExtent`, and flags like `visible` and
  `hasVisualOverflow`.
- This lets a sliver report "I am logically 5000px tall but only painting
  the 800px currently visible" — enabling virtualization: children far
  outside the viewport are never even asked to build/layout
  (`SliverMultiBoxAdaptorWidget`/`RenderSliverList` machinery lazily
  materializes only nearby children based on the scroll offset given in
  constraints).
- A `RenderShrinkWrappingViewport`/`RenderViewport` is the boundary that
  converts ordinary Box constraints from the outside world into a sequence
  of Sliver layout calls to its sliver children, then converts the
  resulting geometry back into a single Size for itself.

Both protocols follow the exact same underlying philosophy: an immutable
description of available space flows down, a description of the resulting
geometry flows back up, and only the parent ever decides final position.

---

## 9. Painting Phase (brief, for context after layout)

Once layout completes for the frame:
1. `flushPaint()` visits nodes needing paint (also tracked via a dirty list,
   also respecting **repaint boundaries** — the paint analog of relayout
   boundaries, opted into via `RepaintBoundary` widgets, which force a node
   onto its own `Layer` so its content can be repainted independent of
   siblings/ancestors).
2. Each node's `paint(context, offset)` is called. The `offset` passed in is
   exactly the offset that was computed and stored during the parent's
   layout step (3c above) — this is the moment position, decided during
   layout, actually gets consumed.
3. Painting recurses depth-first as well, but order here reflects *paint
   order* (back-to-front / z-order defined by child list order, subject to
   any explicit stacking like `Stack`'s children or elevation-based
   ordering), not the layout traversal order per se (they happen to coincide
   because both are structural tree walks, but they are conceptually
   separate passes over the tree, gated by separate dirty flags:
   `_needsLayout` vs `_needsPaint`).
4. `PaintingContext`/`Canvas` calls accumulate into `Layer` objects. Layers
   form their own tree (coarser-grained than the RenderObject tree), which
   is what's finally handed to the engine for compositing/rasterization.

Layout and paint are deliberately decoupled by separate dirty-tracking so
that a pure visual change (e.g. a color animation) need not touch layout at
all, and, conversely, a pure geometry change knows it must also schedule a
repaint of the affected region afterward (`markNeedsLayout` implementations
generally also imply a repaint will be needed once geometry is finalized).

---

## 10. Summary of Core Rules (cheat sheet)

1. Widgets are immutable config; Elements are persistent identity +
   reconciliation; RenderObjects are the actual mutable layout/paint state.
   Layout only ever operates on RenderObjects.
2. Constraints flow strictly parent → child. Sizes flow strictly child →
   parent. Only the parent sets a child's position/offset.
3. A child must choose a size that satisfies the constraints it was given;
   it cannot see or depend on anything outside its own subtree.
4. Layout is depth-first: fully recurse into children (constraints down)
   before reading their sizes back (sizes up) to finish your own layout.
5. A node skips relayout entirely if it's not dirty and constraints are
   unchanged from last time (constraint-equality memoization).
6. A node becomes a *relayout boundary* if it's sized purely by its own
   constraints (tight constraints, or `sizedByParent`, or parent doesn't use
   its size, or it's root). Dirtying stops propagating upward at the nearest
   boundary — bounding the blast radius of any single change.
7. Cross-sibling / "know-before-you-lay-out" needs are handled by a
   separate, uncached, opt-in intrinsic-sizing query API — not by making the
   main protocol multi-pass.
8. Non-box contexts (scrolling) use an analogous but richer protocol
   (Sliver) carrying scroll-position-aware constraints/geometry, enabling
   virtualization of off-screen content.
9. Paint is a distinct pass, gated by its own dirty tracking and its own
   notion of boundary (`RepaintBoundary`), consuming the offsets that layout
   already finalized.

---

## 11. Implementation Notes if Building Something Similar

If you're reimplementing this model, the load-bearing design decisions worth
preserving are, roughly in priority order:

1. **Separate the "what" (declarative config) from the "how" (persistent
   mutable layout node)** — this is what makes cheap re-description +
   expensive-state-reuse possible. Without it, every UI update forces full
   relayout.
2. **Make the constraint object immutable and force children to satisfy it**
   — this is what guarantees a child's layout decision can never silently
   break a parent's assumptions, which in turn is what makes relayout
   boundaries provably safe to skip.
3. **Track two independent dirty flags (layout, paint) with independent
   boundary concepts.** Don't conflate "needs new geometry" with "needs new
   pixels" — a huge fraction of real-world updates only need one of the two.
4. **Compute the relayout-boundary condition structurally** (tight
   constraints / parent-doesn't-use-size / self-sized-by-constraints-only)
   rather than heuristically — it needs to be provably correct, since
   getting it wrong causes stale layouts, a very hard bug class to detect.
5. **Keep "measure before you decide" (intrinsics) as an explicit, separate,
   uncached opt-in API**, not baked into the default traversal — this keeps
   the common case O(n) single-pass and makes the expensive multi-pass cases
   visible and deliberate in the API surface.
6. **Decide child position only in the parent, only after the child's size
   is known**, and store it in a per-child slot owned by the parent (not the
   child) — this keeps children fully agnostic of their placement context
   and reusable across different parent types.