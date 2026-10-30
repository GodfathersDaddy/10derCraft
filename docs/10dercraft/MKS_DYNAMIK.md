# CAD-Baugruppe → lokale Dynamik → Projekt

Die Suite besitzt einen ersten CAD-abgeleiteten Exudyn-Adapter. Die Berechnung
läuft lokal in der vorhandenen optionalen Forschungsumgebung, ohne Server.
SolidCraft selbst bleibt eine Rust-Anwendung. Der Adapter verwendet die öffentlichen
[Create-Funktionen von Exudyn](https://exudyn.readthedocs.io/en/latest/docs/generated/cInterface/MainSystem.html).

## Benutzung

`Start-CAD-Dynamik.cmd` öffnet das Referenzpendel. Oben **MKS** wählen.
**Dynamik · lokaler Exudyn-Lauf** aufklappen, Dauer, Integrationsschritte und
Ausgabepunkte einstellen, dann **Dynamik lokal berechnen**. Die Anzahl der
Schritte muss durch „Ausgabepunkte minus eins“ teilbar sein.

Die resultierende Schwerpunktbahn wird in X/Z angezeigt. Schieberegler,
**Zurück/Weiter** und Wiedergabe zeigen gespeicherte Zeitschritte. Der Tempofaktor
bezieht sich auf die physikalische Ergebniszeit; die Wiedergabe stoppt am Ende.
Mit **CAD-Körper im 3D-Ergebnisfenster anzeigen** erscheinen die tatsächlich
konstruierten Körper in ihren berechneten Lagen, einschließlich Portachsen.
Die 3D-Ansicht verwendet die Standardnavigation und lässt sich separat einpassen.
Sie verändert weder Konstruktionsgeometrie noch gespeicherte Gelenkpositionen.
Der native Befehl `mks.result.frame` liefert die zugehörigen Welttransformationen
auch für MCP und weitere Renderer.

## Abbildungen und Animationen exportieren

GUI und Export verwenden denselben Szenenaufbau mit den nativen CAD-Körpern.
Einzelbilder entstehen über `mks.result.snapshot`, Animationen über
`mks.result.animation`. Beide benötigen ein aktuelles Ergebnis und verändern
die Konstruktion nicht. Vorhandene Dateien bleiben ohne ausdrücklich gesetztes
`overwrite: true` erhalten.

```json
{"command":"mks.result.snapshot","params":{"sample":10,"path":"output/pendel.png","view":"front","width":800,"height":500}}
```

```json
{"command":"mks.result.animation","params":{"path":"output/pendel.gif","view":"iso","width":480,"height":320,"speed":1}}
```

Der GIF-Export verwendet alle gespeicherten Ausgabepunkte und eine gemeinsame
Kamera über deren gesamten Bewegungsraum. Er läuft als GIF-Schleife und hält auch
das letzte Bild für ein Intervall. Das GIF-Format quantisiert die Verzögerung auf
Hundertstelsekunden; die tatsächliche Verzögerung wird im Befehlsresultat angegeben.
Das ist bei sehr kurzen Simulationsläufen von der exakten physikalischen Dauer
zu unterscheiden. Grenzen: 64–1.024 Pixel je Seite, insgesamt acht Millionen
Animationspixel und zwei Millionen Dreiecke über alle Frames. Die PNG-/GIF-Dateien
lassen sich ohne die optionale wissenschaftliche Python-Umgebung aus bereits
gespeicherten Projektergebnissen erzeugen.

Gezielte Exportprüfung: `python tools/verify_assembly_dynamics.py --export-only`.
Die Prüfung liest PNG-Bilddaten erneut ein, vergleicht zwei Körperlagen, zählt die
GIF-Bilder und prüft Zeitangaben, Überschreibschutz, Ressourcenlimit und Ablehnung
historischer Ergebnisse. Artefakte: `output/assembly-dynamics-export/`.

Die gespeicherte 3D-Ansicht lässt sich unabhängig vom Solver prüfen:
`python tools/verify_assembly_dynamics.py --ui-only`.

Die 3D-Vorschau verwendet Darstellungsnetze und den vorhandenen CPU-Renderer,
begrenzt auf 100.000 Dreiecke. Sie ist ein separates Ergebnisfenster, kein
Konstruktionsmodus. Zwischen gespeicherten Ausgabepunkten wird keine neue
physikalische Lösung interpoliert. Bei langsamer Anzeige können visuelle
Zwischenpunkte übersprungen werden; der Solverlauf bleibt davon unabhängig.

Materialien sollten an den CAD-Körpern zugewiesen sein. Die Verwendung der
CAD-Standarddichte erfordert eine ausdrückliche Option. Ein beschädigtes oder
veraltetes CAD-Modell darf nicht als aktuelle Simulationsgrundlage dienen.

## Unterstützter Umfang

- Ein Fundament und ein bis acht bewegliche starre Blöcke in einer offenen Kette
  oder einem offenen Baum mit Dreh- und Schubgelenken.
- Die aktuelle CAD-Pose ist die Anfangslage; alle Anfangsgeschwindigkeiten sind null.
  Native Startwinkel werden nicht doppelt aufgebracht.
- Materialmassen, Schwerpunkte und Schwerpunktträgheit aus der CAD-Auswertung in SI.
- Konstante Gravitation sowie konstante Kräfte/Momente im mitbewegten Portsystem.
- Vorgegebene Gelenkbewegung über ein Winkel-/Längeninkrement als Parameterausdruck:
  halbe Kosinusrampe mit Geschwindigkeit null an Beginn und Ende, danach halten.
  Das Inkrement gilt relativ zur aktuellen CAD-Stellung. Es ist ein idealer Antrieb,
  noch kein Motor-/Reglermodell oder SIL.
- Deklarierter Flächendruck als resultierende Kraft `p · A` entlang der Portnormalen.
- Trajektorien, kinetische und gravitative potenzielle Energie sowie Gelenkkräfte.
  Gelenkkräfte stehen im lokalen Gelenk-0-System gemäß
  [Exudyn ForceLocal](https://exudyn.readthedocs.io/en/latest/docs/generated/items/ObjectJointGeneric.html).

Der Energieverlauf enthält keine Subtraktion der Arbeit äußerer Portlasten oder Antriebe.
Eine Energieänderung unter äußeren Lasten ist deshalb nicht automatisch ein
Integrationsfehler. Numerische Gelenkresiduen werden gesondert geprüft.
Die Resultierende aus einem Flächendruck ersetzt keine FEM-Lastverteilung.

**Explizit gesperrt:** geschlossene Ketten, native Zahnrad-/Bewegungskopplungen,
Kontakt, Anschläge, geometrische Zusatzrelationen, freie/unverbundene Blöcke,
mehrdeutige Einbaupfade und unvollständige Masseneigenschaften. Es werden keine
dieser Bedingungen stillschweigend entfernt. Die vorhandene Planetenstufe ist
daher noch kein unterstützter dynamischer Fall dieses Adapters.

## Aktualisierung und Herkunft

Job und kompaktes Ergebnis werden im Projekt gespeichert. Der Eingabestand
enthält die nativen Definitionen einschließlich eingebetteter STEP-/Netzdaten,
Parameter, Materialien, Baugruppe, aktuelle Einbaupositionen und Projektports.
Der eigene Simulations-Metadatenkey und der Dokumenttitel gehören nicht zum
physikalischen Vergleich. Leere Metadaten werden normalisiert. Laufzeitinterne
Geometrie-Cachekeys eignen sich nicht als persistenter Herkunftsnachweis.

Nach Änderungen bleibt das vorige Ergebnis sichtbar, erhält aber den Status
**historisch**. Die Übernahme externer Ergebnisse und die Replay-Schnittstelle
verlangen einen aktuellen Eingabestand. Ein neuer Lauf übernimmt die geänderte
Konstruktion und berechnet ihre Masseneigenschaften erneut.

Jeder lokale Solverlauf legt Rohauftrag, Ergebnis und Solverausgaben unter
`output/mks-runs/` in einem eigenen Ordner ab. Version und SHA-256 des Workers
sowie ein vom Worker berechneter Auftragshash stehen im Ergebnis. Diese Angaben
dienen der Nachvollziehbarkeit; sie sind keine Signatur oder physikalische
Zertifizierung eines beliebig importierten Ergebnisses.

## Engine / MCP

Alle Wege verwenden dieselben registrierten Befehle:

```json
{"command":"mks.job.run","params":{"duration_s":2,"steps":1000,"samples":101}}
```

`mks.job.create` erstellt einen gespeicherten Auftrag ohne Prozessstart.
`mks.job.accept {result: ...}` importiert externe Worker-Ausgaben nur für den
exakt vorbereiteten aktuellen Auftrag. `mks.job.state` zeigt Job, Ergebnis und
Gültigkeit. `mks.result.frame {sample: 25}` liefert eine lesende Pose in mm und
spaltenweise gespeicherten Matrizen; Solvergrößen bleiben in SI.

Mehrere Lasten und Druck sind über die JSON-Schnittstelle verfügbar:

```json
{"command":"mks.job.run","params":{"duration_s":0.2,"steps":200,"samples":21,"gravity_m_s2":[0,0,0],"port_loads":[{"port":"tip","force_n":[0,0,10]}]}}
```

Der einfache UI-Lasteditor legt pro neuem Lauf höchstens eine Kraft-/Momentenlast
an. Er ist kein Editor für sämtliche gespeicherten Mehrlastaufträge.

Vorgegebene Bewegung, zum Beispiel ein zusätzliches Drehinkrement:

```json
{"command":"mks.job.run","params":{"duration_s":2,"steps":1000,"samples":21,"drivers":[{"joint":1,"displacement":"30 deg","ramp_s":1}]}}
```

Für Schubgelenke ist der Ausdruck eine Länge, etwa `50 mm`. Auch benannte
CAD-/Berechnungsparameter sind zulässig. Jeder Antrieb braucht mindestens zehn
Integrationsschritte für die Rampe; sie muss innerhalb der Simulationsdauer enden.
Die Amplitude ist derzeit auf 3 rad bzw. 10 m begrenzt. Gelenkanschläge bleiben
unabhängig davon nicht unterstützt. Ein antreibendes Moment wird nicht automatisch
aus den hier gespeicherten lokalen Gelenkkräften bestimmt.

## Installation und Grenzen des Prototyps

Die bestehende `simulation/.venv` muss die Pakete aus `simulation/requirements.txt`
enthalten. Optional konfigurieren `SOLIDCRAFT_SIMULATION_PYTHON` und
`SOLIDCRAFT_ASSEMBLY_WORKER` die lokalen Pfade. Der Desktop startet den Prozess
ohne Shell und unter Windows ohne zusätzliches Konsolenfenster. Timeout: 30 s.
Im Browser bleibt der lokale Prozessstart deaktiviert.

Dauer 0,001–10 s, 10–10.000 Integrationsschritte, 2–101 Ausgabepunkte; höchstens
100 Intervalle mal bewegliche Blöcke. Bei mehr Körpern entsprechend weniger
Ausgabepunkte wählen. Die vorhandenen Metadatenlimits gelten auch für gespeicherte
Jobs und Ergebnisse; zu große Quellen/Ergebnisse werden ausdrücklich abgelehnt.
Große importierte Baugruppen benötigen später externe Artefaktablage mit
persistenten Inhaltshashes. Keine automatische CAD-Optimierung, FEM oder SIL.

## Nachweise und nächste Schritte

`simulation/.venv/Scripts/python.exe tools/verify_assembly_dynamics.py` prüft:
physikalisches CAD-Pendel gegen eine unabhängig integrierte nichtlineare
Differentialgleichung, Zeitschrittverfeinerung und Energie; gekoppelte
Zweikörperkette; Schieber gegen konstante Beschleunigung durch Gravitation,
Kraft und Druck; Projektpersistenz, UI, Replay und fehlerhafte Importe.
Rohdaten und Aufnahmen: `output/assembly-dynamics/`. Eine zusätzliche achtgliedrige
Kette prüft Kopplung, Gelenkresiduen, Energie und Wiederladen. Das ist ein
Konsistenztest und keine unabhängige Referenz für acht gekoppelte Körper.
CSV-Daten für Abbildungen erzeugt `tools/export_dynamics_research_data.py`.

Als Nächstes folgen native Zahnradkopplungen, allgemeinere Antriebsprofile,
größere externe Ergebnisartefakte, variantenspezifische Ergebnisablage und die Übergabe
ausgewählter Lastzustände an FEM/ROM. Jede Erweiterung erhält eigene Referenzfälle.
