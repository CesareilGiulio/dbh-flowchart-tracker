const NAMES = { C: "Connor", K: "Kara", M: "Markus" };
const KEY = "flowchart-tracker-v2";
const W = Layout.W, H = Layout.H, MAXCH = 28;
const ZMIN = 0.3, ZMAX = 2, ZFIT = 0.85;
const bridge = window.api || null;

let DATA = { chapters: {} }, S, cur = 1, curSec = 0, filter = "all";
let view = { x: 0, y: 0, k: 1 }, bounds = null;
let nodeEls = new Map(), edgeEls = [];  // elementi del diagramma visibile, per aggiornarli senza ridisegnare

/* ---------- lingua ---------- */
const T = () => STRINGS[S.lang] || STRINGS.it;
const label = n => (S.lang === "en" ? (n.en || n.title) : (n.title || n.en)) || "";
const otherLabel = n => { const o = S.lang === "en" ? n.title : n.en; return o && o !== label(n) ? o : ""; };
const noteOf = n => S.lang === "en" ? (n.note_en || n.note || "") : (n.note || "");
function applyLang() {
  document.documentElement.lang = S.lang;
  document.querySelectorAll("[data-i18n]").forEach(e => { e.textContent = T()[e.dataset.i18n]; });
  document.querySelectorAll("#langSel button").forEach(b => b.classList.toggle("on", b.dataset.lang === S.lang));
}

/* ---------- stato (formato di salvataggio invariato) ---------- */
function placeholder() { return { fromWiki: false, game: null, sections: [{ name: "", char: null, nodes: [] }] }; }
function fromData(n) {
  const d = DATA.chapters[n];
  if (!d || !d.sections || !d.sections.length) return null;
  return { fromWiki: true, rev: d.rev || null, manual: !!d.manual, game: null, sections: d.sections.map(s => ({ name: s.name, char: s.char,
    nodes: s.nodes.map(x => ({ id: x.id, title: x.title, en: x.en || "", type: x.type, note: x.note || "", note_en: x.note_en || "", parents: [...x.parents], unlocked: false })) })) };
}
function merge(next, old) {
  if (!old) return next;
  const ids = new Set(), titles = new Set();
  old.sections.forEach(s => s.nodes.forEach(x => { if (x.unlocked) { ids.add(x.id); titles.add(s.name + "|" + String(x.title).toLowerCase()); } }));
  next.sections.forEach(s => s.nodes.forEach(x => { x.unlocked = ids.has(x.id) || titles.has(s.name + "|" + String(x.title).toLowerCase()); }));
  next.game = old.game ?? null;
  return next;
}
function isUntouchedPlaceholder(c) { return !c.fromWiki && c.sections.length === 1 && c.sections[0].nodes.length <= 1; }
// Aggiunge ai nodi salvati i campi di testo arrivati dopo (es. note_en)
function fillMissingText(c, f) {
  const src = new Map();
  f.sections.forEach(s => s.nodes.forEach(x => src.set(x.id, x)));
  c.sections.forEach(s => s.nodes.forEach(x => {
    const d = src.get(x.id); if (!d) return;
    if (!x.en && d.en) x.en = d.en;
    if (!x.note_en && d.note_en) x.note_en = d.note_en;
  }));
}
function load() { try { const r = localStorage.getItem(KEY); if (r) { const d = JSON.parse(r); if (d && d.v === 2) return d; } } catch (e) {} return null; }
let saveTimer = null;
function save() { clearTimeout(saveTimer); saveTimer = null; try { localStorage.setItem(KEY, JSON.stringify(S)); } catch (e) {} }
// I clic ravvicinati producono un solo salvataggio invece di riscrivere tutto a ogni clic
function scheduleSave() { clearTimeout(saveTimer); saveTimer = setTimeout(save, 400); }
const flushSave = () => { if (saveTimer) save(); };
window.addEventListener("beforeunload", flushSave);
document.addEventListener("visibilitychange", () => { if (document.hidden) flushSave(); });
function applyData(replaceAll) {
  CHAPTERS.forEach(([n]) => {
    const c = S.chapters[n], f = fromData(n);
    if (!c) S.chapters[n] = f || placeholder();
    else if (f && (replaceAll || isUntouchedPlaceholder(c) || (f.rev && f.rev !== c.rev))) S.chapters[n] = merge(f, c);
    else if (f) fillMissingText(c, f);
  });
}

const chap = () => S.chapters[cur];
const sec = () => chap().sections[curSec] || chap().sections[0];
const hasData = c => !!c.fromWiki && c.sections.some(s => s.nodes.length);

