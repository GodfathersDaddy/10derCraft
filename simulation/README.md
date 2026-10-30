# Erste numerische Referenzmodelle

## Aktueller Baugruppenadapter (30.10.2026)

Zusätzlich zu den unten dokumentierten frühen Referenzmodellen übersetzt
`assembly_worker.py` native CAD-Baugruppen in begrenzte offene Exudyn-Starrkörperbäume.
Unterstützt sind Dreh- und Schubgelenke, vorgeschriebene Bewegungsprofile,
Portlasten und Joint-Sensorwerte. Grenzen: acht bewegliche Körper,
1001 Ausgabepunkte, 8000 Körper-Ausgabeintervalle und 30 s lokale Prozesslaufzeit.
Die Daten werden vor der Übernahme gegen den aktuellen Auftrag geprüft.
Das ist keine allgemeine flexible Mehrkörpersimulation.

Neue, geprüfte Beispielprojekte werden über die Beispiele-Ansicht geöffnet.
Der native Videoexport benötigt keinen zusätzlichen Python-Videocodec.
Ältere Abschnitte unten beschreiben ausdrücklich frühere Einzelversuche;
aktuelle Freigaben und Prüfungen stehen im Integrationsbericht.

Diese optionale, separat gestartete Forschungsumgebung verwendet Exudyn und SciPy über deren öffentliche APIs. Die SolidCraft-Anwendung selbst bleibt Rust. Es wird kein Solverquellcode in SolidCraft kopiert oder verlinkt. Die neuen Studien sind **keine automatische Auswertung der Roboterbaugruppe**.

## Umfang

- Rust-Referenz: gleichförmiger rechteckiger Euler–Bernoulli-Kragträger mit Endkraft, Masse, Steifigkeit, Durchbiegung, Biegespannung und Energie.
- SciPy: Masse durch Änderung von Dicke und Höhe minimieren, mit Durchbiegungsgrenze, Maßgrenzen und zusätzlicher Begrenzung auf `L/h >= 10` und `|delta|/L <= 0,02`. Lokaler SLSQP-Lauf; kein allgemeiner Topologieoptimierer und kein globaler Optimalitätsnachweis.
- Exudyn: eindimensionaler Endmassenschwinger mit der analytischen Träger-Endsteifigkeit. Die verteilte Trägheit des Trägers ist nicht enthalten. **Kein FEM-/CMS-ROM.** Die Sprungantwort wird gegen die exakte Schwingerantwort verglichen.
- Versionierte JSON-Aufträge und Ergebnisse mit Eingabesnapshot, Solverversion, Zeitreihe/Optimierungsverlauf, Laufzeit sowie SHA-256 von Auftrag und Worker.

Die Geometrie ist in mm, Kraft in N, E-Modul in MPa und Dichte in kg/m³ angegeben. Exudyn bekommt SI-Werte in m, N, kg und s. Der Träger liegt entlang x, die Querschnittshöhe und Endkraft entlang z. Eigengewicht, Schubverformung, Beulen, Kontakt, plastisches Material und dynamische Baugruppenlasten fehlen in dieser ersten Referenz.

## Einrichtung

Im Projektstamm unter Windows:

```powershell
python -m venv simulation/.venv
.\simulation\.venv\Scripts\python.exe -m pip install -r simulation/requirements.txt
```

Die Umgebung ist lokal bereits mit Python 3.14.7, Exudyn 1.13.6, SciPy 1.18.0 und NumPy 2.5.4 eingerichtet. Andere Plattformen sind noch nicht geprüft. Die `.venv` ist kein zu veröffentlichendes Softwareartefakt. Lizenznachweise für Exudyn stehen unter `../Literatur/06_Lizenznachweise`; für eine spätere Distribution sind alle tatsächlich ausgelieferten Pakete separat zu erfassen.

## Engine und MCP

Nach einem neuen SolidCraft-Build sind diese registrierten Kommandos über denselben Weg wie andere Engine-Kommandos erreichbar:

```json
{"command":"studio.beam.evaluate","params":{"length_mm":1000,"thickness_mm":10,"height_mm":100,"force_n":1000}}
```

Längen können statt numerischer Werte aus vorhandenen CAD-Parametern kommen:

```json
{"command":"studio.job.create","params":{"backend":"scipy_beam_size","parameter_bindings":{"length_mm":"L_arm","thickness_mm":"t_sheet"}}}
```

