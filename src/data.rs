//! Dati dei capitoli (incorporati nell'exe) e progressi del giocatore.
//!
//! Il file dei progressi ha lo stesso formato dei backup («Esporta»), compatibile anche con
//! quelli della versione 1.x (Electron).

use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/chapter_files.rs"));

// CHAPTER_FILES e CHAPTERS ([numero, titolo, personaggi], da data/chapters.json) arrivano da build.rs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Checkpoint, Choice, Objective, Ending }

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub en: String,
    pub kind: Kind,
    pub note: String,
    pub note_en: String,
    pub parents: Vec<String>,
}

#[derive(Debug)]
pub struct Section { pub name: String, pub ch: Option<char>, pub nodes: Vec<Node> }

#[derive(Debug)]
pub struct Chapter { pub n: u32, pub rev: String, pub sections: Vec<Section> }

#[derive(Deserialize)]
struct RawNode { id: String, title: String, #[serde(default)] en: String, #[serde(rename = "type")] kind: String,
    #[serde(default)] note: String, #[serde(default)] note_en: String, #[serde(default)] parents: Vec<String> }
#[derive(Deserialize)]
struct RawSection { #[serde(default)] name: String, #[serde(rename = "char")] ch: Option<String>, nodes: Vec<RawNode> }
#[derive(Deserialize)]
struct RawChapter { chapter: u32, #[serde(default)] rev: String, sections: Vec<RawSection> }

/// Capitoli indicizzati da 0 a 31; None se manca il file.
pub fn load_chapters() -> Vec<Option<Chapter>> {
    let mut out: Vec<Option<Chapter>> = (0..32).map(|_| None).collect();
    for txt in CHAPTER_FILES {
        let r: RawChapter = serde_json::from_str(txt).expect("JSON di un capitolo non valido");
        let sections = r.sections.into_iter().map(|s| Section {
            name: s.name,
            ch: s.ch.and_then(|c| c.chars().next()),
            nodes: s.nodes.into_iter().map(|n| Node {
                kind: match n.kind.as_str() { "checkpoint" => Kind::Checkpoint, "objective" => Kind::Objective, "ending" => Kind::Ending, _ => Kind::Choice },
                id: n.id, title: n.title, en: n.en, note: n.note, note_en: n.note_en, parents: n.parents,
            }).collect(),
        }).collect();
        if (1..=32).contains(&r.chapter) { out[r.chapter as usize - 1] = Some(Chapter { n: r.chapter, rev: r.rev, sections }); }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { It, En }

#[derive(Default, Clone)]
pub struct ChapterProgress {
    /// per sezione: indici dei nodi sbloccati
    pub unlocked: Vec<HashSet<usize>>,
    pub game: Option<f64>,
}

pub struct Progress {
    pub chapters: Vec<ChapterProgress>,
    pub lang: Lang,
    pub spoiler: bool,
}

impl Progress {
    pub fn new(data: &[Option<Chapter>]) -> Self {
        let lang = if system_italian() { Lang::It } else { Lang::En };
        Progress {
            chapters: data.iter().map(|c| ChapterProgress {
                unlocked: c.as_ref().map_or(vec![], |c| c.sections.iter().map(|_| HashSet::new()).collect()),
                game: None,
            }).collect(),
            lang, spoiler: false,
        }
    }

    /// Statistiche di un capitolo: (totale, sbloccati, % calcolata, % mostrata)
    pub fn stats(&self, data: &[Option<Chapter>], i: usize) -> (usize, usize, i64, i64) {
        let t: usize = data[i].as_ref().map_or(0, |c| c.sections.iter().map(|s| s.nodes.len()).sum());
        let u: usize = self.chapters[i].unlocked.iter().map(|s| s.len()).sum();
        let calc = if t > 0 { (u as f64 / t as f64 * 100.0).round() as i64 } else { 0 };
        let shown = self.chapters[i].game.map_or(calc, |g| g.round() as i64);
        (t, u, calc, shown)
    }

    /// Legge un salvataggio o un backup (anche quelli della versione 1.x). I nodi si
    /// riconoscono dall'id oppure, se l'id nel backup non c'è più, da percorso + titolo.
    pub fn apply_json(&mut self, data: &[Option<Chapter>], v: &Value) -> Result<(), String> {
        if v.get("v").and_then(Value::as_i64) != Some(2) || !v.get("chapters").map_or(false, Value::is_object) {
            return Err("formato".into());
        }
        let chs = &v["chapters"];
        for (i, ch) in data.iter().enumerate() {
            let Some(ch) = ch else { continue };
            let Some(saved) = chs.get(ch.n.to_string()) else { continue };
            let mut ids = HashSet::new();
            let mut known = HashSet::new();
            let mut titles = HashSet::new();
            for s in saved.get("sections").and_then(Value::as_array).into_iter().flatten() {
                let sname = s.get("name").and_then(Value::as_str).unwrap_or("");
                for n in s.get("nodes").and_then(Value::as_array).into_iter().flatten() {
                    if let Some(id) = n.get("id").and_then(Value::as_str) { known.insert(id.to_string()); }
                    if n.get("unlocked").and_then(Value::as_bool) == Some(true) {
                        if let Some(id) = n.get("id").and_then(Value::as_str) { ids.insert(id.to_string()); }
                        if let Some(t) = n.get("title").and_then(Value::as_str) { titles.insert(format!("{}|{}", sname, t.to_lowercase())); }
                    }
                }
            }
            let p = &mut self.chapters[i];
            for (si, s) in ch.sections.iter().enumerate() {
                p.unlocked[si] = s.nodes.iter().enumerate()
                    // l'id decide; il titolo serve solo per i nodi che nel backup non ci sono
                    // (capitolo corretto in una versione successiva)
                    .filter(|(_, n)| if known.contains(&n.id) { ids.contains(&n.id) } else { titles.contains(&format!("{}|{}", s.name, n.title.to_lowercase())) })
                    .map(|(k, _)| k).collect();
            }
            p.game = match saved.get("game") {
                Some(Value::Number(x)) => x.as_f64(),
                Some(Value::String(s)) => s.trim().parse().ok(),
                _ => None,
            }.map(|g: f64| g.clamp(0.0, 100.0));
        }
        if let Some(b) = v.get("spoiler").and_then(Value::as_bool) { self.spoiler = b; }
        match v.get("lang").and_then(Value::as_str) { Some("it") => self.lang = Lang::It, Some("en") => self.lang = Lang::En, _ => {} }
        Ok(())
    }

    /// Stesso formato dei backup della versione 1.x, che quindi li può importare.
    pub fn to_json(&self, data: &[Option<Chapter>]) -> Value {
        let mut chs = Map::new();
        for (i, ch) in data.iter().enumerate() {
            let Some(ch) = ch else { continue };
            let p = &self.chapters[i];
            let sections: Vec<Value> = ch.sections.iter().enumerate().map(|(si, s)| json!({
                "name": s.name,
                "char": s.ch.map(|c| c.to_string()),
                "nodes": s.nodes.iter().enumerate().map(|(k, n)| json!({
                    "id": n.id, "title": n.title, "en": n.en,
                    "type": match n.kind { Kind::Checkpoint => "checkpoint", Kind::Objective => "objective", Kind::Ending => "ending", Kind::Choice => "choice" },
                    "note": n.note, "note_en": n.note_en, "parents": n.parents,
                    "unlocked": p.unlocked[si].contains(&k),
                })).collect::<Vec<_>>(),
            })).collect();
            chs.insert(ch.n.to_string(), json!({ "fromWiki": true, "rev": ch.rev, "manual": true, "game": p.game.map(|g| if g.fract() == 0.0 { json!(g as i64) } else { json!(g) }), "sections": sections }));
        }
        json!({ "v": 2, "chapters": chs, "spoiler": self.spoiler, "lang": if self.lang == Lang::It { "it" } else { "en" } })
    }
}

fn system_italian() -> bool {
    sys_locale::get_locale().map_or(false, |l| l.to_lowercase().starts_with("it"))
}

const README_EN: &str = "Flowchart Tracker - portable version\r\n\r\n\
The app saves your progress in this folder (progress.json).\r\n\
Keep the \"data\" folder next to FlowchartTracker.exe: if you move or delete it,\r\n\
the app goes back to saving in %APPDATA%\\Flowchart Tracker.\r\n\
To move the tracker to another PC or a USB stick, copy the whole FlowchartTracker folder.\r\n";

/// Dove si salvano i progressi:
/// - versione portable: cartella `dati` (progressi.json) o `data` (progress.json) accanto all'exe.
///   Lo zip contiene `dati`: al primo avvio su un sistema non in italiano, se è ancora vuota,
///   diventa `data` (con il LEGGIMI in inglese);
/// - altrimenti %APPDATA%\Flowchart Tracker, con il nome del file nella lingua del sistema
///   (si tiene quello già esistente). FT_SAVE lo sostituisce, per i test.
pub fn save_path() -> PathBuf {
    if let Ok(p) = std::env::var("FT_SAVE") { return PathBuf::from(p); }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(PathBuf::from)) {
        let (dati, data) = (exe_dir.join("dati"), exe_dir.join("data"));
        if dati.is_dir() && !system_italian() && !data.exists() && !dati.join("progressi.json").exists()
            && std::fs::rename(&dati, &data).is_ok() {
            let _ = std::fs::remove_file(data.join("LEGGIMI.txt"));
            let _ = std::fs::write(data.join("README.txt"), README_EN);
        }
        if dati.is_dir() { return dati.join("progressi.json"); }
        if data.join("progress.json").exists() || (data.is_dir() && !system_italian()) { return data.join("progress.json"); }
    }
    let base = std::env::var_os("APPDATA").map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Flowchart Tracker");
    let (it, en) = (base.join("progressi.json"), base.join("progress.json"));
    if it.exists() { it } else if en.exists() || !system_italian() { en } else { it }
}

pub fn read_json(path: &std::path::Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Scrive prima un file temporaneo e poi lo rinomina: un'interruzione non lascia il file a metà.
pub fn write_json(path: &std::path::Path, v: &Value) -> std::io::Result<()> {
    if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(v)?)?;
    std::fs::rename(&tmp, path)
}
