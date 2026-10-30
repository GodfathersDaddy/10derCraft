Erweiterung 22.10.2026: Gespeicherte mechanische Projektschnittstellen,
Körper-/Occurrence-Bezüge und explizite starre Lastpfade mit statischer
Kraft-/Momentenbilanz sind implementiert. [Bedienung und Grenzen](MECHANISCHE_SCHNITTSTELLEN.md).

Aktueller Ausbau: Reale Projektvarianten im Start-Bereich, auswählbare Zahnradentwürfe,
Änderungsprüfung und Aktualisierung vorhandener CAD-Features sind implementiert.
Die konkrete weitere Ausbaufolge steht in [SUITE_FAHRPLAN.md](SUITE_FAHRPLAN.md).

Aktualisierung 21.10.2026: Lokales CalcpadCE und die berechnungsgetriebene
KUE-Planetenstufe mit analytischen Evolventen sind implementiert. Die vier Angaben
schalten Berechnung und native Baugruppenkinematik um. Dies erweitert den
Prototypen um echte Berechnungs-/CAD-Kopplung, ersetzt aber keine vollständige
Festigkeitsfreigabe. [Bedienung und Grenzen](PLANETENGETRIEBE.md).

# 10derCraft – Produkt-, Dissertations- und Softwarefahrplan

Ergänzung vom 21.10.2026: Vertiefte Quellenrecherche und daraus abgeleitete Komponentenwahl stehen separat in [RECHERCHE_STAND_DER_TECHNIK.md](RECHERCHE_STAND_DER_TECHNIK.md). Die beschafften Originale, NotebookLM-Importtexte und bibliografischen Nachweise liegen unter [Literatur/INDEX.md](Literatur/INDEX.md). Forschungsentscheidungen und eine Versuchsvorlage werden in [DISSERTATION_FORSCHUNGSLOG.md](DISSERTATION_FORSCHUNGSLOG.md) geführt. Die Recherche belegt Vorläufer und schärft die Forschungshypothesen; sie beweist noch keine Neuheit oder funktionierende Solverintegration.

Stand: 20. Oktober 2026. Arbeitsgrundlage: `Masterarbeit_Zehnder.pdf` und die gemeinsam festgelegten Anforderungen. Dieser Plan beschreibt das Zielprodukt; der erste Softwarestand ist ein grafischer Prototyp ohne Simulationssolver.

## 1. Zielbild

Eine durchgängige lokale Engineering-Umgebung verbindet Anforderungen, technische Berechnungen, Bewegungsskelett, parametrisches CAD, Baugruppenvarianten, statische und flexible Mehrkörpersimulation, Optimierung und später reale Steuerungen. Informationen werden in die Konstruktion zurückgeführt, damit man methodisch schneller zu einem geeigneten Entwurf gelangt.

Hydraulik ist zunächst eine spätere Erweiterung. Schwerpunkt sind Modularität bereits im CAD, Variantenmanagement, Lastübertragung und wiederverwendbare Simulationsrepräsentationen.

### Erster Demonstrator

Ein selbst erstellter generischer Industrieroboter mit sechs Drehachsen und einem Werkzeugflansch. Als austauschbare Endeffektoren dienen Greifer, Schweißwerkzeug und passives Nutzlastmodul. Keine Herstellergeometrie und keine behauptete industrielle Zertifizierung.

Der Arbeitsablauf lautet:

1. Anforderungen und Berechnungen festlegen.
2. Parametrisches Bewegungsskelett mit Gelenken und Anschlussrahmen erzeugen.
3. Bauteile und Baugruppen konstruieren und an Montageanschlüssen konfigurieren.
4. Bewegungseinheiten, Schnittstellen und Lastpfade ableiten.
5. Posen zunächst starr und statisch bewerten.
6. Geeignete Komponenten flexibel modellieren und reduzieren.
7. Bewegungen, Antriebe und Regler ergänzen.
8. Varianten und Entwurfsparameter systematisch vergleichen und optimieren.
9. Ausgewählte Ergebnisse experimentell validieren.
10. Später Controller in Simulation und Hardware betreiben.

