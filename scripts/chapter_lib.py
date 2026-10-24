import json
C,X,O,E="checkpoint","choice","objective","ending"
REQ="Requisito da un capitolo precedente (evidenziato in giallo nel gioco)."
KEY="Evento chiave (evidenziato in giallo nel gioco)."
def chap(num,char,rev,nodes,fname):
    ns=[]
    for t in nodes:
        i,it,en,ty,par=t[:5]; note=t[5] if len(t)>5 else ""
        d={"id":i,"title":it,"en":en,"type":ty,"parents":par}
        if note:d["note"]=note
        ns.append(d)
    ids={n["id"] for n in ns}
    bad=[(n["id"],p) for n in ns for p in n["parents"] if p not in ids]
    assert not bad,bad; assert len(ids)==len(ns),[i for i in ids if sum(n["id"]==i for n in ns)>1]
    json.dump({"chapter":num,"rev":rev,"source":"Struttura trascritta dal diagramma del gioco; nomi italiani tradotti, da verificare",
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
