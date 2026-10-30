# Native MKS-Bibliothek und Robotersteuerung

**Aktualisierung 28.10.2026:** Die nachfolgende Beschreibung dokumentiert die erste Bibliotheksstufe. Inzwischen sind drei Bewegungsprofile, Quellenvorschau sowie gemessene Gelenkpositionen und Geschwindigkeiten ergänzt. Aktuelle Bedienung, Grenzen und Prüfergebnisse stehen in [MKS_BEWEGUNGSQUELLEN_UND_SENSOREN.md](MKS_BEWEGUNGSQUELLEN_UND_SENSOREN.md). Das Roboterbeispiel im Start-Katalog verwendet diese neue Stufe.

Diese Ausbaustufe erweitert die gemeinsame Bibliothek, nicht nur eine Roboter-Demonstration.

## Bibliothek

Links stehen World/Ground, starrer Körper, Weld/Fixed, Revolute, Prismatic, Bewegungssteuerung, Nutzlast, Bewegungseingang, Kraftsensor, Scope und konstante Kraft bereit. Das Plus öffnet den Einfügedialog. Bibliotheksbausteine können auch in die Blockfläche gezogen werden. Bewegung und Sensor werden auf ein Gelenk gezogen, Nutzlast auf einen Körper mit vorhandener CAD-Anschlussschnittstelle. Die rechte bzw. mittlere Maustaste verschiebt den Blockplan; Strg+Mausrad bzw. die Touchpad-Zoomgeste zoomt. Die 3D-Ergebnisansicht rechts benutzt weiterhin die CAD-Navigation.

World ist der vorhandene eindeutige Weltbezug. Ein weiterer unverbundener Weltbezug würde keine zusätzliche physikalische Funktion erfüllen und wird deshalb nicht als zweite Welt erzeugt.

Körper werden über `mks.library.body` als normale CAD-Komponente mit parametrischem Quader und zugewiesener Ersatzdichte erzeugt. Die gewählte Masse ist damit eine wirkliche träge Masse unter Schwerkraft. Bei einer Nutzlast kommt eine native starre Verbindung zur gewählten CAD-Schnittstelle hinzu. Die Schwerpunktslage liegt zunächst im Zentrum des Quaders an der Anschlussschnittstelle. Eine realistischere Schwerpunktlage kann durch Bearbeiten des nativen Bauteils bzw. seiner Platzierung modelliert werden.

`mks.library.joint` erzeugt echte native Dreh-, Schub- oder starre Gelenke zwischen den ausgewählten Occurrences. Punkt und Achse sind Weltkoordinaten; sie werden durch den normalen nativen Gelenkbefehl übernommen. Ein Körper, der noch nicht an World bzw. eine verbundene Kette angeschlossen ist, bleibt ausdrücklich ein schwebender Körper und kann noch nicht dynamisch berechnet werden.

Ein zusätzliches Gewicht wird bei der Starrkörperreduktion mit dem angeschlossenen Teil zu einem starren Block zusammengefasst. Im Diagramm bleibt ein eigener Nutzlast-Baustein mit seiner Anbindung sichtbar. Er ist keine weitere freie Masse und erzeugt keine zusätzliche Zwangsbedingung im Dynamikmodell. Doppelklick öffnet seine Masseneinstellung. `mks.library.mass` ändert die native Ersatzdichte anhand der aktuellen nativen Masse, sodass auch bei bearbeiteten Quaderabmessungen die gewünschte Masse eingestellt wird. Das verändert Trägheit und Schwerkraftlast und macht ältere Ergebnisse historisch.

## Gelenkblock

Der Doppelklickdialog trennt Körperanschlüsse, Bewegungsansteuerung und Messausgänge. Die Bewegung wird als relativer Kosinus-Hub von der aktuellen CAD-Pose mit anschließender Haltephase vorgegeben. Frei bedeutet, dass keine solche Bewegungsbindung gesetzt wird. Native Gelenkgrenzen werden vor dem Lauf geprüft; ein physikalisches Anschlagmodell ist weiterhin nicht enthalten.

Ausgänge: lokale Reaktionskräfte Fx, Fy, Fz und/oder Kraftbetrag. Diese Auswahl wird gespeichert und ist am Joint-Ausgang bzw. Sensoranschluss abgebildet. Die gewählten Ausgänge werden im gemeinsamen Scope mit gemeinsamer Zeitachse zusammengeführt. Die Werte liegen im Exudyn-J0-Gelenkrahmen vor; unterschiedliche Gelenkrahmen dürfen nicht ohne Transformation als gemeinsamer Weltvektor interpretiert werden. Ein initial noch nicht bestimmter Multiplikatorwert bei t=0 wird nicht geplottet.

Nicht enthalten sind ein validierter Antriebsmomentausgang, Feder-/Dämpferparameter, Kontaktanschläge und allgemeine Gelenktypen außerhalb des starren offenen Dreh-/Schubgelenkmodells. Die Oberfläche behauptet diese Funktionen nicht allein wegen ähnlicher Bezeichnungen in Simscape.

## Gemeinsame Bewegungssteuerung

