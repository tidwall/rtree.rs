#[cfg(test)]
mod test;

extern crate pqueue;

use pqueue::Queue;
use std::cmp::Ordering;
use std::default::Default;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;

const MAX_ITEMS: usize = 32;
const MIN_ITEMS: usize = MAX_ITEMS * 20 / 100;

/// An axis-aligned rectangle in D dimensions.
/// `min` and `max` specify the minimum and maximum coordinates along each axis.
///
/// # Type Parameters
/// - `D`: Number of dimensions (const generic)
/// - `C`: Coordinate numeric type (e.g., `f32`, `u64`)
/// 
/// # Example 
/// ```
/// use rtree_rs::Rect;
/// 
/// let rect: Rect<2, f32> = Rect::new([1.0, 1.0], [2.0, 2.0]);
/// assert_eq!(rect.min, [1.0, 1.0]);
/// assert_eq!(rect.max, [2.0, 2.0]);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rect<const D: usize, C>
where
    C: Default,
{
    pub min: [C; D],
    pub max: [C; D],
}

fn min<C: PartialOrd>(a: C, b: C) -> C {
    if a < b {
        a
    } else {
        b
    }
}

fn max<C: PartialOrd>(a: C, b: C) -> C {
    if a > b {
        a
    } else {
        b
    }
}

