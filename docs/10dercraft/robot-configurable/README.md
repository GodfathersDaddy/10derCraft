# Konfigurierbarer 10der-Rohrroboter

Die native Datei `examples/projects/10der-Rohrroboter-Konfigurierbar.solidcraft` ist ein akademischer CAD-Entwurf anhand der bereitgestellten UR7e-STEP-Vorlage. Die verwendete Referenzdatei heisst UR7e.step und hat SHA-256 f27a7b314170af2e5426084b85c85304f1961c803c972bfacdcabf8fddb1fa26. Die beigelegten Herstellerbedingungen gelten fuer die verwendete Referenz.

## Bedienung

Projekt in 10derCraft oeffnen. Unter Start die Variantenansicht waehlen. Im Bereich CAD-Konfigurationen eine Zeile auswaehlen und Konfiguration uebernehmen klicken. Familientabelle bearbeiten oeffnet die native Tabelle mit bearbeitbaren Werten und Feature-Unterdrueckungen. Im CAD stehen dieselben Funktionen unter Konfigurieren zur Verfuegung.

Sechs Zeilen kombinieren Kompakt, Standard und Lang jeweils mit Greifer oder Sauger. Die Werkzeugauswahl schaltet alle erzeugenden Features der jeweiligen Werkzeuggruppe gemeinsam. Die Alternativen sind im Projekt gespeichert. Die Anbindung beliebiger externer Moduldateien ist ein weiterer Ausbauschritt.

## Parameter

Upper_Tube_Length und Forearm_Tube_Length sind die freien Rohrlaengen zwischen den Rohrflanschen. Upper_Tube_Diameter und Forearm_Tube_Diameter bestimmen die Aussendurchmesser. Upper_Tube_Wall und Forearm_Tube_Wall bestimmen die Wandstaerken. Joint_Offset steuert den seitlichen Abstand der Armebenen. Schrauben und Flansche folgen den Rohrmassen und Montagepositionen. Die Sechskantkonturen sind ueber Ausdruecke in normalen Skizzen definiert.

## Aufbau und Grenzen

Die Rohre tragen eigene Anschlussflansche mit Zentrierbund. Gegenflansche und Stutzen bilden separate Schweissteile auf bearbeiteten Gehaeusesitzen. 24 Flanschverschraubungen besitzen Schaft, Kopf mit Innensechskant und Sechskantmutter. Die Gewinde sind geometrisch vereinfacht. Drei blaue Deckel tragen erhabene native 10der-Schriftzuege. Die Basis besitzt sechs M10-Durchgangsbohrungen.

Der Entwurf benoetigt noch eine vollstaendige Antriebs-, Lager-, Naht- und Schraubenauslegung, Montagezugangs- und Kollisionspruefung. Die Gelenkgehaeuse sind vereinfachte Platzhalter. Fuer dieses neue CAD-Modell wurden keine bewegliche Gelenkkette und keine FEM/ROM-MKS-Ergebnisse erzeugt. Es ist keine freigegebene Fertigungsbaugruppe.

## Gezielte Pruefung

Alle sechs Konfigurationen wurden nativ aktiviert. Die Rohre blieben geschlossen. Die Volumina wurden gegen pi*(D*t-t*t)*L verglichen. Die maximale relative Abweichung liegt unter 0,0001. Als tolerierter Grenzwert fuer die tessellierungsbasierte Messung wurde 0,0005 verwendet. Ausschliesslich die gewaehlte Werkzeuggruppe war in der aktiven Geometrie vorhanden. Die Zusammenfassung steht in `verification-summary.json`, der Oberflaechennachweis in `ui-verification.json`.

Referenzhinweis: Copyright 2023 Universal Robots A/S. Use hereof is subject to Universal Robots A/S Terms and Conditions for Use of Graphical Documentation. Die mitgelieferten Bedingungen beschreiben die Nutzung der Referenz und daraus abgeleiteter Arbeiten.
