const { contextBridge, ipcRenderer } = require("electron");
contextBridge.exposeInMainWorld("api", {
  getData: () => ipcRenderer.invoke("get-data"),
  refreshData: () => ipcRenderer.invoke("refresh-data"),
  onProgress: (cb) => ipcRenderer.on("scrape-progress", (_e, m) => cb(m))
});
