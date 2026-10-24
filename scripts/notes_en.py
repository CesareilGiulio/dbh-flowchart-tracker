"""Traduzione inglese delle note dei nodi (campo note_en).

Le note sono composte da un prefisso fisso facoltativo (REQ o KEY) seguito da un testo libero.
`note_en()` traduce il prefisso e cerca il resto in NOTES_EN: se una nota nuova non c'è,
lo script si ferma e chiede di aggiungerla qui.
"""
from chapter_lib import REQ, KEY

REQ_EN = "Requirement from a previous chapter (highlighted in yellow in the game)."
KEY_EN = "Key event (highlighted in yellow in the game)."
DUB = "Collegamento da verificare: "
DUB_EN = "Link to be verified: "

NOTES_EN = {
 "Accanto a un container.": "Next to a container.",
 "Aiutando Josh.": "While helping Josh.",
 "Al bivio.": "At the crossroad.",
 "Al capezzale di Carl.": "At Carl's bedside.",
 "Al confine canadese.": "At the Canadian border.",
 "Al controllo di sicurezza nella stazione.": "At the security check in the bus terminal.",
 "Al piano di sopra.": "Upstairs.",
 "Al piano di sotto.": "Downstairs.",
 "Al posto di blocco.": "At the security checkpoint.",
 "All'arrivo, fuori casa.": "On arrival, outside the house.",
 "All'ultimo piano, per rientrare.": "On the top floor, to get back in.",
 "Alla centrale, prima del seminterrato.": "At the station, before the basement.",
 "Alla scrivania di Hank.": "At Hank's desk.",
 "Alle domande di Alice.": "Answering Alice's questions.",
 "Alternativa a «Salva il pesce».": "Alternative to “Save Fish”.",
 "Aperta da Luther.": "Opened by Luther.",
 "Aperta dall'androide.": "Opened by the android.",
 "Appoggiato a un pilastro.": "Leaning against a pillar.",
 "Checkpoint del percorso di Connor macchina.": "Checkpoint on the machine Connor path.",
 "Checkpoint.": "Checkpoint.",
 "Collegamento da verificare: a questo gruppo arriva anche una linea da «Parla con l'androide salvato».":
  "A line from “Talk to Rescued Android” also reaches this group.",
 "Collegamento da verificare: arrivano linee da «La gente reagisce…», dal gruppo delle scelte finali e dal gruppo «Lascia perdere / Premi».":
  "Lines arrive from “Public Reacts…”, from the group of final choices and from the “Give Up / Press” group.",
 "Collegamento da verificare: attribuito a «Non riesci a salvare gli androidi» come il nodo precedente.":
  "Attributed to “Fail to Save Androids”, like the previous node.",
 "Collegamento da verificare: la linea arriva da sinistra e non si legge da dove parte.":
  "The line comes from the left and its origin can't be read.",
 "Collegamento da verificare: la linea parte dal gruppo del controllo di sicurezza e passa per un'icona di bivio; attribuita a «Scappa» e «Consegna il documento».":
  "The line leaves the security check group and passes through a branch icon; attributed to “Flee” and “Hand Over ID”.",
 "Collegamento da verificare: la linea parte dal gruppo delle scelte finali senza indicare quali.":
  "The line leaves the group of final choices without showing which ones.",
 "Collegamento da verificare: la linea parte dal gruppo «Muoviti / Resta immobile»; attribuita a «Muoviti».":
  "The line leaves the “Move / Remain Still” group; attributed to “Move”.",
 "Collegamento da verificare: la linea parte dal gruppo «Salva gli androidi / Non riesci…»; attribuita a «Non riesci a salvare gli androidi».":
  "The line leaves the “Save Fleeing Androids / Fail…” group; attributed to “Fail to Save Androids”.",
 "Collegamento da verificare: la linea parte dalla parte alta del gruppo; attribuita a «Salva i Jerry» e «Salva Luther».":
  "The line leaves the top of the group; attributed to “Save Jerrys” and “Save Luther”.",
 "Collegamento da verificare: le linee partono dal gruppo «Il soldato ha la meglio su Kara / Kara ha la meglio sul soldato» senza indicare la scelta.":
  "The lines leave the “Soldier Beats Kara / Kara Beats Soldier” group without showing which choice.",
 "Collegamento da verificare: nel diagramma parte da un'icona separata, senza linea dal nodo precedente.":
  "In the flowchart it starts from a separate icon, with no line from the previous node.",
 "Collegamento da verificare: nel diagramma parte da un'icona separata, senza linea dal nodo precedente. Collegata a «Nasconditi dai militari» per posizione.":
  "In the flowchart it starts from a separate icon, with no line from the previous node. Linked to “Hide From Military” based on its position.",
 "Collegamento da verificare: oltre a «In ritardo», una linea parte dal punto in cui convergono i rami verso «At the Bus Terminal»; attribuita ad «Alice trova il suo guanto».":
  "Besides “Delayed”, a line starts where the branches towards “At the Bus Terminal” meet; attributed to “Alice Finds Her Glove”.",
 "Collegamento da verificare: una linea arriva dal gruppo «Scappa / Consegna il documento / Chiedi a Luther…» senza indicare la scelta; attribuita a «Chiedi a Luther di creare un diversivo».":
  "A line arrives from the “Flee / Hand Over ID / Ask Luther…” group without showing which choice; attributed to “Ask Luther to Create Diversion”.",
 "Collegamento da verificare: una seconda linea arriva direttamente dall'obiettivo «Approaching Recall Center».":
  "A second line comes directly from the “Approaching Recall Center” objective.",
 "Con Markus in chiesa.": "With Markus in the church.",
 "Con North in chiesa.": "With North in the church.",
 "Con i soldati che minacciano gli androidi.": "With the soldiers threatening the androids.",
 "Con il soldato impaziente.": "With the impatient soldier.",
 "Con la barca che affonda.": "With the boat sinking.",
 "Controllando Connor.": "Controlling Connor.",
 "Controllando Markus.": "Controlling Markus.",
 "Da terra, col nuovo Connor che si avvicina.": "From the ground, as the new Connor approaches.",
 "Da terra.": "From the ground.",
 "Dalla sala server, per uscire.": "From the server room, to get out.",
 "Davanti a North, nel negozio della CyberLife.": "Facing North, in the CyberLife store.",
 "Davanti ai soldati che alzano le armi.": "As the soldiers raise their weapons.",
 "Davanti ai soldati che sparano sulla folla.": "As the soldiers fire on the crowd.",
 "Davanti al blocco della polizia.": "At the police roadblock.",
 "Dentro il riquadro «Libera la tua gente».": "Inside the “Free Your People” box.",
 "Dopo aver armato l'esplosivo.": "After arming the explosives.",
 "Dopo aver ferito Hank.": "After injuring Hank.",
 "Dopo aver risparmiato Chloe.": "After sparing Chloe.",
 "Dopo aver scelto il nascondiglio.": "After choosing the hiding place.",
 "Dopo aver sparato a Chloe.": "After shooting Chloe.",
 "Dopo aver stordito i soldati.": "After stunning the soldiers.",
 "Dopo aver ucciso i soldati.": "After killing the soldiers.",
 "Dopo che Kara ha lasciato il campo.": "After Kara left the camp.",
 "Dopo che North se ne va.": "After North leaves.",
 "Dopo essere intervenuto.": "After intervening.",
 "Dopo essere stato colpito da Connor.": "After being shot by Connor.",
 "Dopo il conto alla rovescia.": "After the countdown.",
 "Dopo il crollo del ponte.": "After the walkway collapses.",
 "Dopo il serbatoio.": "After the reservoir.",
 "Dopo l'intervento.": "After the intervention.",
 "Dopo la morte di Kara in fila.": "After Kara died in the lines.",
 "Dopo la morte di Kara per mano del drone.": "After Kara was killed by the drone.",
 "Dopo la resa.": "After surrendering.",
 "Dopo la riorganizzazione.": "After regrouping.",
 "Durante la ritirata.": "During the retreat.",
 "Estrai la pistola ma non sparare, continua a intimidirlo.": "Draw the gun but don't shoot; keep intimidating him.",
 "Fallisci il QTE finale: Emma muore.": "Fail the final QTE: Emma dies.",
 "Ferita alla gamba.": "Wounded in the leg.",
 "Fuga finale.": "Final escape.",
 "Fuori dal riquadro: quello con il proprietario.": "Outside the box: the one with the owner.",
 "In fila con Alice.": "In line with Alice.",
 "In fila, per raggiungere Alice.": "In line, to reach Alice.",
 "In fila, per raggiungere Alice. Collegamento da verificare: una linea passa anche da «Osserva le guardie».":
  "In line, to reach Alice. Link to be verified: a line also passes through “Observe Guards”.",
 "In soggiorno: ricostruisci la sua morte.": "In the living room: reconstruct his death.",
 "In terrazza, a sinistra dell'ingresso.": "On the terrace, left of the entrance.",
 "Indaga troppo a lungo senza uscire.": "Investigate for too long without leaving.",
 "Inizio alternativo se Connor è morto in «Public Enemy».": "Alternative start if Connor died in “Public Enemy”.",
 "Inizio alternativo se Markus è morto in «Freedom March».": "Alternative start if Markus died in “Freedom March”.",
 "Inseguendo North.": "While chasing North.",
 "Intervenendo.": "By intervening.",
 "Markus da solo nel negozio.": "Markus alone in the store.",
 "Markus e North nel negozio.": "Markus and North in the store.",
 "Markus tra la gente.": "Markus among the people.",
 "Mentre Josh combatte.": "While Josh fights.",
 "Mentre cerchi di raggiungere Alice.": "While trying to reach Alice.",
 "Mentre difendi la barricata.": "While holding the barricade.",
 "Mentre esplori la casa.": "While exploring the house.",
 "Minacciate dal soldato.": "Threatened by the soldier.",
 "Necessaria per le opzioni sulla pistola e per gli esiti con «Usa la pistola».":
  "Needed for the gun options and for the outcomes with “Use Gun”.",
 "Necessario per sbloccare il capitolo «The Interrogation».": "Needed to unlock the chapter “The Interrogation”.",
 "Nel corridoio d'accesso.": "In the access corridor.",
 "Nel corridoio successivo.": "In the next corridor.",
 "Nel corridoio.": "In the corridor.",
 "Nel dialogo con North.": "In the conversation with North.",
 "Nel parcheggio, a sinistra entrando.": "In the parking lot, on the left as you enter.",
 "Nel portare Luther al sicuro.": "While carrying Luther to safety.",
 "Nel recinto.": "In the enclosure.",
 "Nell'inseguimento del deviante della cucina.": "While chasing the deviant from the kitchen.",
 "Nella barricata smantellata.": "In the dismantled barricade.",
 "Nella fuga finale.": "In the final escape.",
 "Nella sala trasmissioni.": "In the broadcast room.",
 "Nella stanza degli ospiti.": "In the guest room.",
 "Nella stanza delle prove.": "In the evidence room.",
 "North in chiesa.": "North in the church.",
 "North tra la gente.": "North among the people.",
 "Opening non ha un diagramma nel gioco: segna questo nodo quando hai giocato il capitolo.":
  "Opening has no flowchart in the game: mark this node once you have played the chapter.",
 "Per aiutare Josh.": "To help Josh.",
 "Per fuggire.": "To escape.",
 "Per raggiungere Alice.": "To reach Alice.",
 "Per salvare Hank.": "To save Hank.",
 "Percorso di Connor deviante.": "Deviant Connor path.",
 "Percorso di Connor macchina.": "Machine Connor path.",
 "Prima fuga.": "First escape.",
 "Prima variante: Kara troppo lenta.": "First variant: Kara too slow.",
 "Prima variante: Markus gravemente ferito. Collegamento da verificare: la linea parte dal gruppo «Accetta / Rifiuta» senza indicare la scelta.":
  "First variant: Markus badly wounded. Link to be verified: the line leaves the “Accept / Refuse” group without showing which choice.",
 "Prima variante: alla centrale.": "First variant: at the station.",
 "Prima variante: distrutto da Allen.": "First variant: destroyed by Allen.",
 "Prima variante: dopo «Non sparare».": "First variant: after “Don't Shoot”.",
 "Prima variante: fermato dal soldato.": "First variant: stopped by the soldier.",
 "Prima variante: il nuovo Connor uccide Connor.": "First variant: the new Connor kills Connor.",
 "Prima variante: nei corridoi.": "First variant: in the corridors.",
 "Prima variante: nel corridoio.": "First variant: in the corridor.",
 "Prima variante: nell'ascensore.": "First variant: in the elevator.",
 "Prima variante: nella prima fuga.": "First variant: in the first escape.",
 "Prima variante: prima del magazzino.": "First variant: before the warehouse.",
 "Prima variante: scambio di coscienza.": "First variant: consciousness swap.",
 "Prima volta, all'arrivo.": "First time, on arrival.",
 "Prima volta: dopo aver parlato con Alice.": "First time: after talking to Alice.",
 "Prima volta: per raggiungere Alice.": "First time: to reach Alice.",
 "Prima volta: scendendo dal camion.": "First time: getting out of the truck.",
 "Probabilità di successo al 100%: serve aver scoperto il nome del deviante.":
  "100% probability of success: you need to have found out the deviant's name.",
 "Probabilità di successo troppo bassa.": "Probability of success too low.",
 "Quando Alice corre da Kara.": "When Alice runs to Kara.",
 "Quando Alice deve avanzare.": "When Alice has to move forward.",
 "Quando Jericho viene attaccata.": "When Jericho is attacked.",
 "Quando Luther è nervoso.": "When Luther is nervous.",
 "Quando arriva la guardia costiera.": "When the coastguard appears.",
 "Quando i soldati si avvicinano.": "When the soldiers approach.",
 "Quando il soldato cerca chi sposti il corpo.": "When the soldier looks for someone to move the body.",
 "Quando la guardia si avvicina.": "When the guard approaches.",
 "Quando portano via Luther.": "When Luther is taken away.",
 "Quarta variante: aiutando Josh.": "Fourth variant: helping Josh.",
 "Quarta variante: alla breccia.": "Fourth variant: at the breach.",
 "Quarta variante: non riesci ad aiutare Markus.": "Fourth variant: you fail to help Markus.",
 "Quinta variante: nella sala macchine.": "Fifth variant: in the engine room.",
 "Resta sotto la pioggia per diversi minuti senza scegliere un riparo.":
  "Stay in the rain for several minutes without choosing a shelter.",
 "Richiede «Libera i mostri».": "Requires “Free Monsters”.",
 "Riesci nel QTE finale.": "Succeed in the final QTE.",
 "Rimanda alla sezione «Battle for Detroit - Kara Captured» (icona gialla nel gioco).":
  "Points to the “Battle for Detroit - Kara Captured” section (yellow icon in the game).",
 "Risparmia i poliziotti.": "Spare the police officers.",
 "Rivista": "Magazine",
 "Rivista (senza titolo nel diagramma).": "Magazine (no title in the flowchart).",
 "Rivista. Compare solo se in «On the Run» Connor ha inseguito Kara e Alice.":
  "Magazine. Only appears if Connor chased Kara and Alice in “On the Run”.",
 "Rivista. Compare solo se in «On the Run» Connor non ha inseguito Kara e Alice.":
  "Magazine. Only appears if Connor did not chase Kara and Alice in “On the Run”.",
 "Saltando sui soldati.": "By jumping on the soldiers.",
 "Scendendo dal camion.": "Getting out of the truck.",
 "Se conosci già la posizione di Jericho.": "If you already know where Jericho is.",
 "Se hai hackerato la telecamera nell'ascensore.": "If you hacked the camera in the elevator.",
 "Se in «Fugitives» Kara e Alice hanno dormito al motel.": "If Kara and Alice slept at the motel in “Fugitives”.",
 "Se in «Fugitives» Kara e Alice hanno dormito nel rifugio di Ralph.": "If Kara and Alice slept at Ralph's hideout in “Fugitives”.",
 "Se in «Fugitives» Kara e Alice hanno dormito nell'auto abbandonata.": "If Kara and Alice slept in the abandoned car in “Fugitives”.",
 "Seconda variante: Connor uccide il nuovo Connor.": "Second variant: Connor kills the new Connor.",
 "Seconda variante: Luther resta calmo.": "Second variant: Luther stays calm.",
 "Seconda variante: Markus spara a Connor.": "Second variant: Markus shoots Connor.",
 "Seconda variante: aiutando Luther.": "Second variant: helping Luther.",
 "Seconda variante: al bivio.": "Second variant: at the crossroad.",
 "Seconda variante: al piano -49.": "Second variant: on floor -49.",
 "Seconda variante: domande di Hank.": "Second variant: Hank's questions.",
 "Seconda variante: intervenendo.": "Second variant: by intervening.",
 "Seconda variante: lo scontro con Hank.": "Second variant: the fight with Hank.",
 "Seconda variante: nel magazzino.": "Second variant: in the warehouse.",
 "Seconda variante: nell'assalto finale.": "Second variant: in the final assault.",
 "Seconda variante: nella sala di controllo.": "Second variant: in the control room.",
 "Seconda variante: nella stanza delle prove.": "Second variant: in the evidence room.",
 "Seconda variante: superando il filo spinato o nello scontro col soldato. Collegamento da verificare: le linee partono dal gruppo «Il soldato ha la meglio su Kara / Kara ha la meglio sul soldato» senza indicare la scelta.":
  "Second variant: passing the barbed wire or in the fight with the soldier. Link to be verified: the lines leave the “Soldier Beats Kara / Kara Beats Soldier” group without showing which choice.",
 "Seconda variante: superando il filo spinato.": "Second variant: passing the barbed wire.",
 "Seconda volta: davanti alla tenda.": "Second time: in front of the tent.",
 "Seconda volta: mentre cerchi di raggiungere Alice.": "Second time: while trying to reach Alice.",
 "Seconda volta: per fuggire.": "Second time: to escape.",
 "Senza questo il capitolo non si gioca.": "Without this the chapter is not played.",
 "Serve aver accettato la molotov.": "You need to have accepted the Molotov.",
 "Serve il giocattolo rubato al supermercato.": "Requires the toy stolen from the store.",
 "Serve la barretta rubata al supermercato.": "Requires the chocolate bar stolen from the store.",
 "Serve per «I mostri tendono un agguato a Zlatko».": "Needed for “The Monsters Ambush Zlatko”.",
 "Servono i soldi del supermercato e i vestiti della lavanderia.": "Requires the money from the store and the clothes from the laundromat.",
 "Servono le cesoie: puoi rubarle al supermercato.": "Requires the clippers: you can steal them from the store.",
 "Solo se hai preso la pistola del poliziotto.": "Only if you took the police officer's gun.",
 "Solo se in «The Hostage» hai salvato il poliziotto ferito.": "Only if you saved the wounded officer in “The Hostage”.",
 "Sotto il fuoco della polizia.": "Under police fire.",
 "Stessa icona di bivio dell'esito con Luther.": "Same branch icon as the outcome with Luther.",
 "Stesso nome di un esito in «Kara Captured»: nodi separati.": "Same name as an outcome in “Kara Captured”: separate nodes.",
 "Stesso nome di un esito in «Kara Leaving Detroit»: nodi separati.": "Same name as an outcome in “Kara Leaving Detroit”: separate nodes.",
 "Sul nome del cane.": "About the dog's name.",
 "Sul nome del figlio.": "About his son's name.",
 "Sul tetto, con Simon.": "On the roof, with Simon.",
 "Sul tetto.": "On the roof.",
 "Sulla riva del fiume.": "On the riverbank.",
 "Sulla tomba di Carl.": "At Carl's grave.",
 "Tablet nella stanza di Emma. Serve per arrivare a «I cecchini sparano al deviante».":
  "Tablet in Emma's room. Needed to reach “Snipers Shot Deviant”.",
 "Tempo scaduto.": "Out of time.",
 "Terza variante: Hank uccide il nuovo Connor.": "Third variant: Hank kills the new Connor.",
 "Terza variante: nella cabina.": "Third variant: in the cabin.",
 "Terza variante: soldati in arrivo.": "Third variant: soldiers incoming.",
 "Terza variante: tempo scaduto nella fuga.": "Third variant: out of time during the escape.",
 "Terza variante: ucciso da un soldato.": "Third variant: killed by a soldier.",
 "Terza volta: in fila con Alice.": "Third time: in line with Alice.",
 "Terza volta: uscendo dalla tenda.": "Third time: leaving the tent.",
 "Uccidi i poliziotti.": "Kill the police officers.",
 "Una linea arriva anche direttamente da «Attack Successful».": "A line also comes directly from “Attack Successful”.",
 "Una linea arriva direttamente dal gruppo «Raggiungi il Canada…».": "A line comes directly from the “Reach Canada…” group.",
 "Uscito dall'ascensore, a sinistra: il pesce caduto dall'acquario.": "Out of the elevator, on the left: the fish that fell out of the aquarium.",
 "Variante con Connor illeso.": "Variant with Connor unharmed.",
 "Variante con Connor investito.": "Variant with Connor hit by the car.",
 "Verso la stiva.": "Towards the hold.",
 "Visto dal tetto.": "Seen from the roof.",
}


def note_en(note):
    """Traduce una nota italiana; solleva KeyError se manca nel dizionario."""
    if not note:
        return ""
    parts = []
    rest = note
    for it, en in ((REQ, REQ_EN), (KEY, KEY_EN)):
        if rest.startswith(it):
            parts.append(en)
            rest = rest[len(it):].strip()
    if rest:
        if rest in NOTES_EN:
            tr = NOTES_EN[rest]
            if rest.startswith(DUB) and not tr.startswith(DUB_EN):
                tr = DUB_EN + tr[0].lower() + tr[1:]
            parts.append(tr)
        else:
            raise KeyError("Nota senza traduzione in scripts/notes_en.py: " + repr(rest))
    return " ".join(parts)