fn compare<C: PartialOrd>(a: C, b: C) -> Ordering {
    if a < b {
        Ordering::Less
    } else if a > b {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

impl<const D: usize, C> Rect<D, C>
where
    C: PartialOrd + Copy + Sub<Output = C> + Add<Output = C> + Mul<Output = C> + Default,
{
    /// Create a new rectangle from min and max corners.
    ///
    /// # Arguments
    /// * `min` - Lower corner
    /// * `max` - Upper corner
    ///
    /// # Example 
    /// ```
    /// use rtree_rs::Rect;
    /// 
    /// let rect: Rect<2, i32> = Rect::new([1, 1], [2, 2]);
    /// assert_eq!(rect.min, [1, 1]);
    /// assert_eq!(rect.max, [2, 2]);
    /// ```
    pub fn new(min: [C; D], max: [C; D]) -> Rect<D, C> {
        Rect { min, max }
    }

    /// Create a zero-size rectangle at a given point.
    /// 
    /// # Example 
    /// ```
    /// use rtree_rs::Rect;
    /// 
    /// let rect: Rect<2, i32> = Rect::new_point([1, 1]);
    /// assert_eq!(rect.min, [1, 1]);
    /// assert_eq!(rect.max, [1, 1]);
    /// ```
    pub fn new_point(point: [C; D]) -> Rect<D, C> {
        Rect::new(point, point)
    }

    fn expand(&mut self, rect: &Rect<D, C>) {
        for i in 0..D {
            if rect.min[i] < self.min[i] {
                self.min[i] = rect.min[i];
            }
            if rect.max[i] > self.max[i] {
                self.max[i] = rect.max[i];
            }
        }
    }
    fn largest_axis(&self) -> usize {
        if D == 0 {
            return 0;
        }
        let mut axis = 0;
        let mut size = self.max[0] - self.min[0];
        for i in 1..D {
            let asize = self.max[i] - self.min[i];
            if asize > size {
                axis = i;
                size = asize;
            }
        }
        axis
    }
    fn contains(&self, rect: &Rect<D, C>) -> bool {
        if D == 0 {
            return false;
        }
        for i in 0..D {
            if rect.min[i] < self.min[i] || rect.max[i] > self.max[i] {
                return false;
            }
        }
        true
    }
    fn intersects(&self, rect: &Rect<D, C>) -> bool {
        if D == 0 {
            return false;
        }
        for i in 0..D {
            if rect.min[i] > self.max[i] || rect.max[i] < self.min[i] {
                return false;
            }
        }
        true
    }
    fn on_edge(&self, rect: &Rect<D, C>) -> bool {
        for i in 0..D {
            if !(rect.min[i] > self.min[i]) || !(rect.max[i] < self.max[i]) {
                return true;
            }
        }
        false
    }
    fn area(&self) -> C {
        if D == 0 {
            return Default::default();
        }
        let mut area = self.max[0] - self.min[0];
        for i in 1..D {
            area = area * (self.max[i] - self.min[i]);
        }
        area
    }
    fn unioned_area(&self, rect: &Rect<D, C>) -> C {
        if D == 0 {
            return Default::default();
        }
        let mut area = max(self.max[0], rect.max[0]) - min(self.min[0], rect.min[0]);
        for i in 1..D {
            area = area * (max(self.max[i], rect.max[i]) - min(self.min[i], rect.min[i]));
        }
        area
    }
    pub fn box_dist(&self, rect: &Rect<D, C>) -> C {
        let zero = Default::default();
        if D == 0 {
            return zero;
        }
        let mut dist = zero;
        for i in 0..D {
            let x = max(self.min[i], rect.min[i]) - min(self.max[i], rect.max[i]);
            if x > zero {
                dist = dist + (x * x);
            }
        }
        dist
    }
}

impl<const D: usize, C: Copy + Default> Default for Rect<D, C> {
    fn default() -> Rect<D, C> {
        Rect{
            min: [Default::default(); D],
            max: [Default::default(); D],
        }
    }
}

enum Data<const D: usize, C, T>
where
    C: PartialOrd + Copy + Default,
{
    Item(T),
    Nodes(Box<Vec<Node<D, C, T>>>),
}

struct Node<const D: usize, C, T>
where
    C: PartialOrd + Copy + Default,
{
    rect: Rect<D, C>,
    data: Data<D, C, T>,
}

impl<const D: usize, C, T: PartialEq> Node<D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Add<Output = C> + Mul<Output = C> + Default,
{
    fn new(rect: Rect<D, C>) -> Node<D, C, T> {
        Node {
            rect: rect,
            data: Data::Nodes(Box::new(Vec::with_capacity(MAX_ITEMS))),
        }
    }
    fn len(&self) -> usize {
        match &self.data {
            Data::Nodes(nodes) => nodes.len(),
            _ => panic!("not a branch node"),
        }
    }
    fn nodes(&self) -> &Vec<Node<D, C, T>> {
        match &self.data {
            Data::Nodes(nodes) => nodes,
            _ => panic!("not a branch node"),
        }
    }
    fn nodes_mut(&mut self) -> &mut Vec<Node<D, C, T>> {
        match &mut self.data {
            Data::Nodes(nodes) => nodes,
            _ => panic!("not a branch node"),
        }
    }
    fn item(&self) -> &T {
        match &self.data {
            Data::Item(item) => item,
            _ => panic!("not a leaf node"),
        }
    }
    fn choose_least_enlargement(&self, rect: &Rect<D, C>) -> usize {
        if D == 0 {
            return 0;
        }
        let mut j = 0;
        let mut jenlargement = rect.min[0];
        let mut jarea = rect.min[0];
        let nodes = self.nodes();
        for i in 0..nodes.len() {
            let uarea = nodes[i].rect.unioned_area(rect);
            let area = nodes[i].rect.area();
            let enlargement = uarea - area;
            if i == 0 || enlargement < jenlargement || (enlargement == jenlargement && area < jarea)
            {
                j = i;
                jenlargement = enlargement;
                jarea = area;
            }
        }
        j
    }
    fn choose_subtree(&self, rect: &Rect<D, C>) -> usize {
        if D == 0 {
            return 0;
        }
        self.choose_least_enlargement(&rect)
    }
    fn insert(&mut self, rect: Rect<D, C>, data: T, height: usize) {
        if height == 0 {
            // leaf node
            self.nodes_mut().push(Node {
                rect: rect,
                data: Data::Item(data),
            });
        } else {
            // branch node
            let index = self.choose_subtree(&rect);
            let nodes = self.nodes_mut();
            let child = &mut nodes[index];
            child.insert(rect, data, height - 1);
            if child.len() == MAX_ITEMS {
                let right = child.split_largest_axis_edge_snap();
                nodes.push(right);
            }
        }
        if !self.rect.contains(&rect) {
            self.rect.expand(&rect);
        }
    }
    fn recalc(&mut self) {
        let nodes = self.nodes_mut();
        if nodes.len() == 0 {
            return;
        }
        let mut rect = nodes[0].rect;
        for i in 1..nodes.len() {
            rect.expand(&nodes[i].rect);
        }
        self.rect = rect
    }
    fn split_largest_axis_edge_snap(&mut self) -> Node<D, C, T> {
        let rect = self.rect;
        let axis = rect.largest_axis();
        let mut right = Node::new(rect);
        let lchilds = self.nodes_mut();
        let rchilds = right.nodes_mut();
        let mut i = 0;
        while i < lchilds.len() {
            let min = lchilds[i].rect.min[axis] - rect.min[axis];
            let max = rect.max[axis] - lchilds[i].rect.max[axis];
            if min < max {
                // stay left
                i += 1;
            } else {
                // move right
                rchilds.push(lchilds.swap_remove(i));
            }
        }
        // Make sure that both left and right nodes have at least
        // MIN_ITEMS by moving items into underflowed nodes.
        if lchilds.len() < MIN_ITEMS {
            // reverse sort by min axis
            rchilds.sort_by(|a, b| compare(b.rect.min[axis], a.rect.min[axis]));
            while lchilds.len() < MIN_ITEMS {
                lchilds.push(rchilds.pop().unwrap());
            }
        } else if rchilds.len() < MIN_ITEMS {
            // reverse sort by max axis
            lchilds.sort_by(|a, b| compare(b.rect.max[axis], a.rect.max[axis]));
            while rchilds.len() < MIN_ITEMS {
                rchilds.push(lchilds.pop().unwrap());
            }
        }
        // recalculate and sort the nodes
        self.recalc();
        right.recalc();
        self.sort_by_axis(0);
        right.sort_by_axis(0);
        right
    }
    fn push(&mut self, child: Node<D, C, T>) {
        self.nodes_mut().push(child);
    }
    fn sort_by_axis(&mut self, axis: usize) {
        let nodes = self.nodes_mut();
        nodes.sort_by(|a, b| compare(a.rect.min[axis], b.rect.min[axis]));
    }
    fn flatten_into(&mut self, reinsert: &mut Vec<(Rect<D, C>, T)>) {
        let nodes = self.nodes_mut();
        while let Some(mut node) = nodes.pop() {
            match node.data {
                Data::Item(data) => reinsert.push((node.rect, data)),
                _ => node.flatten_into(reinsert),
            }
        }
    }
    pub fn remove(
        &mut self,
        rect: &Rect<D, C>,
        data: &T,
        reinsert: &mut Vec<(Rect<D, C>, T)>,
        height: usize,
    ) -> (Option<(Rect<D, C>, T)>, bool) {
        let nodes = self.nodes_mut();
        if height == 0 {
            // remove from leaf
            for i in 0..nodes.len() {
                if nodes[i].item() == data {
                    let out = nodes.swap_remove(i);
                    let recalced = self.rect.on_edge(&out.rect);
                    if recalced {
                        self.recalc();
                    }
                    return (
                        Some((
                            out.rect,
                            match out.data {
                                Data::Item(data) => data,
                                _ => unreachable!(),
                            },
                        )),
                        recalced,
                    );
                }
            }
        } else {
            for i in 0..nodes.len() {
                if !nodes[i].rect.intersects(rect) {
                    continue;
                }
                let (removed, mut recalced) = nodes[i].remove(rect, data, reinsert, height - 1);
                if removed.is_none() {
                    continue;
                }
                let underflow = nodes[i].len() < MIN_ITEMS;
                if underflow {
                    let nrect = nodes[i].rect;
                    nodes.swap_remove(i).flatten_into(reinsert);
                    if !recalced {
                        recalced = self.rect.on_edge(&nrect);
                    }
                }
                if recalced {
                    self.recalc();
                }
                return (removed, recalced);
            }
        }
        (None, false)
    }
    pub fn search_flat<'a>(&'a self, rect: &Rect<D, C>, items: &mut Vec<(Rect<D, C>, &'a T)>) {
        let nodes = self.nodes();
        for i in 0..nodes.len() {
            if nodes[i].rect.intersects(&rect) {
                match &nodes[i].data {
                    Data::Item(data) => items.push((nodes[i].rect, data)),
                    _ => nodes[i].search_flat(&rect, items),
                }
            }
        }
    }
}

/// An R-tree for fast spatial insert, remove, and query operations.
/// 
/// # Type Parameters
/// - `D`: Number of dimensions (const generic)
/// - `C`: Coordinate numeric type (e.g., `f32`, `u64`)
/// - `T`: type of the item of the node
pub struct RTree<const D: usize, C, T: PartialEq>
where
    C: PartialOrd + Copy + Default,
{
    root: Option<Node<D, C, T>>,
    length: usize,
    height: usize,
}

impl<const D: usize, C, T: PartialEq> RTree<D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Add<Output = C> + Mul<Output = C> + Default,
{
    /// Create an empty R-tree instance.
    /// 
    /// # Example
    /// ```
    /// use rtree_rs::RTree;
    /// 
    /// let rtree: RTree<2, f32, &'static str> = RTree::new();
    /// assert_eq!(rtree.len(), 0);
    /// ```
    pub fn new() -> RTree<D, C, T> {
        RTree {
            root: None,
            length: 0,
            height: 0,
        }
    }

    /// Number of items stored in the tree.
    /// 
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    /// 
    /// let mut rtree = RTree::new();
    /// assert_eq!(rtree.len(), 0);
    /// 
    /// rtree.insert(Rect::new_point([52.0, 7.2]), "hello rustaceans!");
    /// assert_eq!(rtree.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns the bounding rectangle incasing all items, if any.
    /// 
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    /// 
    /// let mut rtree = RTree::new();
    /// 
    /// assert!(rtree.rect().is_none()); 
    /// 
    /// rtree.insert(Rect::new_point([52.0, 7.2]), "bob");
    /// rtree.insert(Rect::new_point([56.0, 15.2]), "john");
    /// rtree.insert(Rect::new_point([85.0, 52.0]), "piet");
    /// 
    /// let rect = rtree.rect()
    ///     .expect("rect should be Some");
    /// 
    /// assert_eq!(rect.min, [52.0, 7.2]);
    /// assert_eq!(rect.max, [85.0, 52.0]);
    /// ```
    pub fn rect(&self) -> Option<Rect<D, C>> {
        match &self.root {
            Some(root) => Some(root.rect),
            None => None,
        }
    }

    /// Insert a new item associated with a rectangle.
    ///     
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    /// 
    /// let mut rtree = RTree::new();
    /// rtree.insert(Rect::new_point([52.0, 7.2]), "hello rustaceans!");
    /// ```
    pub fn insert(&mut self, rect: Rect<D, C>, data: T) {
        if self.root.is_none() {
            self.root = Some(Node::new(rect));
        }
        let root = self.root.as_mut().unwrap();
        root.insert(rect, data, self.height);
        if root.len() == MAX_ITEMS {
            let mut new_root = Node::new(root.rect);
            let right = root.split_largest_axis_edge_snap();
            let left = self.root.take().unwrap();
            new_root.push(left);
            new_root.push(right);
            self.root = Some(new_root);
            self.height += 1;
        }
        self.length += 1;
    }

    /// Remove an item matching a rectangle and payload value.
    /// Returns the removed entry if found.
    /// 
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    /// 
    /// let mut rtree = RTree::new();
    /// let empty_remove = rtree.remove(Rect::new_point([52.0, 7.2]), &"hello");
    /// assert!(empty_remove.is_none());
    /// 
    /// rtree.insert(Rect::new_point([52.0, 7.2]), "hello");
    /// assert_eq!(rtree.len(), 1);
    /// 
    /// let remove = rtree.remove(Rect::new_point([52.0, 7.2]), &"hello")
    ///     .expect("should be Some");
    /// 
    /// assert_eq!(rtree.len(), 0);
    /// 
    /// assert!(remove.0 == Rect::new_point([52.0, 7.2]));
    /// assert!(remove.1 == "hello");
    /// ```
    pub fn remove(&mut self, rect: Rect<D, C>, data: &T) -> Option<(Rect<D, C>, T)> {
        if let Some(root) = &mut self.root {
            let mut reinsert = Vec::new();
            let (removed, recalced) = root.remove(&rect, data, &mut reinsert, self.height);
            if removed.is_none() {
                return None;
            }
            self.length -= reinsert.len() + 1;
            if self.length == 0 {
                self.root = None;
            } else if self.height > 0 && root.len() == 1 {
                let nodes = root.nodes_mut();
                let mut n = nodes.pop().unwrap();
                n.recalc();
                self.height -= 1;
                self.root = Some(n);
            } else if recalced {
                if let Some(root) = &mut self.root {
                    root.recalc();
                }
            }
            while let Some(item) = reinsert.pop() {
                self.insert(item.0, item.1);
            }
            removed
        } else {
            None
        }
    }

    /// Query all items intersecting a given rectangle, appending results to `items`.
    /// 
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    /// 
    /// let mut rtree = RTree::new();
    /// 
    /// 
    /// rtree.insert(Rect::new_point([1.0, 2.0]), "no intersect, x/y to small");
    /// rtree.insert(Rect::new_point([11.0, 19.0]), "intersect");
    /// rtree.insert(Rect::new_point([15.0, 21.0]), "no intersect, y to big");
    /// rtree.insert(Rect::new_point([21.0, 21.0]), "no intersect, x/y to big");
    /// 
    /// let mut found_items = vec![];
    /// rtree.search_flat(Rect::new([10.0, 10.0], [20.0, 20.0]), &mut found_items);
    /// 
    /// assert_eq!(found_items.len(), 1);
    /// assert!(found_items[0] == (Rect::new_point([11.0, 19.0]), &"intersect"));
    /// 
    /// ```
    pub fn search_flat<'a>(&'a self, rect: Rect<D, C>, items: &mut Vec<(Rect<D, C>, &'a T)>) {
        if let Some(root) = &self.root {
            root.search_flat(&rect, items);
        }
    }
}

// iterartors, ScanIterator, SearcIterator, NearbyIterator

/// Item value yielded by R-tree iterators.
/// Reports its bounding rectangle, payload, and distance (for kNN).
pub struct IterItem<'a, const D: usize, C: Default, T> {
    pub rect: Rect<D, C>,
    pub data: &'a T,
    pub dist: C,
}

impl<const D: usize, C, T: PartialEq> RTree<D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    /// Returns an iterator over all items in the R-tree.
    ///
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    ///
    /// let mut rtree = RTree::new();
    /// rtree.insert(Rect::new_point([1.0, 2.0]), "A");
    /// rtree.insert(Rect::new_point([3.0, 4.0]), "B");
    ///
    /// let items: Vec<_> = rtree.iter().map(|item| item.data).collect();
    /// assert_eq!(items, vec![&"A", &"B"]);
    /// ```
    pub fn iter(&self) -> ScanIterator<D, C, T> {
        self.scan()
    }

    /// Returns an iterator that scans all items in the R-tree without filtering.
    ///
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    ///
    /// let mut rtree = RTree::new();
    /// rtree.insert(Rect::new_point([1.0, 2.0]), "A");
    /// rtree.insert(Rect::new_point([3.0, 4.0]), "B");
    ///
    /// let items: Vec<_> = rtree.scan().map(|item| item.data).collect();
    /// assert_eq!(items, vec![&"A", &"B"]);
    /// ```
    pub fn scan(&self) -> ScanIterator<D, C, T> {
        ScanIterator::new(&self.root, self.height)
    }

    /// Returns an iterator over all items intersecting a given rectangle.
    ///
    /// # Arguments
    /// * `rect` - The query rectangle to intersect with stored item rectangles.
    ///
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    ///
    /// let mut rtree = RTree::new();
    /// rtree.insert(Rect::new_point([1.0, 2.0]), "inside");
    /// rtree.insert(Rect::new_point([5.0, 6.0]), "outside");
    ///
    /// let query = Rect::new([0.0, 0.0], [3.0, 4.0]);
    /// let mut results = rtree.search(query);
    ///
    /// let found: Vec<_> = results.map(|item| item.data).collect();
    /// assert_eq!(found, vec![&"inside"]);
    /// ```
    pub fn search<'a>(&self, rect: Rect<D, C>) -> SearchIterator<D, C, T> {
        SearchIterator::new(&self.root, self.height, rect)
    }

    /// Returns an iterator for performing nearest-neighbor (kNN) style queries,
    /// using a custom user-provided distance function.
    ///
    /// # Arguments
    /// * `dist` - A function that computes distance from an item rectangle and optional data reference.
    ///
    /// # Example
    /// ```
    /// use rtree_rs::{RTree, Rect};
    ///
    /// let mut rtree: RTree<2, f32, &'static str> = RTree::new();
    /// rtree.insert(Rect::new_point([0.0, 0.0]), "origin");
    /// rtree.insert(Rect::new_point([5.0, 5.0]), "far");
    ///
    /// // Distance function measuring squared distance to point (1,1)
    /// let nearest = rtree.nearby(|rect, _| {
    ///     let dx = if rect.min[0] > 1.0 { rect.min[0] - 1.0 } else { 1.0 - rect.max[0] };
    ///     let dy = if rect.min[1] > 1.0 { rect.min[1] - 1.0 } else { 1.0 - rect.max[1] };
    ///     dx * dx + dy * dy
    /// });
    ///
    /// let closest = nearest.map(|item| item.data).next();
    /// assert_eq!(closest, Some(&"origin"));
    /// ```
    pub fn nearby<'a, F>(&'a self, dist: F) -> NearbyIterator<D, C, T, F>
    where
        F: FnMut(Rect<D, C>, Option<&'a T>) -> C,
    {
        NearbyIterator::new(&self.root, dist)
    }
}

