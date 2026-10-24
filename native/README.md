# Flowchart Tracker – versione nativa (in prova)

Riscrittura in Rust dell'app, senza browser incorporato: un solo exe di circa 2 MB che usa
intorno ai 10–15 MB di RAM, contro i circa 100 MB della versione Electron. Funzionamento e aspetto sono
gli stessi.

- **Dati dei capitoli:** gli stessi file di `../data/manual/`, incorporati nell'exe durante la
  compilazione. Anche l'elenco dei capitoli viene letto da `../renderer/chapters.js`.
- **Disposizione dei diagrammi:** `src/layout.rs` è la traduzione di `../renderer/layout.js` e dà le
  stesse posizioni. Se si modifica uno dei due, va modificato anche l'altro.
- **Progressi:** `%APPDATA%\Flowchart Tracker\progressi.json`, nello stesso formato dei backup
  («Esporta»), quindi i backup passano da una versione all'altra in entrambe le direzioni.
- **Font:** Inter (SIL Open Font License, `assets/OFL.txt`), ridotto ai soli caratteri usati.

## Compilare

```
cd native
cargo build --release
```

L'exe è `target/release/FlowchartTracker.exe`. GitHub Actions lo compila a ogni push
(artefatto «FlowchartTracker-nativo»).

## Variabili d'ambiente per i test

- `FT_SAVE`: percorso alternativo del file dei progressi
- `FT_DARK`: forza il tema scuro
- `FT_DEBUG`: stampa eventi e tempi di disegno
- `--dump-layout`: stampa le posizioni calcolate per tutti i percorsi, da confrontare con layout.js
