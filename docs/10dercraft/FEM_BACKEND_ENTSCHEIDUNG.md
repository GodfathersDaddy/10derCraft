# FEM-Backend: Stand und Empfehlung, 28.10.2026

## Was derzeit tatsächlich rechnet

Die integrierte Roboterrechnung ist eine eigene Rust-Implementierung: kubische
Euler-Bernoulli-Balkenelemente mit assemblierten Steifigkeitsmatrizen für die
Biegung sowie analytische axiale und Torsionsnachgiebigkeit. Zwei Armglieder,
gesperrte Aktuatoren, lokale TCP-Kraft. Sie nutzt weder PrePoMax noch CalculiX.
Das farbige Bild zeigt Verschiebungsbeträge dieses Modells. Es ist keine allgemeine
Volumen- oder Kontakt-FEM. Die MKS-Bewegungsrechnung verwendet separat Exudyn.

## Unschärfe und Gitter

Die bisherige Ergebnistextur hatte 720 × 440 Pixel unabhängig von der Fenstergröße.
Sie wurde auf großen Bildschirmen hochskaliert. Die Anzeige berechnet die Auflösung
jetzt aus Ansichtsgröße und Pixeldichte, begrenzt auf 2048 × 1400 und etwa zwei
Millionen Pixel. Der installierte GPU-Renderer wird genutzt, CPU als Ersatz.
Beim Drehen bleibt das letzte gültige Bild sichtbar, bis die nächste Ansicht fertig ist.
Die Kamerabedienung läuft weiterhin über dieselbe Navigation wie im CAD.

Das einschaltbare Oberflächengitter stellt die Darstellungsdreiecke dar. Es darf
nicht als Tetraeder- oder Schalen-FEM-Netz ausgegeben werden. Das Analysemodell
besitzt weiterhin seine 1D-Balkenelemente. Echte Volumenvernetzung ist ein eigener
Arbeitsschritt und braucht Netzprüfung sowie Konvergenzuntersuchungen.

## Empfehlung für die allgemeine FEM

PrePoMax ist ein Prä-/Postprozessor, der eigentliche Solver ist CalculiX.
Die offizielle Beschreibung nennt Volumen-/Schalenvernetzung, geometrie- und
netzbasierte Auswahlsätze sowie skalare 3D-Ergebnisse und Animationen.
Für den angestrebten allgemeinen Arbeitsablauf ist dieses etablierte Paket eine
wesentlich passendere Vergleichs- und Einrichtungsumgebung als die jetzige
begrenzte Balkenansicht. Die Balkenrechnung bleibt als schnelle Vorabschätzung nützlich.

Vorgeschlagene Architektur, noch nicht implementiert:

1. 10derCraft verwaltet parametrisches CAD, Varianten und mechanische Anschlüsse.
2. Geometrieexport und ein separat versioniertes Zuordnungsmanifest übertragen
   Bauteil-/Einbau-IDs, Einheiten, Material und Anschlussabsichten.
3. Eine erste Referenzanalyse wird in PrePoMax unabhängig aufgesetzt und geprüft.
   Netz, Kontakte, Lagerungen und Lastflächen müssen ausdrücklich definiert werden.
4. Automatisierbare Rechenaufträge laufen lokal über CalculiX-Eingabedateien und
   den Solverprozess. Möglichkeiten der PrePoMax-Automatisierung müssen anhand
   der konkreten Version geprüft werden; eine fertige API wird hier nicht behauptet.
5. Ergebnisse werden zusammen mit Netz, Quellstand, Lastfall und Einheiten
   eingelesen und auf die richtige CAD-Variante zurückgeführt. Ein STEP-Export allein
   erhält keine qualifizierte Flächenzuordnung; das muss gesondert gelöst werden.

PrePoMax wird zunächst als eigenständiges Programm betrachtet. Die offiziellen
Lizenzangaben nennen GPLv3, während die Repository-Regeln eine Übernahme solcher
Bibliotheken und ihres Codes in den Rust-Kern ausschließen. Eine Einbettung seiner
Oberfläche oder direkte Bibliotheksintegration wird deshalb nicht vorausgesetzt.

## Primärquellen

- [PrePoMax: Funktionsumfang](https://prepomax.fs.um.si/)
- [Versionen, portable Pakete und Lizenz](https://prepomax.fs.um.si/downloads/)
- [Dokumentation, einschließlich Handbuch 2.6.0](https://prepomax.fs.um.si/documentation/)

Am Abrufdatum listet die Downloadseite die veröffentlichte Version 2.6.0 und
neuere Entwicklungs-/RC-Versionen. Eine konkrete produktive Referenzanalyse sollte
Version, mitgelieferte Solver-/Mesher-Versionen und Eingabedateien festhalten.
Es wurde keine PrePoMax-Installation oder Solverintegration vorgenommen und kein
GPL-Quellcode gelesen oder übernommen.
