# Istruzioni per continuare la trascrizione dei diagrammi

Questo progetto è un tracker desktop (Electron, compilato in .exe con GitHub Actions) per segnare
i nodi sbloccati nei diagrammi dei 32 capitoli. I capitoli trascritti dagli screenshot del gioco
stanno in `data/manual/` e hanno la precedenza sui dati scaricati dalla wiki.

## Stato
Fatti (20/32): 1 The Hostage, 3 Shades of Color, 4 A New Home, 5 The Painter, 6 Partners,
7 Stormy Night, 8 Broken, 9 The Interrogation, 10 Fugitives, 11 From the Dead,
12 Waiting for Hank..., 13 On the Run, 14 Jericho, 15 The Nest, 16 Time to Decide, 17 Zlatko,
18 Russian Roulette, 19 Spare Parts, 20 The Eden Club, 21 The Pirates' Cove.

Mancano: 2 Opening, 22 The Bridge, 23 The Stratford Tower, 24 Public Enemy, 25 Midnight Train,
26 Capitol Park, 27 Meet Kamski, 28 Freedom March, 29 Last Chance Connor,
30 Crossroads (Kara/Connor/Markus), 31 Night of the Soul (Markus/Connor),
32 Battle for Detroit (più varianti per personaggio).

## Regole seguite
- Niente percentuali: i dati contengono solo nomi e collegamenti.
- Ogni nodo ha `title` (italiano), `en` (nome originale inglese), `type`, `parents`, `note` facoltativa.
- Tipi: `checkpoint` (nodi CHECKPOINT / LOAD CHECKPOINT), `objective` (i blocchi grandi tipo
  "SEARCH FOR CLUES"), `ending` (esiti finali con immagine, gli ultimi a destra), `choice` (tutto il resto).
- Nodi gialli col lucchetto che dipendono da capitoli precedenti: nodo senza genitori + nota
  "Requisito da un capitolo precedente". Nodi gialli che sbloccano qualcosa dopo: nota "Evento chiave".
- Riviste: titolo lasciato in inglese, «Leggi «Titolo»», nota "Rivista".
- Nodi con lo stesso nome in punti diversi del diagramma restano nodi separati (id diversi).
- I nomi italiani sono traduzioni (non le etichette ufficiali): stile imperativo per le azioni
  ("Esamina…", "Parla con…"), passato per gli esiti finali ("Connor ha sparato al deviante").
- Capitoli con più personaggi: se il gioco mostra diagrammi separati, usare più `sections`,
  ognuna con `name` (es. "Kara Leaving Detroit") e `char` ("K", "C", "M"): l'app mostra il selettore.
  Se è un unico diagramma intrecciato (come On the Run), una sola sezione con `char: null`.

## Come si genera un capitolo
`scripts/chapter_lib.py` contiene `chap(numero, char, rev, nodi, nomefile)`: ogni nodo è una tupla
`(id, titolo_it, titolo_en, tipo, [genitori], nota)`. La funzione verifica che i genitori esistano
e che gli id siano unici, poi scrive il JSON in `data/manual/NN-nome.json`.
Per i capitoli con più sezioni, scrivere il JSON con più elementi in `sections`.
Per correggere un capitolo già fatto: modificare il JSON e cambiare `rev` (es. "manual-2"):
l'app aggiorna il capitolo mantenendo i nodi già sbloccati.