Der Steuerungsblock hat je ein aktivierbares Signal für die vorhandenen nativen Gelenke. Weg-/Winkelausdrücke können normale Projektparameter und Berechnungsreferenzen verwenden. Rampenzeiten sind je Kanal einstellbar; die Simulationszeit ist gemeinsam. Der Block ist eine vorgegebene Bewegungsquelle, noch kein rückgekoppelter Regler oder Software-in-the-loop-System.

Beim Roboter werden sechs Achsen gleichzeitig vorgegeben und die sechs Gelenkreaktionen im Scope überlagert. Das System verwendet die tatsächlichen CAD-Massen und die hinzugefügte Nutzlast. Eine andere Nutzlast soll bei identischer vollständig vorgeschriebener Bewegung die Kräfte ändern, während die Trajektorie gleich bleibt. Genau diese Unterscheidung ist ein Integrationskriterium.

## Prüfverfahren

`tools/verify_mks_robot_library.py` erstellt zuerst eine unabhängige einfache Bibliotheksbaugruppe: 5-kg-Quader unter einem Drehgelenk in ruhender Vertikallage. Erwarteter Reaktionsbetrag: 5 × 9,81 = 49,05 N. Anschließend wird die Masse geändert und die Ungültigkeit des alten Ergebnisses geprüft. Ein ungültiges Gelenk zwischen demselben Körper muss atomar abgewiesen werden.

Danach wird der unveränderte ursprüngliche Roboter als Kopie geöffnet, ein gemeinsamer Steuerungsblock eingerichtet, die Gelenkausgänge bedient und eine Nutzlast über das Bibliotheksformular eingefügt. Geprüft werden Massendifferenz, native starre Reduktion, gleiche vorgegebene Trajektorie bei anderer Masse, veränderte Gelenkkräfte und gemeinsame Kurvenanzeige. Eine vollständige experimentelle Validierung des Roboterarms wird daraus nicht abgeleitet.

Die Ergebnispfade, Rohdaten und echten Screenshots liegen unter `output/mks-robot-library/`. Der tatsächliche Abschlussstatus steht in `verification.json`; Beschreibungen des Prüfverfahrens sind für sich noch kein bestandener Nachweis.

Die physikalische Prüfung ist bestanden: Referenzkraftfehler 7,11 × 10⁻¹⁵ N; Nutzlaständerung auf 8 kg nativ bestätigt; sechs Gelenkkräfte mit 41 Ausgaben; unveränderte CAD-Bewegung bei verändertem Schwerpunkt; verlustfreier Datei-Rundlauf. `ui-verification.json` enthält gesondert den Status der abschließenden Oberflächenprüfung nach den letzten Änderungen an der Leitungsführung.

Auch die abschließende UI-Prüfung ist bestanden: Einfügeformular durch Ziehen eines Joint-Prototyps geöffnet, Körperblock im gezoomten Plan verschoben, Gültigkeit der Physik erhalten, Steuerungs- und Joint-Dialoge bedient, Nutzlastdialog geöffnet, gespeicherte Bewegung abgespielt und sechs Gelenke im Scope angezeigt. Das neue Beispiel ist unter **Start → Beispiele → Roboter – Bewegungssteuerung und Gelenksensoren** auffindbar. Das ursprüngliche Roboterbeispiel bleibt erhalten.

Die Leitungsführung unterscheidet graue mechanische Anschlüsse, eine blaue Bewegungssammelleitung und eine grüne Messsammelleitung. Die Sammelleitungen bündeln die Kanäle grafisch. Die Gelenkkräfte werden weder addiert noch als ein gemeinsamer physischer Kraftvektor behandelt. Bewegungsdaten werden verlustfrei in Metadatenobjekten je Körper gespeichert; die normalen Metadatengrenzen bleiben erhalten.

Die bisherige Grenze von 100 Körper-Ausgabeintervallen wird gezielt auf 400 angehoben, weiterhin höchstens acht bewegliche Blöcke, 101 Ausgabepunkte, 10.000 Integrationsschritte und zehn Sekunden Simulationszeit. Dadurch sind beispielsweise sechs bewegliche Roboterblöcke mit 41 gemeinsamen Ausgabepunkten möglich. Der unveränderte Integrationsalgorithmus wird dadurch nicht genauer; nur die gespeicherte Ausgabe wird dichter.

## Fachliche Referenzen

- [MathWorks: Revolute Joint](https://www.mathworks.com/help/sm/ref/revolutejoint.html): Trennung von Frame-Anschlüssen, Ansteuerung und Messports als Funktionsreferenz, keine Übernahme von Code, Textvorlagen oder Symbolgrafiken.
- [MathWorks: Joint Force and Torque Sensing](https://www.mathworks.com/help/sm/ug/sense-forces-and-torques-at-joints.html): Reaktionskräfte, Antriebsmoment und Gesamtmoment sind unterschiedliche Größen.
- [Modelica MultiBody](https://doc.modelica.org/om/Modelica.Mechanics.MultiBody.html): Orientierung für die Einteilung in World, Parts, Joints, Forces und Sensors. Aktiver Solver bleibt Exudyn; keine OpenModelica-Einbettung.
