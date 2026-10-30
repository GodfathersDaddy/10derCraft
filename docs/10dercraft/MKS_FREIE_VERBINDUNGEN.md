# Unabhängige MKS-Quellen und Verbindungen

Diese Stufe ergänzt die bisherige Bibliothek um **Freie Bewegungsquelle**. Sie ist ein eigener, verschiebbarer und gespeicherter Block. Ein Doppelklick öffnet Name, Parameterausdruck, physikalische Größe, Profil, Rampenzeit, Frequenz und Vorschau.

## Bewegung verbinden

1. Links **Freie Bewegungsquelle → +** wählen und die Quelle speichern.
2. Den blauen Ausgang der Quelle anklicken.
3. Den blauen q-Eingang eines Gelenks anklicken.
4. Das Ersetzen des bisherigen Bewegungseingangs im Verbindungsdialog bestätigen.
5. **Simulation starten** führt denselben lokalen Solverjob wie die bisherige Ansteuerung aus.

Eine Winkelquelle kann mehrere Drehgelenke mit demselben relativen Verlauf antreiben. Für unterschiedliche Hübe werden unterschiedliche Quellen benutzt. Eine Wegquelle passt zu Schubgelenken. Die Quelle bleibt unabhängig von den Gelenken bearbeitbar. Parameterausdrücke werden beim Vorbereiten des Jobs neu ausgewertet; Parameteränderungen machen ältere Ergebnisse historisch.

Eine blaue Leitung anklicken und **Eingang trennen** wählen oder außerhalb eines Eingabefeldes die Entf-Taste drücken. Danach ist das Gelenk frei; ein früher gespeicherter Antrieb wird nicht unbeabsichtigt wieder aktiv. Im Gelenkdialog kann ausdrücklich wieder zu lokalen Bewegungseinstellungen gewechselt werden. Das Löschen einer Quelle trennt sämtliche zugehörigen Eingänge. Änderungen laufen durch rückgängig machbare Befehle.

## Messausgang verbinden

Oben **Anschlüsse verbinden** einschalten, den grünen Gelenkausgang und danach den grünen Scope-Eingang anklicken. Die Messauswahl wird im Projekt gespeichert. Welche Größen ausgegeben werden, wird weiterhin im Gelenkdialog unter **Sensorik / Ausgänge** gewählt. Dort kann der Messausgang auch abgeschaltet werden. Kräfte verschiedener Gelenke werden im Scope verglichen, nicht addiert.

## Leitungen und Grenzen

Die neuen Quellenleitungen benutzen einen zwischengespeicherten rechtwinkligen Weg um die tatsächlichen Blockrechtecke. Verschieben eines Blocks berechnet den Weg neu. Findet der Router wegen überlappender Blöcke oder seiner begrenzten Suchgröße keinen freien Weg, zeigt er dies am Eingang an. Er zeichnet dann keine scheinbar freie Verbindung durch einen Block. Linien dürfen sich weiterhin kreuzen und gemeinsame Teilstrecken besitzen; Kreuzungsbrücken und manuelle Wegpunkte sind noch nicht enthalten. Die bisherigen Sammelleitungen für lokale Antriebe und Messungen bleiben separat.

Der Graph enthält derzeit vorgegebene Bewegungsquellen und native Gelenkeingänge. Er ist kein allgemeines Simulink-/Modelica-Gleichungssystem: frei kaskadierbare Signaloperatoren, PID-Regler, variable Sollkurven aus Tabellen, mehrere Scope-Blöcke und rückgekoppelte Mehrdomänensysteme sind weitere Aufgaben. Mechanische B/F-Verbindungen stammen weiterhin aus der CAD-Baugruppe und werden über native Gelenke bearbeitet.

## Befehle, Speicherung und Prüfung

`mks.diagram.source`, `.connect`, `.disconnect`, `.release`, `.remove` und `.read` sind registrierte Engine-Befehle mit Eingabe-/Ausgabeschemas und damit über die vorhandene Automatisierung erreichbar. Quellen und Verbindungen stehen im Projekt unter `org.solidcraft.mks-diagram`; Blockpositionen bleiben in der Darstellung gespeichert. Fehlende Graphdaten erhalten die bisherige lokale Ansteuerung unverändert. Graph-Eingänge überschreiben ausdrücklich die betreffenden lokalen Treiber bei `mks.job.create` und `mks.job.run`.

`tools/verify_mks_connections.py` prüft Typverträglichkeit, atomare Ablehnung ungültiger Verbindungen, Verzweigung einer Quelle, Trennen und Wiederherstellen lokaler Eingänge, Undo/Redo, Parameterweitergabe, Dateirundlauf sowie reale Bedienhandlungen. Ein kurzer Zweiarm-Lauf wird mit unabhängigen Quintic-Formeln verglichen. Anschließend werden die bisherigen sechs Roboterantriebe durch unabhängige Quellen ersetzt und die gemessenen Zustände verglichen. Der aktuelle Abschlussstatus steht in `output/mks-connections/verification.json`; eine Beschreibung des Verfahrens ersetzt kein bestandenes Ergebnis.
