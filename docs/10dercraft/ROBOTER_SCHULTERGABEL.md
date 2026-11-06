# Parametrische Schultergabel des 8-kg-Roboters

## Nativer Aufbau

Das Beispiel **Roboter 8 kg – Schultergabel und flexible MKS** ersetzt den früheren Schulterstützblock und den überlagernden A1-Antriebssockel durch drei separate Frästeile. Die ersetzten Features und die alte Körperzuordnung bleiben unterdrückt in der Historie erhalten. Die vorhandene Sechsachs-Kinematik und die beiden elastischen Rohrprofile bleiben Bestandteil dieses Modellstands.

| Teil | Rohmaße in mm | Wesentliche Bearbeitungen |
|---|---|---|
| RB-A2-001 Grundplatte | 300 × 310 × 20 | Außenradien R10, sechs Ø11 auf Teilkreis 220 für M10, acht Ø8,5 für die Seitenplatten |
| RB-A2-002 Motorseite | 300 × 287 × 20 | Obere Radien R20, untere R10, Öffnung Ø150, sechs Ø9 auf Teilkreis 190, vier M8-Kernlöcher von unten |
| RB-A2-003 Gegenseite | 300 × 287 × 20 | Obere Radien R20, untere R10, Öffnung Ø210 als Freiraum, vier M8-Kernlöcher von unten |

Die gegenüberliegende Öffnung ist ein Montagefreiraum. Ein vollständig ausgelegter Lagerträger mit Lagersitzen und Wellenanschluss muss noch ergänzt werden.

Breite, Plattendicke, Grundplattentiefe, Eckradius, Einbauhöhen und Teilkreise stehen als benannte Yoke_-Parameter im CAD-Projekt. Boxen, Verrundungen, Bohrungen und kosmetische Gewinde verwenden normale native Features. Die vorhandenen Gelenkkoordinaten und die Katalogmaße des Motors begrenzen zulässige Änderungen dieser Einbaumaße. Eine Änderung der Schulterhöhe erfordert weiterhin die konsistente Änderung der Kinematik und angrenzender Teile.

## Befestigung

- Sechs ISO-4762-M10×50 mit sechs ISO-7089-M10 verbinden die Grundplatte mit dem vorhandenen A1-Drehkranz. Dieser erhält sechs Ø8,5-Kernlöcher und kosmetische M10×1,5-Gewinde.
- Acht ISO-4762-M8×40 mit acht ISO-7089-M8 verbinden die Seitenplatten von unten mit der Grundplatte. Die nominale Einschraublänge beträgt bei den vorliegenden Maßen etwa 18,4 mm.
- Sechs ISO-4762-M8×35 mit sechs ISO-7089-M8 verbinden die Motorseitenplatte mit dem vorhandenen 15-mm-Motoradapter. Dessen sechs frühere Ø9-Durchgänge werden zu Ø6,8-Kernlöchern mit kosmetischem M8×1,25. Die nominale Einschraublänge beträgt etwa 13,4 mm.

Die 40 Normteile sind einzelne Bibliothekskomponenten mit nativen, flächenbezogenen starren Montagegelenken. Die Schraubensitze beziehen sich ausdrücklich auf ihre Scheiben. Der Aufbau kann im normalen Programm über Bauteile, Bohrungen und Gelenke bearbeitet werden. Die Geometrie enthält kosmetische Gewinde und Kernlochdarstellungen. Reale Schraubenfestigkeitsklasse, Oberflächenzustand, Anziehmomente und Fertigungstoleranzen benötigen die weitere Auslegung.

Die drei Platten verwenden vorläufig das generische Material Aluminum mit 2,7 g/cm³. Schrauben und Scheiben sind explizit Steel zugeordnet. Die native geometrische Massenabschätzung ergibt 12,000 kg für diese Unterbaugruppe mit ihren 43 Körpern. Der konkrete Aluminiumwerkstoff ist noch auszuwählen.

## CAD-ROM-MKS

Die Schultergabel und ihre Normteile gehören zum vorgelagerten starren Achsblock. Auch ihr leerer Strukturcontainer erhält eine explizite starre Anbindung. Die abgeleitete MKS meldet anschließend keine schwebenden Blöcke. Die beiden Armrohre bleiben die elastischen ROM-Bereiche. Ihre übrigen CAD-Teile sind weiterhin explizit als starre Anbauteile an den Kraftübertragungsflächen zugeordnet.