## 2. Dissertation

### Gewählter Titel

**Methodik zur automatisierten, durchgängigen virtuellen Entwicklung mechatronischer Systeme auf Basis parametrischer Konstruktion und modularer Simulationsmodelle**

Die Dissertation erweitert die in der Masterarbeit praktisch aufgebaute Entwicklungskette. Die Software ist der Forschungsdemonstrator. Ein eigenständiger wissenschaftlicher Beitrag entsteht durch eine formal beschriebene, automatisierbare Methodik und belastbare Vergleichsergebnisse; allein die Integration vorhandener Solver reicht dafür nicht.

### Vorgesehener Eigenbeitrag

Gemeinsame Konstruktions-, Varianten- und Schnittstelleninformationen erzeugen kompatible Simulationsrepräsentationen. Änderungsanalyse und Qualitätsprüfungen steuern Aktualisierung, Wiederverwendung und die Rückführung von Erkenntnissen in den Entwurf. Die Neuheit wird im Exposé gegen bestehende Forschung abgegrenzt.

Forschungsfragen:

- Wie lassen sich Parameter, Komponenten, Montageanschlüsse, Varianten und Modellrepräsentationen gemeinsam beschreiben?
- Wie werden starre, flexible und angetriebene Modelle daraus automatisch und überprüfbar abgeleitet?
- Wie bleiben physikalische Anschlüsse und Lastübertragung bei Modulwechseln und CAD-Änderungen konsistent?
- Wann dürfen Teilmodelle wiederverwendet, exakt zusammengefasst oder näherungsweise reduziert werden?
- Wie viel Bearbeitungsaufwand und Rechenzeit spart der Ablauf bei ausreichend genauer Ergebnisqualität?

### Forschungsstand und notwendige Abgrenzung

