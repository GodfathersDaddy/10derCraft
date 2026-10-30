# CAD, Systemblöcke und Strukturanalyse

Stand 28.10.2026. Dieser Text beschreibt den Zielworkflow und grenzt ihn von
den bereits umgesetzten Funktionen ab. Der aktuelle Bedienstand steht in
[FEM_MKS_ARBEITSBEREICHE.md](FEM_MKS_ARBEITSBEREICHE.md).

## Gemeinsame Modellidentität

CAD-Bauteile und ihre Einbauvorkommen sind die gemeinsame Quelle. MKS und FEM
erhalten daraus unterschiedliche Analysemodelle. Ein MKS-Starrkörperblock ist
keine automatische FEM-Vernetzung und keine Aussage über die reale Steifigkeit.

Jeder Systemblock soll eine echte, automatisch aus den zugehörigen CAD-Körpern
gerenderte isometrische Vorschau erhalten, ähnlich den Karten der Startansicht.
Keine Symbolbilder und keine Aufnahme der gesamten Baugruppe als Bauteilbild.
Bei mehreren starr verbundenen Bauteilen zeigt die Vorschau die ganze Gruppe.
Sichtbar: Bauteil-/Modulname, Fundamentstatus, Anzahl der enthaltenen Teile,
Anschlussbezeichnungen und Analysezustand. Aufklappen zeigt alle Einbaupfade.
Anklicken soll die zugehörigen Vorkommen im CAD markieren; umgekehrt markiert
eine CAD-Auswahl die betreffenden Blöcke. Ein expliziter Befehl öffnet die
isolierte Geometrieansicht. Geometrie-, Varianten- und Positionsänderungen
invalidieren die passende Vorschau; die Bilderzeugung darf die Bedienung nicht
blockieren. Browser und fehlende Renderer brauchen eine beschriftete Ersatzkarte.

## Ablauf

1. Parameter und Variante wählen, lokal berechnen, geänderte Referenzen prüfen
   und die CAD-Konstruktion aktualisieren.
2. Im CAD Material, Baugruppenstruktur, Gelenke und mechanische Anschlüsse
   definieren. Anschlussdefinition und Lastfall bleiben getrennt: Ein Flansch
   existiert auch ohne aufgebrachte Kraft.
3. In MKS starre Gruppen automatisch ableiten, Fundament, Gelenke, Antriebe und
   Lasten prüfen. Die Blöcke zeigen dieselben Bauteilidentitäten wie das CAD.
4. In FEM die zu untersuchenden Teile auswählen und deren Idealisierung festlegen:
   Balken, Schale oder Volumen. Verbindungen, Lagerungen, Material und Netz
   gehören zum FEM-Modell. Starr verbundene MKS-Teile dürfen hier getrennt bleiben.
5. Zunächst einen statischen Lastfall auswerten. Danach als gesonderte Erweiterung
   ausgewählte MKS-Zeitpunkte mit Kräften, Momenten, Beschleunigungen und
   Koordinatentransformationen an FEM übergeben. Eine reine äußere TCP-Kraft
   ersetzt keine dynamische Lastübertragung einschließlich Trägheitslasten.
6. Verformung, Spannung, Masse und Grenzwertverletzungen zur Variante zurückführen.
   Bei Änderungen betroffene Modelle und Ergebnisse als veraltet markieren;
   erst nach bestätigter Aktualisierung und Neuberechnung freigeben.

## Wo Anschlüsse entstehen sollen

Im CAD: Körper und Vorkommen auswählen, Punkt/Fläche/Flächengruppe markieren,
danach „Mechanischen Anschluss anlegen“. Der vorübergehende Dialog definiert
Name, Geometriebezug, lokalen Ursprung und Achsen sowie zulässige Übertragung.
MKS und FEM verwenden diesen gemeinsamen Anschluss und ergänzen ihre eigenen
physikalischen Bedingungen. Die bestehenden oberen Reiter bleiben erhalten.

