//! Finestre di dialogo per esportare e importare i backup.
//! Su Windows sono quelle native del sistema; altrove (solo per i test) si usano
//! le variabili d'ambiente FT_EXPORT e FT_IMPORT.
use std::path::PathBuf;

#[cfg(windows)]
pub fn save_file() -> Option<PathBuf> {
    rfd::FileDialog::new().set_file_name("flowchart-tracker-backup.json").add_filter("JSON", &["json"]).save_file()
}
#[cfg(windows)]
pub fn open_file() -> Option<PathBuf> { rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() }
#[cfg(windows)]
pub fn alert(msg: &str) {
    rfd::MessageDialog::new().set_title("Flowchart Tracker").set_description(msg).set_level(rfd::MessageLevel::Warning).show();
}

#[cfg(not(windows))]
pub fn save_file() -> Option<PathBuf> { std::env::var_os("FT_EXPORT").map(PathBuf::from) }
#[cfg(not(windows))]
pub fn open_file() -> Option<PathBuf> { std::env::var_os("FT_IMPORT").map(PathBuf::from) }
#[cfg(not(windows))]
pub fn alert(msg: &str) { eprintln!("{msg}"); }
