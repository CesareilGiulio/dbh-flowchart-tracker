const { app, BrowserWindow, ipcMain, shell } = require("electron");
const path = require("path");
const fs = require("fs");

// La cartella dei dati resta "Flowchart Tracker" anche se un giorno cambia il nome del pacchetto:
// lì dentro ci sono i progressi salvati dagli utenti.
app.setPath("userData", path.join(app.getPath("appData"), "Flowchart Tracker"));

// L'app disegna solo un diagramma SVG statico: il rendering software basta e
// risparmia la memoria del processo GPU (circa 30 MB).
app.disableHardwareAcceleration();

// Capitoli trascritti dai diagrammi del gioco (data/manual/*.json)
function loadChapters() {
  const out = { chapters: {} };
  const dir = path.join(__dirname, "data", "manual");
  for (const f of fs.readdirSync(dir).filter(f => f.endsWith(".json")).sort()) {
    try {
      const m = JSON.parse(fs.readFileSync(path.join(dir, f), "utf8"));
      if (m && m.chapter && m.sections) out.chapters[m.chapter] = { rev: m.rev, manual: true, sections: m.sections };
    } catch (e) { console.error(`File non valido: ${f}`, e); }
  }
  return out;
}

ipcMain.handle("get-data", () => loadChapters());

function createWindow() {
  const win = new BrowserWindow({
    width: 1440, height: 900, minWidth: 900, minHeight: 600,
    backgroundColor: "#08111a", autoHideMenuBar: true, title: "Flowchart Tracker",
    icon: path.join(__dirname, "build", "icon.png"),
    webPreferences: { preload: path.join(__dirname, "preload.js"), contextIsolation: true, nodeIntegration: false, sandbox: true, spellcheck: false }
  });
  // I link esterni si aprono nel browser; l'app non naviga mai fuori dai suoi file
  const external = url => { if (/^https:\/\//.test(url)) shell.openExternal(url); };
  win.webContents.setWindowOpenHandler(({ url }) => { external(url); return { action: "deny" }; });
  win.webContents.on("will-navigate", (e, url) => { e.preventDefault(); external(url); });
  win.loadFile(path.join(__dirname, "renderer", "index.html"));
}

app.whenReady().then(createWindow);
app.on("window-all-closed", () => app.quit());
