//! Disposizione dei diagrammi da sinistra a destra (metodo a livelli, "Sugiyama"):
//! 1. colonne: ogni nodo va una colonna dopo il più lontano dei suoi genitori;
//! 2. i collegamenti che saltano colonne ricevono una corsia propria (nodi fittizi);
//! 3. l'ordine dentro ogni colonna è scelto per ridurre gli incroci;
//! 4. la posizione verticale avvicina ogni nodo ai suoi vicini senza sovrapposizioni;
//! 5. tra due colonne ogni nodo di partenza ha la sua linea verticale.

use crate::data::Node;
use std::collections::HashMap;

pub const W: f64 = 204.0;
pub const H: f64 = 40.0;
const GY: f64 = 12.0;
const LANE: f64 = 9.0;
const TRACK: f64 = 7.0;
const GAP_MIN: f64 = 44.0;
const GAP_PAD: f64 = 14.0;

pub struct Edge { pub p: usize, pub c: usize, pub lines: Vec<Vec<(f64, f64)>>, pub dots: Vec<(f64, f64)> }
pub struct Bounds { pub x0: f64, pub x1: f64, pub y0: f64, pub y1: f64 }
pub struct Layout { pub pos: Vec<(f64, f64)>, pub edges: Vec<Edge>, pub bounds: Option<Bounds>, pub crossings: usize }

struct V { layer: usize, real: bool, idx: f64, up: Vec<usize>, down: Vec<usize>, y: f64, o: usize, key: f64, x: f64 }

fn columns(nodes: &[Node]) -> Vec<usize> {
    let map: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let mut kids: Vec<Vec<usize>> = vec![vec![]; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        for p in &n.parents { if let Some(&pi) = map.get(p.as_str()) { kids[pi].push(i); } }
    }
    let mut depth: Vec<Option<usize>> = vec![None; nodes.len()];
    fn d(i: usize, nodes: &[Node], map: &HashMap<&str, usize>, depth: &mut Vec<Option<usize>>, seen: &mut Vec<bool>) -> usize {
        if let Some(v) = depth[i] { return v; }
        if seen[i] { return 0; }
        seen[i] = true;
        let ps: Vec<usize> = nodes[i].parents.iter().filter_map(|p| map.get(p.as_str()).copied()).collect();
        let v = if ps.is_empty() { 0 } else { ps.iter().map(|&p| d(p, nodes, map, depth, seen)).max().unwrap() + 1 };
        depth[i] = Some(v);
        v
    }
    for i in 0..nodes.len() { let mut seen = vec![false; nodes.len()]; d(i, nodes, &map, &mut depth, &mut seen); }
    let mut depth: Vec<usize> = depth.into_iter().map(|v| v.unwrap_or(0)).collect();
    // i nodi senza genitori (requisiti da capitoli precedenti) vanno subito prima del primo figlio
    for (i, n) in nodes.iter().enumerate() {
        if n.parents.iter().any(|p| map.contains_key(p.as_str())) { continue; }
        if let Some(m) = kids[i].iter().map(|&k| depth[k]).min() { depth[i] = m.saturating_sub(1); }
    }
    depth
}

fn crossings(vs: &[V], l: &[usize]) -> usize {
    let mut e: Vec<(usize, usize)> = vec![];
    for &u in l { for &v in &vs[u].down { e.push((vs[u].o, vs[v].o)); } }
    e.sort();
    let mut c = 0;
    for i in 0..e.len() { for j in i + 1..e.len() { if e[j].0 > e[i].0 && e[j].1 < e[i].1 { c += 1; } } }
    c
}
fn total(vs: &[V], layers: &[Vec<usize>]) -> usize { layers.iter().map(|l| crossings(vs, l)).sum() }
fn set_order(vs: &mut [V], l: &[usize]) { for (i, &v) in l.iter().enumerate() { vs[v].o = i; } }

