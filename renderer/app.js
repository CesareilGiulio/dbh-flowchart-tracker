const NAMES = { C: "Connor", K: "Kara", M: "Markus" };
const KEY = "flowchart-tracker-v2";
const W = 204, H = 40, GX = 44, GY = 12;
const api = window.api || null;

let DATA = { chapters: {} }, S, cur = 1, curSec = 0, filter = "all";
let view = { x: 0, y: 0, k: 1 };

/* ---------- stato ---------- */
const uid = () => "u" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
function placeholder() { return { fromWiki: false, game: null, sections: [{ name: "", char: null, nodes: [] }] }; }
function fromData(n) {
  const d = DATA.chapters[n];
  if (!d || !d.sections || !d.sections.length) return null;
  return { fromWiki: true, rev: d.rev || null, manual: !!d.manual, game: null, sections: d.sections.map(s => ({ name: s.name, char: s.char,
    nodes: s.nodes.map(x => ({ id: x.id, title: x.title, en: x.en || "", type: x.type, note: x.note || "", parents: [...x.parents], unlocked: false })) })) };
}
function merge(next, old) {
  if (!old) return next;
  const ids = new Set(), titles = new Set();
  old.sections.forEach(s => s.nodes.forEach(x => { if (x.unlocked) { ids.add(x.id); titles.add(s.name + "|" + x.title.toLowerCase()); } }));
  next.sections.forEach(s => s.nodes.forEach(x => { x.unlocked = ids.has(x.id) || titles.has(s.name + "|" + x.title.toLowerCase()); }));
  next.game = old.game ?? null;
  return next;
}
function isUntouchedPlaceholder(c) { return !c.fromWiki && c.sections.length === 1 && c.sections[0].nodes.length <= 1; }
function load() { try { const r = localStorage.getItem(KEY); if (r) { const d = JSON.parse(r); if (d && d.v === 2) return d; } } catch (e) {} return null; }
function save() { try { localStorage.setItem(KEY, JSON.stringify(S)); } catch (e) {} }
function applyData(replaceAll) {
  CHAPTERS.forEach(([n]) => {
    const c = S.chapters[n], f = fromData(n);
    if (!c) S.chapters[n] = f || placeholder();
    else if (f && (replaceAll || isUntouchedPlaceholder(c) || (f.rev && f.rev !== c.rev))) S.chapters[n] = merge(f, c);
  });
}

const chap = () => S.chapters[cur];
const sec = () => chap().sections[curSec] || chap().sections[0];
const byId = id => sec().nodes.find(n => n.id === id);

function chStats(c) {
  let t = 0, u = 0;
  c.sections.forEach(s => { t += s.nodes.length; u += s.nodes.filter(n => n.unlocked).length; });
  const calc = t ? Math.round(u / t * 100) : 0;
  const shown = (c.game !== null && c.game !== "" && c.game !== undefined) ? Number(c.game) : calc;
  return { t, u, calc, shown };
}

/* ---------- layout: da sinistra a destra ---------- */
function layout(nodes) {
  const map = new Map(nodes.map(n => [n.id, n])), depth = new Map();
  const d = (n, seen = new Set()) => {
    if (depth.has(n.id)) return depth.get(n.id);
    if (seen.has(n.id)) return 0;
    seen.add(n.id);
    const ps = n.parents.map(p => map.get(p)).filter(Boolean);
    const v = ps.length ? Math.max(...ps.map(p => d(p, seen))) + 1 : 0;
    depth.set(n.id, v); return v;
  };
  nodes.forEach(n => d(n));
  const cols = [];
  nodes.forEach((n, i) => { const k = depth.get(n.id); (cols[k] = cols[k] || []).push({ n, i }); });
  const pos = new Map(); let root = 0;
  const step = H + GY;
  cols.forEach((L, k) => {
    if (!L) return;
    L.forEach(o => {
      const py = o.n.parents.map(p => pos.get(p)).filter(Boolean).map(p => p.y);
      o.b = py.length ? py.reduce((a, b) => a + b, 0) / py.length : (root++) * step * 3;
    });
    L.sort((a, b) => a.b - b.b || a.i - b.i);
    const mean = L.reduce((a, o) => a + o.b, 0) / L.length, span = (L.length - 1) * step;
    L.forEach((o, j) => pos.set(o.n.id, { x: k * (W + GX), y: mean - span / 2 + j * step }));
  });
  return pos;
}

/* ---------- rendering ---------- */
const $ = s => document.querySelector(s);
const NS = "http://www.w3.org/2000/svg";
function el(tag, attrs, parent) { const e = document.createElementNS(NS, tag); for (const k in attrs) e.setAttribute(k, attrs[k]); parent && parent.appendChild(e); return e; }
function fit2(t, max = 28) {
  const words = String(t).split(/\s+/), lines = [""];
  words.forEach(w => { const l = lines[lines.length - 1]; if ((l + " " + w).trim().length > max && l) lines.push(w); else lines[lines.length - 1] = (l + " " + w).trim(); });
  if (lines.length > 2) { lines.length = 2; lines[1] = lines[1].slice(0, max - 1) + "…"; }
  return lines;
}
const hasData = n => !!(DATA.chapters && DATA.chapters[n]);

