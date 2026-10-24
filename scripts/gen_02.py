import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from chapter_lib import chap, C

OUT = os.path.join(os.path.dirname(__file__), "..", "data", "manual")

# 2 OPENING: capitolo lineare, nel gioco non ha un diagramma (verificato).
# Un solo nodo da segnare a mano, così il capitolo può arrivare al 100%.
chap(2, "K", "manual-1", [
 ("op-done", "Capitolo giocato", "Chapter Played", C, [],
  "Opening non ha un diagramma nel gioco: segna questo nodo quando hai giocato il capitolo."),
], os.path.join(OUT, "02-opening.json"))
