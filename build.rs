// Incorpora nell'exe i dati dei capitoli (data/manual/*.json e data/chapters.json) e, su Windows, l'icona.
use std::{env, fs, path::Path};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("data/manual");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<_> = fs::read_dir(&dir).expect("data/manual").filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "json")).collect();
    files.sort();
    let mut out = String::from("pub static CHAPTER_FILES: &[&str] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        out += &format!("    include_str!({:?}),\n", f.to_string_lossy());
    }
    out += "];\n";

    // elenco dei capitoli: [numero, titolo, personaggi] (C=Connor K=Kara M=Markus)
    let list = root.join("data/chapters.json");
    println!("cargo:rerun-if-changed={}", list.display());
    let chapters: Vec<(u32, String, String)> = serde_json::from_str(&fs::read_to_string(&list).expect("data/chapters.json")).expect("chapters.json non valido");
    assert_eq!(chapters.len(), 32, "servono 32 capitoli in data/chapters.json");
    out += "pub const CHAPTERS: [(u32, &str, &str); 32] = [\n";
    for (n, t, c) in &chapters { out += &format!("    ({n}, {t:?}, {c:?}),\n"); }
    out += "];\n";
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("chapter_files.rs"), out).unwrap();

    #[cfg(windows)]
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Flowchart Tracker")
            .set("FileDescription", "Flowchart Tracker")
            .set("LegalCopyright", "MIT License");
        res.compile().expect("risorse Windows");
    }
}
