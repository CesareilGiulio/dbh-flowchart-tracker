# Flowchart Tracker

App desktop offline per tenere traccia dei nodi sbloccati nei diagrammi dei 32 capitoli,
in vista del 100% e del platino.

## Ottenere il .exe con GitHub Actions

1. Crea un nuovo repository su GitHub e carica tutti questi file (cartella `.github` compresa).
2. Il push sul branch `main` avvia da solo la build. Puoi anche lanciarla a mano da
   **Actions → Build Windows exe → Run workflow**.
3. A build finita apri l'esecuzione e scarica l'artifact **FlowchartTracker**: dentro c'è `FlowchartTracker.exe`.
   È portable: non serve installarlo.
4. Per avere l'exe come Release scaricabile, crea un tag: `git tag v1.0.0 && git push --tags`.

Durante la build lo script `scripts/fetch-data.js` scarica i nodi di tutti i capitoli dalla
Detroit: Become Human Wiki su Fandom e li include nell'eseguibile. Se in quel momento la wiki
non risponde, la build va avanti lo stesso.

## Build in locale (facoltativo)

```
npm install
npm run fetch-data
npm start          # prova l'app
npm run dist       # crea dist/FlowchartTracker.exe (da Windows)
```

## Capitoli trascritti dal gioco

I file in `data/manual/` contengono capitoli trascritti a mano dai diagrammi del gioco, con i nomi in italiano
e il nome originale inglese. Hanno la precedenza sui dati della wiki. Per correggere un nome basta modificare
il campo `title` e cambiare il valore di `rev`: all'avvio l'app aggiorna il capitolo mantenendo i nodi già sbloccati.

## Note

- I progressi sono salvati in `%APPDATA%\flowchart-tracker`. Usa **Esporta backup** per una copia.
- Le liste dei nodi vengono dalla wiki della community: se a un capitolo manca un nodo che
  vedi nel gioco, aggiungilo a mano dall'app. La % inserita nel campo "% nel gioco" ha la precedenza.
- Dati dei nodi: Detroit: Become Human Wiki (Fandom), licenza CC BY-SA 3.0.