fn sweep(vs: &mut [V], layers: &mut [Vec<usize>], down: bool, median: bool) {
    let seq: Vec<usize> = if down { (1..layers.len()).collect() } else { (0..layers.len()).rev().skip(1).collect() };
    for k in seq {
        let l = layers[k].clone();
        for &v in &l {
            let (a, b) = if down { (&vs[v].up, &vs[v].down) } else { (&vs[v].down, &vs[v].up) };
            let nb = if !a.is_empty() { a } else { b };
            let key = if nb.is_empty() { vs[v].o as f64 } else {
                let mut s: Vec<f64> = nb.iter().map(|&u| vs[u].o as f64).collect();
                s.sort_by(|x, y| x.partial_cmp(y).unwrap());
                let m = s.len() >> 1;
                if median { if s.len() % 2 == 1 { s[m] } else { (s[m - 1] + s[m]) / 2.0 } } else { s.iter().sum::<f64>() / s.len() as f64 }
            };
            vs[v].key = key;
        }
        let mut l = l;
        l.sort_by(|&a, &b| vs[a].key.partial_cmp(&vs[b].key).unwrap().then(vs[a].o.cmp(&vs[b].o)));
        set_order(vs, &l);
        layers[k] = l;
    }
}

fn pair_cross(vs: &[V], u: usize, v: usize) -> usize {
    let mut c = 0;
    for side in 0..2 {
        let (a, b) = if side == 0 { (&vs[u].up, &vs[v].up) } else { (&vs[u].down, &vs[v].down) };
        for &x in a { for &y in b { if vs[x].o > vs[y].o { c += 1; } } }
    }
    c
}

fn transpose(vs: &mut [V], layers: &mut [Vec<usize>]) {
    let mut pass = 0;
    let mut again = true;
    while again && pass < 6 {
        again = false;
        for l in layers.iter_mut() {
            for i in 0..l.len().saturating_sub(1) {
                let (u, v) = (l[i], l[i + 1]);
                if pair_cross(vs, u, v) > pair_cross(vs, v, u) { l.swap(i, i + 1); set_order(vs, l); again = true; }
            }
        }
        pass += 1;
    }
}

fn order(vs: &mut [V], layers: &mut [Vec<usize>]) -> usize {
    let mut best: Option<Vec<Vec<usize>>> = None;
    let mut best_c = usize::MAX;
    for start in [1.0f64, -1.0] {
        for median in [true, false] {
            for l in layers.iter_mut() {
                l.sort_by(|&a, &b| (start * (vs[a].idx - vs[b].idx)).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| if start > 0.0 { vs[a].layer.cmp(&vs[b].layer) } else { vs[b].layer.cmp(&vs[a].layer) }));
                set_order(vs, l);
            }
            let mut it = 0;
            while it < 24 && best_c > 0 {
                sweep(vs, layers, it % 2 == 0, median);
                transpose(vs, layers);
                let c = total(vs, layers);
                if c < best_c { best_c = c; best = Some(layers.to_vec()); }
                it += 1;
            }
        }
    }
    if let Some(b) = best { for (k, l) in b.into_iter().enumerate() { set_order(vs, &l); layers[k] = l; } }
    if best_c == usize::MAX { 0 } else { best_c }
}

fn half(v: &V) -> f64 { if v.real { H / 2.0 } else { 0.0 } }
fn sep(a: &V, b: &V) -> f64 {
    half(a) + half(b) + if a.real && b.real { GY } else if a.real || b.real { LANE + 7.0 } else { LANE }
}

/// Regressione isotonica (pool adjacent violators): posizioni vicine ai desideri, stesso ordine, distanza minima
fn fit_layer(vs: &mut [V], l: &[usize], want: &[f64], weight: &[f64]) {
    let mut off = vec![0.0; l.len()];
    for i in 1..l.len() { off[i] = off[i - 1] + sep(&vs[l[i - 1]], &vs[l[i]]); }
    struct B { s: usize, e: usize, w: f64, m: f64 }
    let mut blocks: Vec<B> = vec![];
    for i in 0..l.len() {
        blocks.push(B { s: i, e: i, w: weight[i], m: want[i] - off[i] });
        while blocks.len() > 1 && blocks[blocks.len() - 2].m >= blocks[blocks.len() - 1].m {
            let b = blocks.pop().unwrap();
            let a = blocks.last_mut().unwrap();
            a.m = (a.m * a.w + b.m * b.w) / (a.w + b.w);
            a.w += b.w;
            a.e = b.e;
        }
    }
    for b in &blocks { for i in b.s..=b.e { vs[l[i]].y = b.m + off[i]; } }
}