function renderGraph() {
  const nodes = hasData(cur) ? sec().nodes : [], pos = layout(nodes);
  const E = $("#edges"), N = $("#nodes"); E.innerHTML = ""; N.innerHTML = "";
  $("#empty").hidden = nodes.length > 0;
  nodes.forEach(n => {
    const b = pos.get(n.id);
    n.parents.forEach(pid => {
      const p = pos.get(pid), pn = byId(pid); if (!p || !pn) return;
      const x1 = p.x + W, x2 = b.x, mx = x2 - GX / 2;
      el("path", { d: `M${x1},${p.y}H${mx}V${b.y}H${x2}`, class: "edge" + (pn.unlocked && n.unlocked ? " lit" : "") }, E);
    });
  });
  nodes.forEach(n => {
    const b = pos.get(n.id), hide = !n.unlocked && S.spoiler;
    const g = el("g", { class: `node ${n.type}${n.unlocked ? " on" : ""}${hide ? " locked" : ""}`, transform: `translate(${b.x},${b.y - H / 2})`, tabindex: 0, role: "button" }, N);
    if (n.type === "ending") el("rect", { x: -3, y: -3, width: W + 6, height: H + 6, class: "outer", rx: 3 }, g);
    el("rect", { width: W, height: H, rx: n.type === "checkpoint" ? H / 2 : 2 }, g);
    const lines = hide ? ["—"] : fit2(n.type === "objective" ? n.title.toUpperCase() : n.title);
    const t = el("text", { x: n.type === "checkpoint" ? 14 : 10 }, g);
    lines.forEach((l, i) => { const s = el("tspan", { x: n.type === "checkpoint" ? 14 : 10, y: H / 2 + 4 + (i - (lines.length - 1) / 2) * 15 }, t); s.textContent = l; });
    g.addEventListener("click", e => { e.stopPropagation(); toggle(n); });
    g.addEventListener("keydown", e => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggle(n); } });
    g.addEventListener("mouseenter", () => showInfo(n, hide));
    g.addEventListener("mouseleave", () => showInfo(null));
  });
  applyView();
}

function showInfo(n, hide) {
  const box = $("#info"); box.textContent = "";
  if (!n || hide) return;
  const b = document.createElement("b"); b.textContent = n.title; box.appendChild(b);
  if (n.en) box.append(" · " + n.en);
  if (n.note) { box.appendChild(document.createElement("br")); box.append(n.note); }
}

function renderSide() {
  const list = $("#chlist"); list.innerHTML = "";
  let full = 0;
  CHAPTERS.forEach(([n, t, who]) => {
    const s = chStats(S.chapters[n]); if (s.shown >= 100) full++;
    if (filter !== "all" && !who.includes(filter)) return;
    const li = document.createElement("li"), b = document.createElement("button");
    b.className = "ch" + (n === cur ? " sel" : "") + (s.shown >= 100 ? " full" : "") + (hasData(n) ? "" : " none");
    b.innerHTML = `<span class="n">${n}</span><span class="t"></span><span class="p">${hasData(n) || s.shown ? s.shown + "%" : ""}</span>`;
    b.querySelector(".t").textContent = t;
    b.onclick = () => { cur = n; curSec = 0; renderAll(); fit(); };
    li.appendChild(b); list.appendChild(li);
  });
  $("#ovCh").textContent = `${full} / 32`;
}

