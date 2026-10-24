import json
SOURCE="Trascritto dai diagrammi del gioco / Transcribed from the in-game flowcharts"
def _node(t):
    from notes_en import note_en
    i,it,en,ty,par=t[:5]; note=t[5] if len(t)>5 else ""
    d={"id":i,"title":it,"en":en,"type":ty,"parents":par}
    if note: d["note"]=note; d["note_en"]=note_en(note)
    return d
C,X,O,E="checkpoint","choice","objective","ending"
REQ="Requisito da un capitolo precedente (evidenziato in giallo nel gioco)."
KEY="Evento chiave (evidenziato in giallo nel gioco)."
def chap(num,char,rev,nodes,fname):
    ns=[_node(t) for t in nodes]
    ids={n["id"] for n in ns}
    bad=[(n["id"],p) for n in ns for p in n["parents"] if p not in ids]
    assert not bad,bad; assert len(ids)==len(ns),[i for i in ids if sum(n["id"]==i for n in ns)>1]
    json.dump({"chapter":num,"rev":rev,"source":SOURCE,
      "sections":[{"name":"","char":char,"nodes":ns}]},open(fname,"w"),ensure_ascii=False,indent=1)
    print(fname,len(ns),"nodi")
def promise(p,par):
    """Talk to Alice -> promessa -> giocattolo/cioccolata -> come dorme Kara"""
    return [
     (p+"-promise","Fai una promessa","Make Promise",X,[par]),
     (p+"-nopromise","Non fare promesse","Don't Make Promise",X,[par]),
     (p+"-toy","Dai il giocattolo","Give Toy",X,[p+"-promise",p+"-nopromise"],"Serve il giocattolo rubato al supermercato."),
     (p+"-choc","Dai la cioccolata","Give Chocolate",X,[p+"-promise",p+"-nopromise"],"Serve la barretta rubata al supermercato."),
    ]
def hair(p,par):
    return [(p+"-hair","Taglia e tingi i capelli","Cut and Color Hair",X,par,KEY),
     *[(p+"-"+k,it,en,X,[p+"-hair"]) for k,it,en in [("brown","Castani","Brown"),("black","Neri","Black"),("white","Bianchi","White"),("blonde","Biondi","Blonde")]],
     (p+"-led","Rimuovi il LED","Remove LED",X,[p+"-brown",p+"-black",p+"-white",p+"-blonde"])]
def chap_sections(num,rev,sections,fname):
    """Capitoli con più diagrammi: sections = [(nome, char, nodi), ...]. Gli id devono essere unici in tutto il capitolo."""
    out=[];allids=[]
    for name,char,nodes in sections:
        ns=[_node(t) for t in nodes]
        ids={n["id"] for n in ns}
        bad=[(n["id"],p) for n in ns for p in n["parents"] if p not in ids]
        assert not bad,bad; assert len(ids)==len(ns),name
        allids+=list(ids); out.append({"name":name,"char":char,"nodes":ns})
    assert len(allids)==len(set(allids)),"id duplicati tra sezioni"
    json.dump({"chapter":num,"rev":rev,"source":SOURCE,
      "sections":out},open(fname,"w"),ensure_ascii=False,indent=1)
    print(fname,", ".join(f"{s['name']}: {len(s['nodes'])} nodi" for s in out))