fn vertical(vs: &mut [V], layers: &[Vec<usize>]) {
    for l in layers {
        let mut y = 0.0;
        for i in 0..l.len() { if i > 0 { y += sep(&vs[l[i - 1]], &vs[l[i]]); } vs[l[i]].y = y; }
    }
    for it in 0..16 {
        let mode = it % 3; // 0: verso i genitori, 1: verso i figli, 2: entrambi
        let seq: Vec<usize> = if mode == 1 { (0..layers.len()).rev().collect() } else { (0..layers.len()).collect() };
        for k in seq {
            let l = &layers[k];
            if l.is_empty() { continue; }
            let mut want = vec![];
            let mut weight = vec![];
            for &v in l {
                let nb: Vec<usize> = match mode { 0 => vs[v].up.clone(), 1 => vs[v].down.clone(), _ => vs[v].up.iter().chain(vs[v].down.iter()).copied().collect() };
                if nb.is_empty() { want.push(vs[v].y); weight.push(0.25); } else {
                    want.push(nb.iter().map(|&u| vs[u].y).sum::<f64>() / nb.len() as f64);
                    weight.push(if vs[v].real { 1.0 } else { 4.0 }); // le corsie stanno dritte
                }
            }
            fit_layer(vs, l, &want, &weight);
        }
    }
}

fn span(vs: &[V], u: usize) -> (f64, f64) {
    let ys = vs[u].down.iter().map(|&v| vs[v].y).chain(std::iter::once(vs[u].y));
    ys.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), y| (a.min(y), b.max(y)))
}
fn inside(y: f64, s: (f64, f64)) -> bool { y > s.0 + 0.5 && y < s.1 - 0.5 }
/// Incroci se la linea di "a" sta a sinistra di quella di "b"
fn left_cost(vs: &[V], a: usize, b: usize) -> usize {
    let (sa, sb) = (span(vs, a), span(vs, b));
    (if inside(vs[b].y, sa) { 1 } else { 0 }) + vs[a].down.iter().filter(|v| !vs[b].down.contains(v) && inside(vs[**v].y, sb)).count()
}
fn order_tracks(vs: &[V], buses: &mut Vec<usize>) {
    let n = buses.len();
    if n < 2 { return; }
    let c: Vec<Vec<i64>> = buses.iter().map(|&a| buses.iter().map(|&b| if a == b { 0 } else { left_cost(vs, a, b) as i64 }).collect()).collect();
    let score: Vec<i64> = (0..n).map(|i| (0..n).map(|j| c[i][j] - c[j][i]).sum()).collect();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| score[i].cmp(&score[j]).then(vs[buses[i]].y.partial_cmp(&vs[buses[j]].y).unwrap()));
    let mut pass = 0;
    let mut again = true;
    while again && pass < n {
        again = false;
        for k in 0..n - 1 { if c[idx[k]][idx[k + 1]] > c[idx[k + 1]][idx[k]] { idx.swap(k, k + 1); again = true; } }
        pass += 1;
    }
    *buses = idx.iter().map(|&i| buses[i]).collect();
}

