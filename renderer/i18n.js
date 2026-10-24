// Testi dell'interfaccia in italiano e inglese. I nomi dei nodi e le note stanno nei dati.
const STRINGS = {
  it: {
    chaptersFull: "capitoli completati",
    all: "Tutti",
    other: "Altro",
    export: "Esporta",
    import: "Importa",
    hideNames: "Nascondi nomi bloccati",
    showNames: "Mostra nomi bloccati",
    gamePct: "nel gioco",
    fit: "Adatta",
    noData: "Diagramma non ancora disponibile.",
    hiddenName: "Bloccato",
    badBackup: "Il file non è un backup valido di questo tracker.",
    path: n => `Percorso ${n}`
  },
  en: {
    chaptersFull: "chapters completed",
    all: "All",
    other: "Other",
    export: "Export",
    import: "Import",
    hideNames: "Hide locked names",
    showNames: "Show locked names",
    gamePct: "in game",
    fit: "Fit",
    noData: "Flowchart not available yet.",
    hiddenName: "Locked",
    badBackup: "This file is not a valid backup of this tracker.",
    path: n => `Path ${n}`
  }
};