function chStats(c) {
  let t = 0, u = 0;
  c.sections.forEach(s => { t += s.nodes.length; u += s.nodes.filter(n => n.unlocked).length; });
  const calc = t ? Math.round(u / t * 100) : 0;
  const shown = (c.game !== null && c.game !== "" && c.game !== undefined) ? Number(c.game) : calc;
  return { t, u, calc, shown };
}

/* ---------- disposizione: vedi layout.js; si calcola una volta per percorso ---------- */
const layoutCache = new WeakMap();
function layoutOf(nodes) {
  let L = layoutCache.get(nodes);
  if (!L) { L = Layout.layout(nodes); layoutCache.set(nodes, L); }
  return L;
}

/* ---------- disegno ---------- */
const $ = s => document.querySelector(s);
const NS = "http://www.w3.org/2000/svg";
function el(tag, attrs = {}, parent) { const e = document.createElementNS(NS, tag); for (const k in attrs) e.setAttribute(k, attrs[k]); if (parent) parent.appendChild(e); return e; }
function wrap(t, max = MAXCH) {
  const lines = [""];
  String(t || "").split(/\s+/).forEach(w => {
    const l = lines[lines.length - 1];
    if ((l + " " + w).trim().length > max && l) lines.push(w); else lines[lines.length - 1] = (l + " " + w).trim();
  });
  if (lines.length > 2) { lines.length = 2; lines[1] = lines[1].slice(0, max - 1).trim() + "…"; }
  else if (lines[lines.length - 1].length > max) lines[lines.length - 1] = lines[lines.length - 1].slice(0, max - 1) + "…";
  return lines;
}
const hidden = n => !n.unlocked && S.spoiler;

function renderGraph() {
  const E = $("#edges"), N = $("#nodes"); E.replaceChildren(); N.replaceChildren();
  nodeEls = new Map(); edgeEls = [];
  const empty = !hasData(chap());
  $("#empty").hidden = !empty;
  if (empty) { bounds = null; return; }
  const nodes = sec().nodes, L = layoutOf(nodes), pos = L.pos, map = new Map(nodes.map(n => [n.id, n]));
  bounds = L.bounds;
  // prima le linee spente, poi quelle accese, così le accese restano sopra
  const lit = e => map.get(e.p).unlocked && map.get(e.c).unlocked;
  [...L.edges].sort((a, b) => lit(a) - lit(b)).forEach(x => {
    const g = el("g", { class: "edge" + (lit(x) ? " lit" : "") }, E);
    el("path", { d: x.d }, g);
    x.dots.forEach(([cx, cy]) => el("circle", { cx, cy, r: 2.2 }, g));
    edgeEls.push({ e: g, p: map.get(x.p), c: map.get(x.c) });
  });
  nodes.forEach(n => {
    const b = pos.get(n.id);
    const g = el("g", { transform: `translate(${b.x},${b.y})`, tabindex: 0, role: "checkbox" }, N);
    el("rect", { class: "box", x: -W / 2, y: -H / 2, width: W, height: H, rx: n.type === "checkpoint" ? H / 2 : 2 }, g);
    if (n.type === "ending") el("rect", { class: "inner", x: -W / 2 + 3, y: -H / 2 + 3, width: W - 6, height: H - 6, rx: 1 }, g);
    el("text", { "text-anchor": "middle" }, g);
    paintNode(g, n); nodeEls.set(n.id, g);
    g.addEventListener("click", () => toggle(n));
    g.addEventListener("keydown", e => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggle(n); } });
    g.addEventListener("mouseenter", () => showInfo(n));
    g.addEventListener("focus", () => showInfo(n));
    g.addEventListener("mouseleave", hideInfo);
    g.addEventListener("blur", hideInfo);
  });
  applyView();
}

// Stato visivo di un nodo: classe, testo (nascosto se bloccato e spoiler attivo), accessibilità
function paintNode(g, n) {
  const hide = hidden(n);
  g.setAttribute("class", `node ${n.type}${n.unlocked ? " on" : ""}${hide ? " hidden" : ""}`);
  g.setAttribute("aria-checked", n.unlocked ? "true" : "false");
  g.setAttribute("aria-label", hide ? T().hiddenName : label(n));
  const tx = g.querySelector("text"), lines = hide ? [T().hiddenName] : wrap(label(n));
  tx.replaceChildren();
  lines.forEach((l, i) => { const s = el("tspan", { x: 0, y: (i - (lines.length - 1) / 2) * 15 + 4.5 }, tx); s.textContent = l; });
}
function showInfo(n) {
  if (hidden(n)) { $("#infoTitle").textContent = T().hiddenName; $("#infoNote").textContent = ""; }
  else {
    $("#infoTitle").textContent = label(n) + (otherLabel(n) ? " · " + otherLabel(n) : "");
    $("#infoNote").textContent = noteOf(n);
  }
  $("#info").hidden = false;
}
function hideInfo() { $("#info").hidden = true; }

