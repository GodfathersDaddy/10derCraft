# Start und geschützte Upstream-Integration

Das Programm wird nur noch mit **Start-10derCraft.cmd** gestartet.
Ohne weitere Argumente erscheint die native Startseite. Unter **Beispiele**
liegen zwölf Projekte, darunter das neue Pendel mit Endgewicht, Roboter,
Planetengetriebe und Berechnung/CAD.
Geöffnete Projekte erscheinen anschließend automatisch unter **Zuletzt verwendet**.
Die bisherigen Einzelstarter sind als Sicherung unter
`archive/start-files-2026-10-28` abgelegt; sie sind keine aktiven Einstiegspunkte.

Die sichtbare Anwendung heißt **10derCraft**. Technische Crate-Namen,
Dateiformate und bestehende Einstellungs-/Wiederherstellungspfade bleiben
kompatibel. Herkunft und Lizenzhinweise zu SolidCraft bleiben erhalten.
Der Projektkatalog liegt in `10derCraft.examples.json`; seine Pfade sind
relativ zum Projektordner. Er wird unabhängig vom Arbeitsverzeichnis gefunden.

## Sicherung und Integration

Vor den Änderungen wurde `backups/before-upstream-20261028-093115` erstellt:
2.292 gesicherte Quelldateien/Modelle, SHA-256-Manifest, Git-Status, getrennte
Patches für Index und Arbeitsverzeichnis und Kopien der Git-Metadaten.
ZIP-Inhalt und Hashes wurden erneut gelesen und geprüft.

Ursprüngliche Basis: `d8eb65d26e36af74fc573c094f3106d0626fbcb6` (0.16.2).
Abgerufener Upstream: `ae07e36a7d09c6095f4de1bc7ab51067e2418093` (0.25.1),
965 zusätzliche Commits, 2.027 geänderte Dateien. Eine Aktualisierung ist
daher eine API- und Oberflächenmigration, kein einfaches Überschreiben.

Die Zusammenführung erfolgt getrennt in
`integration/solidcraft-upstream-2026-10-28` auf einer lokalen Kopie der
vollständigen Weiterentwicklung. Die Herkunft wird in
`integration/upstream-integration.json` festgehalten. Ein abgerufener oder
textuell zusammengeführter Stand gilt erst nach Build und Modellprüfungen
als aktivierbare Version.

## Aktivierter Stand, 28.10.2026

Die geprüfte Integration ist im bisherigen Quellordner `solidcraft` aktiv,
auf Branch `10dercraft/upstream-2026-10-28`.
Merge: `226fc69aa3fa865cf9129100b0600bdf13035986`.
Abschließende UI-Korrektur: `b8e1270e73a5c8106b533740d651ae05760c3efa`.
`Start-10derCraft.cmd` startet die neue gebaute Version 0.25.1.
Die Ergebnisfenster verwenden die neue native UI; die vier Engineering-Reiter
und deine eigenen Funktionen bleiben erhalten.

Der vorherige Branch `tendercraft/studio-prototype` bleibt erhalten.
Sein exakter vorheriger Index und Arbeitsstand einschließlich ungetrackter
Erweiterungen liegt zusätzlich im Git-Stash
`a7f8e204765d524a05948b0fb2a68241b7c5382d`.
Die zweite geprüfte ZIP-Sicherung liegt unter
`backups/before-upstream-20261028-102028`; dort liegt auch die vorige
Programmdatei. Es wurde kein Reset und kein Push nach GitHub durchgeführt.
Die Sicherungen und den Stash nicht als temporäre Build-Dateien löschen.

Prüfungen: Desktop-Build, Clippy für Engine/UI mit `-D warnings`,
WASM-Engine-Check, Laden aller elf Modelle ohne Featurefehler,
Calcpad → Moduländerung → CAD, Ablehnung einer unzulässigen Auslegung,
KUE-Variantenwechsel und Wiederladen, Roboter-Varianten/MKS/Balken-FEM,
Startseitenklick und reale Zuletzt-verwendet-Einträge, PNG/GIF/Kamera und
Originalfarben. Die alten großen Testläufe bleiben pausiert.
Der WASM-Check meldet eine vorhandene Upstream-Warnung zu einem dort
ungenutzten Feld; er besteht. Vollständige CI wurde nicht ausgeführt.

Originale Beispieldateien wurden nicht überschrieben. Neu gespeicherte
Kompatibilitätsversuche und Screenshots liegen unter `output/upstream-acceptance`.
Vorhandene alte Solverergebnisse können wegen des neuen Software-/Dokumentstands
als historisch gelten; dann die betreffende Studie neu berechnen.
Das schützt vor der Anzeige unpassender alter Ergebnisse.

Provenienz und Rückweg: `integration/activation.json`,
`integration/activation-git-log.json`, `integration/local-preservation-audit.json`.
Der Audit prüfte alle 2.179 Quellpfade des lokalen Snapshots und fand keine
unbekannten nachträglichen Änderungen oder neuen unberücksichtigten Dateien.
Bei einer späteren Rückkehr zuerst seitdem entstandene Änderungen sichern;
der alte Branch allein enthält noch nicht den damaligen uncommitteten Stand,
dieser liegt im oben genannten Stash und in den ZIP-Dateien.