struct StackNode<'a, const D: usize, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    nodes: &'a [Node<D, C, T>],
    index: usize,
}

impl<'a, const D: usize, C, T> StackNode<'a, D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    fn new_stack(root: &'a Option<Node<D, C, T>>, height: usize) -> Vec<StackNode<'a, D, C, T>> {
        let mut stack = Vec::with_capacity(height + 1);
        if let Some(root) = &root {
            stack.push(StackNode {
                nodes: match &root.data {
                    Data::Nodes(nodes) => nodes,
                    _ => unreachable!(),
                },
                index: 0,
            });
        }
        stack
    }
}

// scan iterator

pub struct ScanIterator<'a, const D: usize, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    stack: Vec<StackNode<'a, D, C, T>>,
}

impl<'a, const D: usize, C, T> ScanIterator<'a, D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    fn new(root: &'a Option<Node<D, C, T>>, height: usize) -> ScanIterator<'a, D, C, T> {
        ScanIterator {
            stack: StackNode::new_stack(root, height),
        }
    }
}

impl<'a, const D: usize, C, T> Iterator for ScanIterator<'a, D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    type Item = IterItem<'a, D, C, T>;
    fn next(&mut self) -> Option<Self::Item> {
        'outer: while let Some(stack) = &mut self.stack.last_mut() {
            for i in stack.index..stack.nodes.len() {
                stack.index = i + 1;
                if let Data::Item(data) = &stack.nodes[i].data {
                    return Some(IterItem {
                        rect: stack.nodes[i].rect,
                        data,
                        dist: Default::default(),
                    });
                }
                let snode = StackNode {
                    nodes: match &stack.nodes[i].data {
                        Data::Nodes(nodes) => nodes,
                        _ => unreachable!(),
                    },
                    index: 0,
                };
                self.stack.push(snode);
                continue 'outer;
            }
            self.stack.pop();
        }
        None
    }
}

