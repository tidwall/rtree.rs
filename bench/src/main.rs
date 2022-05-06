extern crate rtree_rs;
extern crate rstar;
extern crate lotsa;
extern crate rand;

fn main() {
    const N: usize = 1_000_000;
    
    let mut pts = Vec::new();
    for _ in 0..N {
        let pt = [
            rand::random::<f64>() * 360.0 - 180.0,
            rand::random::<f64>() * 180.0 - 90.0,
        ];
        pts.push(pt);
    }

    // 1%
    let mut r1 = Vec::new();
    for _ in 0..10_000 {
        let p = 0.01;
        let min = [
            rand::random::<f64>() * 360.0 - 180.0,
            rand::random::<f64>() * 180.0 - 90.0,
        ];
        let max = [min[0] + 360.0 * p, min[1] + 180.0 * p];
        r1.push((min, max));
    }
    // 5%
    let mut r5 = Vec::new();
    for _ in 0..10_000 {
        let p = 0.05;
        let min = [
            rand::random::<f64>() * 360.0 - 180.0,
            rand::random::<f64>() * 180.0 - 90.0,
        ];
        let max = [min[0] + 360.0 * p, min[1] + 180.0 * p];
        r5.push((min, max));
    }
    // 10%
    let mut r10 = Vec::new();
    for _ in 0..10_000 {
        let p = 0.10;
        let min = [
            rand::random::<f64>() * 360.0 - 180.0,
            rand::random::<f64>() * 180.0 - 90.0,
        ];
        let max = [min[0] + 360.0 * p, min[1] + 180.0 * p];
        r10.push((min, max));
    }

    println!(">>> rtree_rs::RTree <<<");
    let mut tr = rtree_rs::RTree::new();
    print!("insert:        ");
    lotsa::ops(pts.len(), 1, |i, _| {
        tr.insert(rtree_rs::Rect::new(pts[i],pts[i]), i);
    });
    print!("search-item:   ");
    lotsa::ops(pts.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(pts[i],pts[i])) {
            break;
        }
    });
    print!("search-1%:     ");
    lotsa::ops(r1.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(r1[i].0,r1[i].1)) {}
    });
    print!("search-5%:     ");
    lotsa::ops(r5.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(r5[i].0,r5[i].1)) {}
    });
    print!("search-10%:    ");
    lotsa::ops(r10.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(r10[i].0,r10[i].1)) {}
    });
    print!("remove-half:   ");
    lotsa::ops(pts.len()/2, 1, |i, _| {
        tr.remove(rtree_rs::Rect::new(pts[i*2],pts[i*2]), &(i*2)).unwrap();
    });
    print!("reinsert-half: ");
    lotsa::ops(pts.len()/2, 1, |i, _| {
        tr.insert(rtree_rs::Rect::new(pts[i*2],pts[i*2]), i*2);
    });
    print!("search-item:   ");
    lotsa::ops(pts.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(pts[i],pts[i])) {
            break;
        }
    });
    print!("search-1%:     ");
    lotsa::ops(r1.len(), 1, |i, _| {
        for _ in tr.search(rtree_rs::Rect::new(r1[i].0,r1[i].1)) {}
    });
    print!("remove-all:    ");
    lotsa::ops(pts.len(), 1, |i, _| {
        tr.remove(rtree_rs::Rect::new(pts[i],pts[i]), &i).unwrap();
    });


    println!();
    println!(">>> rstar::RTree <<<");
    let mut tr = rstar::RTree::new();
    print!("insert:        ");
    lotsa::ops(N, 1, |i, _| {
        tr.insert(pts[i]);
    });
    print!("search-item:   ");
    lotsa::ops(N, 1, |i, _| {
        let rect = rstar::AABB::from_corners(pts[i], pts[i]);
        for _ in tr.locate_in_envelope_intersecting(&rect) {
            break;
        }
    });
    print!("search-1%:     ");
    lotsa::ops(r1.len(), 1, |i, _| {
        let rect = rstar::AABB::from_corners(r1[i].0,r1[i].1);
        for _ in tr.locate_in_envelope_intersecting(&rect) {}
    });
    print!("search-5%:     ");
    lotsa::ops(r5.len(), 1, |i, _| {
        let rect = rstar::AABB::from_corners(r5[i].0,r5[i].1);
        for _ in tr.locate_in_envelope_intersecting(&rect) {}
    });
    print!("search-10%:    ");
    lotsa::ops(r10.len(), 1, |i, _| {
        let rect = rstar::AABB::from_corners(r10[i].0,r10[i].1);
        for _ in tr.locate_in_envelope_intersecting(&rect) {}
    });
    print!("remove-half:   ");
    lotsa::ops(pts.len()/2, 1, |i, _| {
        tr.remove(&pts[i*2]).unwrap();
    });
    print!("reinsert-half: ");
    lotsa::ops(pts.len()/2, 1, |i, _| {
        tr.insert(pts[i*2]);
    });
    print!("search-item:   ");
    lotsa::ops(N, 1, |i, _| {
        let rect = rstar::AABB::from_corners(pts[i], pts[i]);
        for _ in tr.locate_in_envelope_intersecting(&rect) {
            break;
        }
    });
    print!("search-1%:     ");
    lotsa::ops(r1.len(), 1, |i, _| {
        let rect = rstar::AABB::from_corners(r1[i].0,r1[i].1);
        for _ in tr.locate_in_envelope_intersecting(&rect) {}
    });
    print!("remove-all:    ");
    lotsa::ops(pts.len(), 1, |i, _| {
        tr.remove(&pts[i]).unwrap();
    });
}
