# Kran: flexible Simulation und nächster Ausbau

Stand 03.11.2026. Die lokale Version ist über Start-10derCraft.cmd erreichbar; neu starten, falls noch eine ältere Instanz läuft.

## Jetzt nachgewiesen

Ein eigenes natives Kranbeispiel besteht aus Fundament, zwei parametrischen Auslegern, zwei angetriebenen Drehgelenken, freiem Lastpendel und 20-kg-Last. Alle Körper und Gelenke entstehen durch reguläre Bibliotheksbefehle und bleiben normale editierbare CAD-Bauteile. Es ist ein vereinfachtes neues Kranmodell, keine vollständige flexible Umrüstung des detaillierten bisherigen Roboterarms.

Beide Ausleger haben separate TET10-Volumen-FEM-Modelle und Hurty-Craig-Bampton-ROMs. Benannte CAD-Stirnflächen sind die Kraftübertragungsschnittstellen. Die native MKS verwendet reale Exudyn-FFRF-Objekte, keine rein optische Biegeanimation. Zwei unterschiedlich schnelle Sinusantriebe mit sanftem Anlauf erzeugen eine zusammengesetzte Bewegung. Lastmasse und Trägheit wirken dynamisch; das Lastpendel ist frei beweglich.

Referenzlauf: 2 s, 4000 Integrationsschritte, 1001 Ausgabezeitpunkte. Größter Gelenkpositionsrest 3,04e-10 m, Achsrest 1,36e-9. Diese Werte prüfen die Zwangsbedingungen, nicht die Genauigkeit des gewählten Netzes oder die reale Maschine. Netz/Moden/Zeitschrittkonvergenz für diesen Bewegungsfall steht noch aus. Die anfänglich unbelasteten elastischen Zustände erzeugen unter Schwerkraft Einschwingvorgänge; die hochfrequenten Kraftanteile sind nicht automatisch Greifkontakt.

Gespeichert: output/crane-flexible/Kran-Zwei-Flexible-Arme.solidcraft, Kran-Flexible-MKS.avi und verification.json. Das Video zeigt die 3D-Bewegung; das Kraftdiagramm ist im MKS-Fenster sichtbar. Wiederholbarer Aufbau: tools/create_flexible_crane.py. Das Skript automatisiert dieselben öffentlichen Befehle wie die Oberfläche; ein vollständig manuell qualifizierter Drag-and-drop-Ablauf ist damit noch nicht bewiesen.

Zusätzlich bestanden: frischer Prozess öffnet FEM/ROM-Projekt; Wechsel auf starre MKS funktioniert; erneute flexible Bindung markiert das starre Ergebnis als historisch. Belege: output/fem-rom-showcase/verification.json. Der Fehler beim Öffnen großer SI-Steifigkeitszahlen wurde behoben, ohne die Grenzen für normale CAD-Koordinaten aufzuweichen.

## Wiederverwendbare Bibliotheksblöcke für Aufnehmen und Ablegen

Nächste Ausbaustufen, noch nicht implementiert:

1. Trajectory Sequence: Zeit-/Winkel-/Weg-Tabelle mit Halten, Heben, Transport, Senken und Rückfahrt; stetige Geschwindigkeiten und Beschleunigungen, Grenzprüfung. Anschlüsse q, q_dot, q_ddot und done.
2. Gripper: zwei Backen mit Öffnungsweg bzw. Kraftantrieb, geometrischen Kontaktflächen und Materialpaar; keine bloße Sichtbarkeitsumschaltung des Werkstücks.
3. Contact Pair: ausdrücklich definierte Körper/Flächen, Normalsteifigkeit, Dämpfung und regularisierte Haft-/Gleitreibung. Ausgänge Normalkraft, Tangentialkraft, Schlupf, Spalt und Kontaktstatus. Schnittstellen zu flexiblen Körpern müssen die Lastarbeit erhalten.
4. Table/Ground Contact: Ablagefläche; beim Ablegen übernimmt der Tisch das Gewicht. Werkstück bleibt danach ein unabhängiger dynamischer Körper.
5. Sequence Controller: anfahren → schließen → Greifnachweis → heben → transportieren → senken → Auflage nachweisen → öffnen → zurückfahren. Übergänge mit Hysterese und Zeitüberschreitung, nicht allein festem Zeitstempel.
6. Force/Torque Sensor, Contact Sensor und Scope: gemeinsame Zeitbasis, mehrere Gelenke und Kontakte, Einheiten und exportierbare Daten.

Die Blöcke sollen normale native Bibliothekselemente mit Doppelklick-Einstellungen, ziehbaren Anschlüssen und identischen MCP-Befehlen werden. Verhalten nach dem Simscape-Prinzip, keine MATLAB-Abhängigkeit oder Übernahme fremder Symbole. Vor einer Kontaktintegration Solverfähigkeit, Lizenz und Zeitschrittstabilität prüfen. Eine geschaltete ideale Befestigung wäre ein separater vereinfachter Greifmodus; sie darf nicht als Reibsimulation bezeichnet werden.

## Abnahmekriterien

- Last bleibt vor dem Greifen auf dem Tisch und nach dem Loslassen dort liegen.
- Heben nur bei nachgewiesen ausreichender Greifkraft; zu kleine Reibzahl führt nachvollziehbar zu Schlupf.
- Gewichtskraft, Greifkräfte und Gelenkreaktionen reagieren auf geänderte Masse.
- Zeitschritt-, Netz- und Modenverfeinerung ändern relevante Maximalwerte innerhalb einer dokumentierten Toleranz.
- Änderungen an CAD, Flächen oder Material machen betroffene ROMs und Simulationen sichtbar veraltet.
- Gesamter Ablauf lässt sich aus der Bibliothek ohne benutzerspezifisches Skript zusammensetzen, speichern und nach Neustart erneut ausführen.

Keine vollständige CI oder experimentelle Validierung dieses Krans beansprucht. Derzeit kein Greifkontakt, kein Loslassen, keine Tischkollision.
