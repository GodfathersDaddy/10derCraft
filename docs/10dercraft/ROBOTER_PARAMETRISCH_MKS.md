# Roboter: Berechnung, CAD-Schnittstellen und Bewegung

Start: `Start-10derCraft.cmd`. Unter Beispiele das Modell **Roboter – parametrisiert und große Bewegung** öffnen. Die Originalbaugruppe bleibt erhalten.

Projektdatei: `output/mks-endless-robot/Roboter-Parametrisch-Grosse-Bewegung.solidcraft`.

## Berechnung und Aktualisierung

Im Reiter Parameter liegt das lokale Calcpad-Berechnungsblatt im Projekt. Eine zusätzliche lesbare Kopie ist `output/mks-endless-robot/Roboter-Berechnung.cpd`.

- `R_Oberarm` und `R_Unterarm` treiben über `ref_R_Oberarm` und `ref_R_Unterarm` die echten nativen CAD-Radiusbemaßungen an.
- `Hub_A1` bis `Hub_A6` treiben über die gleichnamigen `ref_…`-Referenzen die sechs unabhängigen MKS-Bewegungsquellen an.
- Die Berechnung enthält Querschnittsflächen, Flächenträgheitsmomente, idealisierte Zylindermassen und den Vergleich mit dem einzelnen Kragträger. Die im Blatt angegebenen Längen stammen aus dem bestehenden CAD-Balkenmodell; sie sind derzeit Auswertewerte und treiben keine automatische Änderung der Gelenklagen an.
- Nach einer Änderung **lokal berechnen**, geänderte CAD-Referenzen ansehen und **Konstruktion aktualisieren**. Danach MKS erneut starten. Veraltete Simulationen gelten nicht als aktuelle Ergebnisse.

Die vorhandenen Varianten bleiben erhalten; zusätzlich ist „Roboter · großer Bewegungsablauf“ gespeichert. Die Variante wird vor der Simulation gespeichert, damit deren Eingabestand eindeutig bleibt.

## Flächen und Kraftschnittstellen im CAD

1. CAD → Zusammenbau → CAD-Simulationsschnittstellen.
2. Eine ebene Bauteilfläche im 3D-Fenster auswählen.
3. Eine eindeutige ID und eine lesbare Beschriftung eingeben.
4. Kraftaufnahme, Kraftabgabe oder beides wählen. Diese Rolle beschreibt den vorgesehenen Lastpfad; die physikalische Verbindung bleibt wechselseitig.
5. **Flächenübergabe – Fläche aus CAD nachführen** aktivieren. Bei einer Punktkraft ist diese Option aus; eine Punktkraft kann kein freies Moment übertragen.
6. Fläche erfassen. Die Fläche und ihr Rahmen verwenden denselben persistenten nativen CAD-Flächennamen. Ändert sich die Bemaßung, werden Rahmen und Flächeninhalt aus der Geometrie erneut bestimmt. Ein fehlender oder mehrdeutiger Bezug darf keine alte Fläche als gültig weiterverwenden.

Im Reiter Schnittstellen kann die Beschriftung/Rolle später geändert werden. MKS zeigt beschriftete Anschlüsse an den Körperblöcken; über dem Anschluss stehen im Hinweis auch ID, Körper und Fläche. Bis zu vier Anschlüsse erscheinen direkt am Block. Weitere Anschlüsse sind in den CAD-Schnittstellen beziehungsweise Körpereinstellungen verfügbar.

Im Beispiel sind Fundament, Oberarm, Unterarm und Werkzeug mit Flächenanschlüssen versehen. Die bereits vorhandenen sechs CAD-Gelenke behalten ihre ursprünglichen Achsen und Lagerpositionen. Eine Lastfläche ersetzt nicht automatisch eine Gelenkachse. Neue Gelenke lassen sich zwischen zwei CAD-Schnittstellen erstellen.

Starre MKS überträgt eine Kraft-/Momentresultierende im Flächenrahmen. Daraus folgt noch keine räumliche Kontakt- oder FEM-Druckverteilung. Der ermittelte Flächeninhalt verwendet die native Geometrieauswertung mit 0,01 mm Toleranz.

## MKS und Video

Das Gitter wird über den gesamten sichtbaren, verschiebbaren Bereich gezeichnet, auch außerhalb des Blockbereichs und bei negativen Weltkoordinaten. Auf freiem Hintergrund mit mittlerer oder rechter Maustaste verschieben; Blöcke mit links verschieben. „Automatisch anordnen“ setzt die Diagrammansicht zurück.

Die sechs Sinusquellen verwenden 0,8 s weichen Anlauf und Frequenzen von 0,25 bis 0,5 Hz. Die Amplituden sind 90°, 35°, 50°, 90°, 45° und 120°. Ausgeführt werden 8 s mit 3.960 Integrationsschritten und 67 gespeicherten Zuständen. Die vorhandene Nutzlast beträgt 8 kg. Die Joint-Sensoren gehen in einen gemeinsamen Scope; dort sind lokale Gelenkkräfte und gemessene Gelenkkoordinaten verfügbar. „Abspielen“ öffnet die drehbare 3D-Wiedergabe rechts.

`output/mks-endless-robot/Roboter-Grosse-Bewegung.avi` enthält die tatsächlichen berechneten Körperposen. 66 Ausgabeintervalle bei 8,25 Bildern/s ergeben 8 s Laufzeit; der letzte Endzustand ist separat als PNG gespeichert. Eine feste Kamera und ein für alle Bilder identischer Bildausschnitt halten die Bewegung vergleichbar. Rohbilder und Rohdaten bleiben erhalten. Eine kleine GIF-Vorschau liegt daneben.

Die Bewegung ist eine Starrkörpersimulation mit vorgeschriebenen Achsbewegungen. Kollisionsfreiheit, Antriebsdimensionierung und dynamische elastische Verformung sind damit nicht nachgewiesen. Die separat gespeicherte Balken-FEM beschreibt die vorhandene lineare, statische Idealisation mit verriegelten Antrieben unter 100 N TCP-Last; sie ist keine flexible MKS.

## Reproduzierbare Nachweise

- `verification.json`: Radius-/Flächennachführung, Fehlerabweisungen, gemessene Achsausschläge, Dateirundlauf, Original-Hash.
- `ui-verification.json`: gezielte Bedienprüfung der neuen Felder und Diagrammnavigation.
- `dynamics-result.json`, `simulation-state.json`, `Gelenkbewegung.csv`, `Gelenkkraefte.csv`: berechnete Ergebnisse und Eingangszustand.
- `robot-beam-fem.json`: statische Balken-FEM mit Annahmen und Grenzen.
- `video-verification.json`: Container, Bildgröße, Bildanzahl und Laufzeit.
- `provenance.json`: ausgelieferte Programmversion und Dateihashes.

Alte umfangreiche Testläufe bleiben pausiert. Diese Nachweise sind gezielte Integrations- und Bedienprüfungen; eine experimentelle Validierung und eine Netz-/Zeitschritt-Konvergenzstudie gehören in einen gesonderten Dissertationsversuch.
