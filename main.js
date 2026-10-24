const { app, BrowserWindow, ipcMain, shell } = require("electron");
const path = require("path");
const fs = require("fs");
const { scrapeAll } = require("./scraper");

const userFile = () => path.join(app.getPath("userData"), "flowcharts.json");
const bundledFile = path.join(__dirname, "data", "flowcharts.json");

function readData(file) {
  try {
    const d = JSON.parse(fs.readFileSync(file, "utf8"));
    if (d && d.chapters && Object.keys(d.chapters).length) return d;
  } catch {}
  return null;
}

// I file in data/manual/ (trascritti dai diagrammi del gioco) hanno la precedenza sui dati della wiki
function withManual(d) {
  const dir = path.join(__dirname, "data", "manual");
  try {
    for (const f of fs.readdirSync(dir).filter(f => f.endsWith(".json"))) {
      const m = JSON.parse(fs.readFileSync(path.join(dir, f), "utf8"));
      if (m && m.chapter && m.sections) d.chapters[m.chapter] = { rev: m.rev, manual: true, sections: m.sections };
    }
  } catch {}
  return d;
}

ipcMain.handle("get-data", () => withManual(readData(userFile()) || readData(bundledFile) || { chapters: {} }));
ipcMain.handle("refresh-data", async (e) => {
  const d = await scrapeAll((m) => e.sender.send("scrape-progress", m));
  if (Object.keys(d.chapters).length) fs.writeFileSync(userFile(), JSON.stringify(d));
  return withManual(d);
});

function createWindow() {
  const win = new BrowserWindow({
    width: 1440, height: 900, minWidth: 900, minHeight: 600,
    backgroundColor: "#08111a", autoHideMenuBar: true, title: "Flowchart Tracker",
    webPreferences: { preload: path.join(__dirname, "preload.js"), contextIsolation: true, nodeIntegration: false }
  });
  win.webContents.setWindowOpenHandler(({ url }) => { shell.openExternal(url); return { action: "deny" }; });
  win.loadFile(path.join(__dirname, "renderer", "index.html"));
}

app.whenReady().then(createWindow);
app.on("window-all-closed", () => app.quit());
