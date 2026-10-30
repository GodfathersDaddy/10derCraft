# Spätere SolidCraft-Updates übernehmen

`origin` bezeichnet das eigene 10derCraft-Repository,
`upstream` das Original unter https://github.com/tangency-labs/solidcraft.
Die Zusammenführung auf 0.27.1 erhält beide bisherigen Entwicklungszweige sowie
die erste README des Zielrepositories als Git-Vorfahren. Dadurch kann Git bei
späteren Updates wieder eine gemeinsame Ausgangsbasis finden.

## Ablauf

1. Eigene Änderungen und Modelle sichern. `git status --short` zeigt, was noch
   nicht in einem Commit steckt. Die vollständige Projektprüfung bleibt vor
   jedem eigenen Commit erforderlich.
2. `git fetch upstream main` lädt die Originaländerungen, ohne Dateien im
   Arbeitsverzeichnis zu ersetzen.
3. Auf einem eigenen Integrationszweig `git merge upstream/main` ausführen.
   Konflikte anhand beider Änderungen lösen. Eine ganze Datei pauschal durch
   eine Seite zu ersetzen kann eigene Funktionen oder neue Originalfunktionen
   entfernen.
4. `cargo xtask ci` ausführen, danach den Desktop bauen und die wichtigsten
   Projekte öffnen. Berechnung → Referenzübernahme → CAD-Aktualisierung,
   Gelenke/Quellen/Sensoren und Videoexport gezielt prüfen.
5. Den geprüften Integrationsstand in den eigenen Hauptzweig übernehmen und
   normal nach `origin` schieben. Ein normaler Push schützt zwischenzeitlich
   hinzugekommene Remote-Commits vor Überschreiben.

## Stellen, die beim Vergleich Aufmerksamkeit benötigen

| Bereich | Eigene Erweiterung | Typische Integrationsfrage |
|---|---|---|
| Engine-Befehle | `mks_*`, `worksheet*`, `robot*`, `mechanics`, `gear*` | Befehlsregister, Symbole und Eingabe-/Ausgabeverträge zusammenführen; geprüfte Handler über `checked!` anbinden |
| Dokument | Berechnung, Referenzen, Schnittstellen, eigene Ergebnisse | Neue Dokumentfelder in Vergleichen berücksichtigen; Ergebnisdaten von CAD-Eingabesnapshots getrennt halten |
| Geometrie | Echte Evolventenkurve und -extrusion | Neue vollständige `match`-Ausdrücke müssen die eigenen Varianten behandeln oder ausdrücklich als nicht unterstützt zurückgeben |
| Oberfläche | Projektseiten, MKS und Berechnungsblatt | Gemeinsame Container, Navigation, neue Grafik-Frame-Felder und Caches übernehmen |
| Beispiele | `10derCraft.examples.json`, `examples/projects` | Pfade, eingebettete Berechnung, Varianten und aktuelle Ergebniszustände erhalten |
| Lokale Helfer | `simulation/`, `integrations/calcpad-local/` | Quellen und Einrichtungsdateien versionieren; installierte Umgebungen bleiben lokale Build-Ergebnisse |
| GitHub-Prüfung | Eigene CI auf GitHub-Rechnern | Originalvorlage liegt unter `.github/upstream-workflows/`; neue Originalprüfungen bewusst übernehmen |

Die übernommenen Original-Lizenzen, Copyrights und Dateiformat-Migrationen bleiben
erhalten. Die sichtbare Produktbezeichnung lautet 10derCraft. Technische
Crate-Namen, kompatible Dateiendungen und bestehende Konfigurationspfade müssen
dafür nicht pauschal umbenannt werden.

## Lokaler Start

`Start-10derCraft.cmd` unterstützt sowohl den bisherigen Arbeitsordner mit
Unterordner `solidcraft/` als auch einen direkt geklonten Repository-Stamm.
`Build-10derCraft.ps1` baut die passende Quelle. Ein geöffnetes älteres
Programm aktualisiert sich nicht durch den Git-Push; zum Verwenden des neuen
Builds den Starter erneut öffnen. Bereits geöffnete Modelle zuvor normal speichern.

Nach dem Klonen bleiben die Beispiele direkt im Repository. Die letzte
getestete lokale Python-Umgebung und der Calcpad-Helfer werden entsprechend
der README eingerichtet und nicht aus einem anderen Rechner kopiert.