| Anschluss | Gemeinsame Definition | MKS-Verwendung | FEM-Verwendung |
|---|---|---|---|
| Punktkraft | Punkt und lokaler Rahmen | Kraft am Hebelarm | Punkt-/Knotenlast; lokale Spannung kritisch interpretieren |
| Rahmen | Ursprung, Achsen, Kraft und Moment | Gelenk, Antrieb oder äußere Last | Explizit gewählte Kopplung an die Struktur |
| Fläche | CAD-Flächenbezug und Orientierung | Resultierende an einem Rahmen | Druck, verteilte Last, Lagerung oder Kontakt nach Auswahl |

Eine Fläche allein legt weder Einspannung noch starre Kopplung fest.
Kraft/Moment sind Resultierende; Druck ist eine Flächenverteilung. Die spätere
FEM-Abbildung muss Kräfte und Momente erhalten. Keine pauschale Verteilung
auf Netzknoten ohne dokumentiertes Kopplungsmodell. Geänderte oder verlorene
Flächenreferenzen müssen gemeldet werden, niemals still auf eine andere Fläche zeigen.

## FEM als Blockstruktur

Sinnvoll als Übersicht über Bauteile, Verbindungen, Lastfälle und Ergebnisse.
Die eigentliche Netzerstellung und Randbedingungsauswahl erfolgt am 3D-Modell.
Beispiel: Oberarm — Achsverbindung — Unterarm — Werkzeuganschluss. Ein Block
kann ein detailliertes FEM-Modell oder später ein reduziertes elastisches Modell
repräsentieren. Eine beidseitige flexible MKS/FEM-Kopplung ist ein späterer,
gesondert zu validierender Ausbau, kein Ergebnis einer gezeichneten Verbindung.

## Vorhanden und noch zu bauen

Vorhanden: native MKS-Gruppierung mit Mitgliedsnamen und Einbaupfaden,
Gelenken und Ports; mechanischer Porteditor für Körper/Vorkommen, lokale
Position/Orientierung, Punktkraft, Rahmenresultierende und deklarierte Druckfläche.
Erreichbar im MKS-Modell unter „Kraftschnittstellen und statische Lastfälle
bearbeiten“, dort „Schnittstelle definieren / ändern“.
Die deklarierte Fläche wird bisher nicht aus einer CAD-Flächenauswahl gemessen.
Der Roboter besitzt eine begrenzte Balken-FEM, keine allgemeine Volumen-FEM.

Inzwischen umgesetzt: eigener MKS-Arbeitsbereich mit bauteilbezogenen
Blockvorschaubildern, FEM-Vorbereitung mit sichtbarem CAD sowie eine integrierte
Verformungsansicht und PNG-Export der vorhandenen Roboter-Balkenrechnung.

Noch zu bauen: CAD/Block-Auswahlsynchronisation,
persistente CAD-Flächenreferenzen im Porteditor, allgemeine FEM-Modellbildung,
automatische konservative MKS-Lastübertragung und flexible Rückkopplung.

## Umsetzung und Nachweis

Erster Schritt: Bildkarten mit vollständiger Bauteilzuordnung, Auswahl und
revisioniertem Vorschaucache. Danach geometrische Anschlussauswahl und
Prüfung verlorener Referenzen. Erst danach allgemeine FEM und Lastübertragung.
Alle Schreibaktionen über registrierte Befehle für dieselbe Bedienbarkeit per MCP.

Gezielte Nachweise: zwei gleichnamige Teile an verschiedenen Einbaupfaden richtig
zuordnen; starre Mehrteilgruppe vollständig abbilden; Variantenwechsel invalidiert
Bilder und Analysestand; Kraft und Moment vor/nach einer Lastübertragung vergleichen;
verlorene Flächenreferenz blockiert die Freigabe; FEM-Netzkonvergenz getrennt von
der MKS-Zeitintegration untersuchen. Für die Dissertation Bearbeitungszeit,
manuelle Zuordnungsschritte und Fehler gegenüber einem getrennten Workflow erfassen.
