# rtree.rs

[![license](https://img.shields.io/crates/l/rtree_rs.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/d/rtree_rs.svg)](https://crates.io/crates/rtree_rs)
[![version](https://img.shields.io/crates/v/rtree_rs.svg?)](https://crates.io/crates/rtree_rs/)
[![documentation](https://docs.rs/rtree_rs/badge.svg?)](https://docs.rs/rtree_rs/)

A fast [R-tree](https://en.wikipedia.org/wiki/R-tree) for Rust. Ported from [an implementation](https://github.com/tidwall/rtree) that's designed for [Tile38](https://github.com/tidwall/tile38).

![Cities](cities.png)

## Features

- [Optimized](#algorithms) for fast inserts and updates. Ideal for both static and moving data.
- Supports the standard `insert`, `remove`, and `search` operations
- Includes `nearby` function for performing nearest neighbors (kNN) iterations

## Usage

```rust
use rtree_rs::{RTree, Rect};

let mut tr = RTree::new();

// Insert some points
tr.insert(Rect::new_point([-112.0078, 33.4373]), "PHX");
tr.insert(Rect::new_point([-118.4071, 33.9425]), "LAX");
tr.insert(Rect::new_point([-73.7822, 40.6441]),  "JFK");

// Search a rectangle
for item in tr.search(Rect::new([-112.1, 33.4], [-112.0, 33.5])) {
    println!("{}", item.data);
}

// OUTPUT:
// PHX
```

## Algorithms

This implementation is a variant of the original paper:  
[R-TREES. A DYNAMIC INDEX STRUCTURE FOR SPATIAL SEARCHING](http://www-db.deis.unibo.it/courses/SI-LS/papers/Gut84.pdf)

### Inserting

From the root to the leaf, the rectangles which will incur the least enlargement are chosen. Ties go to rectangles with the smallest area.

When a rectangle does not incur any enlargement at all, it's chosen immediately and without further checks of other rectangles in the same node.

### Splitting

This implementation attempts to minimize intensive operations such as pre-sorting the children and comparing overlaps & area sizes. The desire is to do simple single axis distance calculation on each child only once, with a target 50/50 chance that the child might be moved in-memory.

When a rect has reached it's max number of entries it's largest axis is calculated and the rect is split into two smaller rects, named `left` and `right`. Each child rects is then evaluated to determine which smaller rect it should be placed into. Two values, `min-dist` and `max-dist`, are calcuated for each child. 

- `min-dist` is the distance from the parent's minumum value of it's largest axis to the child's minumum value of the parent largest axis.
- `max-dist` is the distance from the parent's maximum value of it's largest axis to the child's maximum value of the parent largest axis.

When the `min-dist` is less than `max-dist` then the child is placed into the `left` rect, otherwise the child is placed into the `right` rect. 

If either left or right has fewer than the minimum allowable entries, then it will take enough entries from the other rectangle to ensure that the minumum is reached. The chosen ones are based on the distance of the farthest edge to the other rectangle's axis.

Finally, the child rectangles are sorted in their parent node by the minimum x value.

### Deleting

Similar to the original algorithm.
A target rect is deleted directly. When the number of children in a rectangle falls below the minumum allowed, that child is removed from the tree and all of it's leaf items are re-inserted into the tree starting at the root.

### Searching

Same as the original algorithm.
