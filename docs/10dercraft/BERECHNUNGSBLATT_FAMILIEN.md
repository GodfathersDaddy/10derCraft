# Direkte Berechnung, Bibliothek und Familientabellen

Über Start-10derCraft.cmd neu öffnen; bereits laufende Fenster behalten ihre alte
Programmversion. Im Parameter-Reiter stehen oben Bibliothek, Familientabellen und
Symbole / Griechisch. Bibliothek verwendet das geschlossene/geöffnete Buch.

Referenzen: Referenz auswählen einschalten und den markierten Variablennamen
noch einmal anklicken. Dadurch wird die Auswahl aufgehoben. Bestehende CAD-Maße
behalten ihren zuletzt übernommenen Wert. Erneutes Markieren verbindet wieder
denselben CAD-Parameter. Kein Maß wird zum Abwählen gelöscht.

Ergebniseinheiten stehen näher an der Zahl. Bruch-Einheiten werden mit Bruchstrich
gerendert; Zahl und Einheit werden an ihrer mathematischen Grundlinie ausgerichtet.
Das betrifft auch Einheiten mit Potenzen. Intern bleibt die volle Genauigkeit.

Bibliothek: suchen, Datensatz aufklappen und einen einzelnen Kennwert auf das
Berechnungsblatt ziehen. Der Wert wird als normale Berechnung an der Ablageposition
eingefügt. Doppelte Namen werden nicht überschrieben. Der Katalog zeigt weiterhin
Quelle und Geltungsbereich. Die Schaltfläche zum Einfügen eines ganzen Datensatzes
bleibt vorhanden. Quellenprovenienz des Katalogs wird beim Ziehen einzelner Werte
noch nicht als eigenständiger Herkunftsdatensatz in der Formel gespeichert.

Familientabellen: Eine Familie benennen, Spalten bearbeiten, Zeilen hinzufügen und
Zellinhalte als Berechnungsausdrücke mit Einheiten eingeben. Speichern und auswählen
berechnet die betreffende Zeile. Gespeicherte Familien lassen sich oben im Editor
wieder laden. Die Vorlage FlangeBolt benennt thread_diameter, thread_pitch,
tap_drill_diameter und clearance_hole_diameter. Nullwerte sind Platzhalter, keine
normgerechte Schraubenauswahl. Eigene Tabellen können andere Felder besitzen.

Die ausgewählte Zeile liefert normale benannte Größen wie
FlangeBolt_clearance_hole_diameter. Ein CAD-Maß verwendet die markierte Referenz,
beispielsweise ref_FlangeBolt_clearance_hole_diameter. Beim Zeilenwechsel bleiben
die Namen gleich; die ausstehenden CAD-Änderungen werden mit Konstruktion
aktualisieren übernommen. Die Tabelle steuert Maße, setzt aber noch kein
vollständiges Schraubenbauteil automatisch in eine Baugruppe ein.

Dies ist ein Namensraum für Tabellenparameter, kein objektorientierter Parser
mit Punktzugriff, Vererbung oder Methoden. Zellen werden lokal durch Calcpad
berechnet; die Reihenfolge der Spalten ist für gegenseitige Abhängigkeiten relevant.
Die ausgewählten Familienparameter werden vor abhängigen Berechnungen definiert.
Familien sind im CAD-Projekt und im Export des Berechnungsblatts gespeichert.

Griechisch: Eine Formel aktivieren, Cursor setzen und oben unter Symbole / Griechisch
das gewünschte Zeichen anklicken. Zeichen werden in die aktive Eingabe eingesetzt.

Aktuelle Bedienung: Felder mit einfachem Ziehen verschieben, Text mit Doppelklick
bearbeiten. Klassenartige Eigenschaftsvorschläge gelten für alle Bibliotheks- und
Familieneinträge; Pfeile wählen, Tab übernimmt den berechenbaren Parameternamen.
