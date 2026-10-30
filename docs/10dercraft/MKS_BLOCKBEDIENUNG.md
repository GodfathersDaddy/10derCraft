# MKS: Gelenkblöcke, Bewegung und Kraftverlauf

Start: **Start-10derCraft.cmd → Beispiele → Pendel - Ground, Revolute, Endgewicht**.
Danach oben **MKS** wählen.

Das Beispiel besteht aus einem festen Fundament und einem normalen parametrischen
CAD-Pendelarm mit Endgewicht. Die starr befestigten Teile sind gemeinsam ein
beweglicher Körperblock. Dazwischen liegt ein eigener **Revolute**-Block mit
mechanischen B-/F-Anschlüssen. Bilder stammen aus den tatsächlichen CAD-Körpern.

1. **Simulation starten** berechnet den lokalen Exudyn-Lauf. Das freie Pendel
   startet aus 20 Grad und schwingt unter Schwerkraft.
2. Die berechnete Bewegung wird direkt auf der MKS-Seite wiedergegeben. Mit
   Abspielen/Pause und Ausgabepunkt lässt sich der Verlauf untersuchen.
3. Den **Revolute-Block anklicken**, um seine Bewegungs- und Messausgänge einzustellen.
4. **Vorgegebenen Bewegungseingang verbinden** einschalten, beispielsweise `10 deg`
   und eine Rampenzeit von `0.5 s` einstellen. Eine eigene Quelle wird über einen
   Signaldraht mit dem Joint verbunden. Erneut starten.
5. **Gelenkkraftsensor mit Scope verbinden** aktiviert den Sensorknoten. Im Scope
   Fx, Fy, Fz oder den Kraftbetrag wählen. Die Gelenk-Checkboxen erlauben mehrere
   Kurven gleichzeitig; dieselbe Bedienung gilt für den Roboterarm.

Die Kurven gehören zum zuletzt gerechneten Auftrag. Bei abweichendem CAD oder
geänderten noch nicht gerechneten Bewegungseingaben werden sie nicht als aktuell
angezeigt. Antriebsdefinitionen werden mit dem Rechenauftrag im Projekt gespeichert;
Sensor-/Kurvenauswahl ist zunächst eine Ansichtseinstellung.

Die Verbindungen werden aus der CAD-Baugruppe abgeleitet; Signale werden über den
Gelenkeditor zugeschaltet. Freies Platzieren von Blöcken und Ziehen beliebiger
Leitungen sind noch kein Bestandteil dieses ersten Grapheditors. Der Solver
unterstützt offene starre Dreh-/Schubgelenkketten, keine allgemeine Simulation
sämtlicher Simscape-Blöcke, Kontakte oder flexibler Körper. Der Kraftsensor zeigt
Exudyn ForceLocal in N, kein Antriebsmoment. Der initiale t=0-Kraftwert ist von
der Anzeige ausgeschlossen und in den Rohdaten weiterhin nachvollziehbar.

Beispielparameter: `L = 960 mm`, `P = 80 mm`, `q0 = 20 deg`, Referenzdichte
7850 kg/m³. Masse und Trägheit folgen aus CAD. Der Solver erhält SI-Einheiten.
Kein zusätzlicher Berechnungsserver ist erforderlich.

Belege: `output/mks-pendulum/verification.json`, Gelenkkraft-CSV, GIF und native
Screenshots. Dokumentation und methodischer Vergleich:
[Simscape-Quellen](Literatur/Simscape_Multibody_Bewegung_und_Sensoren.md).

### Neue Bedienung des grafischen Arbeitsbereichs

Die aktualisierte Anleitung steht in
[MKS und Berechnungsblatt](MKS_UND_BERECHNUNGSBLATT_2026-10-28.md).
Sie ersetzt die frühere Beschreibung der untereinander angeordneten Ansicht:
Bibliothek links, verschiebbare Blöcke in der Mitte, Wiedergabe rechts und
Symbolwerkzeuge oben. Gelenke werden per Doppelklick geöffnet; Bewegung und
Kraftsensor werden aus der Bibliothek auf ein Gelenk gezogen. Konstante Kräfte
werden über die vorhandenen Kraftschnittstellen eines Körperblocks eingestellt.