pub fn layout(nodes: &[Node]) -> Layout {
    let depth = columns(nodes);
    let map: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let mut vs: Vec<V> = nodes.iter().enumerate().map(|(i, _)| V { layer: depth[i], real: true, idx: i as f64, up: vec![], down: vec![], y: 0.0, o: 0, key: 0.0, x: 0.0 }).collect();
    let mut chains: Vec<(usize, usize, Vec<usize>)> = vec![];
    for (ci, c) in nodes.iter().enumerate() {
        for pid in &c.parents {
            let Some(&pi) = map.get(pid.as_str()) else { continue };
            let mut chain = vec![pi];
            for l in depth[pi] + 1..depth[ci] {
                vs.push(V { layer: l, real: false, idx: f64::INFINITY, up: vec![], down: vec![], y: 0.0, o: 0, key: 0.0, x: 0.0 });
                chain.push(vs.len() - 1);
            }
            chain.push(ci);
            for w in chain.windows(2) { vs[w[0]].down.push(w[1]); vs[w[1]].up.push(w[0]); }
            chains.push((pi, ci, chain));
        }
    }
    let nl = vs.iter().map(|v| v.layer + 1).max().unwrap_or(0);
    let mut layers: Vec<Vec<usize>> = vec![vec![]; nl];
    for (i, v) in vs.iter().enumerate() { layers[v.layer].push(i); }

    let cross = order(&mut vs, &mut layers);
    vertical(&mut vs, &layers);

    // linee verticali tra le colonne: una per ogni nodo di partenza che cambia altezza
    struct Gap { track: HashMap<usize, usize>, n: usize, width: f64 }
    let gaps: Vec<Gap> = layers.iter().map(|l| {
        let mut buses: Vec<usize> = l.iter().copied().filter(|&u| vs[u].down.iter().any(|&v| (vs[v].y - vs[u].y).abs() > 0.5)).collect();
        order_tracks(&vs, &mut buses);
        let track = buses.iter().enumerate().map(|(i, &u)| (u, i)).collect();
        let n = buses.len();
        Gap { track, n, width: GAP_MIN.max(GAP_PAD * 2.0 + (n as f64 - 1.0) * TRACK) }
    }).collect();
    let mut col_x = vec![0.0; nl];
    let mut x = 0.0;
    for k in 0..nl { col_x[k] = x; x += W + gaps[k].width; }
    for v in vs.iter_mut() { v.x = col_x[v.layer]; }
    let track_x = |u: usize| {
        let gp = &gaps[vs[u].layer];
        vs[u].x + W / 2.0 + (gp.width - (gp.n as f64 - 1.0) * TRACK) / 2.0 + gp.track[&u] as f64 * TRACK
    };
    let edges = chains.iter().map(|(p, c, chain)| {
        let mut lines = vec![];
        let mut dots = vec![];
        for (i, &u) in chain.iter().enumerate() {
            let vu = &vs[u];
            if !vu.real { lines.push(vec![(vu.x - W / 2.0, vu.y), (vu.x + W / 2.0, vu.y)]); } // attraversa la colonna in corsia
            let Some(&v) = chain.get(i + 1) else { continue };
            let vv = &vs[v];
            let (sx, ex) = (vu.x + W / 2.0, vv.x - W / 2.0);
            if (vv.y - vu.y).abs() <= 0.5 { lines.push(vec![(sx, vu.y), (ex, vu.y)]); continue; }
            let tx = track_x(u);
            lines.push(vec![(sx, vu.y), (tx, vu.y), (tx, vv.y), (ex, vv.y)]);
            // pallino dove la linea verticale si dirama
            if vu.down.len() > 1 {
                let ys: Vec<f64> = vu.down.iter().map(|&w| vs[w].y).collect();
                let lo = ys.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                if vv.y > lo.min(vu.y) + 0.5 && vv.y < hi.max(vu.y) - 0.5 { dots.push((tx, vv.y)); }
                if vu.y > lo + 0.5 && vu.y < hi - 0.5 { dots.push((tx, vu.y)); }
            }
        }
        Edge { p: *p, c: *c, lines, dots }
    }).collect();
    let bounds = if nodes.is_empty() { None } else {
        let mut b = Bounds { x0: f64::INFINITY, x1: f64::NEG_INFINITY, y0: f64::INFINITY, y1: f64::NEG_INFINITY };
        for v in &vs { b.x0 = b.x0.min(v.x - W / 2.0); b.x1 = b.x1.max(v.x + W / 2.0); b.y0 = b.y0.min(v.y - half(v)); b.y1 = b.y1.max(v.y + half(v)); }
        Some(b)
    };
    Layout { pos: (0..nodes.len()).map(|i| (vs[i].x, vs[i].y)).collect(), edges, bounds, crossings: cross }
}
