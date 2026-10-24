// Disposizione dei diagrammi da sinistra a destra (metodo a livelli, "Sugiyama"):
// 1. colonne: ogni nodo va una colonna dopo il più lontano dei suoi genitori;
// 2. i collegamenti che saltano colonne ricevono una corsia propria (nodi fittizi),
//    così non passano mai dietro a una casella;
// 3. l'ordine dentro ogni colonna è scelto per ridurre gli incroci;
// 4. la posizione verticale avvicina ogni nodo ai suoi vicini senza sovrapposizioni;
// 5. tra due colonne ogni nodo di partenza ha la sua linea verticale.
const Layout = (() => {
  const W = 204, H = 40, GY = 12;       // casella e spazio verticale tra caselle
  const LANE = 9;                       // spazio tra corsie di collegamenti lunghi
  const TRACK = 7, GAP_MIN = 44, GAP_PAD = 14;

  function columns(nodes) {
    const map = new Map(nodes.map(n => [n.id, n])), depth = new Map(), kids = new Map();
    nodes.forEach(n => n.parents.forEach(p => { if (map.has(p)) (kids.get(p) || kids.set(p, []).get(p)).push(n.id); }));
    const d = (n, seen = new Set()) => {
      if (depth.has(n.id)) return depth.get(n.id);
      if (seen.has(n.id)) return 0;
      seen.add(n.id);
      const ps = n.parents.map(p => map.get(p)).filter(Boolean);
      const v = ps.length ? Math.max(...ps.map(p => d(p, seen))) + 1 : 0;
      depth.set(n.id, v); return v;
    };
    nodes.forEach(n => d(n));
    // i nodi senza genitori (requisiti da capitoli precedenti) vanno subito prima del primo figlio
    nodes.forEach(n => {
      if (n.parents.some(p => map.has(p))) return;
      const k = (kids.get(n.id) || []).map(id => depth.get(id));
      if (k.length) depth.set(n.id, Math.max(0, Math.min(...k) - 1));
    });
    return { map, depth };
  }

  // Grafo a livelli con i nodi fittizi; ogni collegamento originale diventa una catena di segmenti
  function build(nodes) {
    const { map, depth } = columns(nodes);
    const V = [], byId = new Map();
    const add = (id, layer, real, idx) => { const v = { id, layer, real, idx, up: [], down: [], y: 0 }; V.push(v); byId.set(id, v); return v; };
    nodes.forEach((n, i) => add(n.id, depth.get(n.id), true, i));
    const edges = [];
    let dn = 0;
    nodes.forEach(c => c.parents.forEach(pid => {
      if (!map.has(pid)) return;
      const chain = [byId.get(pid)];
      for (let l = depth.get(pid) + 1; l < depth.get(c.id); l++) chain.push(add("~" + dn++, l, false, Infinity));
      chain.push(byId.get(c.id));
      for (let i = 0; i + 1 < chain.length; i++) { chain[i].down.push(chain[i + 1]); chain[i + 1].up.push(chain[i]); }
      edges.push({ p: pid, c: c.id, chain });
    }));
    const layers = [];
    V.forEach(v => (layers[v.layer] = layers[v.layer] || []).push(v));
    for (let i = 0; i < layers.length; i++) layers[i] = layers[i] || [];
    return { V, byId, layers, edges };
  }

  /* ---------- ordine nelle colonne ---------- */
  const setOrder = L => L.forEach((v, i) => { v.o = i; });
  function crossings(L) {   // incroci tra la colonna L e la successiva
    const e = [];
    L.forEach(u => u.down.forEach(v => e.push([u.o, v.o])));
    e.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    let c = 0;
    for (let i = 0; i < e.length; i++) for (let j = i + 1; j < e.length; j++) if (e[j][0] > e[i][0] && e[j][1] < e[i][1]) c++;
    return c;
  }
  const total = layers => layers.reduce((s, L) => s + crossings(L), 0);
  function sweep(layers, down, useMedian) {
    const seq = down ? layers.map((_, i) => i).slice(1) : layers.map((_, i) => i).reverse().slice(1);
    seq.forEach(k => {
      const L = layers[k];
      L.forEach(v => {
        // i nodi senza vicini da quel lato guardano l'altro lato (es. i requisiti)
        const nb = (down ? v.up : v.down).length ? (down ? v.up : v.down) : (down ? v.down : v.up);
        if (!nb.length) { v.key = v.o; return; }
        const s = nb.map(u => u.o).sort((a, b) => a - b), m = s.length >> 1;
        v.key = useMedian ? (s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2) : s.reduce((a, b) => a + b, 0) / s.length;
      });
      L.sort((a, b) => a.key - b.key || a.o - b.o);
      setOrder(L);
    });
  }
  // scambia caselle vicine quando lo scambio riduce gli incroci con le colonne accanto
  function pairCross(u, v) {
    let c = 0;
    for (const side of ["up", "down"]) for (const a of u[side]) for (const b of v[side]) if (a.o > b.o) c++;
    return c;
  }
  function transpose(layers) {
    for (let pass = 0, again = true; again && pass < 6; pass++) {
      again = false;
      layers.forEach(L => {
        for (let i = 0; i + 1 < L.length; i++) {
          const u = L[i], v = L[i + 1];
          if (pairCross(u, v) > pairCross(v, u)) { L[i] = v; L[i + 1] = u; setOrder(L); again = true; }
        }
      });
    }
  }
  // Varie strategie (mediana o media, partendo dall'ordine dei dati o a rovescio):
  // si tiene l'ordine con meno incroci
  function order(layers) {
    let best = null, bestC = Infinity;
    for (const start of [1, -1]) for (const useMedian of [true, false]) {
      layers.forEach(L => { L.sort((a, b) => start * (a.idx - b.idx) || start * (a.layer - b.layer)); setOrder(L); });
      for (let it = 0; it < 24 && bestC > 0; it++) {
        sweep(layers, it % 2 === 0, useMedian); transpose(layers);
        const c = total(layers);
        if (c < bestC) { bestC = c; best = layers.map(L => L.slice()); }
      }
    }
    best.forEach((L, k) => { layers[k] = L; setOrder(L); });
    return bestC;
  }

  /* ---------- posizione verticale ---------- */
  const half = v => v.real ? H / 2 : 0;
  const sep = (a, b) => half(a) + half(b) + (a.real && b.real ? GY : a.real || b.real ? LANE + 7 : LANE);
  // Regressione isotonica (pool adjacent violators): posizioni il più vicine possibile ai
  // desideri, nello stesso ordine e alla distanza minima
  function fitLayer(L, want, weight) {
    const off = [0];
    for (let i = 1; i < L.length; i++) off[i] = off[i - 1] + sep(L[i - 1], L[i]);
    const blocks = [];
    L.forEach((v, i) => {
      blocks.push({ s: i, e: i, w: weight[i], m: want[i] - off[i] });
      while (blocks.length > 1 && blocks[blocks.length - 2].m >= blocks[blocks.length - 1].m) {
        const b = blocks.pop(), a = blocks[blocks.length - 1];
        a.m = (a.m * a.w + b.m * b.w) / (a.w + b.w); a.w += b.w; a.e = b.e;
      }
    });
    blocks.forEach(b => { for (let i = b.s; i <= b.e; i++) L[i].y = b.m + off[i]; });
  }
  function vertical(layers) {
    layers.forEach(L => { let y = 0; L.forEach((v, i) => { if (i) y += sep(L[i - 1], v); v.y = y; }); });
    const mean = a => a.reduce((s, u) => s + u.y, 0) / a.length;
    for (let it = 0; it < 16; it++) {
      const mode = it % 3;   // 0: verso i genitori, 1: verso i figli, 2: entrambi
      const seq = mode === 1 ? layers.map((_, i) => i).reverse() : layers.map((_, i) => i);
      seq.forEach(k => {
        const L = layers[k];
        if (!L.length) return;
        const want = [], weight = [];
        L.forEach(v => {
          const nb = mode === 0 ? v.up : mode === 1 ? v.down : v.up.concat(v.down);
          want.push(nb.length ? mean(nb) : v.y);
          weight.push(nb.length ? (v.real ? 1 : 4) : 0.25);   // le corsie stanno dritte
        });
        fitLayer(L, want, weight);
      });
    }
  }

  /* ---------- ordine delle linee verticali tra due colonne ---------- */
  // Incroci se la linea di "a" sta a sinistra di quella di "b": la verticale di a taglia
  // l'uscita orizzontale di b, e la verticale di b taglia gli ingressi di a
  function span(u) { const ys = u.down.map(v => v.y).concat(u.y); return [Math.min(...ys), Math.max(...ys)]; }
  const inside = (y, s) => y > s[0] + 0.5 && y < s[1] - 0.5;
  function leftCost(a, b) {
    const sa = span(a), sb = span(b);
    return (inside(b.y, sa) ? 1 : 0) + a.down.filter(v => !b.down.includes(v) && inside(v.y, sb)).length;
  }
  function orderTracks(buses) {
    if (buses.length < 2) return;
    const n = buses.length, C = buses.map(a => buses.map(b => a === b ? 0 : leftCost(a, b)));
    // prima stima: quante volte conviene stare a sinistra, poi scambi tra vicini finché migliora
    const idx = buses.map((_, i) => i);
    const score = i => idx.reduce((s, j) => s + C[i][j] - C[j][i], 0);
    idx.sort((i, j) => score(i) - score(j) || buses[i].y - buses[j].y);
    for (let pass = 0, again = true; again && pass < n; pass++) {
      again = false;
      for (let k = 0; k + 1 < n; k++) if (C[idx[k]][idx[k + 1]] > C[idx[k + 1]][idx[k]]) { [idx[k], idx[k + 1]] = [idx[k + 1], idx[k]]; again = true; }
    }
    const sorted = idx.map(i => buses[i]);
    buses.splice(0, n, ...sorted);
  }

  /* ---------- coordinate e percorsi ---------- */
  function layout(nodes) {
    const g = build(nodes), layers = g.layers;
    const crossCount = order(layers);
    vertical(layers);
    // linee verticali tra le colonne: una per ogni nodo di partenza che cambia altezza
    const gaps = layers.map(L => {
      const buses = L.filter(u => u.down.some(v => Math.abs(v.y - u.y) > 0.5));
      orderTracks(buses);
      const track = new Map(); buses.forEach((u, i) => track.set(u, i));
      return { track, n: buses.length, width: Math.max(GAP_MIN, GAP_PAD * 2 + (buses.length - 1) * TRACK) };
    });
    const colX = []; let x = 0;
    layers.forEach((L, k) => { colX[k] = x; x += W + gaps[k].width; });
    const pos = new Map();
    g.V.forEach(v => { v.x = colX[v.layer]; if (v.real) pos.set(v.id, { x: v.x, y: v.y }); });
    const trackX = u => {
      const gp = gaps[u.layer], i = gp.track.get(u);
      return u.x + W / 2 + (gp.width - (gp.n - 1) * TRACK) / 2 + i * TRACK;
    };
    const edges = g.edges.map(e => {
      let d = "";
      const dots = [];
      e.chain.forEach((u, i) => {
        if (!u.real) d += `M${u.x - W / 2},${u.y}H${u.x + W / 2}`;       // attraversa la colonna in corsia
        const v = e.chain[i + 1]; if (!v) return;
        const sx = u.x + W / 2, ex = v.x - W / 2;
        if (Math.abs(v.y - u.y) <= 0.5) { d += `M${sx},${u.y}H${ex}`; return; }
        const tx = trackX(u);
        d += `M${sx},${u.y}H${tx}V${v.y}H${ex}`;
        // pallino dove la linea verticale si dirama verso questo nodo
        const ys = u.down.map(w => w.y);
        if (u.down.length > 1 && v.y > Math.min(...ys, u.y) + 0.5 && v.y < Math.max(...ys, u.y) - 0.5) dots.push([tx, v.y]);
        if (u.down.length > 1 && u.y > Math.min(...ys) + 0.5 && u.y < Math.max(...ys) - 0.5) dots.push([tx, u.y]);
      });
      return { p: e.p, c: e.c, d, dots };
    });
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
    g.V.forEach(v => { x0 = Math.min(x0, v.x - W / 2); x1 = Math.max(x1, v.x + W / 2); y0 = Math.min(y0, v.y - half(v)); y1 = Math.max(y1, v.y + half(v)); });
    return { pos, edges, bounds: nodes.length ? { x0, x1, y0, y1 } : null, crossings: crossCount };
  }

  return { layout, W, H };
})();
if (typeof module !== "undefined") module.exports = Layout;