function variantLabel(s, j) {
  const base = (s.name || "").includes(" - ") ? s.name.split(" - ").slice(1).join(" - ") : (s.name || "");
  const n = base.replace(/\b(Kara|Connor|Markus)(['’]s)?\b/gi, "").replace(/\s+/g, " ").trim();
  return n ? n.charAt(0).toUpperCase() + n.slice(1) : T().path(j + 1);
}
function tab(text, on, onclick, small) {
  const b = document.createElement("button");
  b.className = on ? "on" : ""; b.textContent = text; b.onclick = onclick;
  if (small) { const s = document.createElement("small"); s.textContent = small; b.appendChild(s); }
  return b;
}
function renderTabs() {
  const ct = $("#charTabs"), pt = $("#pathTabs"); ct.replaceChildren(); pt.replaceChildren();
  const secs = chap().sections;
  if (secs.length < 2) return;
  const groups = [];
  secs.forEach((s, i) => { const k = s.char || ""; let g = groups.find(g => g.k === k); if (!g) groups.push(g = { k, items: [] }); g.items.push(i); });
  const order = { K: 0, C: 1, M: 2, "": 3 };
  groups.sort((a, b) => order[a.k] - order[b.k]);
  const active = sec().char || "";
  const go = i => { curSec = i; hideInfo(); renderAll(); fit(); };
  if (groups.length > 1) groups.forEach(g => ct.appendChild(tab(g.k ? NAMES[g.k] : T().other, g.k === active, () => go(g.items[0]))));
  const g = groups.find(g => g.k === active);
  if (g && g.items.length > 1) g.items.forEach((i, j) => {
    const s = secs[i], u = s.nodes.filter(n => n.unlocked).length;
    pt.appendChild(tab(variantLabel(s, j), i === curSec, () => go(i), `${u}/${s.nodes.length}`));
  });
}

function renderSide() {
  const list = $("#chlist"); list.replaceChildren();
  CHAPTERS.forEach(([n, t, who]) => {
    if (filter !== "all" && !who.includes(filter)) return;
    const c = S.chapters[n], s = chStats(c);
    const li = document.createElement("li"), b = document.createElement("button");
    b.className = "ch" + (n === cur ? " sel" : "") + (hasData(c) ? "" : " nodata");
    const sp = (cls, txt) => { const e = document.createElement("span"); e.className = cls; e.textContent = txt; b.appendChild(e); };
    sp("n", n); sp("t", t); sp("p" + (s.shown >= 100 ? " full" : ""), s.shown + "%");
    if (n === cur) b.setAttribute("aria-current", "true");
    b.onclick = () => { cur = n; curSec = 0; hideInfo(); renderAll(); fit(); };
    li.appendChild(b); list.appendChild(li);
  });
  let full = 0;
  CHAPTERS.forEach(([n]) => { if (chStats(S.chapters[n]).shown >= 100) full++; });
  $("#ovCh").textContent = `${full} / 32`;
}

function renderTop() {
  const [, t] = CHAPTERS[cur - 1], s = chStats(chap());
  $("#chTitle").textContent = t;
  $("#chPct").textContent = s.calc + "%";
  $("#gamePct").value = chap().game ?? "";
  $("#spoilBtn").textContent = S.spoiler ? T().showNames : T().hideNames;
  renderTabs();
}

function renderAll() { applyLang(); renderSide(); renderTop(); renderGraph(); }
function commit() { save(); renderAll(); }

// Un clic aggiorna solo il nodo, le sue linee e i contatori: niente ridisegno completo
function toggle(n) {
  n.unlocked = !n.unlocked; scheduleSave();
  const g = nodeEls.get(n.id);
  if (g) paintNode(g, n);
  edgeEls.forEach(x => {
    if (x.p !== n && x.c !== n) return;
    const on = x.p.unlocked && x.c.unlocked;
    x.e.classList.toggle("lit", on);
    if (on) x.e.parentNode.appendChild(x.e);   // le linee accese stanno sopra
  });
  updateCounts();
  if (g && (g.matches(":hover") || document.activeElement === g)) showInfo(n);
}
function updateCounts() {
  const s = chStats(chap());
  $("#chPct").textContent = s.calc + "%";
  const p = document.querySelector(".ch.sel .p");
  if (p) { p.textContent = s.shown + "%"; p.classList.toggle("full", s.shown >= 100); }
  let full = 0;
  CHAPTERS.forEach(([c]) => { if (chStats(S.chapters[c]).shown >= 100) full++; });
  $("#ovCh").textContent = `${full} / 32`;
  const t = document.querySelector("#pathTabs button.on small");
  if (t) t.textContent = `${sec().nodes.filter(x => x.unlocked).length}/${sec().nodes.length}`;
}

/* ---------- vista: scorrimento e zoom ---------- */
const svg = $("#svg"), world = $("#world");
function applyView() { world.setAttribute("transform", `translate(${view.x},${view.y}) scale(${view.k})`); }
function fit() {
  if (!bounds) return;
  const cw = svg.clientWidth || 800, ch = svg.clientHeight || 600, m = 40;
  const bw = bounds.x1 - bounds.x0, bh = bounds.y1 - bounds.y0;
  const k = Math.max(ZFIT, Math.min(1, (cw - 2 * m) / bw, (ch - 2 * m) / bh));
  const x = bw * k <= cw - 2 * m ? (cw - bw * k) / 2 - bounds.x0 * k : m - bounds.x0 * k;
  const y = bh * k <= ch - 2 * m ? (ch - bh * k) / 2 - bounds.y0 * k : m - bounds.y0 * k;
  view = { k, x, y }; applyView();
}
function zoomAt(f, mx, my) {
  const k = Math.min(ZMAX, Math.max(ZMIN, view.k * f));
  view.x = mx - (mx - view.x) * k / view.k; view.y = my - (my - view.y) * k / view.k; view.k = k; applyView();
}
svg.addEventListener("wheel", e => {
  e.preventDefault();
  if (e.ctrlKey) { const r = svg.getBoundingClientRect(); zoomAt(Math.exp(-e.deltaY * 0.002), e.clientX - r.left, e.clientY - r.top); }
  else if (e.shiftKey && !e.deltaX) { view.x -= e.deltaY; applyView(); }
  else { view.x -= e.deltaX; view.y -= e.deltaY; applyView(); }
}, { passive: false });
let drag = null;
svg.addEventListener("pointerdown", e => {
  if (e.target.closest(".node")) return;
  drag = { x: e.clientX, y: e.clientY, vx: view.x, vy: view.y };
  svg.setPointerCapture(e.pointerId); svg.classList.add("panning");
});
svg.addEventListener("pointermove", e => { if (!drag) return; view.x = drag.vx + e.clientX - drag.x; view.y = drag.vy + e.clientY - drag.y; applyView(); });
const endDrag = () => { drag = null; svg.classList.remove("panning"); };
svg.addEventListener("pointerup", endDrag);
svg.addEventListener("pointercancel", endDrag);
$("#zoomIn").onclick = () => zoomAt(1.2, svg.clientWidth / 2, svg.clientHeight / 2);
$("#zoomOut").onclick = () => zoomAt(1 / 1.2, svg.clientWidth / 2, svg.clientHeight / 2);
$("#fitBtn").onclick = fit;
window.addEventListener("resize", fit);

/* ---------- controlli ---------- */
$("#filters").onclick = e => {
  const b = e.target.closest("button"); if (!b) return;
  filter = b.dataset.f;
  document.querySelectorAll("#filters button").forEach(x => x.classList.toggle("on", x === b));
  renderSide();
};
$("#spoilBtn").onclick = () => { S.spoiler = !S.spoiler; commit(); };
$("#langSel").onclick = e => { const b = e.target.closest("button"); if (!b) return; S.lang = b.dataset.lang; commit(); };
$("#gamePct").oninput = e => {
  const v = e.target.value;
  chap().game = v === "" ? null : Math.max(0, Math.min(100, Number(v)));
  scheduleSave(); updateCounts();
};
$("#exportBtn").onclick = () => {
  const blob = new Blob([JSON.stringify(S, null, 1)], { type: "application/json" });
  const a = document.createElement("a"); a.href = URL.createObjectURL(blob); a.download = "flowchart-tracker-backup.json"; a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
};
$("#importBtn").onclick = () => $("#importFile").click();
$("#importFile").onchange = e => {
  const f = e.target.files[0]; if (!f) return;
  const r = new FileReader();
  r.onload = () => {
    try {
      const d = JSON.parse(r.result);
      if (!d || !d.chapters || d.v !== 2) throw 0;
      d.lang = d.lang || S.lang; S = d; applyData(false); curSec = 0; commit(); fit();
    } catch (err) { alert(T().badBackup); }
  };
  r.readAsText(f); e.target.value = "";
};

(async function init() {
  if (bridge) { try { DATA = await bridge.getData(); } catch (e) {} }
  S = load() || { v: 2, chapters: {}, spoiler: false };
  if (!S.lang) S.lang = /^it\b/i.test(navigator.language || "") ? "it" : "en";
  applyData(false);
  save();
  renderAll(); requestAnimationFrame(fit);
})();
