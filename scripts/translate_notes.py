"""Aggiunge note_en e aggiorna source in tutti i file di data/manual/.

Serve per i capitoli scritti a mano (1-21), che non hanno uno script generatore;
gli script gen_*.py lo fanno già da soli. Si può rilanciare quante volte si vuole.
"""
import sys, os, json, glob
sys.path.insert(0, os.path.dirname(__file__))
from chapter_lib import SOURCE
from notes_en import note_en

DIR = os.path.join(os.path.dirname(__file__), "..", "data", "manual")
for f in sorted(glob.glob(os.path.join(DIR, "*.json"))):
    d = json.load(open(f, encoding="utf-8"))
    d["source"] = SOURCE
    for s in d["sections"]:
        for n in s["nodes"]:
            if n.get("note"):
                n["note_en"] = note_en(n["note"])
            else:
                n.pop("note_en", None)
    json.dump(d, open(f, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print("note_en aggiornate in", len(glob.glob(os.path.join(DIR, "*.json"))), "file")
