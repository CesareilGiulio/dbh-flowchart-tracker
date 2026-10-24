// Scarica i diagrammi dei capitoli dalla Detroit: Become Human Wiki (Fandom, CC BY-SA)
// e li converte in nodi per il tracker.
const CHAPTERS = require("./renderer/chapters.js");
const API = "https://detroit-become-human.fandom.com/api.php";
const NAMES = { C: "Connor", K: "Kara", M: "Markus" };

async function getWikitext(page) {
  const u = `${API}?action=parse&page=${encodeURIComponent(page)}&prop=wikitext&format=json&formatversion=2&redirects=1`;
  try {
    const r = await fetch(u, { headers: { "User-Agent": "FlowchartTracker/1.0 (personal progress tracker)" } });
    if (!r.ok) return null;
    const j = await r.json();
    return (j.parse && j.parse.wikitext) || null;
  } catch { return null; }
}

function candidates(title) {
  const t = title.replace(/…/g, "...");
  const set = [`${t} (chapter)`, t, `The ${t}`, t.replace(/\.+$/, ""), t.replace(/'/g, "’")];
  return [...new Set(set)];
}

function clean(s) {
  let t = String(s);
  t = t.replace(/<ref[^>]*\/>/gi, "").replace(/<ref[^>]*>[\s\S]*?<\/ref>/gi, "");
  let prev;
  do { prev = t; t = t.replace(/\{\{[^{}]*\}\}/g, ""); } while (t !== prev);
  t = t.replace(/\[\[(?:File|Image|Category):[^\]]*\]\]/gi, "");
  t = t.replace(/\[\[[^\]|]*\|([^\]]*)\]\]/g, "$1").replace(/\[\[([^\]]*)\]\]/g, "$1");
  t = t.replace(/\[https?:\/\/\S+\s([^\]]*)\]/g, "$1").replace(/\[https?:\/\/\S+\]/g, "");
  t = t.replace(/'{2,}/g, "").replace(/<[^>]+>/g, "");
  t = t.replace(/&nbsp;/g, " ").replace(/&amp;/g, "&").replace(/&quot;/g, '"').replace(/&#39;/g, "'");
  return t.replace(/\s+/g, " ").trim();
}

function parseItem(raw, depth) {
  let t = clean(raw);
  const notes = [];
  const cp = /\(\s*(?:load\s+)?checkpoint\s*\)/i;
  const type = cp.test(t) ? "checkpoint" : "choice";
  t = t.replace(cp, " ");
  const i = t.search(/\s[-–—]\s/);
  if (i > 0) { notes.push(t.slice(i).replace(/^\s*[-–—]\s*/, "")); t = t.slice(0, i); }
  t = t.replace(/\(([^)]*)\)/g, (_, x) => { if (x.trim()) notes.push(x.trim()); return " "; });
  t = t.replace(/^["“]|["”]$/g, "").replace(/\s+/g, " ").trim().replace(/[:;,.]$/, "").trim();
  return { title: t, type, note: notes.join(" · "), depth };
}

function parseFlowchart(wt) {
  const lines = wt.split(/\r?\n/);
  const start = lines.findIndex(l => /^==\s*Flowchart\s*==\s*$/i.test(l.trim()));
  if (start < 0) return null;
  const sections = [];
  let cur = { name: "", items: [] };
  for (let k = start + 1; k < lines.length; k++) {
    const l = lines[k].trim();
    if (/^==[^=]/.test(l)) break;
    const h = l.match(/^(={3,})\s*(.*?)\s*\1$/);
    if (h) { if (cur.items.length) sections.push(cur); cur = { name: clean(h[2]), items: [] }; continue; }
    const b = l.match(/^([*#]+)\s*(.+)$/);
    if (b) {
      const it = parseItem(b[2], b[1].length);
      if (it.title && it.title.length <= 90) cur.items.push(it);
    }
  }
  if (cur.items.length) sections.push(cur);
  return sections;
}

function charOf(name, chapterChars) {
  for (const [k, v] of Object.entries(NAMES)) if (new RegExp(`\\b${v}`, "i").test(name)) return k;
  return chapterChars.length === 1 ? chapterChars : null;
}

function toNodes(items, prefix) {
  const nodes = [];
  let lastTop = null;
  const stack = [];
  items.forEach((it, i) => {
    const id = `${prefix}n${i}`;
    let parents = [];
    if (it.depth <= 1) {
      if (lastTop) parents = [lastTop];
      lastTop = id; stack.length = 0; stack[1] = id;
    } else {
      const p = stack[it.depth - 1] || lastTop;
      if (p) parents = [p];
      stack[it.depth] = id; stack.length = it.depth + 1;
    }
    nodes.push({ id, title: it.title, type: it.type, note: it.note, parents });
  });
  const tops = nodes.filter((n, i) => items[i].depth <= 1);
  const last = tops[tops.length - 1];
  if (last && last.type !== "checkpoint") last.type = "ending";
  return nodes;
}

async function scrapeChapter([n, title, chars]) {
  for (const page of candidates(title)) {
    const wt = await getWikitext(page);
    if (!wt) continue;
    const secs = parseFlowchart(wt);
    if (!secs || !secs.length) continue;
    return {
      page,
      sections: secs.map((s, si) => ({
        name: s.name,
        char: charOf(s.name, chars),
        nodes: toNodes(s.items, `c${n}s${si}`)
      }))
    };
  }
  return null;
}

async function scrapeAll(progress = () => {}) {
  const out = { source: "Detroit: Become Human Wiki (Fandom) — CC BY-SA 3.0", fetched: new Date().toISOString(), chapters: {} };
  for (const ch of CHAPTERS) {
    progress(`Capitolo ${ch[0]}/32: ${ch[1]}`);
    const r = await scrapeChapter(ch);
    if (r) out.chapters[ch[0]] = r;
    await new Promise(r => setTimeout(r, 250));
  }
  progress(`Fatto: ${Object.keys(out.chapters).length}/32 capitoli scaricati`);
  return out;
}

module.exports = { scrapeAll, parseFlowchart, toNodes, clean, charOf };
