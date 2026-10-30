# MKS und grafisches Berechnungsblatt

## Bedienung

Die vorhandene CAD-Ansicht bleibt erhalten. Im MKS-Reiter liegt die Blockbibliothek links, der Blockplan in der Mitte und nach dem Start einer aktuellen Simulation die drehbare 3D-Wiedergabe rechts. Die Symbolleisten verwenden dieselben vorhandenen Symbole, Schrift- und Farbwerte wie CAD.

- Blöcke mit der linken Maustaste ziehen. Ihre Verbindungen folgen. Positionen werden im Projekt gespeichert.
- Doppelklick auf ein Gelenk: Bewegungsvorgabe, Weg-/Winkelausdruck, Rampenzeit und Kraftsensor einstellen.
- Bewegungseingang oder Gelenkkraftsensor aus der Bibliothek auf ein Gelenk ziehen.
- Eine konstante Kraft aus der Bibliothek auf einen beweglichen Körper ziehen. Im Dialog eine vorhandene CAD-Kraftschnittstelle aktivieren und Fx/Fy/Fz einstellen. Die Werte beziehen sich auf den mitbewegten Schnittstellenrahmen. Druckflächen werden durch diesen einfachen Krafteditor noch nicht bearbeitet.
- Doppelklick auf World/Ground: Schwerkraft einstellen. Eine CAD-Endmasse wirkt bereits unter Schwerkraft; ihre Gewichtskraft darf nicht zusätzlich doppelt angesetzt werden.
- Simulation starten; Abspielen/Pause und Ausgabepunkt stehen oben. Rechts gelten dieselben Dreh- und Zoomgesten wie im CAD.
- Scope oben einblenden oder den Scope-Block doppelt anklicken. Gezeigt werden lokale Gelenkreaktionskräfte in N, keine Antriebsmomente. Der initial noch nicht gültige Multiplikatorwert bei t=0 wird ausgelassen.
- Automatisch anordnen setzt nur die grafischen Positionen zurück.

Die Körper und mechanischen B/F-Verbindungen stammen aus der CAD-Baugruppe. Starr verbundene Bauteile sind ein Block. Neue mechanische Körper und Gelenke werden derzeit im CAD erzeugt. Im MKS-Plan lassen sich Signalanschlüsse über Drag-and-drop und die Gelenkeinstellungen ändern; beliebiges freies Neuverdrahten der mechanischen Baugruppe ist noch nicht enthalten.

## Grafisches Parameterblatt

Der Parameter-Reiter zeigt ein gerastertes Berechnungsblatt mit frei verschiebbaren Bereichen. Doppelklick öffnet die zugrunde liegende Definition. Die Darstellung unterstützt einfache Hochzahlen, Brüche und tiefgestellte Bezeichner. Das ist eine erste grafische Darstellung über den vorhandenen Rechenengines, kein vollständiger Mathcad-Formeleditor mit Cursorpositionen innerhalb eines zweidimensionalen Ausdrucks.

Oben können Formeln, Texte und Ergebnisdiagramme eingefügt werden. Bei neuen nativen Formeln muss die Zieleinheit passend angegeben werden, beispielsweise `mm`, `deg` oder leer für einen Skalar. Die vorhandene native Einheitenprüfung gilt weiterhin. Diagramme vergleichen benannte skalare Ergebnisse derselben Einheit als Balken; sie ersetzen noch keine Funktionsplots oder Parameterstudien.

Calcpad rechnet weiterhin lokal. Einfache Zuweisungszeilen erscheinen als grafische Formelbereiche. Änderungen an diesen Bereichen bearbeiten den eingebetteten Quelltext; danach lokal berechnen, Referenzen prüfen und Konstruktion aktualisieren. Komplexere Calcpad-Anweisungen bleiben unter **Quelltext / Tabellen** bearbeitbar. Native CAD-Formeln verwenden weiterhin den gemeinsamen CAD-Parametergraphen.

Varianten und die bewusste Übernahme publizierter `ref_…`-Werte bleiben erhalten. Das Layout einschließlich Texten und Diagrammen wird mit dem Projekt sowie beim `.scmath`-Export gespeichert.

## Architektur und Grenzen

`worksheet.presentation` ist ein über MCP und die Steuerschnittstelle erreichbarer, rückgängig machbarer Befehl zum Lesen und Speichern der Darstellung. Layoutdaten werden getrennt von Geometrie, Masseneigenschaften und Solverergebnissen gespeichert. Die MKS-Quellprüfung ignoriert diese reinen Darstellungsdaten. Das Verschieben eines Blocks macht einen gültigen Physiklauf deshalb nicht ungültig. Veränderte ausstehende Simulationsvorgaben werden dagegen vor einer neuen Rechnung nicht mit alten Ergebnissen überlagert.

Aktiver Solver bleibt Exudyn für starre offene Dreh-/Schubgelenkketten. Die Bibliothek enthält bereits Körper/World aus CAD, vorhandene Gelenke, Bewegung, Gelenkkraftsensor, Scope und konstante Portkräfte. OpenModelica wurde nicht installiert oder eingebettet. Die öffentliche Modelica-Dokumentation dient nur als Orientierung für Bibliothekskategorien; kein GPL-Quellcode und keine fremden Symbolgrafiken wurden übernommen.

Die mechanische Neuverdrahtung im Blockplan, allgemeine Modelica-Modelle, freie Gleichungssysteme, echte flexible MKS und ein vollständiger visueller Mathematikeditor bleiben nächste Ausbaustufen.

## Nachweise

Kurzer Bedien- und Integrationscheck: `tools/verify_engineering_canvas.py`. Er erzeugt reale Screenshots in `output/engineering-canvas/` und prüft Blockverschiebung, Ergebnisgültigkeit, Doppelklick, lokale Berechnung, CAD-Referenzübernahme, Layout-Rundlauf, Bibliotheksanschluss und drehbare Wiedergabe. Die Vorher-Aufnahmen liegen in `output/engineering-canvas-before/`. Breite alte Testreihen bleiben pausiert.

Die Aufnahmen zeigen die tatsächliche Anwendung. Ein zusätzlicher Ausschnittvergleich der Symbolleisten dient der visuellen Kontrolle; er ist keine Behauptung pixelidentischer Oberflächen.
