# Stand seit dem Planetengetriebe

Lokale Calcpad-Berechnung im Projekt, exportierte Referenzen, explizites
CAD-Aktualisieren und Berechnungsvarianten. Analytische native gerade
Außen-/Innenverzahnung und bewegliche Planetenbaugruppe. Weitere Zahnradarten
sind teilweise als Vorauslegung vorhanden; ihre vollständige CAD-Erzeugung
ist nicht allgemein umgesetzt.

Native Baugruppen werden in starre MKS-Blöcke mit Gelenken und mechanischen
Ports übersetzt. CAD-abgeleitete Masse, Schwerpunkt, Trägheit, lokaler
Exudyn-Lauf, vorgegebene Bewegungen, Ergebnisgültigkeit, 3D-Wiedergabe und
PNG/GIF-Export sind implementiert. Begrenzungen siehe MKS_DYNAMIK.md.

Der Roboter hat jetzt ein geprüftes gemeinsames Beispiel über Parameter,
CAD, sechsachsige starre MKS und Balken-FEM. Zwei Querschnittsvarianten,
Kraftpfad bis zum Fundament, Speichern/Laden und Veralten der Ergebnisse
wurden getestet. Start-Roboter-Alle-Domaenen.cmd öffnen.

Noch offen: Volumen-FEM, flexible FEM/MKS-Rückkopplung, allgemeine
Topologieoptimierung, vollständiges Werkzeugvariantenmanagement und SIL/
Hardwareansteuerung. Die Balken-FEM ist eine dokumentierte Idealisierung.

Anleitung: ROBOTER_DOMAENEN.md. Quantitative Belege, Screenshots, CSV und
NotebookLM-Bericht: output/robot-domains. Dissertation: DISSERTATION_FORSCHUNGSLOG.md.
