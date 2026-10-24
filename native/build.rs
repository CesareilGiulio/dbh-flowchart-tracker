// Incorpora nell'exe i dati dei capitoli (../data/manual/*.json) e, su Windows, l'icona.
use std::{env, fs, path::Path};

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/manual");
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
    out += &chapter_list();
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("chapter_files.rs"), out).unwrap();

    #[cfg(windows)]
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../build/icon.ico")
            .set("ProductName", "Flowchart Tracker")
            .set("FileDescription", "Flowchart Tracker")
            .set("LegalCopyright", "MIT License");
        res.compile().expect("risorse Windows");
    }
}

// Legge l'elenco dei capitoli da renderer/chapters.js: righe del tipo [1,"The Hostage","C"]
fn chapter_list() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../renderer/chapters.js");
    println!("cargo:rerun-if-changed={}", path.display());
    let src = fs::read_to_string(&path).expect("renderer/chapters.js");
    let body = &src[src.find("= [").expect("CHAPTERS") + 3..];
    let mut items = vec![];
    let mut rest = body;
    while let Some(i) = rest.find('[') {
        rest = &rest[i + 1..];
        let end = rest.find(']').expect("]");
        let part = &rest[..end];
        rest = &rest[end + 1..];
        let comma = part.find(',').unwrap();
        let n: u32 = part[..comma].trim().parse().expect("numero capitolo");
        let strs: Vec<&str> = part[comma + 1..].split('"').collect(); // ["", titolo, ",", personaggi, ""]
        items.push(format!("    ({}, {:?}, {:?}),\n", n, strs[1], strs[3]));
    }
    assert_eq!(items.len(), 32, "servono 32 capitoli in chapters.js");
    format!("pub const CHAPTERS: [(u32, &str, &str); 32] = [\n{}];\n", items.concat())
}
