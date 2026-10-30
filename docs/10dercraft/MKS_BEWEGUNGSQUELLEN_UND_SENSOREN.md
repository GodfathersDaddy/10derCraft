# MKS: Bewegungsquellen und Sensoren

**Neuere Ausbaustufe:** Unabhängige Quellen und explizite Quellen-/Messverbindungen sind in [MKS_FREIE_VERBINDUNGEN.md](MKS_FREIE_VERBINDUNGEN.md) beschrieben. Die folgenden Angaben dokumentieren die davor eingeführten Profile und Messgrößen.

Stand: 28.10.2026. Start über **Start-10derCraft.cmd**, anschließend **Start → Beispiele → Roboter - Bewegungssteuerung und Gelenksensoren → MKS**. Das einfache Beispiel heißt **MKS - Bewegungsquellen und Sensoren**.

## Bedienung

Die linke Bibliothek enthält 16 Einträge in fünf Kategorien: World/Körper, Gelenke, Bewegungsquellen, Sensoren sowie Lasten/Ausgabe. Die Suche findet deutsche Beschriftungen und interne Kennungen wie `sine`. Über Plus wird das Zielgelenk gewählt; Quellen und Sensoren können auch auf ein Gelenk gezogen werden. Ein Doppelklick auf die freie Zeichenfläche öffnet die Schnellauswahl. Knoten bleiben verschiebbar; mittlere/rechte Maustaste verschiebt den Plan, Strg+Mausrad zoomt.

Ein Doppelklick auf ein Gelenk öffnet die Reiter **Verbindungen**, **Ansteuerung** und **Sensorik / Ausgänge**. Auch die kleinen B/F-, q- und Messanschlüsse am Gelenk öffnen den passenden Reiter. Das sind Zugänge zu den nativen Verbindungen und Einstellungen; ein beliebiger frei verdrahtbarer Mehrdomänen-Signalgraph ist damit noch nicht implementiert.

In der Ansteuerung werden Hub/Winkel als Parameterausdruck, Rampenzeit und Profil eingestellt. Der gemeinsame Steuerungsblock bearbeitet mehrere Achsen. Die Vorschau zeigt den normierten Verlauf q/A; sie verwendet denselben registrierten Quellenbefehl wie die Profildefinition.

Unter Sensorik sind Fx, Fy, Fz, Kraftbetrag, Gelenkkoordinate q und Geschwindigkeit wählbar. **Simulation starten** berechnet anschließend die Bewegung. Der Scope trennt Kraft [N], Winkel [deg], Winkelgeschwindigkeit [deg/s], Weg [mm] und Geschwindigkeit [mm/s]. Mehrere Gelenke lassen sich innerhalb derselben physikalischen Größe vergleichen. Die rechte 3D-Ansicht kann mit der CAD-Navigation gedreht werden.

## Profile und Messwerte

Mit u = clamp(t/T, 0, 1) und Hub A gilt:

- Kosinus: q_rel = A · (1 − cos(πu))/2; anschließend halten.
- Quintic: q_rel = A · (10u³ − 15u⁴ + 6u⁵); anschließend halten. Geschwindigkeit und Beschleunigung sind an den Übergängen stetig null. Der Ruck ist nicht überall stetig.
- Sinus: q_rel = A · (10u³ − 15u⁴ + 6u⁵) · sin(2πft); nach dem Hochlauf periodisch weiterlaufen.

Die gemessene Koordinate enthält den nativen Anfangswert. Sie und ihre Geschwindigkeit werden aus den berechneten Körperzuständen ermittelt, nicht aus einer erneut gezeichneten Sollkurve. Rohdaten verwenden rad, rad/s, m und m/s; die Oberfläche rechnet passend zum Gelenktyp um. Rotationswinkel werden an den Ausgabezeitpunkten entrollt; für schnelle oder freie Mehrfachumdrehungen muss die Ausgabedichte ausreichend sein.

Sinusquellen benötigen mindestens 50 Integrationsschritte und zehn Ausgabeintervalle je Periode. Die Prüfung gegen native Gelenkgrenzen verwendet konservativ den gesamten Bereich Anfangswert ± |A|. Alte Dateien ohne Profil bleiben Kosinus-Antriebe; alte Ergebnisse ohne Koordinaten müssen für die neuen Sensoren neu berechnet werden.

## Nachweise

`tools/verify_mks_sources.py` prüft drei Bewegungsprofile an einer nativen 5-kg-Schubgelenkbaugruppe gegen unabhängige Formeln für Position, Geschwindigkeit und m(g+a). Zusätzlich prüft es einen Drehgelenk-Anfangswinkel von 20°, ungültige Profile, unzureichende Sinus-Abtastung, Grenzverletzungen sowie die Oberfläche. Größter Kraftfehler der drei Referenzläufe: 0,004537 N. Ergebnisse und CSV: `output/mks-sources/verification.json` und `*-Referenz.csv`.

`tools/verify_mks_robot_profiles.py` prüft alle sechs Roboterachsen mit 41 Ausgabezeitpunkten gegen die vorgegebenen Quintic-Bewegungen. Maximaler Winkelfehler: 2,36 × 10⁻¹² rad; maximaler Geschwindigkeitsfehler: 9,95 × 10⁻⁶ rad/s. Das ursprüngliche Robotermodell bleibt unverändert. Bericht: `output/mks-sources/robot-verification.json`; Daten: `Roboter-q-Geschwindigkeit-Kraft.csv`.

Das sind gezielte numerische und Integrationsprüfungen, keine experimentelle Validierung. Kraftdaten sind lokale Gelenkreaktionen im Exudyn-J0-Rahmen; sie sind kein validierter Antriebsmomentausgang. Der nicht bestimmte initiale Kraftwert bei t=0 wird nicht geplottet.

## Grenzen und Fortsetzung

Der lokale Solver bleibt Exudyn. Unterstützt wird die vorhandene offene Starrkörperkette mit Dreh-/Schubgelenken. OpenModelica, allgemeine Gleichungskopplung, rückgekoppelte PID-Regler, Kontaktdynamik, Federn/Dämpfer und geschlossene Schleifen wurden in dieser Stufe nicht ergänzt. Ein vollständiger Simscape-Ersatz erfordert vor allem einen typisierten Signal-/Frame-Graph mit expliziten Verbindungen, Validierung und Übersetzung ins Solvermodell. Die vorhandenen nativen Befehle, Bauteile, Quellen und gemessenen Ausgänge bilden dafür die Grundlage.

Funktionale Orientierung: [MathWorks Revolute Joint](https://www.mathworks.com/help/sm/ref/revolutejoint.html). Eigene Implementierung und vorhandene 10derCraft-Oberfläche; kein übernommener proprietärer Quelltext oder Symbolsatz.