function tabRow(items, cls) {
  const r = document.createElement("div"); r.className = "tabs" + (cls ? " " + cls : "");
  items.forEach(([label, on, fn, extra]) => {
    const b = document.createElement("button"); b.textContent = label; if (on) b.className = "on";
    if (extra) { const sm = document.createElement("small"); sm.textContent = extra; b.appendChild(sm); }
    b.onclick = fn; r.appendChild(b);
  });
  return r;
}
function renderTop() {
  const [, t] = CHAPTERS[cur - 1], s = chStats(chap());
  $("#chTitle").textContent = t;
  $("#chPct").textContent = s.calc + "%";
  $("#gamePct").value = chap().game ?? "";
  $("#spoilBtn").textContent = S.spoiler ? "Mostra nomi bloccati" : "Nascondi nomi bloccati";
  const box = $("#charSel"); box.innerHTML = "";
  const secs = chap().sections;
  if (!hasData(cur) || secs.length < 2) return;
  const groups = [];
  secs.forEach((x, i) => { const k = x.char || ""; let g = groups.find(g => g.k === k); if (!g) groups.push(g = { k, items: [] }); g.items.push(i); });
  const order = { K: 0, C: 1, M: 2, "": 3 }; groups.sort((a, b) => order[a.k] - order[b.k]);
  const active = sec().char || "", go = i => () => { curSec = i; renderAll(); fit(); };
  if (groups.length > 1) box.appendChild(tabRow(groups.map(g => [g.k ? NAMES[g.k] : "Altro", g.k === active, go(g.items[0])])));
  const g = groups.find(g => g.k === active);
  if (g && g.items.length > 1) box.appendChild(tabRow(g.items.map((i, j) => {
    const x = secs[i], label = (x.name || "").replace(/\b(Kara|Connor|Markus)(['’]s)?\b/gi, "").trim() || `Percorso ${j + 1}`;
    return [label, i === curSec, go(i), `${x.nodes.filter(n => n.unlocked).length}/${x.nodes.length}`];
  }), "sub"));
}

function renderAll() { renderSide(); renderTop(); renderGraph(); }
function toggle(n) { n.unlocked = !n.unlocked; save(); renderSide(); renderTop(); renderGraph(); }

/* ---------- pan e zoom ---------- */
const svg = $("#svg"), world = $("#world");
function applyView() { world.setAttribute("transform", `translate(${view.x},${view.y}) scale(${view.k})`); }
function fit() {
  const nodes = hasData(cur) ? sec().nodes : [], pos = layout(nodes);
  const cw = svg.clientWidth, chh = svg.clientHeight;
  if (!pos.size) { view = { x: 40, y: chh / 2, k: 1 }; applyView(); return; }
  let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
  pos.forEach(p => { x0 = Math.min(x0, p.x); x1 = Math.max(x1, p.x + W); y0 = Math.min(y0, p.y - H); y1 = Math.max(y1, p.y + H); });
  const k = Math.min(1, Math.max(.85, Math.min((cw - 80) / (x1 - x0), (chh - 80) / (y1 - y0))));
  view = { k, x: Math.max(40, (cw - (x1 - x0) * k) / 2) - x0 * k, y: chh / 2 - (y0 + y1) / 2 * k };
  applyView();
}
function zoomAt(f, mx = svg.clientWidth / 2, my = svg.clientHeight / 2) {
  const k = Math.min(2.5, Math.max(.15, view.k * f));
  view.x = mx - (mx - view.x) * k / view.k; view.y = my - (my - view.y) * k / view.k; view.k = k; applyView();
}
let drag = null;
svg.addEventListener("pointerdown", e => { if (e.target.closest(".node")) return; drag = { x: e.clientX, y: e.clientY, vx: view.x, vy: view.y }; svg.setPointerCapture(e.pointerId); svg.classList.add("panning"); });
svg.addEventListener("pointermove", e => { if (!drag) return; view.x = drag.vx + e.clientX - drag.x; view.y = drag.vy + e.clientY - drag.y; applyView(); });
svg.addEventListener("pointerup", () => { drag = null; svg.classList.remove("panning"); });
svg.addEventListener("wheel", e => {
  e.preventDefault();
  if (e.ctrlKey || e.metaKey) { const r = svg.getBoundingClientRect(); zoomAt(Math.exp(-e.deltaY * .002), e.clientX - r.left, e.clientY - r.top); }
  else { view.x -= e.shiftKey ? e.deltaY : e.deltaX; view.y -= e.shiftKey ? 0 : e.deltaY; applyView(); }
}, { passive: false });

/* ---------- controlli ---------- */
$("#zoomIn").onclick = () => zoomAt(1.2);
$("#zoomOut").onclick = () => zoomAt(1 / 1.2);
$("#fitBtn").onclick = fit;
$("#spoilBtn").onclick = () => { S.spoiler = !S.spoiler; save(); renderAll(); };
$("#gamePct").oninput = e => { const v = e.target.value; chap().game = v === "" ? null : Math.max(0, Math.min(100, Number(v))); save(); renderSide(); };
$("#filters").onclick = e => { const b = e.target.closest("button"); if (!b) return; filter = b.dataset.f; document.querySelectorAll("#filters button").forEach(x => x.classList.toggle("on", x === b)); renderSide(); };
$("#exportBtn").onclick = () => {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([JSON.stringify(S, null, 1)], { type: "application/json" }));
  a.download = "flowchart-tracker-backup.json"; a.click(); setTimeout(() => URL.revokeObjectURL(a.href), 1000);
};
$("#importBtn").onclick = () => $("#importFile").click();
$("#importFile").onchange = e => {
  const f = e.target.files[0]; if (!f) return;
  const r = new FileReader();
  r.onload = () => {
    try { const d = JSON.parse(r.result); if (!d.chapters || d.v !== 2) throw 0; S = d; applyData(false); curSec = 0; save(); renderAll(); fit(); }
    catch (err) { alert("Il file non è un backup valido."); }
  };
  r.readAsText(f); e.target.value = "";
};
window.addEventListener("resize", fit);

(async function init() {
  if (api) { try { DATA = await api.getData(); } catch (e) {} }
  S = load() || { v: 2, chapters: {}, spoiler: false };
  applyData(false); save();
  renderAll(); requestAnimationFrame(fit);
})();