// search iterator -- much like the scan iterator but with a intersects guard.

pub struct SearchIterator<'a, const D: usize, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    stack: Vec<StackNode<'a, D, C, T>>,
    rect: Rect<D, C>,
}

impl<'a, const D: usize, C, T> SearchIterator<'a, D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
{
    fn new(
        root: &'a Option<Node<D, C, T>>,
        height: usize,
        rect: Rect<D, C>,
    ) -> SearchIterator<'a, D, C, T> {
        SearchIterator {
            stack: StackNode::new_stack(root, height),
            rect,
        }
    }
}

impl<'a, const D: usize, C, T> Iterator for SearchIterator<'a, D, C, T>
where
    C: PartialOrd + Copy + Sub<Output = C> + Add<Output = C> + Mul<Output = C> + Default,
{
    type Item = IterItem<'a, D, C, T>;
    fn next(&mut self) -> Option<Self::Item> {
        'outer: while let Some(stack) = &mut self.stack.last_mut() {
            for i in stack.index..stack.nodes.len() {
                if !stack.nodes[i].rect.intersects(&self.rect) {
                    continue;
                }
                stack.index = i + 1;
                if let Data::Item(data) = &stack.nodes[i].data {
                    return Some(IterItem {
                        rect: stack.nodes[i].rect,
                        data,
                        dist: Default::default(),
                    });
                }
                let snode = StackNode {
                    nodes: match &stack.nodes[i].data {
                        Data::Nodes(nodes) => nodes,
                        _ => unreachable!(),
                    },
                    index: 0,
                };
                self.stack.push(snode);
                continue 'outer;
            }
            self.stack.pop();
        }
        None
    }
}

