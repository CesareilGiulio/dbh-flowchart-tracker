const { contextBridge, ipcRenderer } = require("electron");
contextBridge.exposeInMainWorld("api", {
  getData: () => ipcRenderer.invoke("get-data")
});