Gebundene CAD-Parameter müssen eine Längeneinheit tragen. Kraft und Materialwerte bleiben explizit numerisch, da SolidCrafts vorhandenes Einheitensystem zunächst Länge/Winkel abbildet. Ein Feld darf entweder einen Zahlenwert oder eine Bindung haben. Werte werden bei der Auftragserstellung aufgelöst. Nach Änderungen der gebundenen CAD-Parameter wird ein altes Ergebnis beim Import abgelehnt.

`studio.job.create` gibt einen Auftrag zurück, startet aber keinen Prozess. `studio.job.accept {result: ...}` übernimmt nur Ergebnisse für den exakt aktuellen Auftrag. Die Übernahme ändert keine CAD-Geometrie. Übernommene Werte bleiben externe Referenzergebnisse; die Übereinstimmung des Auftrags allein ist kein physikalischer Validierungsnachweis.

Für einen laufenden, neu gebauten Prototyp mit `--control 7879` und Token unter `output/control/7879.token` kann der externe Helfer die drei Schritte ausführen:

```powershell
.\simulation\.venv\Scripts\python.exe -B tools/run_reference_job.py scipy_beam_size
.\simulation\.venv\Scripts\python.exe -B tools/run_reference_job.py exudyn_beam_reference
```

Option `--params meine-eingaben.json` übergibt eigene numerische Eingaben oder Parameterbindungen. Jeder Lauf bekommt einen eigenen Ordner unter `output/reference-runs`. Der Helfer verwendet dieselben Engine-Kommandos, die auch MCP aufrufen kann. Es ist kein neuer MCP-Server erforderlich.

## Bereits ausgeführte gezielte Prüfung

```powershell
.\simulation\.venv\Scripts\python.exe -B tools/verify_reference_adapters.py
```

Echte Ergebnisse liegen unter [output/reference-adapters](../output/reference-adapters/verification.json). Geprüft wurden das analytische Dimensionierungsoptimum des einfachen Falls, die Exudyn-Antwort gegen eine geschlossene Lösung, Zeitschrittkonvergenz sowie Ablehnung unzulässiger Aufträge. Das ersetzt keine Validierung des späteren Robotermodells.

Die bisherigen großen Projekt-Testläufe wurden nicht fortgesetzt. Der neue Desktop-Build unter `solidcraft/target/native-ui/ci/solidcraft.exe` enthält die Referenzkommandos. Studien öffnen erst über einen Funktionsknopf im Studienreiter; der Launcher verwendet die neue Datei. Der externe Worker funktioniert unabhängig davon bereits mit den gespeicherten JSON-Beispielaufträgen:

```powershell
.\simulation\.venv\Scripts\python.exe -B simulation/reference_worker.py output/reference-adapters/exudyn_beam_reference.job.json --out output/mein-neues-ergebnis.json
```

Vorhandene Ergebnisdateien werden vom Worker nicht überschrieben.

## Öffentliche technische Referenzen

- [Exudyn-Dokumentation](https://exudyn.readthedocs.io/en/latest/)
- [Koordinaten-Feder-Dämpfer](https://exudyn.readthedocs.io/en/latest/docs/generated/items/ObjectConnectorCoordinateSpringDamper.html)
- [SciPy SLSQP](https://docs.scipy.org/doc/scipy/reference/optimize.minimize-slsqp.html)

Die Implementierung ist neu geschriebener Adaptercode. Solver werden als installierte Pakete genutzt; es wurde kein fremder Beispielquelltext übernommen.


## CAD-abgeleitete Baugruppendynamik, 22.10.2026

Zusätzlich zum unabhängigen Beam-Referenzworker existiert jetzt
assembly_worker.py. Er übernimmt native starre CAD-Blöcke, deren
Masseneigenschaften und offene Dreh-/Schubgelenkketten. Lokaler Start
über mks.job.run; kein Server. Konstante Projektportlasten und ideale
vorgegebene Kosinusrampen sind unterstützt. Job und kompaktes Ergebnis
bleiben mit Eingabestand im CAD-Projekt gespeichert; Rohläufe werden archiviert.

Anleitung, Umfang und Grenzen: [MKS_DYNAMIK.md](../MKS_DYNAMIK.md).
Gezielte Abnahme aus dem Projektroot:
simulation/.venv/Scripts/python.exe tools/verify_assembly_dynamics.py.
Die alten unabhängigen Referenzmodelle bleiben für ihre beschriebenen
Vergleichsaufgaben bestehen; sie sind nicht der neue Baugruppenadapter.