struct NearbyItem<'a, const D: usize, C, T>
where
    C: PartialOrd + Copy + Default,
{
    dist: C,
    node: &'a Node<D, C, T>,
}

impl<'a, const D: usize, C, T> PartialEq for NearbyItem<'a, D, C, T>
where
    C: PartialOrd + Copy + Default,
{
    fn eq(&self, other: &NearbyItem<'a, D, C, T>) -> bool {
        self.dist.eq(&other.dist)
    }
}

impl<'a, const D: usize, C, T> PartialOrd for NearbyItem<'a, D, C, T>
where
    C: PartialOrd + Copy + Default,
{
    fn partial_cmp(&self, other: &NearbyItem<'a, D, C, T>) -> Option<Ordering> {
        self.dist.partial_cmp(&other.dist)
    }
}

pub struct NearbyIterator<'a, const D: usize, C, T, F>
where
    C: PartialOrd + Copy + Default,
{
    queue: Queue<NearbyItem<'a, D, C, T>>,
    dist: F,
}

impl<'a, const D: usize, C, T, F> NearbyIterator<'a, D, C, T, F>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
    F: FnMut(Rect<D, C>, Option<&'a T>) -> C,
{
    fn new(root: &'a Option<Node<D, C, T>>, dist: F) -> NearbyIterator<'a, D, C, T, F> {
        let mut queue = Queue::new();
        if let Some(root) = root {
            queue.push(NearbyItem {
                dist: Default::default(),
                node: root,
            });
        }
        NearbyIterator { queue, dist }
    }
}

impl<'a, const D: usize, C, T, F> Iterator for NearbyIterator<'a, D, C, T, F>
where
    C: PartialOrd + Copy + Sub<Output = C> + Mul<Output = C> + Default,
    F: FnMut(Rect<D, C>, Option<&'a T>) -> C,
{
    type Item = IterItem<'a, D, C, T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.queue.pop() {
            match &item.node.data {
                Data::Item(data) => {
                    return Some(IterItem {
                        rect: item.node.rect,
                        data: data,
                        dist: item.dist,
                    });
                }
                Data::Nodes(nodes) => {
                    for i in 0..nodes.len() {
                        self.queue.push(NearbyItem {
                            dist: (self.dist)(
                                nodes[i].rect,
                                match &nodes[i].data {
                                    Data::Item(data) => Some(data),
                                    _ => None,
                                },
                            ),
                            node: &nodes[i],
                        });
                    }
                }
            }
        }
        None
    }
}