Die neue Geometrie wurde vor der erneuten Bewegung über die nativen FEM-ROM-Befehle aktualisiert und reduziert. Die sechs Bewegungsquellen steuern die achtsekündige Bewegung aus statisch vorgespanntem Zustand. Das Ergebnis wurde nach dem Speichern und nach erneutem Öffnen als aktuell erkannt.

Der Lauf mit 4.000 Integrationsschritten und 501 Ausgaben ergab am Oberarm eine maximale elastische Knotenverschiebung von 0,13936 mm und am Unterarm 0,07588 mm. A2 erreicht einen Spitzenbetrag von lokalem Mz von 597,266 Nm und einen RMS-Wert von 373,040 Nm. Der maximale Gelenkpositionsrest beträgt 3,48e-9 m. Dieser Rest beschreibt die Erfüllung der Zwangsbedingungen.

Die native Solverrechnung benötigte in diesem Lauf 78,44 Sekunden, davon 75,41 Sekunden für die Zeitintegration. Gleichzeitige Builds und verschiedene Maschinenlasten verhindern einen isolierten Laufzeitvergleich mit anderen Ausführungen.

## Geometrische Kontrolle und Dateien

Alle drei Platten werden als geschlossene Volumenkörper gemessen. Die Timeline enthält im geöffneten Endmodell keine Fehler und keine Warnungen. Die begrenzte Kontrolle zwischen den drei Platten und A2-Schulter, Lagerschild, Nabenkappe sowie Adapter meldet keine Volumenüberschneidungen und keine ungeprüften Paare. Die Grundplatte besitzt 8 mm Abstand zum betrachteten Schulterkörper. Die Motorseitenplatte berührt ihren Adapter wie vorgesehen. Andere Bauteilpaare und die gesamte Bewegung benötigen eine gesonderte Kollisionsuntersuchung.

Die Belege enthalten STEP-Frästeile, Stückliste, Volumina, Masse, begrenzte Abstandsprüfung und den Rechenbericht unter evidence/robot-shoulder-yoke. Das vollständige native Projekt liegt unter examples/projects. Das Rezept examples/robot-shoulder-yoke-rom.json baut auf dem bisherigen Beispiel Roboter-8kg-Zwei-ROM.solidcraft auf und führt die Modellierung über normale Befehle aus.

## Offene mechanische Auslegung

Die motorseitige Verschraubung und die drei getrennten Platten sind weiter ausgearbeitet. Der tatsächliche A2-Getriebeaufbau, die Lagerpatrone, die Wellenanbindung und die zugehörigen bearbeiteten Gehäuse benötigen weiterhin eine vollständige Konstruktion. Die ideale starre ROM-Flächenanbindung bildet diese reale Anschlussnachgiebigkeit bislang vereinfacht ab. Vorhandene Achsgehäuse, weitere Motoren und Verkleidungen enthalten noch vereinfachte Geometrien. Netz-, Moden-, Zeitschritt-, Festigkeits-, Ermüdungs- und thermische Nachweise sowie Fertigungszeichnungen stehen aus. Die Gesamtbaugruppe besitzt weiterhin keine Fertigungsfreigabe.

Breite automatisierte Tests und vollständige CI bleiben auf Benutzerwunsch pausiert. Ausgeführt wurden normale native Konstruktion und Projektrechnung sowie Build, statische Prüfungen und dokumentierte Ansichten.

## Allgemeine Normteilmontage

Der normale Befehl parts.insert verarbeitet einen Bohrungs- oder Scheibenpunkt jetzt in Weltkoordinaten. Die Hostsuche berücksichtigt tatsächliche zylindrische Flächen und ihre Platzierung. Damit gelingt der zuvor fehlschlagende automatische M10-Schraubensitz auf einer bereits platzierten Scheibe. Die gemessene Schraubenspitze liegt bei z = 215 mm und die Kopfoberseite bei z = 275 mm, passend zur Sitzhöhe 265 mm und Schraubenlänge 50 mm. Der gezielte Regressionsfall wurde in der bestehenden Testdatei ergänzt. Seine automatische Ausführung bleibt mit den übrigen Tests pausiert.

## Darstellung und Ergebnisexport

Die detaillierte Baugruppe lässt sich mit dem begrenzt erweiterten Darstellungsbudget im MKS abspielen und als natives AVI/MJPEG mit 30 FPS und 241 Bildern exportieren. Das Budget beträgt 250.000 Dreiecke und 500.000 Vertices über alle dargestellten CAD- und FEM-Flächen. Die erfasste FEM-Ansicht nutzt 300-fache Darstellungsüberhöhung bei t = 1,264 s. Die berechneten Verschiebungen behalten ihren physikalischen Maßstab.