- [Sadowski 2014: automatische CAD-zu-MKS-Modellerstellung](https://depositonce.tu-berlin.de/items/f579937f-d11d-4671-8b81-530ce0a799c8).
- [Nolan et al. 2015: Simulation Intent](https://doi.org/10.1016/j.cad.2014.08.030).
- [Capalbo et al. 2024: parametrische flexible MKS](https://doi.org/10.1016/j.mechmachtheory.2023.105529).
- [Bai et al. 2026: persistente CAD-Netz-Zuordnung und inkrementelle Aktualisierung](https://doi.org/10.1093/jcde/qwag073).
- [Vanpaemel et al.: gekoppelte Topologieoptimierung flexibler MKS](https://doi.org/10.1016/j.mechmachtheory.2023.105344).

### Forschungsfahrplan

Planungsannahme: 36 Monate bei überwiegend vollzeitiger Bearbeitung; Hochschule, Betreuung, Wochenstunden und Werkstattzugang sind noch offen.

| Phase | Zeitraum | Ergebnis |
|---|---|---|
| D1 | Monat 1–3 | Literaturmatrix, Anforderungen, Exposé, Eigenbeitrag und Versuchsplan |
| D2 | Monat 4–8 | Gemeinsame Modellbeschreibung, Anschlüsse, Varianten und Ableitungsregeln |
| D3 | Monat 9–15 | Automatische starre/flexible Modellerzeugung und numerische Vergleichsstudie |
| D4 | Monat 16–21 | Änderungen, Modulwechsel, Wiederverwendung und Informationsrückführung |
| D5 | Monat 22–28 | Optimierung, Prüfstand und zweiter Mechanismus |
| D6 | Monat 29–36 | Veröffentlichungen, reproduzierbare Auswertung und Dissertation |

Nach D1 muss die Abgrenzung zu den engsten Vorarbeiten überzeugen. Falls der Gesamtansatz zu breit ist, wird der Eigenbeitrag auf die konsistente Ableitung und Aktualisierung austauschbarer Simulationsrepräsentationen konzentriert.

Vorgesehene Kapitel: Problemstellung; Forschungsstand; Anforderungen und Methodik; gemeinsames Modell; automatische Ableitung und Aktualisierung; Implementierung; numerische und experimentelle Bewertung; Optimierung und Übertragbarkeit; Grenzen und Ausblick.

Arbeitsform zunächst Monographie mit begleitenden Veröffentlichungen; endgültige Form mit der Betreuung festlegen.

## 3. Varianten und Montageanschlüsse

| Ebene | Beispiel |
|---|---|
| Parameterkonfiguration | Andere Blechdicke oder Greiferbackenlänge |
| Modulbelegung | Greifer statt Schweißwerkzeug am selben Flansch |
| Simulationsrepräsentation | Starr, flexibel oder reduziert für dieselbe physische Komponente |

Ein Montageanschluss besitzt eine stabile Identität, ein lokales Bezugssystem, Montageparameter, Verbindungstyp und kompatible Modulfamilien. Mechanische und gegebenenfalls elektrische, pneumatische oder signalbezogene Anschlüsse bleiben typisiert.

Ein Modulwechsel aktualisiert Geometrie, Masse, Schwerpunkt, Trägheit, Anschlüsse und betroffene Studien. Unpassende Module liefern konkrete Diagnosen. Eine geometrische Berührung allein beweist keine physikalische Verbindung.

Eine Projektvariante speichert Modulbelegungen, Parameter, Modellrepräsentationen, Bewegungsfall, Lasten und Auswertung. Varianten referenzieren gemeinsame Bauteile statt alle Modelle zu duplizieren.

## 4. Lastpfade und Statik

Erster Lastpfad: Werkzeugkontakt → Endeffektor → Flansch → Handachsen → Armglieder → Grundachsen → Basis.

Die erste Rechnung untersucht gehaltene Posen mit Eigengewicht und einer äußeren Werkzeugkraft bzw. einem Werkzeugmoment. Die Gelenkantriebe halten die Pose; ihre Haltemomente werden berechnet. Ausgegeben werden Schnittstellenkräfte und -momente in benannten Bezugssystemen sowie Basisreaktionen.

Bei mehreren Abstützungen zeigt die Software mögliche Lastwege. Die tatsächliche Lastaufteilung benötigt Steifigkeits- und Verbindungsmodelle. Eine graphische Verbindungslinie ist noch kein berechneter Kraftanteil.

## 5. Zusammenfassung und ROM

Unverändert bedeutet unveränderter Aufbau, Parameter und benötigte äußere Anschlüsse. Innere dynamische Zustände können sich trotzdem ändern.

| Teilmodell | Behandlung |
|---|---|
| Fest verbundene starre Teile | Exakte Zusammenfassung mit Gesamtmasse, Schwerpunkt und Trägheit |
| Elastische Struktur | FEM-basiertes ROM mit erhaltenen mechanischen Schnittstellen |
| Elektrischer Antrieb | Gekapseltes Modell; Näherungsreduktion erst nach Qualifikation |
| Regler/Software | Ausführbares Modul mit Ein-/Ausgängen und Abtastzeiten |
| Hydraulik | Später gekapseltes oder geeignet reduziertes Modell |
| Gemischtes Modul | Gemeinsame äußere Anschlüsse, intern passende Domänenmodelle |

Kapselung, exakte Zusammenfassung und Näherungsreduktion werden unterschieden. Eine FMU ist nicht automatisch ein ROM. Zu einem ROM gehören Ursprungsrevision, Schnittstellen, Ausgangsgrößen, Methode, Modellannahmen und geprüfter Betriebsbereich. Benötigte Sensoren müssen in der Reduktionsaufgabe berücksichtigt werden.

Geänderte relevante Parameter, Kopplungsbedingungen oder Auswertungsanforderungen lösen eine erneute Eignungsprüfung aus. Die erste Implementierung verwendet konservative Neuberechnung statt ungeprüfter Wiederverwendung.

## 6. Architektur und Open-Source-Auswahl

SolidCraft liefert CAD, Oberfläche und Befehlsarchitektur. Eigene Rust-Module verwalten Projekt, Anschlüsse, Varianten, Abhängigkeiten und Befehle. Ein separater Python-Prozess führt spätere numerische Aufgaben aus. UI, CLI und MCP verwenden dieselben Befehle.

Das gemeinsame Projektmodell enthält Parameter/Formeln/Einheiten, Bauteildefinitionen/Instanzen, Skelett/Gelenke/Anschlüsse, Repräsentationen, Studien und revisionsgebundene Ergebnisse. Definitionen zunächst als versioniertes JSON, große numerische Daten später in HDF5.

**Vorhandene SolidCraft-Funktionen zuerst qualifizieren und wiederverwenden:** Der bezogene Quellstand enthält bereits Parameterbefehle einschließlich Abhängigkeitsgraph und Ausdrucksauswertung sowie Konfigurationstabellen, Baugruppen und Gelenke. Die neue Umgebung soll diese Funktionen anbinden. Der grafische Prototyp verwendet dafür noch keine echten Varianten- oder Gelenkdefinitionen. Die nächste technische Etappe prüft die vorhandenen APIs an kleinen Referenzmodellen, bevor eigene parallele Datenstrukturen entstehen.

Das technische Rechenblatt benötigt eine gemeinsame verbindliche Parameterquelle mit Einheiten, Revisionen und nachvollziehbaren Abhängigkeiten. SymPy/Pint sind Kandidaten für zusätzliche symbolische Berechnung und Einheitenprüfung im numerischen Prozess. Ein zweiter, unabhängig gepflegter Parametersatz im Backend würde die Durchgängigkeit gefährden; Werte und Änderungen müssen über einen definierten Vertrag mit dem CAD synchronisiert werden.

| Aufgabe | Auswahl | Einordnung |
|---|---|---|
| CAD/UI | [SolidCraft](https://github.com/tangency-labs/solidcraft) | MIT oder Apache-2.0; geprüften Commit festhalten |
| FEM/Netz | [Netgen/NGSolve](https://docu.ngsolve.org/latest/i-tutorials/) | LGPL-2.1-Texte in Hauptprojekten; erste Kette mit Volumenmodellen |
| Flexible MKS/ROM | [Exudyn](https://exudyn.readthedocs.io/en/stable/docs/RST/FlexibleBodyFFRFTutorial.html) | Eigene permissive Lizenz plus Drittkomponenten; FFRF und HCB |
| Numerik/Optimierung | [SciPy](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.differential_evolution.html) | Kleine Räume vollständig auswerten; größere mit festem Budget untersuchen |
| Formeln/Einheiten | [SymPy](https://docs.sympy.org/latest/index.html), [Pint](https://pint.readthedocs.io/en/stable/) | Gemeinsame Parameterquelle erhalten |
| Regleranalyse | [python-control](https://github.com/python-control/python-control) | BSD-3-Clause; optionale Abhängigkeiten gesondert prüfen |
| Erweiterte Reduktion | [pyMOR](https://github.com/pymor/pymor) | BSD-2-Clause; späterer Kandidat |
| Multidisziplinäre Optimierung | [OpenMDAO](https://github.com/OpenMDAO/OpenMDAO) | Apache-2.0; nicht automatisch Ableitungen durch jede CAD/FEM-Kette |
| Unabhängige FEM | [CalculiX](https://www.dhondt.de/) | Gesonderter Adapter und Lizenzprüfung |
| Spätere Systemmodelle | [OpenModelica](https://openmodelica.org/), [FMPy](https://github.com/CATIA-Systems/FMPy) | Funktionen und Kopplung am konkreten FMU prüfen |
| Spätere Hydraulik | [OpenHydraulics](https://github.com/modelica-3rdparty/OpenHydraulics) | Version 2.0.0 BSD-3-Clause; frühere Versionen anders lizenziert |
| Spätere Hardware | [ros2_control](https://github.com/ros-controls/ros2_control) | Apache-2.0; eigene zeitkritische Laufzeit |

Alternativen: [Chrono](https://projectchrono.org/pychrono/) als Vergleichsbackend; [Calcpad](https://calcpad.eu/) als Blatt-/Backendkandidat; [OpenLSTO](https://github.com/M2DOLab/OpenLSTO) für spätere Topologieoptimierung.

Besondere Integrationsgrenzen: NGSolve besitzt Schalenmodelle, aber viele Exudyn-FEM-Hilfsfunktionen erwarten drei Verschiebungsfreiheitsgrade je Knoten. Ein Schalen-ROM-Weg benötigt eigene Qualifikation. FMI-Anschlüsse allein gewährleisten keine stabile physikalische Kopplung. Topologieoptimierung und CAD-Rekonstruktion sind zusätzliche Entwicklungsaufgaben.

## 7. Softwareetappen

| Etappe | Ergebnis / Abnahme |
|---|---|
| S0 | Offiziellen Quellstand beziehen, Herkunft sichern, Ausgangsbuild herstellen |
| S1 | Grafischer Prototyp mit Roboter, Endeffektoren, Varianten, Lastpfaden und Systemmodell; keine simulierten Ergebnisse vortäuschen |
| S2 | Anschlüsse und Lastpfade prüfen, fehlende Verbindungen markieren |
| S3 | Starre Statik, Haltemomente und Basisreaktionen gegen Referenzen prüfen |
| S4 | Varianten vergleichen, Ergebnisse zurückführen, veraltete Studien kennzeichnen |
| S5 | Ein Armglied/Werkzeug flexibel rechnen; äußere Anschlüsse erhalten |
| S6 | Unveränderte Teilmodelle erkennen und geprüfte Artefakte wiederverwenden |
| S7 | Bewegungen, Dynamik und Gewichtsoptimierung ergänzen |
| S8 | Weitere Domänen, Software-in-the-Loop und Geräteanbindung |

Der grafische Prototyp zeigt geplante Bedienung und Zustände. Er implementiert weder Statik noch FEM noch ROM. Lokale Demo-Auswahl, Ansichtswechsel und erklärende Vorschauen sind zulässig und ausdrücklich als Demonstration gekennzeichnet.

## 8. Validierung und Rückführung

Referenzsuite: analytischer Kragträger; flexibles Pendel; Arm mit mehreren Schnittstellen; Zweiachsarm mit bekannten Lastfällen; generischer Sechsachsroboter; später geschlossener Mechanismus.

Kritische Prüfungen:

- Globales Kräfte- und Momentengleichgewicht und korrekte Koordinatentransformationen.
- Werkzeugwechsel aktualisiert Masse, Schwerpunkt und Haltemomente.
- Fehlende Anschlüsse werden erkannt; Instanzen bleiben unterscheidbar.
- Varianten bestehen Speichern/Laden und Undo/Redo.
- Veränderte Teile verwenden keine veralteten ROMs.
- Unveränderte Teile verwenden nur passende qualifizierte Artefakte.
- Reduktion erhält benötigte Anschlüsse und Ausgänge.
- CAD, Blockmodell und Lastpfade zeigen dieselbe aktive Variante.

Wissenschaftlicher Vergleich: manueller Ablauf; automatischer vollständiger Neuaufbau; entwickelte modulare Methodik. Gemessen werden Bearbeitungszeit, Handgriffe, Fehlzuordnungen, Reparaturaufwand, Gesamt-Rechenzeit und Abweichungen der Zielgrößen.

Erste Optimierung: Masse durch Änderung von Dicke und Profilhöhe minimieren, während eine festgelegte elastische Spitzenabweichung eingehalten wird. Fertigbare Dicken diskret behandeln, Endentwürfe mit genauerer Rechnung überprüfen. Kein unbelegter Anspruch auf ein globales Optimum.

Prüfstand schrittweise: fest eingespannter elastischer Arm, eine Drehachse, später zweite Achse. Kalibrierungs- und Validierungsversuche trennen. Messunsicherheit und numerische Konvergenz dokumentieren.

## 9. Dokumentation und Upstream

Alle Anforderungen, Architekturentscheidungen, Methoden, Versuche, Fehler und Ergebnisse werden fortlaufend dokumentiert. Läufe enthalten Modellrevision, Parameter, Netz, Solverversion/-konfiguration, Zufallsstartwerte und Rohdaten. Grafiken werden per Skript daraus erzeugt.

Das offizielle SolidCraft-Repository bleibt als `upstream` erreichbar. Die Erweiterung entsteht auf einem eigenen Branch; die ursprüngliche Upstream-Revision bleibt dokumentiert. Eigene Änderungen möglichst in zusätzlichen Modulen bündeln und die Berührungspunkte zur UI klein halten.

Updates zuerst in einem Integrationsbranch übernehmen, bauen und gegen Referenzprojekte prüfen. Erst danach den Arbeitsbranch aktualisieren. Konflikte sind möglich, insbesondere wenn Upstream dieselben UI- oder Dokumenttypen verändert. Generell nützliche Erweiterungspunkte können später als separat prüfbare Upstream-Beiträge vorbereitet werden.

Das Produktlizenzmodell bleibt offen. Konkrete Versionen, transitive Abhängigkeiten und übernommene Altbestände vor Weitergabe prüfen. Eine Prozessgrenze ersetzt die Lizenzprüfung nicht.


## Implementierungsstand 21.10.2026

Die erste numerische Stufe ist umgesetzt: analytischer Rechteck-Kragträger in Rust, CAD-Längenparameterbindungen, versionierte externe Solveraufträge und kontrollierter Ergebnisimport. Exudyn 1.13.6 und SciPy 1.18.0 sind in einer getrennten Forschungsumgebung installiert und mit kleinen Referenzfällen geprüft. [Anleitung](simulation/README.md), [Versuchsregister](DISSERTATION_FORSCHUNGSLOG.md), [Ergebnisse](output/reference-adapters/verification.json).

Die Original-SolidCraft-Oberfläche mit zusätzlichen Reitern ist neu gebaut und im laufenden Programm geprüft. Reiterwechsel ändern nur die Toolbar; Eingabefenster öffnen ausdrücklich über Funktionsknöpfe. Der neue Launcher startet ohne erzwungenen Studio-Demomodus. [UI-Nachweis](output/native-shell/README.md).

FEM, CMS-ROM, persistente Engineering-Konfiguration, automatische Auswertung der Roboterbaugruppe und Rückschreiben optimierter Geometrie bleiben offen. Die alte große Testsuite bleibt pausiert.


### Umsetzung 2026-10-21: Allgemeine Berechnungen

Native Berechnungsseite und .scmath-Dokumente verwenden den CAD-Parametergraphen. Benannte Formeln treiben Skizzen und Featuremasse; atomare Uebernahme, Speichern/Wiederoeffnen und eine echte UI-Aenderung an einem unabhaengigen Bauteil geprueft. Beschreibung: MATHE_CAD.md. Naechste getrennte Aufgaben: zusammengesetzte physikalische Einheiten, Gleichungssysteme und analytischer Evolventenkurventyp ohne Spline-/Polygonersatz.

### Umsetzung 2026-10-21: Lokales Calcpad und Berechnungsvarianten

CalcpadCE als lokaler Prozess ohne Berechnungsserver eingebunden. Im Reiter Parameter: Quelltext, Berechnen, Ergebnisreferenzen, gespeicherte Variantenauswahl. CAD-Referenzen bleiben bis zur expliziten Uebernahme unveraendert; CAD zeigt ausstehende Aenderungen. Allgemeiner Nachweis an einer Skizzenplatte bestanden, vier gezielte Engine-Tests bestanden. Start-Mathe-CAD.cmd oeffnet das neue Calcpad-Beispiel. Aktueller Umfang und Grenzen in MATHE_CAD.md und integrations/calcpad-local/README.md. Offen bleiben typografischer 2D-Editor, Calcpad-Solver-/Dokumentdirektiven, komplette KUE-Auslegung und exakte Evolventengeometrie inklusive Topologie-Neuerzeugung.


### Umsetzung 2026-10-22: Native CAD-Baugruppe als MKS-Blockstruktur

Die strukturelle Ableitung ist umgesetzt: Komponenten-Einbaupfade, starre
Gelenke, Fundament, bewegliche Gelenke und Projekt-Kraftschnittstellen werden
als gemeinsames Blockmodell aus dem aktuellen CAD-Stand dargestellt. Kein
separat gepflegtes Ersatzmodell. Mehrdeutige Instanzen, freie Bloecke und
noch nicht uebersetzte Kopplungen werden kenntlich gemacht.

Gezielte Referenzpruefung: tools/verify_native_mks.py. Einstieg:
Start-MKS-Baugruppenmodell.cmd; Beschreibung MKS_BAUGRUPPENMODELL.md.
Naechste Abnahme: Materialmassen, Schwerpunkte und Traegheitstensoren gegen
analytische Koerper, anschliessend Exudyn-Pendel mit Perioden-/Energievergleich.
Dynamische Mehrkoerpersimulation und flexible FEM-Kopplung bleiben offen.


### Umsetzung 2026-10-22: Physikalische Blockeigenschaften

Masse, Schwerpunkt und Traegheitstensor aus Material und CAD-Netzintegration
sind lokal ueber mks.mass_properties verfuegbar. Eigener Knopf in MKS;
Aktualitaetsanzeige nach Aenderungen. Unvollstaendige Bloecke bekommen keine
Ersatzmasse. Beschreibung: MKS_MASSENEIGENSCHAFTEN.md; gezielte Pruefung:
python tools/verify_native_mks.py --mass-only. Danach folgt die Exudyn-Uebergabe.


### Umsetzung 2026-10-22: CAD -> Exudyn -> Projektergebnis

Erster realer Baugruppenadapter mit lokalen Exudyn-Laeufen, offenen Dreh-/
Schubgelenkbaeumen, CAD-Massen, Projektportlasten und vorgegebenen
Kosinusrampen. Ergebnisse bleiben mit Eingabesnapshot im Projekt gespeichert;
Aenderungen machen sie historisch. UI-Schwerpunktwiedergabe und enginebasierte
Welttransformationen fuer Koerper und Ports vorhanden.
Einstieg Start-CAD-Dynamik.cmd bzw. Start-CAD-Antriebsbewegung.cmd.
Grenzen und API: MKS_DYNAMIK.md. Kein FEM-/SIL-Modell und noch keine
native Zahnradkopplung oder animierte 3D-CAD-Ergebnisansicht.


### Fortsetzung 2026-10-22: 3D-Replay und Abbildungsexport

Separate 3D-Ergebnisansicht mit nativen CAD-Koerpern und Portachsen umgesetzt,
ohne gespeicherte CAD-Positionen zu veraendern. Gemeinsamer Engine-Szenenaufbau
fuer GUI und neue MCP-Befehle mks.result.snapshot (PNG) und
mks.result.animation (GIF). Pruefung von Wiedergabe, Dateiformaten und
Ueberschreib-/Gueltigkeitsgrenzen. Die oben als offen genannte erste
3D-Ergebnisdarstellung ist damit erledigt; FEM/ROM, Zahnradkopplung und SIL
bleiben weitere Produktbausteine. Details MKS_DYNAMIK.md.

23.10.2026: Roboter-Domänenversuch bestanden. Vorhandene native Geometrie,
lokale Calcpad-Referenzen, zwei Querschnittsvarianten, sechsachsige Exudyn-MKS,
gesperrter Kraftpfad und erste Balken-FEM am selben CAD-Stand. Start über
Start-Roboter-Alle-Domaenen.cmd. Details/Abgrenzung ROBOTER_DOMAENEN.md;
Abnahmedaten und CSV unter output/robot-domains. Volumen-FEM, dynamische
Lastübergabe, flexible Rückwirkung und SIL bleiben offen.
