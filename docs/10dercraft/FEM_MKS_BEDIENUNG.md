# FEM, ROM und MKS bedienen

## Einheitliche Arbeitsumgebung

Parameter, FEM und MKS verwenden die CAD-Menükomponente mit Arbeitsbereichskarte, Reitern, Originalsymbolen und horizontalem Überlauf. Das CAD bleibt beim Bearbeiten der FEM-Einstellungen sichtbar. FEM-Eigenschaften erscheinen rechts.

## Vom CAD zur flexiblen Simulation

1. Im CAD das Bauteil auswählen und seinen Werkstoff zuweisen. Benutzerdefinierte Werkstoffkarten benötigen isotrope Elastizitätsdaten. Die eingebauten allgemeinen Stahl- und Aluminiumwerte sind illustrative Vorgaben.
2. Kraftübertragungsflächen als benannte Interfaces definieren. Für einen flexiblen Körper mindestens Referenzanschluss und Lastanschluss verwenden. Starre Anbauteile und Gelenke den entsprechenden Anschlüssen zuordnen.
3. Im FEM-Reiter Modell den Körper und die Interfaces wählen. Werkstoff aus CAD übernehmen aktivieren. Ältere Projekte behalten ihre gespeicherten FEM-Werte, bis diese Übernahme ausdrücklich aktiviert wird.
4. Unter Netz Anfangsunterteilung, quadratische Elemente und optional fehlergesteuertes Netz wählen. Berechnen löst die statische Studie und erzeugt das reduzierte Modell.
5. Ergebnisse zeigen die unverformte Kontur, blaue Referenzanschlüsse und rote statische Kraftpfeile. Die Anzeige kann automatisch überhöht und animiert werden. Diese statische Animation interpoliert die Darstellung und stellt keinen zeitabhängigen Lastfall dar.
6. Unter ROM das reduzierte Modell für MKS aktivieren. Nach Änderungen an Geometrie, Material oder Anschlüssen neu rechnen.
7. Im MKS Bewegungsquellen mit Gelenken verbinden, Simulation starten und Gelenkkräfte beziehungsweise Antriebsmomente auswerten. Für gekoppelte flexible Ergebnisse wird die echte ROM-Zeitreihe dargestellt. Statische Lastpfeile werden in dieser Ansicht ausgeblendet.

## Adaptive Vernetzung

Der Solver löst einen Vorlauf, rekonstruiert ein volumengewichtetes nodales Spannungsfeld und bewertet die lokale Abweichung mit einer Nachgiebigkeitsgewichtung. Markierte längste Kanten werden mit gemeinsamen Mittelpunkten in sämtlichen angrenzenden Tetraedern geteilt. Quadratische Knoten werden anschließend konsistent neu erzeugt. Anschlussflächen werden erneut geometrisch zugeordnet. Nach jeder Verfeinerung folgen statische Lösung und ROM-Reduktion.

Gespeichert werden Netzgrößen, Fehlerindikator, Zielwert, erreichte Konvergenz und Abbruchgrund. Das Netzbudget oder die Schrittzahl können die Rechnung vor dem Ziel beenden. Der Indikator ist keine garantierte Fehlerschranke. Verfügbare Volumennetze sind die unterstützten primitiven Körper und importierte konforme Tetraedernetze. Ein allgemeiner CAD-Netzgenerator für beliebige Baugruppen ist damit nicht enthalten.

Die methodische Grundlage ist die spannungsbasierte Fehlerrekonstruktion von Zienkiewicz und Zhu. Die konkrete Implementierung ist projektspezifisch. [Originalarbeit](https://doi.org/10.1002/nme.1620240206) und [MFEM-Dokumentation](https://docs.mfem.org/html/classmfem_1_1ZienkiewiczZhuEstimator.html).

## Signalports und Links

Die MKS-Bibliothek enthält Eingangsport, Ausgangsport und Signal-Link. Blöcke lassen sich platzieren, verschieben und per Doppelklick bearbeiten. Verbindungen lassen sich zwischen den Pins ziehen. Pfeile kennzeichnen die Richtung. Datentypfarben gelten gemeinsam für Pin und Leitung. Reell, Integer, Boolean und Dreiervektor besitzen unterschiedliche Farben. Die physikalische Größe wird zusätzlich auf Verträglichkeit geprüft.

Ein benannter Ausgang kann über einen gleich benannten Link wiederverwendet werden. Bewegungsquellen können über solche Portketten ein Gelenk antreiben. Konstante Werte und Gelenksensordaten können über Ports ausgelesen werden. Gelenkantriebe benötigen eine Bewegungsquelle. Eine geschlossene Regelschleife wird damit noch nicht integriert.

Sensorwerte benötigen ein aktuelles Ergebnis. Kraft und Moment als Dreiervektor liefern die drei lokalen Komponenten. Der reelle Sensorwert liefert jeweils die erste lokale Komponente. Die Sensordaten werden zwischen gespeicherten Zeitpunkten interpoliert. Anzeige- und Beobachteränderungen verwerfen ein Ergebnis nicht. Änderungen am wirksamen Bewegungseingang machen es ungültig.

Native gespeicherte Befehle sind `mks.diagram.source`, `mks.diagram.route`, `mks.diagram.connect` und `mks.diagram.sample`. Fehlende Links, Zyklen und unverträgliche Datentypen erzeugen verständliche Fehler, ohne eine ungültige Verbindung zu speichern.

## Prüfung dieses Zwischenstands

Breite Tests und vollständige CI bleiben entsprechend der Nutzeranweisung pausiert. Build, tatsächliche Referenzrechnungen und gezielte Bedienprüfungen werden im Forschungslog mit Ergebnissen dokumentiert.


## Rohrroboter und sichtbare Durchbiegung

Unter Beispiele den Roboter 8 kg mit Rohrbaugruppe öffnen. Im MKS die Simulation starten. Über der rechten 3D-Ansicht lässt sich die FE-Überlagerung vergrößern, beispielsweise auf 1000×. Die blaue Netzüberlagerung zeigt die zeitabhängige ROM-Verformung in diesem Darstellungsmaßstab. Die eigentliche Bauteilgeometrie bleibt physikalisch skaliert. Zur Beurteilung der Größe die numerischen Verschiebungswerte verwenden. Ein mittlerer Doppelklick passt die Ansicht an das Vorschaufenster an.

Die FEM-Ergebnisse besitzen rechts eine veränderbare Seitenleiste für Verformungsfaktor, Animation und Ergebniswahl. Links bleibt die große Modellansicht frei. Die ganze Rohrbaugruppe verwendet idealisierte Anschlüsse. Ein Schweißnahtnachweis muss gesondert erfolgen.
