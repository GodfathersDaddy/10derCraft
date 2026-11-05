# Dissertation – Forschungsentscheidungen und Versuchsregister

## MKS-Workflow: Quellen am Gelenk, Mehrkanal-Scope und Wiedergabe

Allgemeine Bedienfunktionen ergänzt: direkt aus einem nativen Dreh-/Schubgelenk
eine dimensionspassende unabhängige Bewegungsquelle anlegen; danach explizite
Verbindungsbestätigung über den bestehenden Graphbefehl. Vorhandene passende
Quellen werden ebenfalls angeboten. Kein zweiter Robotersolver und keine
implizite Ersetzung eines Gelenkeingangs. Der Quelleneditor und die bestehende
Validierung typisierter Verbindungen bleiben maßgeblich.

Scope erhält Sammelauswahl für Gelenkkraftbeträge, Drehgelenkwinkel und
Schubwege sowie Leeren der Messauswahl. Einschalten eines Messkanals hebt
nun dessen frühere Scope-Trennung auf. Auswahl bleibt in den vorhandenen
Darstellungsmetadaten. Diagrammklick pausiert, setzt die Wiedergabezeit und
öffnet die berechnete 3D-Stellung. Tooltip zeigt den nächsten vorhandenen
Abtastwert; keine Genauigkeitsbehauptung zwischen Solver-Abtastpunkten.
Wiedergabe 0,1x bis 4x und Wiederholung sind flüchtige Ansichtseinstellungen.
Die obere MKS-Leiste scrollt horizontal statt umbrechender Zusatzzeilen.

Build und gezieltes Clippy -D warnings bestanden. tools/verify_mks_workflow_update.py
bestand echte UI-Schritte: Winkelpresets, Leeren, Kraftpresets, Diagramm-zu-3D,
Anzeigeänderungen erhalten gültige Physik; native Quelle aus Gelenk erzeugt,
Verbindung bestätigt und alte Ergebnisse korrekt ungültig. Anschließend sechs
Roboter-Kraftkanäle und sechs Winkelkanäle dargestellt. Zweiarm-Beispiel und
Roboter benötigten je eine gezielte Referenzrechnung; der Roboterlauf war auf
1 s, 1000 Schritte und 41 Ausgabepunkte begrenzt. Beide Originaldateien blieben
unverändert. Dies ist keine experimentelle Validierung der Gelenkkräfte.

Ein erster Test scheiterte am Doppelklick nach Schließen von Anzeige-Panels.
Der erfolgreiche Ablauf öffnet den Gelenkdialog über den normalen B-Anschluss;
eine verbesserte Zuverlässigkeit aller Doppelklick-Konstellationen ist damit
nicht nachgewiesen. Anleitung MKS_WORKFLOW.md, Screenshots und Bericht unter
output/mks-workflow-update. Der bestehende Startknopf wählt die neue Version
0928d3dea7fe78d2e9f84674f7b20a64d602c7c63ae728b2ef2316501d1264d2.

## MKS-Leitungsführung: Anschlussrichtung statt pauschalem Rechtsausgang

Abschluss: Desktop-Build erfolgreich, statische Prüfung des eingebundenen
Originalmoduls mit -D warnings erfolgreich. UI-Prüfung mit gespeicherter
Zweiarm-Baugruppe bestanden, Leitungen sichtbar, Quelldatei unverändert und
kein Solver erneut ausgeführt. Screenshot MKS-Leitungen.png und verification.json
unter output/mks-routing-update. Die geprüfte Version wurde über launch-target.txt
für den bestehenden Startknopf ausgewählt; Programm-SHA-256:
dd7b0a838604890b121dec06abb774acf2d21b025166b4557bd48beddb8fef10.

Konkreter Darstellungsfehler: Der Manhattan-Router nahm für jeden Quellanschluss
einen Austritt nach rechts an. Zielanschlüsse wurden hingegen bereits nach ihrer
Blockseite behandelt. Damit war die Darstellung bei anders angeordneten oder
verschobenen Blöcken nicht richtungssymmetrisch. Beide Enden verwenden nun die
nächstgelegene Blockkante. Anschlussstücke werden vor benachbarten aufgeweiteten
Blockrechtecken verkürzt, damit enge, noch routbare Zwischenräume erhalten bleiben.
Die Hindernissuche, ihr Cache und die Trennung von Darstellung und Physik bleiben
erhalten. Keine Änderung am Solver, an Gelenkkräften oder an CAD-Kopplungsdaten.

Drei gezielte Prüfungen des tatsächlichen mks_wire_ui.rs-Moduls bestanden:
alle vier Portseiten ohne Blockdurchquerung, enge gegenüberliegende Ports in
beiden Richtungen und Umgehung eines mittleren Hindernisses in beiden Richtungen.
Ausgeführt mit dem kleinen Harness output/mks-routing-check, das das Originalmodul
einbindet und keine zweite Routingimplementierung enthält. Kein großer Solverlauf.
UI-Prüfung und Screenshot: tools/verify_mks_routing_update.py und
output/mks-routing-update/. Freie manuelle Wegpunkte und die Entflechtung sämtlicher
Leitungskreuzungen sind mit dieser begrenzten Korrektur nicht gelöst.

### 2026-10-29: Erneuter Inventor-Importversuch

Benutzer fordert den tatsächlichen Import der body_parts-Baugruppe an. Eigene
AutoCAD-Core-Console mit prozesslokalem Konfigurationspfad scheitert vor dem
Import an der Konfigurationsdatei. Normaler AutoCAD-Start mit selbst erzeugter
leerer DXF und Kopie eines IPT-Bauteils bleibt an einem acmgd.dll-ARX-
Exception-Dialog hängen; erster Skriptmarker wurde nicht geschrieben.
Computer-Use-Appfreigabe zum Lesen des Dialogs abgelaufen. Daher kein Nachweis
einer funktionierenden Übersetzung und weiterhin kein Geometrieimport.
Keine Ersatzkörper als erfolgreiche Übernahme ausgegeben. Befund und Dateien
unter output/body-parts-import/README.md und desktop-converter/.

## 2026-10-29: Parameterlatenz, Bedienbarkeit und Importgrenzen

Hypothese: unoptimierte eigene Geometriecrates erklären die lange Änderung des
Besenhalters. Gezielter Vergleich bei identischem Durchmesserwechsel 24 -> 25 mm:
55,55 s vorher, 53,01 s mit Optimierung Stufe 2. Die Hypothese reicht als Erklärung
nicht aus; der kleine Unterschied ist kein allgemeiner Beschleunigungsnachweis.
Die sechs betroffenen Booleschen Features bleiben der Engpass. Ein geschlossener
Körper und sämtliche gespeicherten Messgrößen stimmen innerhalb der dokumentierten
Toleranz überein. Vollständige Daten und Grenzen: PARAMETER_PERFORMANCE.md sowie
output/parameter-performance/{before,after}/benchmark.json und comparison.json.

Methodische Trennung: Rechenzeit und Bedienbarkeit sind unterschiedliche Größen.
Native Blattänderungen, Änderungen im Parameterdialog und explizites Übernehmen
werden über die vorhandene Hintergrundberechnung ausgeführt. Die Rechenzeit
wird dadurch nicht als verschwunden dargestellt. Keine Kernelalgorithmen geändert.
Erster kalter Modellaufbau nach Buildwechsel: 83,17 s, gegenüber 0,89 s mit gültigem
altem Cache; diese Ladezeiten sind wegen unterschiedlicher Cachebedingungen nicht
als direkter Geschwindigkeitsvergleich zulässig.

Oberflächenprüfung bestanden: eine einzige horizontale Werkzeugzeile, Mausrad
scrollt horizontal, sichtbarer Scrollbalken. Native Formel Side von 10 auf 12 mm
per Doppelklick und normalem Gleichheitszeichen geändert; ein Quader mit 20 mm
Breite und 30 mm Höhe hat danach 7200 mm³ Volumen. Screenshots und Prüfbericht
unter output/parameter-performance/ui. Anfangs fehlte die lokale Einstellung
für das Mausrad; korrigiert. Ein weiterer Klicktestfehler entstand durch fehlende
Mauspositionierung vor dem Reiterklick; der erfolgreiche Test positioniert die
Maus vor dem Klick. Build und gezieltes Clippy mit -D warnings bestanden.

Inventor: neun rechtshändige orthonormale CSV-Platzierungen mit Originalnamen
und SHA-256-Prüfsummen gesichert. Kein nativer Inventor-Decoder implementiert,
keine Geometrie importiert und keine Gelenke aus Platzierungen erfunden. Benutzer
genehmigte AutoCAD ausschließlich als Konverter dieser eigenen Dateien. Lokale
Prüfung scheiterte an Konfiguration bzw. Initialisierung; neutrale STEP-Geometrie
bleibt erforderlich. Siehe output/body-parts-import/README.md und assembly-manifest.json.

Prüfwerkzeug: Server-Wartezeitüberschreitungen sind keine fehlgeschlagenen
Mutationen. tools/cad_test_client.py wartet auf derselben Verbindung mittels
control.pending auf das Originalergebnis, statt Änderungen erneut zu senden.
Veröffentlichungsnachweis: output/parameter-performance/published.json; gemessene
und abschließend um die reine UI-Scrollkorrektur ergänzte Programmversion werden
mit getrennten Prüfsummen geführt. Der vorhandene Startknopf wählt die neue Datei;
eine bereits geöffnete Benutzerinstanz wurde nicht beendet.

## Arbeitstitel

Methodik zur automatisierten, durchgängigen virtuellen Entwicklung mechatronischer Systeme auf Basis parametrischer Konstruktion und modularer Simulationsmodelle.

## Eintrag R-2026-10-21: Stand der Technik und Architektur

**Ausgangsfrage:** Welche Teile der gewünschten CAD–MKS–FEM/ROM–Optimierungs-Kette sind bereits verfügbar, und wo könnte ein eigenständiger Methodenbeitrag liegen?

**Grundlagen:** Eigene Masterarbeit S00; externe Quellen S01–S32 im [Literaturindex](Literatur/INDEX.md). Begründungen und genaue Verweise stehen im [Recherchebericht](RECHERCHE_STAND_DER_TECHNIK.md). Beschaffungs-/Lesestatus stehen im [Rechercheprotokoll](Literatur/RECHERCHEPROTOKOLL.md).

**Erkenntnis:** CAD-Import, flexible ROM-Körper, Workflow-Automatisierung, gekoppelte Optimierung und standardisierte Nachvollziehbarkeit haben belegte Vorläufer. Der vorgeschlagene Beitrag konzentriert sich deshalb auf konsistente mechanische Schnittstellen und die geprüfte Aktualisierung variantenfähiger Simulationsmodelle nach CAD-Änderungen. Neuheit und Verbesserung sind noch nicht bewiesen.

**Vorläufige Entscheidung:** Exudyn als erster MKS-Kandidat, Netgen/NGSolve für FEM-/ROM-Evaluierung, SciPy für erste Parameteroptimierung. Externe numerische Prozesse mit versioniertem Datenvertrag. Eigene mechanische Portsemantik, später Standardschnittstellen zu FMI/SSP. Keine eigene allgemeine Solverentwicklung.

**Alternativen:** Chrono als weiterer MKS-Kern; OpenModelica/OMSimulator für akausale Modellierung bzw. Co-Simulation; OpenMDAO für umfangreichere gekoppelte Optimierung. Diese Alternativen sind nicht grundsätzlich ausgeschlossen. Zuerst den kleinsten vollständigen Anwendungsfall nachweisen.

**Abgrenzung:** Hydraulik, Echtzeit-HIL und allgemeine Topologieoptimierung sind spätere Erweiterungen. Die Oberfläche bleibt die ursprüngliche SolidCraft-Oberfläche mit zusätzlichen Reitern. Die Masterarbeitsmethodik ist Ausgangspunkt, keine externe Validierung unserer Erweiterung.

**Durchgeführte Prüfung:** 30 externe Dokumente tatsächlich lokal beschafft, davon 10 PDFs; technische PDF-Öffnung und Textauslese geprüft. Titelseite von S02 gerendert und visuell geprüft. Masterarbeit zusätzlich lokal im Literaturordner vorhanden. Für S22/S23 HTTP 403 protokolliert, keine Volltextdateien behauptet. 31 TXT-Dateien einschließlich S00 als Importalternativen vorbereitet.

**Nicht durchgeführt:** Keine Solver-Eignungsversuche, keine neuen Software-Builds und keine Wiederaufnahme der pausierten Tests. Es gibt hier noch keine Messdaten für Zeitersparnis, Genauigkeit oder Optimierungserfolg.

**Geplante Abbildungen:** Prozessvergleich Masterarbeit/Automatisierung; mechanischer Portvertrag; Änderungsabhängigkeiten; Variantenwechsel; Genauigkeit-Aufwand-Kurven. Ergebnisse erst aus tatsächlich ausgeführten und protokollierten Versuchen erzeugen.

## Eintrag E-2026-10-21: erste numerische Referenzmodelle

**Status:** Tatsächlich ausgeführte, kleine Referenzrechnungen; keine Untersuchung des vollständigen Roboters und kein Beleg einer schnelleren Entwicklungsmethodik. Der frühere Rechercheeintrag beschreibt den damaligen Stand vor dieser Implementierung.

**Frage:** Können die ersten Adapter einen analytisch kontrollierbaren Lastfall und eine beschränkte Dimensionierung mit reproduzierbaren Eingaben/Ergebnissen abbilden?

**Software:** Python 3.14.7, Exudyn 1.13.6, SciPy 1.18.0, NumPy 2.5.4. Eigene Worker-Datei und Eingabeaufträge werden jeweils mit SHA-256 dokumentiert. Rust-Engine und Oberfläche werden getrennt vom pausierten alten Buildverzeichnis geprüft. [Maschinenlesbares Protokoll](output/reference-adapters/verification.json).

**Referenz B1:** Rechteckiger, gleichförmiger Kragträger. L = 1000 mm, t = 10 mm, h = 100 mm, E = 210000 MPa, rho = 7850 kg/m³, Endkraft = 1000 N in z. Keine Eigenlast und kein Schubanteil. Analytisch: I = t·h³/12, delta = F·L³/(3·E·I), m = rho·L·t·h·10⁻⁹. Ausgangswerte: 7,85 kg, 1,9047619 mm Durchbiegung, 60 MPa Biegespannung. Der getrennt kompilierte Rust-Rechenkern besteht die Handrechnungs-/Skalierungsprüfung einschließlich Vorzeichenwechsel und fehlerhafter Eingaben.

**Versuch E1 – Dimensionierung:** SciPy SLSQP mit auf [0,1] skalierten Entwurfsgrößen, Dicke 2–20 mm, Höhe 10–200 mm; zusätzlich L/h ≥ 10 und |delta|/L ≤ 0,02 als Gültigkeitsgrenzen des Referenzmodells. Ziel minimale Masse, Durchbiegung höchstens 2 mm. Ergebnis: t ≈ 9,5238095 mm, h ≈ 100 mm, Masse ≈ 7,4761905 kg. Die numerische Durchbiegung liegt um ca. 7,8·10⁻¹¹ mm über 2 mm; der dokumentierte Machbarkeitstest verwendet relative Toleranz 10⁻⁷. Das einfache Ergebnis stimmt mit der unabhängig eingesetzten analytischen Lösung dieses Sonderfalls überein. Keine allgemeine Globaloptimalitätsaussage und keine Spannungs-/Beulfestigkeitsfreigabe.

**Versuch E2 – Dynamik:** Exudyn-Endmassenschwinger mit k = 525000 N/m, Endmasse 5 kg, Dämpfungsgrad 0,02 und einer Kraftsprunganregung von 1000 N. Trägheit des Trägers wird in diesem ersten Modell weggelassen; es ist ausdrücklich kein FEM-/CMS-ROM. Generalized-alpha mit Spektralradius 1, Zeitdauer 0,2 s; Eigenfrequenz des Referenzschwingers ca. 51,5721 Hz. Vergleich aller Sensormesspunkte mit der geschlossenen gedämpften Sprungantwort.

**Zeitschrittkonvergenz und zunächst fehlgeschlagene Prüfung:** Bei 2000 Schritten betrug die maximale Verschiebungsabweichung ca. 0,0030651 mm. Das überschritt das zuvor definierte relative Fehlerbudget von 0,1 % der statischen Referenzdurchbiegung (hier ca. 0,0019048 mm); der Lauf wurde deshalb als fehlgeschlagen markiert. Bei 4000 Schritten sank die Abweichung auf ca. 0,00076628 mm, Verhältnis der Fehler ca. 3,99994. Das passt im untersuchten Fall zur erwarteten quadratischen Zeitkonvergenz. Die kleinere Schrittweite ist im neuen Beispielauftrag voreingestellt; die Genauigkeitsgrenze wurde nicht gelockert. Keine nachträgliche Filterung der Rohdaten.

**Weitere Prüfungen:** Unerfüllbare Dimensionierungsaufgabe wird als `infeasible` ausgegeben; unbekannte Backends und zu grobe Zeitschritte werden abgelehnt. Beide Worker wurden zusätzlich über ihre tatsächliche JSON-Datei-/Kommandozeilenschnittstelle ausgeführt. Der Worker verweigert das Überschreiben vorhandener Ergebnisdateien.

**Artefakte für Abbildungen:** [Exudyn-Zeitreihe](output/reference-adapters/exudyn_response.csv), [Optimierungsverlauf](output/reference-adapters/scipy_iterations.csv) und vollständige JSON-Ergebnisse im selben Ordner. Mögliche Grafiken: numerische vs. exakte Sprungantwort; Fehler bei dt und dt/2; Masse und Durchbiegung über SLSQP-Iterationen. Die bisherigen Laufzeiten sind Einzelmessungen ohne kontrollierte Performance-Versuchsplanung.

**Implementierte Rückführung:** Längen können dieselben dimensionierten CAD-Parameter verwenden. Eingaben und Ergebnisse werden als Snapshots geführt. Änderungen gebundener Parameter machen den Auftrag veraltet; ein späterer Ergebnisimport wird abgelehnt. Optimierungsergebnisse werden noch nicht automatisch in die CAD-Konstruktion geschrieben.

**Offen:** Native Desktop-Sichtprüfung des aktuellen Quellstands, Ausführung der neuen Session-Regressionstests, CAD-Anbindung der Querschnittsgeometrie, FEM-/CMS-Reduktion, echte Mehrkörperbaugruppe und persistente Studienverwaltung. Die alte große Suite bleibt pausiert; keine pauschale Aussage „alle Tests bestanden“.

## Vorlage für jeden späteren Versuch

```text
Versuchs-ID / Datum:
Forschungsfrage / Hypothese:
Zugehöriger Beitrag / Benchmark:
Quellen und genaue Fundstellen:
Begründung der Methode und verworfene Alternativen:
Git-Commit + uncommitted diff / Artefakt-Hashes:
CAD-Modell / Varianten-ID / Portvertrag-Version:
Parameter mit Einheiten und zulässigen Bereichen:
Material / Geometrie / Netz / Randbedingungen:
ROM-Methode / Modenzahl / Matrix- und Rahmenkonvention:
Solver und Version / Integrator / Toleranzen / Zeitschritt:
Hardware / parallele Prozesse / Abbruchkriterien:
Referenzmodell / Messaufbau / Messunsicherheit:
Vorab definierte Akzeptanzgrenzen:
Rohdatenpfad / Auswertungsskript / Abbildungspfad:
Aktive Bearbeitungszeit / automatische Rechenzeit:
Ergebnis / Fehlermaße / Wiederholungen / Streuung:
Abweichungen vom geplanten Vorgehen:
Entscheidung / Einschränkungen / nächster Schritt:
```

Die Vorlage enthält keine vorweggenommenen Ergebnisse. Eine erfolgreiche Programmkompilierung ist kein physikalischer Validierungsnachweis; eine genaue Einzelrechnung ist kein Nachweis eines schnelleren Entwicklungsverfahrens.


## UI-Entscheidung 21.10.2026: Originaloberfläche und bedarfsweise Werkzeuge

Die Rückmeldung des Nutzers stellt klar: Zusätzliche Reiter dürfen nicht zu einer dauerhaft anderen Arbeitsoberfläche führen. Deshalb bleiben der native CAD-Viewport, Browser, Timeline und die ursprünglichen Reiter erhalten. Die zusätzlichen Reiter enthalten native Toolbar-Kommandos. Schnittstellen, Studien und Blockmodell öffnen erst auf Anforderung ein schließbares Fenster; für das Rechenblatt wird das vorhandene SolidCraft-Parameterfenster wiederverwendet. Der Launcher lädt standardmäßig keinen Studio-Demonstrator mehr.

**Nachweise:** Separater Desktop-Build erfolgreich. Die tatsächliche Fensterprüfung hat alle ursprünglichen und zusätzlichen Reiter angeklickt und einen identischen CAD-Zeichenbereich festgestellt. Studien- und Schnittstellenfenster wurden ausdrücklich geöffnet und geschlossen. Ein normaler Engine-Aufruf öffnet keine Eingabemaske. Das Überlaufmenü wurde bei 900 logischen Pixeln Fensterbreite geprüft. Die ursprünglichen Reiterpositionen wurden aus dem echten deutschen UI-Screenshot abgelesen; neue Funktionsknöpfe und Überlaufmenüpunkte werden über veröffentlichte Handles angeklickt.

Der Quellvergleich bestätigt: Zentraler CAD-Renderblock und ursprünglicher Viewport entsprechen der offiziellen Ausgangsversion. Prüfprotokoll, Bilder und Build-Prüfsumme liegen unter [output/native-shell](output/native-shell/README.md). Die alte große Suite wurde nicht fortgesetzt. Diese Bedienentscheidung ist kein zusätzlicher Nachweis der physikalischen Modellgenauigkeit.


## 2026-10-21: Native Roboterbaugruppe

27 CAD-Koerper in acht Komponenten gruppiert, Basis geerdet, sechs Drehgelenke und starre Werkzeugmontage erstellt. Alle Achsen einzeln und inverse Kinematik sowie echter Mauszug im nativen Programm geprueft. Separate CAD-Datei output/Modular-Robot-Assembly.solidcraft. Ergebnisse und Grenzen: output/ROBOTER_BAUGRUPPE.md; Rohprotokoll output/native-shell/robot-assembly-verification.json. Kein Nachweis einer dynamischen MBS oder FEM. Port-Nachfuehrung und physikalisch getrennte Werkzeugvarianten bleiben offen.


## 2026-10-21: Detailliertere Robotergeometrie

67 Detailkoerper fuer Motoren, Lagerdeckel, Verschraubungen, Dichtungen, Kabelpakete und Greifer hinzugefuegt; Zuordnung zu bestehenden beweglichen Komponenten. Sechs einzelne Achsbewegungen ohne Gelenkkonflikte oder schwebende Ketten geprueft. Eigener Demonstrationsentwurf; keine Herstellerdaten und keine Validierung von Fertigung, Masse oder Steifigkeit. Kabel bleiben pro Armglied starr. Reproduzierbare Befehle: output/robot-detail-recipe.json; Modell und Grenzen: output/ROBOTER_DETAILLIERUNG.md.


## 2026-10-21: Parametrisches Planetengetriebe als zweite Referenzbaugruppe

Separate native CAD-Datei output/planetary/Planetengetriebe.solidcraft: sechs Komponenten, fuenf Drehgelenke, vier Bewegungskopplungen; geerdetes Hohlrad. Eigener analytischer Evolventen-Generator mit abgetasteten Flanken und vereinfachtem Zahnfuss. Startwerte z_s=24, z_p=24, z_r=72, Modul 2 mm, Breite 20 mm, i=4, n_in=1500 rpm. Native Parametergleichungen und zusaetzliches interaktives HTML-Berechnungsblatt. Keine proprietaere Mathcad-Datei; noch keine automatische Uebernahme der HTML-Eingaben. Topologieaenderungen brauchen vollstaendige Neuerzeugung.

Gezielte Nachweise: 90 Grad Input -> 22.5 Grad Output, -67.5 Grad Planetendrehung relativ zum Traeger; n_in 1200 -> n_out 300 und n_planet_abs -600 rpm; tatsaechlicher Mauszug aendert Gelenke; Speichern/Wiederoeffnen erhaelt 5 Gelenke und 4 Kopplungen. Zusaetzlich 73 Winkelstellungen mit insgesamt 219 Planetenstellungen numerisch auf eingedrungene Profilstuetzpunkte untersucht: keine Treffer. Kein kontinuierlicher Kontakt-/Interferenznachweis. Geometrie- und Bewegungsnachweise stehen in verification.json, sampled-mesh-check.json und echten CAD-Bildschirmaufnahmen.

Loesungsweg: getrennte starre Komponenten, Willis-Kinematik als Winkelkopplungen statt Kontaktmodell; parametrierte Profilpunkte und Breite, getrennte Generierung der Zahnanzahl. Fehler im eigenen Erzeugungsskript: Symbol m wurde als Einheit Meter interpretiert; durch eindeutig benannten Parameter modul ersetzt. Hohlradextrusion auf die tatsaechliche Ringregion beschraenkt. Grenzen: kein Lastkollektiv, keine Werkstoff-/Lager-/Festigkeitsauslegung, keine Fertigungsfreigabe; Zahnfuess und Flanken sind angenaehert. Primaerquelle KHK, Internal Gears 2025; Quellenverweis in Literatur/Planetengetriebe.


## 2026-10-21: Allgemeines natives Berechnungsdokument treibt CAD

Neue Engine-Befehle worksheet.read/set/describe/save/load/open, fuer jedes Projekt verwendbar. Berechnungen sind vorhandene native Benutzerparameter; kein zweiter Auswerter und keine duplizierten Ergebnisse. Projekttitel und Notizen in der vorhandenen namespaced Metadata gespeichert. Eigenes offenes solidcraft-math/1 JSON-Format (.scmath) zum Austausch. Definitionen werden zusammen mit dem CAD-Projekt gespeichert. Native Projektseiten Start, Parameter, CAD, FEM, MKS; Varianten auf Start zugaenglich. Keine fremde Mathcad-Software eingebunden.

Methodik: Allgemeiner Nachweis an einer neuen Skizzenplatte statt einer getriebespezifischen Sonderloesung. Laenge 120 -> 180 mm per echter UI-Tastatureingabe; Breite=Laenge/3 von 40 -> 60 mm; Extrusion Dicke=5 mm; Volumen 24000 -> 54000 mm3 gemessen. Zyklische Definition abgelehnt, letzter gueltiger Zustand erhalten. Speichern/Wiederoeffnen bestanden. Rohprotokoll und native Bildschirmaufnahmen: output/math-cad/. Einzelne kurze native Engine-Tests zu direktem CAD-Mass, Skizzenkoordinaten und Atomaritaet; alte breite Testlaeufe bleiben pausiert.

Grenzen: Skalare sowie Laengen/Winkel/dimensionslose Einheiten; keine zusammengesetzten physikalischen Einheiten, symbolische Algebra, Matrizen oder Gleichungssysteme. Exportierte externe Dateien muessen bewusst neu geladen werden. Topologieaenderungen brauchen entsprechende native Features. Analytische Evolventenkurve gemaess Nutzeranforderung weiterhin offen; weder Spline noch Polygon wird als deren Erfuellung ausgegeben.

Abschlusspruefung desselben Builds: 2 gezielte Rust-Tests bestanden (1863 andere Tests herausgefiltert, Laufzeit 0.02 s nach erstmaliger Kompilierung); Clippy Engine/UI mit -D warnings bestanden. Native Maus-/Tastatureingabe, .scmath-Dateiimport (150 mm -> 37500 mm3), Wiederherstellung, Projekt-Neuladen und direkter --worksheet-Start bestanden. Build- und Quellenhashes: output/math-cad/build-verification.json. Mathedateien und Projekt sind ohne Python-Laufzeit benutzbar.


## 2026-10-21: Lokales CalcpadCE, freigegebene CAD-Referenzen und Berechnungsvarianten

Fragestellung: Wie bleiben Berechnung, Variantenentscheidung und CAD-Aktualisierung nachvollziehbar, ohne jeden Zwischenstand sofort als Konstruktion zu uebernehmen? Umsetzung: CalcpadCE als lokaler, kurzlebiger Prozess mit JSON-Ein-/Ausgabe; keine Serverarchitektur. Revision 0cf438f7d4a78a7ee50724eefb712c23f2ec09c5 (MIT). Kleine eigene Erweiterung exportiert praezise reelle Skalare und Einheiten vor dem Aufraeumen des Parsers. Keine Zahlenextraktion aus gerundetem HTML. Die native Oberflaeche bleibt im Reiter Parameter.

Methode: Berechnungsquelle -> lokales Ergebnis -> publizierter Referenzname ref_X -> explizite Aenderungspruefung -> atomare CAD-Neuberechnung. Vor Uebernahme gelten im CAD die bisherigen Referenzwerte. Varianten speichern Definitionen und Calcpad-Quelltext im Projekt; Auswahl berechnet diese neu. Eine geaenderte Quelle sperrt alte Ergebnisse bis zur erfolgreichen Berechnung. Direkte alte CAD-Parameter ohne Referenz bleiben unmittelbar wirksam.

Gezielte Engine-Pruefung: vier Tests bestanden, 1863 weitere Tests herausgefiltert, Ausfuehrung 0.02 s. Geprueft: explizite Uebernahme, Variantenwechsel ohne vorzeitige CAD-Aenderung, Projektpersistenz, Ablehnung benutzter Referenzloeschung, Einheit cm -> interne mm, Sperre veralteter Calcpad-Ergebnisse sowie die beiden vorhandenen allgemeinen Skizzen-/CAD-Nachweise. Separate kurze Prozesspruefungen bestaetigen zusammengesetzte Einheiten und die Ablehnung unbekannter Variablen und noch nicht unterstuetzter Direktiven.

Befunde des Entwicklungswegs: Namespaced Projektmetadaten verlangen Objekte statt Arrays; Referenzliste deshalb als items in einem Objekt gespeichert. Calcpad erfasst die strukturierte Fehlerliste nur bei aktiviertem Debug-Modus; aktiviert und durch einen negativen Fall nachgewiesen. Der Parser loescht seinen Rechenzustand nach Berichtserstellung; Export erfolgt davor. Diese Entscheidungen sind fuer die Reproduzierbarkeit und einen Vergleich mit rein dateibasierten Kopplungen zu dokumentieren.

Grenzen: Textbasierter Calcpad-Editor mit Ergebnistabelle, noch kein typografischer 2D-Matheeditor. Keine Direktiven-/Solver-Makrointegration, kein beliebiger Topologiewechsel. KUE-Vollauslegung und exakte analytische Evolventenkurven bleiben offen. Native Backend- und UI-Nachweise werden getrennt von Fertigungs-, Festigkeits- und Simulationsvalidierung gefuehrt. Die alte grosse Testsuite bleibt pausiert.

Desktop-Abschlussnachweis: tools/verify_local_calcpad.py bestanden. Variante 2 im echten Auswahlfeld angeklickt und aus dem angezeigten Editor erneut berechnet; L=180 mm und B=60 mm. Bis zum echten Buttonklick bleibt die Platte bei 24000 mm3, danach 54000 mm3. Ungueltige Quelle und veraltete Ergebnisse abgelehnt; CAD-Datei wieder geoeffnet und Variante 1 wiederhergestellt. Bildschirmaufnahmen und Rohdaten: output/local-calcpad-check/. Bei der visuellen Kontrolle erkannter veralteter Editorinhalt nach Variantenwechsel korrigiert; Wiederberechnung als zusaetzliche UI-Regression geprueft. Build-/Quellhashes: build-verification.json.


## 2026-10-21: Analytische Verzahnung und gekoppelte KUE-Varianten

Forschungsbaustein: Ein Berechnungsdokument bestimmt nicht nur Abmessungen,
sondern Zahnzahlen, neue CAD-Topologie und die Kinematik der Baugruppe. Die
Freigabe neuer Referenzwerte wird zusammen mit Gelenkankern, Erdung und
Willis-Kopplungen auf einer Arbeitskopie ausgefuehrt. Erst bei gueltiger
Neuberechnung wird diese gemeinsam uebernommen. Zusaetzliche fremde Komponenten
werden beim KUE-Abgleich nicht pauschal zurueckgesetzt.

Loesungsweg Geometrie: Eigener nichtpolynomialer Kurventyp im Apache-2.0-
Truck-Modellierer, x(t)=rb(cos(t)+t sin(t)), y(t)=rb(sin(t)-t cos(t)); exakte
Ableitungen und analytische lineare Extrusionsflaeche. Dediziertes natives
Zahnradfeature statt einer angepassten Spline-Skizze. Grundkreis, Kopf-/Fusskreis,
Bohrung und Phasenlage werden parametrisch aufgebaut. Speichern/Laden uebertraegt
die analytischen Typen. Eine angeforderte Umwandlung in polynomiale Splines und
der aktuell nicht unterstuetzte STEP-Export werden ausdruecklich abgewiesen.
Unterhalb des Grundkreises: radiale Tangente und Kreisverrundung; keine behauptete
werkzeuggenerierte Trochoide. Darstellungstessellation bleibt von Modellgeometrie
und Messproben getrennt.

Befunde/Fehlversuche: Der Evolventenbeginn t=0 hat verschwindende erste Ableitung;
die Flaechennormale verwendet dort die analytische Grenztangente. Die bisherige
polynomiale Lift-up-Schnittstelle war fuer diesen Kurventyp ungeeignet und wurde
fallibel gemacht. Calcpad akzeptiert Gleichheitsvergleich mit U+2261 und
Kleiner-gleich mit U+2264, nicht == bzw. <=. Der Funktionsname lautet ceiling.
Die urspruengliche Ganzzahl-Modulrundung wurde durch eine Projekt-Vorzugsreihe
ersetzt. Neue Schnittstellen wurden vor dem Baugruppennachweis einzeln geprueft.

KUE-Methode: Vier Angabedatensaetze aus S. 6 und vier Schaltungen aus S. 7; erster
Planetensatz z_s=30, drei gleichverteilte Planeten. Restuebersetzung wird auf
Stufe 2 verteilt, ganzzahlige Zahnzahlen/Montagebedingung quantisiert und die
Gesamtdrehzahlabweichung geprueft. Anwendungsfaktor wird einmal beim Eingangsmoment
angesetzt. Alle Stufen werden berechnet, erste Stufe konstruiert. Beidseitig
abgestuetzte Bolzen werden auf Spannung, Durchbiegung und Neigung vorausgelegt.
Zahnfuss-Vorauslegung ist ein vereinfachtes Modell, keine ISO-6336-Rechnung.

Reproduzierbare Artefakte: `tools/verify_kue_planetary.py`, Calcpad-Quelle
`solidcraft/crates/engine/src/calculations/kue-stage.cpd`, Ausgaben unter
`output/kue-stage/`, standalone Rechenergebnisse unter
`output/kue/preferred-module-cases.json`. Fachquellen und Grenzen siehe
`PLANETENGETRIEBE.md`. Der isolierte Kerneltest prueft Gleichungsauswertung,
Vernetzung und Persistenz echter Evolventen sowie Ablehnung eines Spline-Lifts.
Die breite alte Testsuite bleibt pausiert.

Vorgesehene Auswertung/Grafiken: Abhaengigkeitsgraph Berechnung -> Referenzen ->
CAD-Feature -> Gelenke; Modul/Volumen ueber Drehmoment; Zahnzahlen und erreichte
Uebersetzung ueber Zielwert; vier Schaltungen mit Drehrichtungen; Aenderungsliste
vor Freigabe; Eingriffsmesswerte ueber Bewegungsphase. Endliche
Durchdringungsproben und analytische Eingriffsbedingungen sind getrennt zu
berichten, ebenso Softwarevalidierung und Fertigungs-/Festigkeitsnachweis.


Befund der ersten Desktop-Integration: Trotz korrekt berechneter standalone
JSON-Testfaelle schlug die native Berechnung mit mathematischen Vergleichszeichen
fehl. Ursache war die Windows-Codepage des .NET-Standardstreams. Der Helfer setzt
Ein- und Ausgabe nun explizit auf UTF-8 ohne BOM. Wichtig fuer die Methodik:
ASCII-escaptes JSON in einem Test kann einen Zeichensatzfehler des echten
Prozessprotokolls verdecken. Der erneute Desktoplauf verwendet den realen
Rust-Protokollpfad mit nichtescapten mathematischen Zeichen.


Abschlussnachweis der nativen Geometrie-/Kinematikkopplung: Zehn gezielte Faelle
bestanden (vier KUE-Angaben, R=3.4 mit 1000/8000 Nm, R=3.8 mit 1000 Nm,
R=4.6 mit 5000 Nm, Sonnenzahnzahl 36, Eingangsdrehzahl 24/min). Pro Fall
121 Eingangsstellungen, alle drei Planeten, jeweils Sonnen- und Hohlradeingriff:
insgesamt 7260 untersuchte Paarstellungen. Keine Durchdringungsprobe oberhalb
der Toleranz 0.0002*m; kontinuierliche Willis-Phaseninvarianten und analytische
Arbeitskontaktgrenzen separat geprueft. Gemessene B-rep-Kanten 4*z und
Extrusionsflaechen 2*z pro Zahnrad. Die tatsaechlichen nativen
Vorkommenstransformationen stimmen mit den erwarteten Planetenpositionen und
Drehwinkeln ueberein, auch bei feststehendem Traeger.

Echte UI-Eingabe T_in=1000N*m -> Berechnen -> Aenderungsliste ->
Konstruktion aktualisieren bestanden. Manueller Modul 0 mm abgelehnt, zuvor
gueltiges Sonnenvolumen erhalten. Wiedergeoeffnetes Projekt behaelt die
analytische Geometrie. Zusatzprobe derselben Zielpunktoperation wie beim
Baugruppenziehen: Traegerziel 5 Grad erreicht mit 4.999999999999985 Grad,
keine Gelenkkonflikte. Originale CAD-Werkzeugleiste bei zusaetzlichen oberen
App-Reitern visuell kontrolliert.

Automatische KUE-Module: 20/16/10/6 mm. Sonnen-/Planeten-/Hohlradzaehne:
30/21/72 fuer Angaben 1/2 und 30/36/102 fuer Angaben 3/4. Erreichte
Generatordrehzahlen: 1012, 1224, 1435.5556 und 1825.2632/min, jeweils innerhalb
+/-5 Prozent. Bei gleicher Uebersetzung steigt der Modul fuer 1000 -> 8000 Nm
von 2.5 -> 5 mm. Daten fuer eigene Grafiken liegen als verification.csv und
vollstaendiges verification.json vor; Konstruktion und Bildschirmaufnahmen im
selben Ausgabeordner.

Allgemeiner Referenzfall unabhaengig vom Getriebe erneut bestanden: Calcpad-
Variante treibt Skizze und Extrusion von 24000 auf 54000 mm3 erst nach expliziter
Uebernahme. Fehlerfall und Wiederladen bestanden. Clippy fuer Kernel/Engine/UI
mit -D warnings bestanden. Alter breiter Testsatz weiterhin pausiert.

Offene wissenschaftliche Validierung: tragfaehiger Vergleich gegen einen
unabhaengigen Geometrie-/Kontaktansatz, Werkzeughuellkurve, ISO-Tragfaehigkeit,
Lagerauswahl und Reibung, flexible Koerper, dynamische Zahnkontaktkraefte sowie
reproduzierbare Zeit-/Fehlervergleiche gegen manuelle Variantenbearbeitung.
Die erreichte Softwarefunktion allein ist kein Nachweis einer neuen
wissenschaftlichen Methode oder einer fertigen Dissertation.


Visuelle Abschlusskontrolle: Eingabepaare in ein responsives Raster ueberfuehrt,
explizite Feldbreiten gesetzt und die gesamte Berechnungsseite scrollbar gemacht.
Zusaetzliche Probe bei 900x700: sieben Eingaben sichtbar; Mathedateifeld durch
echtes Mausrad-Scrollen erreichbar. Native Bildschirmaufnahmen fuer grosse und
kleine Fenster im Nachweisordner. Finale Build- und Quellenhashes inklusive
Projekt/Mathedatei in output/kue-stage/build-verification.json. Die zehn
Geometriefaelle wurden vor den ausschliesslichen Layoutkorrekturen durchlaufen;
der finale Build wurde fuer UI-Uebernahme, Wiederladen, Zielpunktbewegung und
kleines Fenster separat erneut geprueft. Kein erneuter breiter Testlauf.


## 2026-10-21: Erweiterter Berechnungs- und Varianten-Audit auf Nutzerauftrag

Erneuter Abgleich mit der KUE-Aufgabe, insbesondere S. 6-9, deckt drei Luecken
auf: Windkraftvariante (Zuteilung aus Teil 1) und Aufbauvariante (alphabetische
Zuteilung) sind unabhaengige Achsen; die bisherigen vier Presets prueften nur
die Diagonale dieser 4x4-Matrix. Die bisherige Ausgabe der Stufendrehzahlen und
-momente enthielt nur Betraege, obwohl die Aufgabe Drehrichtungen verlangt.
Der Quantisierer erzwang Planetenzahnzahlen als Vielfache von drei und verwarf
damit geeignete Kombinationen fuer Sonnenzaehne 31/32.

Korrekturen: Eigene Aufbauvariante im eingebetteten Calcpad-Dokument und im
Eingaberaster, Presets n/n bleiben kompatibel. Vorzeichen aus der jeweiligen
Schaltung fuer n_1/n_2/n_Generator und T_1/T_2/T_Generator; Modulberechnung der
zweiten Stufe verwendet Momentbetrag. Drehzahltoleranz verwendet Betrag.
Montagerest (-z_s modulo 3) wird bei der Wahl der Planetenzahnzahl beruecksichtigt.

Pruefmethode: `tools/verify_kue_calculations.py` ruft den wirklichen lokalen
Calcpad-Prozess ueber UTF-8 auf. Unabhaengige numerische Referenz in SI; keine
Produkt-Auswertefunktionen importiert. Zahnzahlwahl durch Enumeration passender
Kandidaten statt gleicher Rundungsformel; gleich gute ganzzahlige Alternativen
zugelassen. Physikalische Einheiten werden ausserhalb Calcpads normalisiert.
Zusaetzliche Gleichgewichtskontrolle P=T*2*pi*n mit vorzeichenbehafteten Werten.
Alle 96 exportierten fachlichen Skalare pro Fall geprueft, nur eingebaute
mathematische Konstanten ausgenommen.

Ergebnis dieses Rechenaudits: 180/180 Faelle und 17280 Skalarkontrollen bestanden.
Abgedeckt: alle 16 Windkraft-/Aufbaukombinationen, Eingangsmomente 100/1000/8000 Nm,
Zielverhaeltnisse 3.2/3.8/4.6, Sonnenzaehne 31/32/36, Breitenfaktoren 10/20,
manueller Modul, geaenderte Drehzahlen und Eingriffswinkel. Dies prueft die
implementierten Vorauslegungsmodelle, nicht deren Eignung als vollstaendigen
KUE-Nachweis. Im ersten Auditorlauf fehlte das Einheitenalias N*Mittelpunkt*m;
die 48 betreffenden Faelle waren ein Fehler im Pruefadapter, keine Produktfehler.
Alias ergaenzt und den vollstaendigen Rechenaudit erneut bestanden.

Artefakte: output/kue-comprehensive/calculation-audit.json; eigene Desktoppruefung
unter output/kue-comprehensive/desktop; abschliessender Bericht wird getrennt
von Rohdaten gefuehrt. Kataloglager, zwei detaillierte Welle-Nabe-Nachweise,
reale Werkzeughuellkurven und der vollstaendige Gehaeuse-/Montagenachweis sind
weiterhin eigene, noch nicht implementierte KUE-Arbeitspakete.


Abschluss dieses erweiterten Audits: Alle 28 CAD-/Bewegungsfaelle bestanden,
einschliesslich aller 16 Grundkombinationen, Sonne 31/32/36, verschiedener
Uebersetzungen, Drehmomente, Drehzahlen, Breiten, manuellem Modul 4.7 mm und
Eingriffswinkel 25 Grad. 121 Stellungen pro Fall, drei Planeten und zwei
Zahneingriffspaare ergeben 20328 gepruefte Paarstellungen. Keine
Durchdringungsprobe oberhalb der Toleranz. Pro nativem Fall nochmals 96
Berechnungsgroessen gegen die unabhaengige SI-Referenz verglichen.

14 gezielte unzulaessige bzw. nicht unterstuetzte Eingaben wurden abgewiesen;
Referenzen blieben exakt gleich, das zuvor gueltige Sonnenvolumen unveraendert.
Faelle: Modul 0/zu klein/falsche Dimension, Zahnzahl 0/nicht ganzzahlig,
Eingangsmoment negativ/0/extrem gross, Drehzahl 0, Breitenfaktor 0,
Aufbauvariante 5, nicht baubares Zielverhaeltnis, unterkritischer Eingriffswinkel
fuer die vorhandenen Zahnzahlen und unbekanntes Symbol. Negative Eingangslasten
sind damit als derzeit nicht unterstuetzte Auslegung markiert, nicht als
physikalisch grundsaetzlich unmoeglich bewertet.

Echte UI-Eingabe, Berechnen, bewusste CAD-Uebernahme, Speichern/Wiederladen,
5-Grad-Zielpunktbewegung und acht Eingabefelder/Scrollen bei 900x700 erneut
bestanden. Clippy Engine/UI mit -D warnings bestanden. Zusammenfassung in
PRUEFBERICHT_KUE.md, Rohdaten/CSV/Quellenhashes in output/kue-comprehensive.
Startdateien verweisen auf dieses neue Testprojekt; das vorherige Beispiel
unter output/kue-stage bleibt erhalten. Kein Anspruch, damit die noch fehlenden
Lager-/Welle-Nabe-/Gehaeuse-/Werkzeugnachweise erbracht zu haben.


## Bearbeitbarkeit wie bei manueller CAD-Erstellung

Befund nach Nutzerpruefung: Die bisherigen direkten Zylinder-Features besassen
keine Kreisskizzen; beim eigenen Evolventenfeature fehlten Bearbeitungsdialog
und Zulassung im normalen timeline.redefine-Pfad. Parametrische Erzeugung ueber
MCP allein ist noch keine vollstaendige interaktive Bearbeitbarkeit.

Korrektur: Runde Traeger-/Bolzen-/Buchsenkoerper und deren Ausschnitte verwenden
nun dieselben registrierten Befehle wie der Mensch: Skizze auf XY mit
parametrischem Versatz, Kreis, echtes treibendes Radiusmass mit ref_-Ausdruck,
Skizze beenden, normale Extrusion/Ausschnitt. 16 solche Skizzen und 16
Extrusionen. Die analytischen Zahnraeder behalten echte Evolventen und bekommen
einen normalen Feature-Editor einschliesslich Innenverzahnung und Zentrum.
Der eng begrenzte Konstruktor timeline.redefine akzeptiert jetzt genau diesen
neuen Modellierbefehl, keine beliebigen Inspektions-/Dateibefehle.

Nachweis tools/verify_editable_planetary.py: 16 echte Referenz-Bemassungen,
Zahnradbreite per UI ref_b -> ref_b+1mm und zurueck mit passender Volumenaenderung,
normaler Extrusionseditor, Skizzenmass per Doppelklick ref_pin_r+1mm, vier
Varianten und Sonnenzahnzahl 31, Wiederladen. Alle bestanden. Artefakte unter
output/kue-editable; Startdateien oeffnen dieses separate Projekt, alte Projekte
bleiben erhalten. Neue Wiederholung von UI-Aufnahmen nutzt eindeutige
Recovery-Verzeichnisse, damit eigene alte Test-Autosaves die Ansicht nicht verdecken.

## Allgemeines Zahnradwerkzeug: Methodik und Entwicklungsgrenzen

Die beiden Nutzerauftraege werden in derselben Arbeitssitzung nacheinander bearbeitet:
zuerst interaktive Bearbeitbarkeit der Planetenstufe, danach allgemeines Zahnradwerkzeug.
Fachliche Recherche und Funktionsmatrix: ZAHNRADWERKZEUG_RECHERCHE.md.
Drei oeffentliche Hersteller-PDFs von KG Stock Gears wurden lokal gespeichert;
Original-URLs, Abrufdatum, Dateigroesse und SHA-256 liegen in
Literatur/Planetengetriebe/Zahnradwerkzeug/quellen.json.

Methodische Trennung: Bezugsabmessungen/Kinematik sind nicht gleichbedeutend mit
korrekter Herstellgeometrie, kollisionsfreiem Eingriff oder Tragfaehigkeit.
Der neue registrierte Engine-Befehl gear.evaluate berechnet die Zahnradfamilien
ohne Dokumentmutation. gear.design speichert einen expliziten Ergebnis-Snapshot
und erzeugt nur fuer vorhandene Generatoren native Geometrie. Native Konstruktion
behaelt die urspruenglichen Parameterausdruecke; Snapshots aktualisieren sich nicht
heimlich. Bei noch fehlenden Flankengeneratoren wird keine Ersatzform erzeugt.

Implementiert: normale CAD-Dialogintegration mit Familienwahl und passenden
Eingaben, Formeln und Einheiten im Ergebnis, Warnungen/Geltungsbereich,
Speicherung von Eingaben und Ergebnisprotokollen im Projekt, Wiederaufruf der
letzten Eingaben. Exakte Stirn-/Hohlradgeometrie ist an den bestehenden nativen
Evolventenkonstruktor angeschlossen. Schraub-/Pfeil-/Schnecken-/Kegelradflanken
bleiben explizit offen. Auch Profilverschiebung und normgerechte Nachweise fehlen.
Die gesamte urspruengliche Forderung nach allen Zahnradfunktionen ist damit noch
nicht erfuellt; die konkrete Reihenfolge der fehlenden Entwicklungen ist dokumentiert.

Fuer wissenschaftliche Abbildungen: Gegenueberstellung von (a) Berechnungsgraph,
(b) nativen treibenden Skizzenmassen und (c) Feature-Neuaufbau; getrennte
Darstellung von Faehigkeitsstatus, Gueltigkeitsbereich und Fehlerpfad. Ein
Screenshot eines aehnlich aussehenden Zahnrads ersetzt keinen Flankennachweis.
Hypothese fuer die spaetere Evaluation: Die explizite Rueckverfolgbarkeit von
Berechnungsresultat zu nativer Bemassung reduziert Such-/Korrekturzeit bei
Variantenwechseln. Dies ist noch kein experimentell belegtes Ergebnis.

Abschluss der gezielten Pruefung: Build und Clippy fuer Engine/UI mit -D warnings
bestanden. tools/verify_editable_planetary.py erneut bestanden und saubere Screenshots
ohne Recovery-Popup erzeugt. tools/verify_gear_tool.py bestanden: 30 Auslegungen
(zehn Familien mit je drei Modul-/Zahnzahl-/Drehmomentkombinationen), acht ungueltige
Eingaben ohne Veraenderung von Timeline/Auslegungsprotokoll, echter Aufruf ueber
das Erstellen-Menue, Berechnung und CAD-Erzeugung per Dialog, zwei weitere native
Raeder, Modul 2->3 mm und Zahnzahl 30->36 als echte Referenzausdruecke mit
Neuaufbau, 144 analytische Evolventenkanten am 36-Zaehne-Rad, Speichern/Wiederladen
von 13 Auslegungsprotokollen und natives Geometrie-Roundtrip.

Erkenntnisse aus Fehlversuchen: Einheitenlose Zaehnezahlen duerfen nicht als
Laengenparameter angelegt werden (Testaufbau korrigiert, keine Einheitenpruefung
entfernt). Der erste Dialog hatte die Berechnen-Aktion unter einer langen
Eingabeliste; diese sitzt jetzt oben, Eingaben und Ergebnisse sind getrennte
Ansichten im normalen Dialog. Ein alter Test adressierte einen nicht publizierten
Cancel-Handle; auf den vorhandenen Escape-Bedienweg korrigiert. Dies war ein
Testtreiberfehler, kein Geometrie- oder Speicherfehler.

Rohdaten: output/gear-tool/verification.json, output/kue-editable/verification.json;
aktuelle Quell-/Artefakthashes: output/gear-tool/build-verification.json.
Die Planetenpruefung erfolgte vor der letzten reinen Layoutkorrektur des neuen
Zahnrad-Dialogs; der anschliessende Zahnradtest auf dem finalen Build prueft dessen
Menue, Dialog, Parametrik und Persistenz. Die breite alte Testsuite blieb pausiert.

## Suite-Ausbau: echte Varianten, Aenderungspruefung und Aktualisierung

Ausgangsbefund: Die allgemeine Zahnradfunktion legte neue Auslegungs-Snapshots ab,
bot jedoch keinen gezielten Wiederaufruf aller Entwuerfe und keine Aktualisierung
ihres vorhandenen CAD-Features an. Die alte Start-Variantenansicht zeigte primär
Roboterdemo-Optionen, auch wenn ein reales Berechnungsprojekt geladen war.

Entscheidung: Zuerst die bestehende Berechnungs-/CAD-Kette durchgaengig machen.
Weitere Generatoren und Solver sollen auf einer nachvollziehbaren Varianten- und
Ergebniszuordnung aufbauen. Der weitere Ablauf steht in SUITE_FAHRPLAN.md.

Implementierung: Append-only gespeicherte Entwuerfe bekommen optionale Namen und
bleiben ueber ihren bestehenden Index adressierbar. Alte Projekte erhalten
Anzeigenamen ohne Migration. gear.review vergleicht die gespeicherten Ergebniswerte
mit der aktuellen Auswertung und die erwartete Feature-Definition mit der nativen
Konstruktion. Zahlenvergleiche nutzen eine skalierte Toleranz von 1e-10;
Definitionen werden separat als Parameterausdruecke verglichen. Gleichwertige,
anders geschriebene Ausdruecke koennen deshalb als abweichend erscheinen.

gear.update arbeitet auf einem temporaeren Dokument, baut vorhandene Zahnraeder
mit timeline.redefine neu auf und behaelt ihre Feature-ID. Fehler duerfen weder
Entwurf noch Konstruktion veraendern. Eine geloeschte Verknuepfung wird gemeldet
und nicht unbemerkt durch einen neuen Koerper ersetzt. gear.compare vergleicht
standardmaessig historische Ergebnis-Snapshots; optional wertet es beide Entwuerfe
im aktuellen gemeinsamen Parameterkontext aus. Ungleiche Einheiten werden nicht
subtrahiert. Alle Funktionen sind registrierte Engine-/MCP-Befehle.

UI: Auswahl gespeicherter Entwuerfe, Name, Zustand, explizite Aktualisierung und
Vergleich im normalen Zahnrad-Dialog. Die Start-Variantenansicht nutzt reale
Projektvarianten, dieselbe bewusste CAD-Referenzuebernahme wie die Parameterseite
und eine Uebersicht der Zahnradentwuerfe. Keine permanenten CAD-Eingabepanels.

Versuchsaufbau tools/verify_suite_variants.py: Lokales Calcpad-Blatt mit Modul,
Zaehnezahl und Breite; Varianten Kompakt (2 mm, 30, 20 mm) und Robust (3 mm, 36,
25 mm). Noch nicht uebernommene Ergebnisse lassen CAD unveraendert. Nach expliziter
Referenzuebernahme ergeben sich 120 bzw. 144 echte Evolventenkanten. Der Vergleich
liefert 48 mm Teilkreisdurchmesser- und 5 mm Breitenunterschied. Ein manuell
geaendertes CAD-Breitenmass wird als abweichende Definition erkannt.

Weitere Checks: partielle Positionseingabe bei alter Array-Platzierung, Erhalt von
Referenzausdruecken und Feature-ID bei UI-Aktualisierung, sechs atomar abgewiesene
Updates, echte Variantenwahl und Referenzuebernahme in der Start-Ansicht,
Speichern/Wiederladen sowie Erkennung einer geloeschten Verknuepfung.
Ein erster UI-Test nutzte waehrend der nativen Fenstergroessenaenderung alte
Widgetkoordinaten; kurze Stabilisierung des Testfensters korrigiert den Testtreiber.
Ein verschachteltes if wurde nach Clippy-Hinweis semantisch gleich zusammengefasst.

Grenzen: Noch keine isolierten Kopien des gesamten Parametergraphen je Entwurf,
keine allgemeine FEM-/MKS-Ergebnisverwaltung und keine neuen Flankengeneratoren
in diesem Schritt. Eine stabile Feature-ID garantiert nicht die Identitaet aller
Zahnflanken nach Topologieaenderung. Bedienzeitstudien und Messversuche stehen aus;
eine behauptete Zeitersparnis ist weiterhin eine Forschungshypothese.

Rohdaten und Screenshots: output/suite-variants/. Das Startprojekt wird vor den
negativen Loeschtests gespeichert; Start-Suite-Varianten.cmd oeffnet dieses Projekt.

Abschluss: Finaler Desktop-Build und Clippy Engine/UI mit -D warnings bestanden.
Die Suite-Variantenpruefung wurde auf diesem Build erfolgreich wiederholt; der
bestehende gezielte Zahnradtest bestand ebenfalls mit 30 Rechnungen, acht
Fehlerfaellen, UI-Erzeugung, Referenz-Neuaufbau und Speicherung. Aktuelle Hashes
und Pruefstatus: output/suite-variants/build-verification.json. Die alten breiten
Testlaeufe bleiben pausiert. Keine FEM-, ROM- oder Baugruppen-MKS-Loesung wurde
fuer diesen Varianten-Ausbauschritt behauptet oder ausgefuehrt.

## 22.10.2026: Projektschnittstellen und statische Resultantenweiterleitung

Ausgangsbefund: Die bisherigen mechanischen Ports und Kanten lebten nur in der
Sitzung des Engineering-Prototyps. Fuer eine durchgaengige Berechnungs-/CAD-/CAE-Kette
fehlten gespeicherte Zuordnungen und ein erster numerisch pruefbarer Lastpfad.

Implementiert ist deshalb ein getrenntes Projektmodell mit nativen Koerper- und
exakten Occurrence-Bezuegen, parametrischen Ursprungsausdruecken und normierten
Orientierungen. Punktkraft, abstrakter Kraft-/Momentenrahmen, Flaechenresultante
und gleichmaessige Normaltraktion sind unterschiedliche Vertraege. Flaechen werden
noch deklariert, nicht aus topologischen CAD-Flaechen abgeleitet. Die alten
Sitzungsentwuerfe bleiben klar getrennt. Details: MECHANISCHE_SCHNITTSTELLEN.md.

Methodische Wahl: Zunaechst explizit deklarierte starre Baumstrukturen statt einer
unbelegten automatischen Gelenk-/FEM-Kopplung. Jede Last hat einen eindeutigen Weg
zum Fundament. Verzweigungen werden linear ueberlagert. Schleifen und mehrere
Eltern eines Ports werden abgelehnt. Die Resultanten werden mit der Weltrotation
des Ports und M_neu = M_alt + r x F transportiert; mm werden fuer Nm in Meter
umgerechnet. An einem Punktkraft-Uebergang darf kein erforderliches Moment bleiben.
Das liefert eine nachpruefbare statische Referenz fuer spaetere Kopplungsverfahren,
aber keine dynamischen Gelenkreaktionen, Spannungen oder Verformungen.

Registrierte Befehle: mechanics.port, mechanics.connect, mechanics.disconnect,
mechanics.case, mechanics.evaluate, mechanics.solve, mechanics.state. Daten und
Ergebnisse werden im Projekt gespeichert. Ein Ergebnis haelt seinen exakten
Eingabestand mit CAD-Features, CAD-Parametern, Lastfall, Portdefinitionen und
ausgewerteten Rahmen fest. Aenderungen markieren es konservativ als historisch;
fehlende CAD-Koerper und ungueltige Wege werden gemeldet. Positionsaenderungen
verwenden die native Occurrence-Platzierung einschliesslich noch nicht erfasster
Bewegungen. Das spaetere Capture derselben Position darf die Physik nicht aendern.

UI: Die vorhandenen Engineering-Fenster zeigen gespeicherte Koerper-/Occurrence-
Bloecke mit Ports und Lastpfadlinien. Eingabeabschnitte werden aufgeklappt.
Die statischen Lastfaelle lassen sich ueber dieselben Engine-Kommandos berechnen
wie in MCP. Die CAD-Hauptoberflaeche behaelt ihre bisherige Form.

Versuchsaufbau tools/verify_mechanical_project.py: native Fundamentgeometrie,
Komponente mit parametrischer Laenge und wirklicher Occurrence-Position; fuenf
Ports, vier Lastfaelle. Unabhaengige SI-Referenzen fuer einen 1-m-Hebel mit 100 N,
200 mm2 mit 2 MPa, Verzweigungssuperposition, 90-Grad-Baugruppenrotation,
zusaetzliche lokale Portrotation und lokale Fundamentkomponenten. Acht numerische
Referenzpruefungen sowie Aenderungen von Laenge und vorlaeufiger Position.
Zwoelf abgewiesene ungueltige Kommandos muessen die gespeicherte Struktur unveraendert
lassen. Weitere Pruefungen: punktfoermiger Uebergang mit unzulaessigem Moment,
unterbrochener Lastpfad, UI-Berechnung, Wiederladen und geloeschter CAD-Koerper.

Fehlversuche: Ein Test suchte das zu loeschende Feature anhand angenommener
Anzeigenamen; er verwendet nun die tatsaechlich vom Konstruktor gelieferte ID.
Clippy forderte die uebliche +=-Schreibweise fuer Vektorsummen; dies aendert die
Mathematik nicht. Eine vom Nutzer geoeffnete EXE blockierte den Build. Ihre Datei
wurde im Buildordner umbenannt; die laufenden Nutzerinstanzen wurden nicht beendet.

Grenzen fuer die Dissertation: Keine automatische Starrgruppenbildung, keine
Topologiestabilitaet ausgewaehlter Flaechen, keine FE-Knotenlastverteilung und kein
Nachweis realer Steifigkeit. Ports eines Root-Koerpers sind im Root-CAD-System
definiert; ein Geometrie-Move allein bewegt diesen Rahmen nicht. Die konservative
Snapshotpruefung kann auch bei fachlich irrelevanten Dokumentaenderungen anschlagen.
Gleichgewichtsresiduen validieren die Resultantentransformation, nicht saemtliche
physikalischen Modellannahmen. Eine Zeitersparnis bleibt experimentell zu pruefen.

Rohdaten und UI-Aufnahmen: output/mechanical-project/. Das dortige Startprojekt
wird vor dem negativen CAD-Loeschtest gespeichert. Alte breite Testlaeufe bleiben
pausiert; ausgefuehrt werden nur die gezielten Pruefungen dieses Funktionsschritts.

Abschluss dieses Ausbaus: Finaler Desktop-Build und Clippy Engine/UI mit
-D warnings bestanden. Die mechanische Projektpruefung bestand auf dem finalen
Build mit acht numerischen Referenzen, zwoelf atomaren Ablehnungen, UI-Berechnung,
Wiederladen und Erkennung geloeschter CAD-Geometrie. Die bereits vorhandene
Calcpad-/Variantenpruefung wurde nach dem Schnittstellen-Ausbau ebenfalls erfolgreich
ausgefuehrt; der letzte, danach hinzugefuegte enge Flaechenvertragscheck betrifft
nur mechanics.port. Aufnahmen und Rohdaten sind lokal abgelegt, Quell-/Artefakthashes
stehen in output/mechanical-project/build-verification.json.


## Eintrag R-2026-10-22: Native Baugruppe in starre MKS-Bloecke reduzieren

Fragestellung: Kann derselbe CAD-Baugruppenstand ohne manuelle Zweitmodellierung
in eine nachvollziehbare mechanische Blockstruktur ueberfuehrt werden?

Methode: Alle Koerper eines exakten Komponenten-Einbaupfads bilden zunaechst
einen Block. Union-Find vereinigt diese anhand aktiver starrer Gelenke sowie
eindeutiger Erdung am Dokument-Root. Bewegliche Gelenke bleiben Graphkanten.
Projektports werden ueber Quellkoerper und vollstaendigen Einbaupfad zugeordnet;
ihre Bezugssysteme werden aus Parameterausdruecken und aktueller Pose aufgeloest.
Unterkomponenten gelten allein durch die Baumhierarchie nicht als starr verbunden.

Alternativen: Jeden Koerper als eigenen MKS-Koerper exportieren wuerde starre
Verbaende unnoetig aufteilen. Die gesamte Unterbaugruppe pauschal zu vereinigen
wuerde bewegliche Gelenke verlieren. Der gewaehlte Ansatz folgt expliziten
nativen Beziehungen. Wiederholte Instanzen werden getrennt expandiert; native
Gelenkreferenzen ohne eindeutigen Pfad werden diagnostiziert und ausgelassen.

Versuchsaufbau: Fundament mit geerdeter Halterung; Arm am Drehgelenk; starr
angebundenes Werkzeug; Schieber; separates freies Teil. Erwartet werden vier
reduzierte Bloecke und zwei bewegliche Gelenke. Bei Unterdrueckung der starren
Werkzeuganbindung muessen fuenf Bloecke entstehen. Zwei Ports des Armes und
Werkzeugs muessen demselben reduzierten Block angehoeren. Bei 45 Grad Rotation
muss die Portbasis analytisch sin/cos(45 Grad) folgen. Parameteraenderung von
L=50 auf 100 mm muss sowohl das Armvolumen auf 40000 mm^3 aendern als auch den
Port um 50 mm entlang seiner gedrehten lokalen X-Achse verschieben.

Weitere Negativ-/Erhaltungsfaelle: doppelte Unterbaugruppe mit mehrdeutigem
innerem Gelenk, Graphschleife, pending Pose, unveraendertes Dokument durch
Extraktion, identische Ableitung nach Speichern/Laden. Bestehende Planetenstufe
wird mit ihren nativen Bewegungskopplungen gelesen; diese werden ausdruecklich
als noch nicht fuer einen Dynamikadapter uebersetzt ausgewiesen.

Implementierung: engine/cmd/mks.rs und ui-egui/mks_ui.rs; oeffentlicher Befehl
mks.model; Austauschschema solidcraft.rigid-blocks/1. Kein neuer persistenter
Dokumenttyp: Das Zwischenmodell wird aus gespeicherter CAD-/Portinformation
rekonstruiert. Block-IDs sind darstellungsintern; persistente Bezuege bleiben
Quellkoerper plus Einbaupfad. Nur die jeweils aktuelle Ableitung wird angezeigt.

Abgrenzung: Dies ist keine neue allgemeine MKS-Methode und kein Nachweis eines
Dissertationsbeitrags. Weder Trägheitstensoren noch Zeitintegration, dynamische
Reaktionen oder eine globale Beweglichkeits-/Konsistenzanalyse sind implementiert.
Geschlossene Graphen allein beweisen keine physikalische Beweglichkeit.
Forschungsbeitrag und Zeitersparnis muessen spaeter ueber reproduzierbare
Aenderungsszenarien, Fehlerklassen und Vergleich mit manueller Uebertragung
experimentell gezeigt werden.

Abbildungen/Datengrundlage: output/native-mks/native-baugruppen-bloecke.png
zeigt die echte Anwendung. verification.json enthaelt strukturierte Vorher-/
Nachhermodelle fuer spaetere Graphen, Gruppengroessen und Aenderungsdiagramme.
Die Erstpruefung bestand. Clippy fand danach einen unnoetigen Borrow; korrigiert.
Die erste UI-Aufnahme zeigte das Fenster teilweise am Bildschirmrand; die
Fensterbegrenzung wurde angepasst. Keine alten Langzeittestlaeufe gestartet.

Abschlusspruefung: Finaler Desktop-Build und Clippy Engine/UI mit -D warnings
bestanden. Native MKS-Pruefung auf dem finalen Executable mit 13 Pruefbereichen
und zusaetzlichem Lesen der Planetenstufe bestanden. Die statische
Schnittstellenpruefung bestand mit acht numerischen Referenzen, zwoelf
atomaren Ablehnungen, UI und Wiederladen; danach wurde nur die Spaltenzahl
der neuen MKS-Zeichnung an die Fensterbreite angepasst. Die abschliessende
Aufnahme zeigt alle vier Bloecke und die Kraftports innerhalb der Ansicht.
Quell-, Artefakt- und Testbericht-Hashes: output/native-mks/build-verification.json.


## Eintrag R-2026-10-22: Materialmasse und Traegheit der reduzierten Bloecke

Ziel: Die aus CAD abgeleiteten starren Verbaende sollen numerisch brauchbare
Masseneigenschaften erhalten, ohne eine zweite manuelle Massendefinition.
Die strukturelle Ableitung bleibt schnell; mks.mass_properties ist eine
separate, lesende Auswertung. Es wird keine Dynamikloesung behauptet.

Grundlage und Verfahrenswahl: Wiederverwendung der vorhandenen, separat
geprueften Volumenmomentintegration Mesh::mass_properties im Geometriekern.
Die im vorhandenen URDF-Export verwendeten physikalischen Konventionen
(Materialdichte, Rotation, Steiner) dienen als interne Architekturgrundlage.
Keine externe Quellcodeuebernahme und keine neue Laufzeitabhaengigkeit.

Verfahren: Zwei Tessellierungen je Quellkoerper mit relativer Sehnenabweichung
1e-4 und 5e-5 der Koerpergroesse, absolut mindestens 1e-5 mm beim feinen Netz.
Netz um den Boundingbox-Mittelpunkt zentrieren, Randpunkte verschweissen,
Geschlossenheit und konsistente Orientierung pruefen. Volumenmomente ueber
orientierte Tetraeder integrieren. Bei Aenderungen von Volumen oder groesstem
normalisiertem Tensorfehler ueber 0.5 Prozent wird der Block nicht freigegeben.
Dies ist eine empirische Konvergenzpruefung, keine zertifizierte Fehlerschranke.

Transformation: I_world = R I_local R^T; dichtegewichteter Schwerpunkt und
Steinerscher Satz fuer den gesamten Block. Dichte g/cm^3 -> kg/mm^3 mit 1e-6;
Laenge mm -> m mit 1e-3; Traegheit kg*mm^2 -> kg*m^2 mit 1e-6. Tensorachsen
sind Weltachsen; Bezug ist der jeweilige gemeinsame Blockschwerpunkt.
Koerperversatz, exakte Einbaupfade und pending Positionen werden uebernommen.
Mehrfach verbaute Komponenten teilen die lokale Integration, nicht ihre
Weltposition oder ihren Anteil an der Gesamtmasse.

Referenzen: Quader 100x20x10 mm mit rho=7800 kg/m^3; 45-Grad-Drehung mit
gemischtem Tensoreintrag; zweite Komponente 40x30x20 mm mit rho=2700 kg/m^3
und versetztem Einbau; analytisch kombinierter Schwerpunkt und Steineranteil.
Zusaetzlich wiederholte Instanz, Dichtewechsel auf 1000 kg/m^3, Parameterwechsel
L=100 -> 200 mm, Vollzylinder r=10/h=30 mm, Speichern/Laden und UI-Pruefung.
Ein offenes Dreiecksnetz muss als unvollstaendig erkannt werden. Leere
Komponenten erhalten keinen kuenstlichen Traegheitstensor.

Fehlversuch: Die erste Zylinderpruefung forderte fuer analytisch verschwindende
gemischte Eintraege eine absolute Toleranz von 1e-12 kg*m^2. Die numerische
Tessellierung lieferte -1.4241406884188113e-11 kg*m^2. Die Referenzpruefung
verwendet nun eine am groessten analytischen Tensoreintrag skalierte absolute
Toleranz; beim Zylinder 0.2 Prozent, beim Quader weiterhin 1e-8 relativ.
Der Produktionsalgorithmus wurde deshalb nicht veraendert. Rohwerte bleiben
im JSON erhalten. Das ist keine Behauptung exakt verschwindender Netzfehler.

Gueltigkeit: UI-Ergebnisse gelten fuer den ausgewerteten Dokumentstand.
CAD-, Material- oder Positionsaenderungen markieren sie als veraltet.
Fehlende/ungueltige Materialzuordnung verwendet sichtbar die vorhandene
CAD-Standarddichte. Ueberlappende Koerper werden addiert, nicht vereinigt.
Teilweise integrierbare Bloecke erhalten keine freigegebene Gesamtmasse.
Ergebnisse bleiben aktuell sitzungsbezogen; persistente Solverherkunft und
ein Exudyn-Adapter folgen gesondert.

Daten fuer Abbildungen: output/native-mks-mass/verification.json enthaelt
analytische Erwartungen, Toleranzen und vollstaendige Auswertungen je Fall;
progress.json wird vor jeder Referenzpruefung aktualisiert. Die echte
UI-Aufnahme masseneigenschaften.png zeigt Masse, Schwerpunkt und Tensor.
Moegliche Auswertung: Masse und Schwerpunkt vor/nach Parameterwechsel,
relative Tensorabweichungen sowie Zahl unvollstaendiger Bloecke.

Dissertationsbezug: Integration bekannter Starrkoerperphysik allein ist kein
neuer wissenschaftlicher Beitrag. Sie schafft die gepruefte Datengrundlage
fuer den spaeteren methodischen Vergleich der automatischen Modellaktualisierung.
Naechste Abnahme: eindeutig definierte offene Gelenkkette nach Exudyn mit
vollstaendigen Massen, gefolgt von Pendelperiode und Energievergleich.

Zusaetzliche Gueltigkeitsbedingung: Massenauswertung verweigern, wenn der
native CAD-Evaluator Parameter-/Featurefehler oder stale Geometrie meldet.
Gezielter Negativfall: Quaderlaenge L=0 mm. Ein atomarer CAD-Fehler
oder die ausdrueckliche Ablehnung der Massenauswertung ist erforderlich;
ein Ergebnis aus alter Geometrie darf nicht als aktueller Stand erscheinen.

Korrektur des zusaetzlichen Negativtests: L=-1 mm war keine verlaessliche
ungueltige CAD-Eingabe. Der native Quader kann eine negative Ausdehnung
als umgekehrte Richtung behandeln; die Massenauswertung durfte diese
Gueltigkeit nicht allein wegen des Vorzeichens verwerfen. Der Test verwendet
daher L=0 mm als degenerierten Volumenkoerper. Dies war eine falsche
Testannahme, kein nachgewiesenes Versagen der Fehler-/Stale-Sperre.

Abschluss: Finaler Desktop-Build, Clippy Engine/UI mit -D warnings und die
gezielte Massenauswertung auf dem finalen Executable bestanden. Sieben
analytische Referenzauswertungen plus Instanz-, UI-, Gueltigkeits- und
Negativpruefungen sind gespeichert. Die strukturelle MKS-Regression bestand
vor der letzten, nur die Massenauswertung betreffenden CAD-Fehlersperre und
der Zahlenformatierung. Keine breite alte Testsuite ausgefuehrt.
Quell-/Artefakthashes und Pruefliste: output/native-mks-mass/build-verification.json.


## Eintrag R-2026-10-22: Erster CAD-abgeleiteter Exudyn-Dynamikadapter

Forschungsfrage: Kann die aus nativen Baugruppenbeziehungen abgeleitete
Blockstruktur mit ihren CAD-Masseneigenschaften ohne manuelle Zweitmodellierung
in eine reale dynamische Referenzrechnung uebergeben und nachvollziehbar
zurueckgefuehrt werden?

Verfahrensentscheidung: Offene, geerdete Starrkoerperbaeume mit Dreh- und
Schubgelenken. Exudyn CreateRigidBody und CreateGenericJoint ueber deren
oeffentliche Python-API in einem optionalen lokalen Prozess. Keine
Solverquelltexte uebernommen, keine Einbettung in den Rust-Prozess, kein
Berechnungsserver. Python bleibt die vorhandene externe Forschungsumgebung.

Datenvertrag: solidcraft.assembly-job/1 enthaelt die aktuelle CAD-Pose,
materialabhaengige Massen und Schwerpunkte in SI, Tensoren in anfangs
weltparallelen Koerperachsen, Gelenkbezugssysteme und Portlasten.
Alle Anfangsgeschwindigkeiten sind null. Die initiale native Gelenkstellung
ist Teil der Referenzpose; sie wird nicht irrtuemlich nochmals aufgebracht.
Die Referenzachsen der Exudyn-Koerper sind initial weltparallel, die CAD-
Koerperlagen bleiben in ihren Einbau- und Versatztransformationen erhalten.

Gelenkuebergabe: Alle Translationen sowie zwei Rotationen fuer ein Drehgelenk
sperren; bei einem Schubgelenk Translation entlang seiner Z-Achse freigeben,
die anderen fuenf Koordinaten sperren. Rahmen und Anker muessen mit der
aktuellen CAD-Pose uebereinstimmen. Begrenzungen, Schleifen, Zahnrad-/
Bewegungskopplungen, Kontakt und mehrdeutige Instanzen werden abgelehnt.
Die Auswertung behauptet keine allgemeine automatische MKS-Konvertierung.

Lasten: Konstante Kraft/Moment im mitbewegten Projektport; Punktkraftports
nehmen keine Momente an. Druck wird am deklarierten Flaechenport als
Resultierende p*A entlang der Portnormalen uebergeben. Das ist keine
FEM-Flaechenlastverteilung. Bewegte Geometrie wird nicht waehrend der
Zeitintegration neu konstruiert.

Persistenz: Optionaler Job und kompaktes Ergebnis unter eigenem Metadata-Key.
Ein Eingabesnapshot ohne diesen eigenen Key verhindert Selbstinvalidierung.
Geometriefingerprints erkennen zusaetzlich neu ausgewertete externe Geometrie.
Die Uebernahme verlangt den exakt vorbereiteten Job und unveraenderte
aktuelle CAD-/Material-/Parameter-/Positionseingaben. Alte Ergebnisse bleiben
als historisch sichtbar. Rohauftrag, Ergebnis und stderr werden unter
output/mks-runs je lokalem Lauf archiviert, auch bei Solverfehlern.

Erste Versuche: Physikalisches CAD-Pendel (Stab 1000x20x20 mm, Stahl,
Anfangsauslenkung 3 Grad) mit 500 bzw. 1000 Schritten. Unabhaengige
nichtlineare Referenz: theta_ddot = -m*g*d/I_pivot*sin(theta), integriert
mit SciPy solve_ivp, rtol=1e-11, atol=1e-13. Die Pendelperiode aus der
Kleinwinkelformel dient nur als Zusatzkenngroesse, nicht als exakte
Referenz fuer endliche Auslenkungen. Energie und Gelenkresiduen werden
gesondert protokolliert.

Fehlversuch 1: SensorObject Force wird von GenericJoint nicht angeboten.
Die installierte API und offizielle Dokumentation verlangen ForceLocal.
Auf ForceLocal korrigiert; Ausgabe ist im lokalen Gelenk-0-Rahmen.
Fehlversuch 2: Speichern eines unbenannten Projekts setzt den Titel auf
den Dateinamen. Ein vollstaendiger Dokumentvergleich machte dadurch das
Ergebnis historisch. Isolierter Differenzversuch ergab nur /document/name.
Der nichtphysikalische Titel wird nun aus dem Snapshot ausgeschlossen.
Die uebrigen Eingaben und Geometriefingerprints bleiben erhalten.

Weitere Abnahmen: gekoppelte Zweikoerperkette mit Zeitschrittverfeinerung,
analytischer Schieber mit Gravitation und Portkraft/Druck, verfaelschte
Zeitgitter/Rotationen, Materialaenderung, Wiederladen, UI-Ausloesung und
lesende Replay-Transformationen fuer CAD-Koerper und Projektports.
Rohdaten: output/assembly-dynamics/. Skript:
tools/verify_assembly_dynamics.py mit der vorhandenen simulation/.venv.

Primärreferenzen: Exudyn MainSystem CreateRigidBody/CreateGenericJoint,
https://exudyn.readthedocs.io/en/latest/docs/generated/cInterface/MainSystem.html
und GenericJoint-Ausgaben/Constraints,
https://exudyn.readthedocs.io/en/latest/docs/generated/items/ObjectJointGeneric.html
API am 22.10.2026 gegen die lokal installierte Version 1.13.6 geprueft.

Fortsetzung: Der gekoppelte Fall zeigte zwei weitere Herkunftsprobleme.
Nach Entfernen des eigenen Simulationskeys blieb ein zuvor nicht vorhandenes
leeres Metadata-Objekt uebrig. Dieses wird nun normalisiert. Ausserdem
unterschieden sich Geometrie-Cachekeys nach Wiederladen, obwohl die native
Definition unveraendert war. Solche Laufzeitidentitaeten wurden aus dem
persistenten Vergleich entfernt. Die vollstaendige native Dokumentquelle
enthaelt bereits eingebetteten STEP-Text und Netzdaten; sie ist hier die
reproduzierbare Vergleichsbasis. Der erste Entwurf mit Cachekeys wurde damit
verworfen. Isolierte Differenzdaten: output/assembly-dynamics-source/.

Abnahmestand: Nichtlineares physikalisches Pendel mit SciPy DOP853 als
unabhaengiger ODE-Referenz. Maximaler Winkelunterschied 500 Schritte:
7.763232141926382e-6 rad; 1000 Schritte: 1.9363214189271893e-6 rad.
Das Verhaeltnis von etwa 4 entspricht der beobachteten Zeitschrittverfeinerung
zweiter Ordnung fuer diesen Fall. Relative Energieaenderung ohne Portlasten
oder Antrieb: 8.324151767724492e-11. Kleinwinkelperiode: 1.6380284811952406 s.
Diese Werte sind Beobachtungen dieses Referenzmodells, keine allgemeine
Fehlergarantie fuer Exudyn oder beliebige Baugruppen.

Berechnungsvarianten: Basis L=1000 mm, Lang L=1200 mm im nativen
Berechnungswerkzeug gespeichert. Auswahl Lang aktualisiert CAD, Masse und
parametrischen Spitzenport; das vorige Dynamikergebnis wird historisch.
Neuberechnung uebernimmt Massefaktor 1.2 und Schwerpunktabstandsfaktor 1.2;
Kleinwinkelperiode der langen Variante 1.794342886514208 s. Dies ist ein
Nachweis der Aenderungskette, kein Optimierungs- oder Zeitersparnisnachweis.

Gekoppelter Fall: Zwei massive Stabkoerper 1000/800 mm, Drehgelenke an
Fundament und gemeinsamer Spitze. Dauer 0.5 s. Endlagenunterschied gegen
800 Schritte bei 200 Schritten 7.145768764106878e-7 m und bei 400 Schritten
1.4292007965410842e-7 m. Energie und Gelenkresiduen getrennt geprueft.
Eine rein innere Schrittverfeinerung ersetzt keinen unabhaengigen
Mehrkoerperreferenzsolver; dieser Teil ist ein Konvergenz-/Konsistenznachweis.

Weitere Erweiterung: Ideale Antriebe mit einer halben Kosinusrampe, danach
Halten. Winkel-/Laengeninkrement wird als nativer Parameterausdruck
aufgeloest; Rampenzeit mindestens zehn Integrationsschritte. Umsetzung
ueber den oeffentlichen GenericJoint-Offsetcallback; keine Benutzerskripte.
Geprueft wurden 50 mm Schub und ein Drehinkrement aus driveAngle=10 deg.
Die maximale Winkelabweichung des geprueften Sollverlaufs betraegt
6.661338147750939e-16 rad. Das ist ein kinematischer Constraintnachweis,
kein Nachweis eines realen Motors oder Reglers. Die Antriebsarbeit ist
nicht aus der gespeicherten Energie subtrahiert. ForceLocal ist gespeichert;
eine allgemeine Motormomentenauswertung wird nicht behauptet.

UI-Abnahme: Lokalen Lauf ausloesen, Ergebnisgueltigkeit nach Materialwechsel
und Antriebscheckbox mit neuer Berechnung direkt im MKS-Fenster geprueft.
Zeitregler und Wiedergabe zeigen Schwerpunktbahnen, keine animierten
3D-CAD-Koerper. mks.result.frame liefert dafuer lesend die berechneten
Welttransformationen von nativen Koerpern, Einbaupfaden und Projektports.
Die Projektkonstruktion wird weder beim Rechnen noch beim Replay geaendert.

Daten fuer Abbildungen und NotebookLM: tools/export_dynamics_research_data.py
exportiert pendel_zeitverlauf.csv mit Referenz, beiden Zeitschrittweiten,
Fehler und Energie; versuchskennwerte.csv sowie CSV-Trajektorien fuer
alle gespeicherten Laeufe. Anleitung DATEN_FUER_ABBILDUNGEN.md im Ergebnisordner.
Es wurde kein neues Plotpaket als Laufzeitabhaengigkeit installiert.
Die Daten sind fuer Standard-Plotwerkzeuge vorbereitet.

Offen: echte 3D-Ergebnisdarstellung; native Zahnrad-/Kopplungsbedingungen;
Kontakt und geschlossene Mechanismen; Motor-/Reglermodelle und SIL;
variantenweise Ergebnisdatenbank; dynamische Lastuebergabe an FEM/ROM
mit den jeweils erforderlichen Kraeften, Momenten und Traegheitslasten.
Die vorhandene Resultierende eines Druckports ist keine FEM-Kopplung.

Umfangserweiterung nach bestandenen Zwei-Koerperreferenzen: Grenze auf acht
bewegliche Bloecke erweitert. Die gemeinsame Grenze von 100 Koerper-/
Ausgabeintervallen bleibt bestehen, um die persistente Metadatenablage
zu begrenzen; die UI waehlt bei mehr als fuenf beweglichen Bloecken
standardmaessig elf Ausgabepunkte. Ein zusaetzlicher Achtgelenkfall mit
5 Grad Anfangsauslenkung, acht 100-mm-Staeben, 200 Schritten auf 0.1 s
prueft erfolgreiche Kopplung, Energie, Gelenkresiduen und Wiederladen.
Die harte Modellgrenze ist eine Prototypentscheidung, keine Exudyn-Grenze.

Abschluss des Ausbaus: Finaler Desktop-Build und Clippy Engine/UI mit
-D warnings bestanden. Dynamikpruefung mit 12 gespeicherten Solverfaellen,
zusaetzlicher UI-Berechnung, 11 gezielten Ablehnungen und Wiederladen
bestanden. Masseneigenschaftspruefung ebenfalls auf dem finalen Executable
erfolgreich. Die breite alte Testsuite blieb pausiert. Gespeicherte
Einzellast-/Antriebseinstellungen werden beim Oeffnen im Dialog uebernommen;
komplexe Mehrlast-/Mehrfachantriebs- oder Druckauftraege werden vom einfachen
UI-Editor nicht stillschweigend ueberschrieben.
Quell-/Artefakthashes, Paketversionen und Referenzkennwerte:
output/assembly-dynamics/build-verification.json.

Letzte reine Darstellungsanpassung: Beschriftung am rechten Rand der
Schwerpunktbahn nach innen setzen. Danach UI-Kurzpruefung auf finalem
Executable mit geladenem Projekt, erhaltener Antriebsreferenz driveAngle
und aktueller Trajektorie bestanden; Clippy und Formatpruefung bestanden.
Solver und Worker blieben gegenueber den bestandenen numerischen Pruefungen
unveraendert. Abschliessende Hashes wurden entsprechend aktualisiert.


Fortsetzung desselben Ausbaus: Die zuvor noch offene 3D-Ergebnisdarstellung
ist nun als separates, temporaeres SolidCraft-Fenster umgesetzt. Es verwendet
die nativen CAD-Darstellungsnetze mit den aus dem Ergebnis abgeleiteten
Koerpertransformationen und zeigt die mitbewegten Portachsen. Keine neuen
Konstruktionsfeatures, keine Aenderung der Gelenkdefinitionen oder CAD-Positionen.
Die bestehende Navigation wird ueber navigation::handle/NavCaps::RENDER benutzt.
Vor/zurueck, Zeitregler und tempoeinstellbare Wiedergabe folgen der gespeicherten
physikalischen Zeitachse und stoppen am Ende. Es werden Ausgabezeitpunkte
wiedergegeben, keine zusaetzlichen Loesungen zwischen ihnen interpoliert.

Abnahme: gespeichertes Projekt oeffnen, 3D-Fenster einschalten, Ausgabepunkt
0 und 10 anzeigen, bis zum Endpunkt 20 abspielen. document.inspect bleibt
identisch und das Ergebnis aktuell. Bilder cad-ergebnis-3d-start.png und
cad-ergebnis-3d-bewegt.png belegen die echte Anwendung. Die Szene wird
begrenzt auf 100000 Dreiecke; eine abgelehnte Darstellung verwirft keine
numerischen Ergebnisse. Die Kamera ist eine Ansichtseinstellung, keine
mechanische Randbedingung.

Weitere Verallgemeinerung: Der Szenenaufbau wurde in einen gemeinsamen
Engine-Helfer verschoben. GUI und MCP-PNG/GIF-Export verwenden dieselben
Geometrie-/Porttransformationen. Die GUI liefert ihre Theme-Palette, der
Headless-Export verwendet die vorhandene native Standardpalette.
mks.result.snapshot exportiert einen Ausgabezeitpunkt als PNG;
mks.result.animation exportiert alle gespeicherten Zeitpunkte mit einer
festen Kamera ueber den gesamten Bewegungsraum. Konstruktion bleibt unveraendert.

Exportabnahme: PNG-IDAT-Bilddaten erneut dekomprimiert und unterschiedliche
Koerperlagen nachgewiesen. GIF89a-Struktur erneut gelesen: 21 Bilder,
240x160 Pixel, 0.01 s angegebene Bildverzoegerung. Bestehende Dateien ohne
Overwrite bleiben erhalten; zu grosse Exporte und historische Ergebnisse
werden abgelehnt. Ausgabe output/assembly-dynamics-export/verification.json.
GIF-Zeit ist auf Hundertstelsekunden quantisiert und enthaelt die Haltezeit
des letzten Bildes; keine Behauptung beliebiger exakter Echtzeitwiedergabe.

## 23.10.2026 – Roboterarm über Parameter, CAD, MKS und Balken-FEM

Fragestellung: Kann dieselbe native Baugruppe mit expliziten Berechnungsreferenzen
und Kraftschnittstellen für Variantenvergleich, starre Bewegung und eine erste
Strukturidealisierung verwendet werden, ohne Ergebnisse nach Änderungen als
weiterhin gültig auszugeben?

Implementierung: robot.prepare erweitert eine Arbeitskopie der vorhandenen
realistischen Baugruppe atomar um eine lokale Calcpad-Datei, zwölf exportierte
Referenzen, zwei Radiusvarianten, angenommene Materialdichten und sechs
persistente Ports. Radien verwenden reguläre native CAD-Bemaßungen.
Die alternativ mitmodellierten Schweiß- und Nutzlastkörper samt ihren
BodyTransfer-Features wurden in dieser Arbeitskopie entfernt. Ein erster
Versuch löschte nur die ursprünglichen Körperfeatures und ließ ungültige
Transfers zurück; der atomare Commit verweigerte das Modell. Die explizite
Entfernung der betroffenen Transfers behebt dies. Ausgangsdatei unverändert.

Methode MKS: sechs vorgeschriebene ideale Kosinusrampen, 0,5 s, 400 Schritte,
11 Ausgabepunkte, sechs bewegte starre Blöcke, native CAD-Masseneigenschaften,
Schwerkraft und TCP-Portkraft. Eine Gelenkgrenze wird nur zugelassen, wenn
die vorgeschriebene monotone Rampe vollständig darin bleibt. Freie Bewegung
mit Anschlaggesetz ist weiterhin nicht implementiert. Endpunktprüfung gegen
sechs native joint.drive-Aufrufe: TCP-Abweichung 5,584390806e-10 mm;
TCP-Weg 174,3627464 mm. Dies validiert Kinematik und Adapterabbildung,
nicht gemessene Motor-, Reibungs- oder Regelungsdynamik.

Methode FEM: assemblierte kubische Hermite-Biegeelemente für zwei aus nativen
Zylinderdefinitionen idealisierte Armglieder; root fixiert, andere Module
und Aktuatoren starr. Längen, Radien, Weltachsen und TCP-Lage aus dem aktuellen
CAD. Axial- und Saint-Venant-Torsionsnachgiebigkeit des Kreisvollquerschnitts
analytisch ergänzt. Armbeiträge über starre Hebel zum TCP superponiert.
Materialannahme E=70000 MPa, nu=0,33; externe Portkraft 100 N. Eigengewicht
ist in dieser FEM nicht enthalten. Es handelt sich nicht um dynamische
Spannungsrückrechnung aus MKS-Reaktionslasten.

Gezielte Abnahme tools/verify_robot_domains.py bestanden: zwei Varianten,
100/200 N, FE-Lösungen gegen unabhängige geschlossene Kraft-/Momentlösungen,
1/16 Elemente, statischer Lastpfad mit Grundreaktion +100 N und -119,1 Nm,
MKS gegen native CAD-Kinematik, Parameterreferenzen, Speichern/Laden,
Ergebnisaktualität, geänderte Gelenkstellung, drei ungültige Eingaben sowie
native FEM-/MKS-Fenster. Bei konstantem EI und reiner Endbelastung ist die
exakte Biegelinie kubisch. Elementzahlgleichheit ist hier eine Konsistenzprüfung,
kein allgemeiner Konvergenznachweis für beliebige FEM.

Ergebnisse: Basisradien 77/65 mm; Leichtbau 65/55 mm. Bewegte CAD-Masse
149,3272172 kg bzw. 134,8759976 kg, bei additiven illustrativen Vollkörpern
und angenommenen Dichten. TCP-Verschiebung 0,05790060523 mm bzw.
0,1136190839 mm; Verhältnis 1,962312543. Kraftverdopplung verdoppelt die
lineare Verschiebung. Keine Herstellerdaten oder Gewichtsoptimierung behauptet.
Beide Arme sind für Euler-Bernoulli eher kurz/dick; Schubnachgiebigkeit und
realistische Gehäusequerschnitte müssen vor Auslegungsentscheidungen folgen.

Zusätzlicher gefundener Fehler: kleine versetzte Schraubenzylinder wurden
bei der Massenauswertung als offene Netze abgewiesen. Die bisherige
Verschweißdistanz size*1e-9 war für ihre tessellierten Nahtpunkte zu klein.
Jetzt begrenzte Wiederholungen bis zur bereits beim nativen Export verwendeten
Obergrenze size*1e-6. Geschlossene, entgegengesetzt orientierte Kanten und
Zwei-Auflösungs-Abgleich bleiben vorgeschrieben; kein pauschales Einsetzen
von Ersatzmassen. Tatsächlich verwendete Distanz steht im Bericht.
Die gezielte Massenabnahme mit acht analytischen Referenzen einschließlich
einer 4-mm-Schraube bei [1028,-36,969] mm und Ablehnung offener Netze besteht.

Artefakte: output/robot-domains/verification.json, zwei native CAD-Dateien,
Roboter-FEM.png, Roboter-MKS.png, Roboter-Bewegung.png. Drei CSV-Dateien,
VERSUCHSBERICHT.md und provenance.json werden durch
tools/export_robot_research_data.py erzeugt. Für NotebookLM sind Bericht,
ROBOTER_DOMAENEN.md und die Primärquellen zusammen einzulesen. Rohläufe
liegen zusätzlich unter output/mks-runs. Die alten großen Testläufe bleiben
pausiert; diese Prüfungen sind begrenzte neue Referenzrechnungen.

Offen: Volumen-/Timoshenko-FEM, experimentelle Identifikation, FEM-Eigengewicht,
Gelenk-/Getriebenachgiebigkeit, dynamische Lastzustandsübergabe, CMS/FFRF,
flexible Rückwirkung, allgemeines Werkzeugvariantenmanagement, SIL und
reale Geräteansteuerung. Produktintegration und numerische Teilmodellprüfung
allein begründen noch keine wissenschaftliche Neuheit; eine Dissertation
muss Methode und Zeit-/Fehlergewinn gegen definierte Vergleichsabläufe testen.

Abschlussprüfungen dieser Erweiterung: nativer Desktop-Build, Clippy für Engine
und UI mit -D warnings sowie WASM-Engine-Check bestanden. Separater Exporttest
tools/verify_robot_exports.py besteht: Original-Körperfarben aus dem nativen
Dokument statt nur Kernel-Farben übernommen; PNG mit allen Filtertypen erneut
dekodiert (8215 orange Pixel), explizite Kamera reproduziert identische Bytes,
GIF mit elf Ausgabepunkten und 0,05 s Bildverzögerung erzeugt, CAD und beide
gespeicherten Ergebnisdomänen unverändert aktuell. Die erste Prüfroutine nahm
ungefilterte PNG-Zeilen an; vor Akzeptanz wurde sie um die standardmäßigen
PNG-Filter einschließlich Paeth ergänzt. Kein Solverfehler. Numerische
Roboterabnahme blieb unverändert; nur die Ergebnisdarstellung wurde korrigiert.

## 28.10.2026 – Upstream-Integration und einheitlicher Programmeinstieg

SolidCraft-Basis von 0.16.2 (`d8eb65d2`) auf Upstream 0.25.1 (`ae07e36a`)
aktualisiert: 965 zusätzliche Commits, 2.027 geänderte Dateien. Vorab zwei
geprüfte ZIP-/SHA-256-Sicherungen einschließlich lokalem Index-/Arbeitsdiff
und Modellen erstellt. Zusammenführung zunächst in einer isolierten lokalen
Kopie mit vollständigem Snapshot der Eigenentwicklung, 15 Textkonflikte.
Kein Überschreiben durch einen neuen Download. Angepasste Übergänge:
designbezogener Metadata-Owner, gecachter Gelenk-Parameterkontext,
SceneLine-Alpha, neue Dialog-Deskriptoren und die gemeinsame UI-Kit-Oberfläche.
Technische Formate, Crate-/MCP-IDs und Einstellungs-/Wiederherstellungspfade
bleiben kompatibel. Sichtbare Produktbezeichnung und Katalogtexte heißen
10derCraft; SolidCraft-Herkunft und Lizenztexte bleiben erhalten.

Einzige aktive Startdatei: Start-10derCraft.cmd. Elf vorhandene Projekte werden
über einen runtime-seitig gefundenen relativen Katalog auf der nativen Startseite
angeboten. Ein realer Kartenklick lädt das Projekt und registriert es in
Zuletzt verwendet; es wird kein fiktiver Öffnungsverlauf eingetragen.
Die früheren Einzelstarter liegen nur noch im Archiv.

Gezielte Kompatibilitätsabnahme bestanden: alle elf Originalprojekte ohne
Featurefehler geladen und ihre Dateibytes unverändert nachgewiesen; Änderung
des Getriebemoduls aus der lokalen Berechnung treibt die native Konstruktion;
Variante 2 und Wiederladen funktionieren. Eine zuerst gewählte Verringerung auf
3 mm wurde wegen S_Zahn <1 abgewiesen. Anschließend zulässige Vergrößerung auf
1,2 des Ausgangsmoduls geprüft. Die Ablehnung ist erhaltene Auslegungslogik,
kein zu umgehender Import- oder Regressionsfehler.

Roboterabnahme auf neuer Basis: Basis-FEM 0,05790060522510633 mm,
Leichtbau 0,1136190838913221 mm bei 100 N und derselben Ausgangsstellung.
Diese Werte stimmen bis zu numerischer Rundung mit der vorherigen
Implementierung überein. Sechsachsiger TCP-Weg 174,36274640511874 mm;
Abweichung zur nativen Gelenkkinematik 5,583313334e-10 mm.
Kraftverdopplung, ungültige Netzeinstellungen, Achsgrenzen, Ergebnisgültigkeit
und Speichern/Laden weiterhin geprüft.

Versuchsablauf präzisiert: Vor der direkten nativen Kinematikprüfung wird ein
Engine-Checkpoint gesetzt und anschließend wiederhergestellt. Ein erneutes
Öffnen desselben Pfades kann in der neuen Mehrdokument-Oberfläche einen
bereits offenen bearbeiteten Stand wieder auswählen. Ohne definiertes
Zurücksetzen wurden zunächst unterschiedliche Gelenkstellungen verglichen
(Leichtbau 0,1102251 mm); dieser Wert ist kein Vergleich bei gleicher Pose.
Relative Poseänderung wird jetzt an J3 geprüft, statt allein an J2: Eine
gemeinsame Starrkörperdrehung bei mitgedrehter Portlast darf die
Verschiebungsnorm nicht physikalisch verändern.

Die native Upstream-Dateisperre führte beim parallelen Beispieltest korrekt
zu einem Hinweis auf eine bereits geöffnete Datei. Die Prüfroutine öffnet
dieses Modell dann schreibgeschützt, statt den Schutz abzuschalten.
Abschließende tatsächliche Screenshots aufgenommen; eine durch die neue
Trunkierung zu schmale FEM-Tabelle erhielt feste Mindestspaltenbreiten.

Build und Clippy Engine/UI bestehen; WASM-Engine-Check besteht mit einer
vorhandenen Upstream-Dead-Code-Warnung. Kein vollständiger CI-/Langzeittestlauf;
die früher pausierten großen Läufe bleiben pausiert. Daten und Belege unter
output/upstream-acceptance; Originalmodelle nicht überschrieben.
Aktivierung auf eigenem Branch 10dercraft/upstream-2026-10-28, Merge 226fc69a,
UI-Nachtrag b8e1270e. Vorheriger exakter Index-/Arbeitsstand zusätzlich im
Stash a7f8e204765d524a05948b0fb2a68241b7c5382d. Kein Remote-Push.
Dies dokumentiert Versionsintegration und Reproduzierbarkeit, keine neue
physikalische Validierung oder zusätzliche wissenschaftliche Neuheit.


## 28.10.2026: Konzept fuer geometrisch zugeordnete Systembloecke

Anforderung: echte CAD-Bauteilvorschau je Logikblock und gemeinsame geometrische
Anschluesse fuer CAD, MKS und FEM. Konzept und Bestandsabgrenzung stehen in
WORKFLOW_CAD_MKS_FEM.md. MKS-Starrkoerpergruppen und FEM-Strukturaufteilung
sind unterschiedliche Sichten derselben Baugruppe. Flaechenlasten brauchen
geometrische Referenzen und explizite Verteilungs-/Kopplungsmodelle. Die bisherigen
deklarierten Portflaechen ersetzen das nicht. Forschungsnachweise: Zuordnungsfehler,
Zeitaufwand, Erhaltung von Kraft/Moment und getrennte Netz-/Zeitkonvergenz.
Diese Ergaenzung ist ein Konzept; keine neuen UI- oder Solverfunktionen implementiert
und keine physikalischen Tests ausgefuehrt.


## 28.10.2026: Geometrische MKS-Blockbilder und sichtbare Roboterverformung

Umgesetzt: MKS als eigener grosser Arbeitsbereich mit sieben aus realen
CAD-Koerpern und exakten Einbaupositionen erzeugten Blockbildern. Bilderzeugung
im Hintergrund, Modellrevision als Gueltigkeitsbezug. FEM-Reiter oeffnet eine
eigene Werkzeugleiste ueber dem CAD; Ergebnisdarstellung integriert, kein
Fenster beim Reiterwechsel. Voruebergehende Eingabeeditoren bleiben explizit.

PrePoMax wurde anhand der oeffentlichen Produkt-/Downloadseite und des verlinkten
Handbuchs als Workflowvergleich betrachtet; keine Codeuebernahme. Quellen und
Lizenzabgrenzung: Literatur/PrePoMax_Workflow_2026-10-28.md. PDF-Download war
nicht verfuegbar, deshalb keine lokale PDF-Kopie behauptet.

Neue Ergebnisdarstellung: unveraenderte CAD-Konstruktion als Referenz; die zwei
berechneten Kreisquerschnitt-Balken erhalten axiale Darstellungsunterteilungen.
Das ist notwendig, weil ein normales CAD-Zylindernetz mit zwei Endringen die
berechnete Kruemmung nicht darstellen kann. Kubische Hermite-Interpolation der
Biegeverschiebung und ihrer Ableitung, axiale Verschiebung, Torsion und serielle
Uebertragung der vorgelagerten Balkenbewegung. Physikalische Verschiebungsbetraege
werden in 32 gemeinsamen Farbbaendern gezeigt; der Faktor x1000 aendert nur die
Geometrieanzeige. Gehaeuse, Werkzeug und weitere Module bleiben transparent als
UNVERFORMTE Referenz, ihre Elastizitaet wurde nicht berechnet.

Registrierter PNG-Befehl robot.fem.snapshot mit Ein-/Ausgabeschema, validiertem
Darstellungsfaktor, Kamera, aktuellem FEM-Quellstand und explizitem Ueberschreibflag.
Historische Ergebnisse werden abgewiesen. UI und Export verwenden denselben
Engine-Szenenaufbau. Keine neue persistente Dateiformatstruktur erforderlich.

Gezielte Abnahme tools/verify_analysis_workspaces.py erfolgreich: alle sieben
Blockbilder, FEM-Reiter ohne Fenster, integriertes Ergebnis, PNG-Vergleich Faktor
0/1000, drei ungueltige Faktorwerte abgewiesen. Vorhandene Exudyn-MKS mit sechs
Antrieben einmal berechnet. Zusaetzlich fuenf quasistatische Stellungen entlang
der halben Kosinusrampe, jeweils 16 Elemente je Armglied. TCP-Verschiebungen [mm]:
0.05790060522510633, 0.05758675846317701, 0.05691272285102443,
0.056365024170250355, 0.056174427134860656. Darstellungsfeld-Maximum am
Armglied in Ausgangsstellung: 0.03490946641181372 mm; der TCP liegt ausserhalb
des elastischen Balkens und hat wegen des starren Werkzeughebels mehr Weg.

Nachweisdateien: output/analysis-workspaces/verification.json, numerische Knoten,
PNG je Pose, GIF-Posenfolge, native UI-Screenshots und separate Projektkopie.
Originalprojekt SHA256 vor/nach unveraendert. Nach der Posenfolge werden exakt
Ausgangsstellung sowie aktuelle FEM-/MKS-Ergebnisse wiederhergestellt. Die erste
Abnahme fand einen Fehler im Testskript: edit.restore_checkpoint verbraucht den
Checkpoint; vor jeder weiteren Pose wird deshalb ein neuer angelegt.

Grenzen: quasistatische Posenfolge unter TCP-Portlast mit gesperrten Aktuatoren,
keine dynamische Spannungsauswertung, kein FEM-Eigengewicht, keine Kontakt- oder
Volumen-FEM und keine elastische Rueckkopplung. Eine CAD-Flaechenauswahl wird noch
nicht automatisch als FEM-Druckflaeche oder Lagerung uebernommen. Das sichtbare
Ergebnis ist eine nachvollziehbare erste Balkenauswertung, keine Hersteller-
oder experimentelle Validierung und kein Nachweis wissenschaftlicher Neuheit.

Native Build und gezielte Clippy-Pruefung fuer Engine/UI erfolgreich; grosse
CI-/Alttestlaeufe bleiben wie angeordnet pausiert. Bedienung und Architektur:
FEM_MKS_ARBEITSBEREICHE.md sowie WORKFLOW_CAD_MKS_FEM.md.

Abschliessend auch die ueber Start-10derCraft.cmd verwendete EXE geprueft:
alle sieben Bilder, FEM-Start ohne Fenster, Berechnung per Schaltflaeche,
integrierte Verformung und Rueckkehr zum CAD erfolgreich (ui-final.json).
Programmdatei und Quellhashes: output/analysis-workspaces/provenance.json.
Vorherige EXE als solidcraft-before-analysis-workspaces-20261028-*.exe erhalten.


## 28.10.2026: FEM-Anzeigekorrektur und MKS mit Gelenk-/Signalbloecken

FEM-Unschaerfe lokalisiert: bisher 720x440 Pixel, danach starke Hochskalierung.
Die Ansicht nutzt nun Groesse/Pixeldichte, maximal 2048x1400 bzw. rund 2 Mio.
Pixel, installierten GPU-Renderer mit CPU-Ersatz und bestehende CAD-Navigation.
Geprueft bei 2048x868: echter Rechtsmaus-Orbit und Mausrad-Zoom, Gitterumschaltung,
FEM-Ergebnis bleibt aktuell. Das Gitter ist ein Oberflaechen-DARSTELLUNGSGITTER,
kein erzeugtes Volumen-FEM-Netz. Belege output/fem-view-quality/verification.json.
Backend-Abgrenzung und Empfehlung: FEM_BACKEND_ENTSCHEIDUNG.md. Eigene Rust-
Balken-FEM als Vorabschaetzung; PrePoMax/CalculiX fuer allgemeine FEM empfohlen,
noch nicht integriert. Offizielle Dokumentation v2.6.0 zusaetzlich recherchiert.

Nach Nutzereingrenzung zuerst einfaches Pendel statt komplexer Roboterkinematik:
Ground -> eigener Revolute-Block -> Pendelarm inkl. starr angebautem Endgewicht.
Allgemeine UI auch fuer den Roboter. Bewegungsquelle mit Signalverbindung zum
Joint, editorbasierte Zuschaltung, Kraftsensor -> Scope, mehrere Gelenkkurven.
Jobausfuehrung weiterhin ausschliesslich mks.job.run; Kraftdaten sind originale
Exudyn ForceLocal-Werte in J0, N. Keine behaupteten Antriebsmomente. Wiedergabe
in der MKS-Seite, keine neue separate Simulatoroberflaeche. Konfiguration der
Antriebe wird im Job gespeichert; Sensorauswahl ist eine Ansichtseinstellung.

Pendel normal mit nativen CAD-Quadern aufgebaut: L=960 mm, Querschnitt 20x20 mm,
Endgewicht 80x80x80 mm, Referenzdichte 7850 kg/m3. Massen 3.0144+4.0192=7.0336 kg.
Endgewicht ist echte Masse samt Traegheit, keine doppelte Gewichtslast. Explizit
ungrounded bewegliche Vorkommen: der erste native CAD-Part ist sonst automatisch
geerdet. Dieser Default wurde im ersten Fixtureversuch durch die MKS-Topologie-
pruefung erkannt (Bewegungsgelenk innerhalb eines geerdeten Starrblocks).

Unabhaengiger Vergleich: physikalisches Pendel I*qdd=-m*g*d*sin(q), RK4 mit dt
0.0002 s gegen Exudyn dt=0.002 s, 41 Ausgabepunkte ueber 2 s. Winkelabweichung
maximal 7.01441045099227e-06 rad. Lagerkraftbetrag unabhaengig aus m*a_COM minus
Gravitation: maximale Abweichung 0.047602536692451736 N. Initialer t=0-
Multiplikatorwert ist noch kein qualifizierter Kraftmesswert; in Rohdaten
behalten, im Scope von der Auswertung ausgeschlossen.

Wichtiger weiterer Befund: vorgeschriebene Kosinusbewegung mit spektralem Radius
1 hatte nach dem Anhalten wechselnde Reaktionskraftbetraege von etwa 63 bis
91 N. Physikalisch muss der ruhende Haltefall 68.999616 N ergeben. Kleine
kinematische Zwangsrestfehler allein qualifizieren daher keine Kraftzeitreihe.
Keine Glaettung der Ergebnisdaten vorgenommen. Generalized-alpha im externen
Adapter bei vorgeschriebenen Antrieben auf rho_infinity=0.6 umgestellt, bei
freien Bewegungen rho=1 belassen. Neuer numerics-Eintrag dokumentiert die Wahl;
alte angetriebene Laeufe ohne Stabilisierung werden nicht als neue Kraftauswertung
im Scope verwendet, koennen aber weiterhin als Bewegung abgespielt werden.

Haltefallfehler nach Korrektur: 1.2177977737337642e-08 N bei 1000 Schritten,
2.834984513810923e-08 N bei 2000 Schritten. Abweichung waehrend vorgeschriebener
Bewegung gegen m*a_COM-Fg bei 1000 Schritten: 0.0044345642347849434 N.
Die Zeitpunkte t=0 und der Beschleunigungssprung der C1-Kosinusquelle bei
0.5 s werden nicht als eindeutige stationaere Kraftreferenz verwendet. Kuenftig
auch C2-Profile und Sensitivitaet gegen Integratoreinstellungen untersuchen;
rho=0.6 ist eine hier gepruefte Wahl, kein universelles Optimum.

Belege und Reproduktion: tools/verify_mks_signal_graph.py,
output/mks-pendulum/verification.json, Gelenkkraft-CSV, GIF, normale native
Projektdateien und echte UI-Screenshots. Offizielle Methodikquellen und Grenzen
in Literatur/Simscape_Multibody_Bewegung_und_Sensoren.md, Bedienung in
MKS_BLOCKBEDIENUNG.md. Die Umsetzung bildet den Grundablauf ab und ist keine
vollstaendige Simscape-Implementierung. Keine experimentelle Validierung.

Abschliessender Vergleich waehrend der vorgeschriebenen Pendelbewegung:
maximaler Kraftbetragsfehler 0.0044345642347849434 N bei 1000 Schritten,
7.091387556812379e-05 N bei 2000 Schritten (jeweils gegen analytische
COM-Beschleunigung, ohne den exakten Beschleunigungssprung). Dies zeigt in
diesem Fall eine deutliche Verbesserung durch Zeitschrittverfeinerung, keinen
allgemeinen Konvergenzordnungsnachweis fuer beliebige Modelle.

Auch Roboter mit sechs vorgegebenen Gelenken neu gerechnet und zwei gemeinsame
Kraftkurven im Scope bedient: output/mks-signal-robot/verification.json,
Roboter-Gelenkkraefte.csv und Screenshot. Originalprojekt SHA256 unveraendert.
Keine experimentelle Roboter-Kraftvalidierung daraus abgeleitet. Zw?lf native
Beispiele, weiterhin nur eine Startdatei. Neue Quellen und Quellenhinweise sind
lokal in Literatur/Simscape_Multibody_Bewegung_und_Sensoren.md auffindbar.

Abschluss: auch die normale Starter-EXE geprueft (zwoelf Beispiele, Gelenkgraph,
Simulation starten per UI, integrierte Wiedergabe und Scope). Beleg unter
output/mks-pendulum-final/verification.json. Native Build und Clippy -D warnings
fuer die betroffene UI bestanden; grosse alte Testlaeufe/volle CI bleiben pausiert.
Quellenstand inklusive neuem Modul, externem Worker und Testhelfern als geprueftes
source-overlay-20261028.zip mit Einzelhashes und EXE-SHA unter
output/mks-pendulum/provenance.json gesichert. Vorherige EXE bleibt erhalten,
kein Remote-Push und keine automatischen Aenderungen an den Originalmodellen.

Der numerisch fehlerhafte Vorher-Lauf ist nicht nur textlich beschrieben:
output/mks-pendulum/numerics-comparison enthaelt beide rohen Rechenauftraege,
Ergebnisse, CSV-Vergleich und beide Workerstaende. Alter Worker aus dem Backup
wiedergewonnen und gegen den SHA256 des urspruenglichen Resultats verifiziert.
Maximaler Halte-Kraftfehler im archivierten Vorher-Fall: 22.478073060853816 N,
im Nachher-Fall etwa 1.22e-08 N. Diese Daten erlauben eine eigene Vergleichsgrafik.

## 28.10.2026 – Grafischer MKS-Arbeitsbereich und Berechnungsblatt

Ziel: den Wechsel zwischen CAD, Berechnung und Simulation innerhalb derselben
Programmhülle vereinheitlichen. Die Nutzerkorrektur ist verbindlich:
Blockbibliothek links, Blockplan in der Mitte, drehbare Simulation rechts.
Werkzeuge liegen oben; Einstellungen erscheinen erst beim Öffnen eines Blocks.

Methode: vorhandene CAD-Ribbon-Symbole und Themenwerte wiederverwenden;
vorhandene native Körper-/Gelenkidentitäten statt unabhängiger Kopien benutzen;
physikalische Verbindungen aus CAD übernehmen; Bewegung, Sensor und konstante
Portkraft über die Bibliothek anschließen. Doppelklick öffnet Einstellungen.
Eine reine Layoutänderung wird getrennt von der Physik gespeichert und aus
dem MKS-Quellfingerabdruck ausgeschlossen. Ausstehende geänderte Rechenvorgaben
werden in der Oberfläche nicht mit alten Ergebnisbildern überlagert.

Das Parameterblatt erhält verschiebbare Formel-, Text- und Diagrammbereiche.
Die grafische Darstellung von Potenzen, Brüchen und Indizes ist von der
Auswertung getrennt: native CAD-Parameter und lokales Calcpad rechnen weiter.
Referenzen werden weiterhin ausdrücklich über Konstruktion aktualisieren
übernommen. Das Blattlayout wird in Projekten und in .scmath mitgespeichert.

Versuch: tools/verify_engineering_canvas.py betreibt eine eigene Testinstanz,
nimmt echte Vorher-/Nachher-Screenshots auf und überprüft die Bedienkette.
Der erste vollständige Bedienversuch bestand: Block ziehen und speichern bei
weiter gültiger Physik; Gelenk doppelklicken; native Formel bearbeiten;
Calcpad-Formel grafisch ändern, lokal berechnen und CAD-Referenzen übernehmen;
Bewegungsquelle auf Gelenk ziehen; Simulation rechts ansehen und Kamera drehen.
Die erweiterte Abschlussprüfung ergänzt Layout-Rundlauf, ungültige Koordinaten
und konstante Portkraft. Ihr endgültiger Status steht maschinenlesbar unter
output/engineering-canvas/verification.json; keine Ergebnisse daraus ableiten,
bevor dort die aktualisierte erfolgreiche Prüfung vorliegt.

Visuelle Kontrolle: Screenshots werden mit demselben Modell und derselben
Fenstergröße aufgenommen. CAD-, MKS- und Parameter-Symbolleisten werden separat
ausgeschnitten und gegenübergestellt. Damit wird Gestaltung vergleichbar;
eine Usability-Studie oder Zeitersparnis ist dadurch noch nicht nachgewiesen.

Forschungsgrenzen: keine vollständige Simscape-/Mathcad-Nachbildung, kein
OpenModelica-Backend, kein freies mechanisches Neuverdrahten im Blockplan,
keine flexible MKS. Das grafische Matheblatt ist eine erste Darstellung mit
Textformel-Editor im Doppelklickdialog. Ergebnisdiagramme vergleichen Skalare
derselben Einheit; Funktionsplots und symbolische Systeme fehlen noch.
Komplexe Calcpad-Kontrollstrukturen bleiben im Quelltext bearbeitbar.

Quellenentscheidung: öffentliche Modelica.MultiBody-Kategorien (World, Parts,
Joints, Forces, Sensors) als fachliche Orientierung, keine Übernahme von
OpenModelica-Quellcode oder fremden Symbolen. Quellen und Einordnungen:
Literatur/Modelica_Blockbibliothek_2026-10-28.md. Bedienung und Architektur:
MKS_UND_BERECHNUNGSBLATT_2026-10-28.md. Große alte Testreihen bleiben pausiert.

Abschlussprüfung dieser Ausbaustufe: alle fünf Prüffolgen im aktualisierten
output/engineering-canvas/verification.json bestanden, einschließlich zusätzlicher
Portkraft, .scmath-Layout-Rundlauf und sichtbarem Kraftdiagramm. Native EXE gebaut;
Clippy für solidcraft-ui-egui mit -D warnings bestanden. Screenshots wurden
anschließend tatsächlich betrachtet: Bibliothek links, Blockplan mit Bildern,
Scope unten und drehbare Wiedergabe rechts; Parameterblatt mit Brüchen,
Hoch-/Tiefstellung und Ergebnisdiagramm.

Gefundener Integrationsfehler und Abhilfe: Beim Anschluss einer Portkraft ergänzt
die Engine optionale Eingabefelder, beispielsweise pressure_mpa: null. Ein direkter
Vergleich mit dem ursprünglich zusammengestellten Formularobjekt hielt deshalb
auch eine erfolgreiche neue Rechnung fälschlich für abweichend. Nach erfolgreichem
Rechnen übernimmt die Oberfläche nun die kanonischen gespeicherten Eingaben,
bevor sie über die Ergebnisanzeige entscheidet. Der Zusatzkraft-Bedienversuch
wurde danach erneut erfolgreich ausgeführt. Dies ist eine Anzeigekorrektur,
kein Nachweis einer neuen physikalischen Genauigkeit des Solvers.

## 28.10.2026 – Wiederverwendbare MKS-Bibliothek, Roboter und Nutzlast

Die Weiterentwicklung betrifft allgemeine native Elemente statt einer gesonderten
Roboter-Darstellung. Neue registrierte Befehle mks.library.body,
mks.library.joint und mks.library.mass haben Eingabe-/Ausgabeschemata. Die
Bibliothek enthält World, Körper, Weld, Revolute, Prismatic, Bewegungssteuerung,
Nutzlast, Bewegungseingang, Sensor, konstante Kraft und Scope. Ein Körper bzw.
eine Nutzlast wird als normale CAD-Komponente mit Quader, benannten
Abmessungsparametern, zugewiesener Ersatzdichte und gegebenenfalls einer nativen
starren Anschlussverbindung erzeugt. Eine Masseneinstellung verändert deshalb
native Masseneigenschaften und Trägheit, nicht nur eine Kraftannotation.

Gelenkdialoge trennen Verbindungen, Ansteuerung und Sensorik. Messausgänge sind
Fx, Fy, Fz und Kraftbetrag im Exudyn-J0-Rahmen. Der gemeinsame Steuerungsblock
ordnet vorhandenen nativen Gelenken Parameterausdrücke und Rampenzeiten zu.
Dies ist eine vorgegebene Bewegungsquelle ohne Rückkopplungsregelung. Die
ausgewählten Gelenkreaktionen werden im gemeinsamen Scope überlagert, nicht
summiert. Antriebsmomente und allgemeine Kontakt-/Feder-/Dämpfergelenke sind
weiterhin nicht validiert und werden nicht als vorhanden dargestellt.

Unabhängiger Referenzfall: Ein über die allgemeine Bibliothek erzeugter nativer
5-kg-Quader befindet sich in ruhender Vertikallage unter einem Drehgelenk.
Erwartung |F| = m*g = 49,05 N. Maximaler Fehler ohne den initialen t=0-Wert:
7,105427357601002e-15 N. Danach Masse auf 8 kg geändert, native Masse geprüft,
altes Ergebnis als historisch erkannt; Prismatic- und Weld-Fabriken sowie die
atomare Abweisung eines Gelenks zwischen demselben Körper geprüft.

Robotervergleich: sechs native bewegliche Gelenke, gemeinsame Zeitachse mit
41 Ausgaben, 1.000 Integrationsschritte über eine Sekunde. Vergleich ohne und
mit 5-kg-Nutzlast; anschließend 8-kg-Nutzlast. Die vorgegebene CAD-Bewegung
bleibt gleich, die Reaktionskräfte ändern sich. Maximaler Unterschied der
CAD-Platzierung nach Korrektur des geänderten Schwerpunktbezugs:
3,814892846065732e-12. Maximale Komponentenänderung der Kräfte durch 5 kg
zusätzlich: Gelenk 1 49,95744306670349 N; 2 49,95744306670326 N;
3 49,96164127161387 N; 4 50,37920876117917 N; 5 50,26654022987954 N;
6 49,95152035455449 N. Diese sind Differenzen einzelner lokaler Komponenten,
keine gemeinsam im Weltrahmen aufaddierten Kraftvektoren.

Methodischer Punkt: Der Solver speichert Körperpositionen am Schwerpunkt.
Zusätzliche Nutzlast verschiebt diesen. Deshalb werden für den Vergleich der
CAD-Bewegung Rotation R und Translation t = x_COM - R*x_COM_initial verglichen;
ein direkter Vergleich der Schwerpunkttrajektorien wäre hier irreführend.

Die 64-KiB-Grenze je Metadatenobjekt wurde beim dichteren Roboterergebnis als
reale Integrationsgrenze gefunden. Abhilfe: verlustfreie Speicherung der Posen
in separaten Objekten je Körper, indiziert vom Hauptergebnis. Die allgemeine
Projektgrenze bleibt bestehen; die Daten werden weder gerundet noch ausgedünnt.
Die MKS-Quellprüfung ignoriert ausschließlich diese Ergebnisobjekte und die
Darstellung, damit das Speichern nicht das eigene Ergebnis ungültig macht.
Ein erneutes Öffnen einer separaten Dateikopie ergab exakt gleiche Posen und
Kraftdaten. Alte Dateien mit inline gespeicherten Trajektorien bleiben lesbar.

Nutzerkorrektur zur Leitungsführung: Die erste orthogonale Einzelverdrahtung war
noch unübersichtlich. Überarbeitet zu direkten grauen mechanischen Verbindungen,
blauer Bewegungssammelleitung und grüner Messsammelleitung mit Abzweigen außerhalb
der Bauteilblöcke. Das Bündeln ist grafisch; die einzelnen Gelenkkanäle bleiben
getrennt. Eine abschließende UI-Prüfung erfolgt nach der letzten Darstellungskorrektur.

Nachweise: tools/verify_mks_robot_library.py;
output/mks-robot-library/verification.json, Roboter-Gelenkkraefte.csv,
native Roboter- und Referenzmodelle sowie echte Screenshots. Originalmodell
SHA256 unverändert. Kein experimenteller Validierungsnachweis des Roboters.
Bedienung und fachliche Referenzen: MKS_BIBLIOTHEK_UND_ROBOTER.md.

Abschluss: Die gesonderte UI-Prüfung nach der letzten Routingkorrektur ist ebenfalls
bestanden (output/mks-robot-library/ui-verification.json). Tatsächlich geprüft:
Joint-Prototyp per Drag-and-drop auf die Zeichenfläche, gespeicherte Verschiebung
eines Körperblocks ohne Verlust gültiger Physik, lesbare Controller-/Joint-Reiter,
editierbarer Nutzlastdialog sowie Wiedergabe und gemeinsamer Scope. Die großen
alten Testläufe bleiben pausiert. Native Build und UI-Clippy mit -D warnings
bestanden. Neues zusätzliches Beispiel im Home-Katalog; Originalroboter erhalten.


## 28.10.2026 — MKS-Quellen, reale Messausgänge und kategorisierte Bibliothek

Ziel: Wiederverwendbare, in native CAD-Gelenke integrierte Quellen und Sensoren;
Simscape dient als Funktionsreferenz, die Oberfläche bleibt im 10derCraft-Stil.
16 Bibliothekseinträge sind nach Körpern, Gelenken, Quellen, Sensoren und Ausgabe
geordnet. Suche, Zielgelenkauswahl, Schnellauswahl auf der Zeichenfläche und
anklickbare Gelenkports ergänzen Doppelklick und vorhandenes Drag-and-drop.
Die portbasierte freie Verdrahtung eines allgemeinen Signalgraphen bleibt offen.

Neue Quellen: Kosinusübergang, Quintic-Übergang 10u³−15u⁴+6u⁵ und Sinus mit
Quintic-Hochlauf. Quellen werden als Profil und Parameter im normalen Job
gespeichert; mks.signal.sample erzeugt die normierte Vorschau. Kein zweiter
UI-Solver. Die Quintic-Rampe hat stetige Geschwindigkeit und Beschleunigung
an den Enden, aber keinen überall stetigen Ruck. Sinus-Abtastung und konservative
native Weggrenzen werden vor dem Lauf geprüft. Alte Jobs bleiben Kosinus-Jobs.

Messausgänge: Gelenkkoordinate und Geschwindigkeit werden aus tatsächlich
berechneten Körperzuständen gewonnen. Rohdaten in SI; Scope getrennt nach
Kraft, Winkel, Winkelgeschwindigkeit, Weg und Geschwindigkeit. Anfangskoordinaten
bleiben enthalten. Fehler während der Entwicklung: Der native Anfangswinkel
lag bereits in rad vor und wurde zunächst irrtümlich nochmals umgerechnet.
Ein unabhängiger 20°-Startwinkeltest deckte dies auf; nach Korrektur beträgt
der Fehler 5,47e-11 rad. Ein Nullstart allein hätte diesen Fehler nicht erkannt.

Gezielte Referenzläufe: nativer 5-kg-Körper am Schubgelenk, drei Profile,
je 1 s / 1000 Integrationsschritte / 41 Ausgaben. Vergleich mit analytischen
Positionen, Ableitungen und |m(g+a)|. Maximale Kraftfehler: Kosinus 0,004537 N,
Quintic 0,000615 N, Sinus 0,002922 N. Der nicht bestimmte initiale Kraftwert
und beim Kosinus der exakte Beschleunigungssprung werden im Vergleich getrennt
behandelt; keine Glättung der Rohdaten. CSV enthalten die Kennzeichnung t>0.

Sechsachsiger Roboter mit 8-kg-Nutzlast: gemessene q- und Geschwindigkeitskurven
gegen unabhängige Sollprofile geprüft. Maximal 2,36e-12 rad bzw. 9,95e-6 rad/s.
Originalmodell unverändert. Echte UI-Screenshots dokumentieren Kategorien,
Gelenkdialog mit Profilvorschau, Messports, Kurven und drehbare 3D-Ansicht.
Build und gezieltes UI-Clippy bestanden; die alten großen Testläufe bleiben
pausiert. Berichte und Rohdaten: output/mks-sources/. Bedienung und Grenzen:
MKS_BEWEGUNGSQUELLEN_UND_SENSOREN.md.

Forschungsgrenzen: numerische Integrationsprüfung, keine experimentelle
Roboter-Validierung. Lokale Gelenkreaktionen sind keine Antriebsmomente.
Koordinaten-Entrollen benötigt ausreichende Ausgabedichte. Kein neuer PID,
Kontakt, geschlossener Mechanismus oder OpenModelica-Solver. Nächster methodischer
Schritt: typisierte Frame-/Signalverbindungen, Konsistenzprüfung und nachvollziehbare
Übersetzung des bearbeitbaren Graphen in denselben nativen Solverjob.


## 28.10.2026 — Unabhängige MKS-Quellen und explizite Verbindungen

Nutzerziel: modularer Aufbau, Bearbeiten von Blöcken und Verbindungen statt
fest verdrahteter Demonstrationen. Ergänzt: unabhängige Quellen mit eigenen
Parameterausdrücken; typisierte Verbindung auf q-Eingänge nativer Gelenke;
mehrere Ziele je Quelle; explizites Ersetzen, Trennen und lokales Wiederansteuern.
Graphbefehle besitzen Schemas und sind rückgängig machbar. Quellendaten und
Verbindungen werden im Projekt gespeichert, Positionen getrennt als Darstellung.
Der Compiler übersetzt den Graphen in die bestehenden mks.job.create/run-Treiber.
Der Solver und die gemessenen Ausgänge benutzen damit keinen zweiten Physikpfad.

Trennungssemantik: Ein explizit getrennter Eingang unterdrückt einen früher
gespeicherten Treiber. Erst eine ausdrückliche Freigabe aktiviert die lokalen
Einstellungen erneut. Entfernen einer Quelle trennt ihre Ziele. Eine Winkelquelle
darf keine Wegachse antreiben. Parameter werden beim Job erneut ausgewertet.
Graphänderungen machen konservativ alte Ergebnisse historisch; Verschieben von
Blöcken nicht. Quellennamen ändern derzeit ebenfalls den Quellnachweis, obwohl
dies für die Physik nicht notwendig wäre.

Oberfläche: Bibliothekseintrag Freie Bewegungsquelle, eigener Editor mit Vorschau,
Anschlussmodus, Ausgang→Eingang-Klickfolge, ausdrücklicher Austauschdialog,
anwählbare Quellenleitungen und Löschen/Trennen. Messausgang→Scope speichert die
normale Messauswahl. Mechanische Gelenke bleiben durch CAD-Verbindungen definiert.
Für neue Quellenleitungen: begrenzte Manhattan-A*-Suche auf einem Sichtbarkeitsgitter
um aufgeweitete Blockrechtecke. Ergebnis nach Geometrie zwischengespeichert.
Kein freier Pfad führt zu einer Warnung, nicht zu einer Leitung durch ein Bauteil.
Linienkreuzungen, gemeinsame Abschnitte und fehlende manuelle Wegpunkte bleiben
offene Darstellungsfragen; bestehende Sammelleitungen verwenden ihren alten Weg.

Gezielte Prüfung: Zweiarm-Baugruppe, native Bibliothekskörper und Drehgelenke,
gemeinsame Quintic-Quelle. Typfehler/fehlende Quellen/fehlende Gelenke und
unbeabsichtigter Austausch atomar abgelehnt; Trennung unterdrückt alte Treiber;
lokale Freigabe, Undo/Redo, Parameterweitergabe und Dateirundlauf geprüft.
UI-Prüfung umfasst Quellenerstellung, Verschieben ohne Physikänderung,
Verbinden/Trennen und Messausgang zum Scope. Anschließend sechs Roboterantriebe
über unabhängige Quellen angesteuert und Zustände mit dem bisherigen Lauf verglichen.
Numerische Fehler und vollständige Checkliste: output/mks-connections/verification.json.
Originalbeispiel unverändert. Build und gezieltes Clippy bestanden; große alte CI pausiert.

Kein allgemeiner Mehrdomänen-Gleichungslöser, keine PID-Rückführung, keine
experimentelle Validierung. Nächste Architekturfragen: eigenständige Signaloperatoren,
tabellarische Sollprofile, Zyklenprüfung, eindeutige Solver-Kausalität und mehrere
Ausgabeblöcke. Bedienung: MKS_FREIE_VERBINDUNGEN.md.


## 28.10.2026 — CAD-Schnittstellen, tatsächliches Ziehen und getrennte Messleitungen

Anforderung: Das Modell muss ohne Skripte über die normale Oberfläche bearbeitbar
sein. Körper und mechanische Verbindungen sollen aus dem CAD kommen. Die CAD-
Baugruppe bleibt deshalb Eigentümer von Geometrie, Massen, starren Gruppen und
Gelenken. MKS ergänzt die Bewegungs- und Messsignale. Keine zweite mechanische
Baugruppe im Blockeditor und keine neu gezeichneten Ersatzteile für den Roboter.

Neue normale CAD-Schaltfläche im Reiter Zusammenbau: CAD-Simulationsschnittstellen.
Erfassung einer ausgewählten ebenen Fläche als nativer, benannter Gelenkursprung
mit persistentem Flächennamen. Eine Mechanical-Port-Referenz joint_origin benutzt
genau diesen Rahmen erneut, statt separat gespeicherte Koordinaten nachzuführen.
Zwei solche Rahmen werden über den normalen CAD-Gelenkbefehl zu Starr-, Dreh-
oder Schubgelenken verbunden. Die einfache Auswahl unterstützt direkt eingesetzte
Komponenten; komplexere Ursprünge bleiben Sache des vorhandenen nativen CAD-Tools.

Semantik: Flächenrahmen überträgt eine Resultierende aus Kraft und Moment;
Punktkraft im Flächenzentrum überträgt kein freies Moment und darf keinen vollen
mechanischen Gelenkanschluss vortäuschen. Ein Flächenbezug im starren MKS ist
keine verteilte FEM-Last, kein Kontaktmodell und keine Druckinterpolation.
Native Gelenke dürfen die Teile beim Ausrichten bewegen; die Simulation selbst
ändert die gespeicherte CAD-Konstruktion nicht.

Bedienung: Tatsächlicher Mausdruck→Ziehen→Loslassen von Quellenausgang auf q und
von Gelenk-/Sensorausgang auf Scope. Klick→Klick bleibt zusätzlich möglich.
Hinweise haben einen eigenen Bereich fester Höhe, damit der Zielanschluss während
des Ziehens nicht durch einen umgebrochenen Ribbon wegspringt. Vorschaukoordinaten
werden in den lokalen, gezoomten Blockplan transformiert. Sensor-Doppelklick öffnet
die Messseite, Quellen-Doppelklick die Ansteuerung. Eine getrennte Messleitung
erhält den Sensor und seine Einstellungen; unscoped_sensors wird gespeichert.
Ohne dieses Feld behalten ältere Projekte ihre verbundenen Messungen.

Leitungen: Neue getrennte Scope-Eingänge statt einer scheinbar gemeinsamen
Kraft-Sammelleitung. Orthogonale Pfade mit Pfeilrichtung, Kanalabstand und einem
Manhattan-Suchzustand inklusive Richtungswechselkosten. Adaptive kurze Endstücke
passen zwischen eng benachbarte Blöcke. Pfade werden nach Geometrie gecacht;
keine behauptete globale Schaltplanoptimierung. Kreuzungsbrücken und manuelle
Wegpunkte sind weiterhin keine zugesagten Funktionen.
Die abschließende Darstellung führt jede Messung auf einer eigenen, 14 Diagramm-
Einheiten getrennten Spur außerhalb der Bauteile. Der automatisch platzierte
Scope reserviert dafür einen Leitungsbereich. Die sechs globalen Spurpositionen
werden im UI-Nachweis gesondert auf Abstand geprüft.

UI-Befund: Erste akzeptanznahe Prüfung fand abgeschnittene Felder im CAD-Dialog.
Aufteilung in Fläche erfassen / CAD-Gelenk / Schnittstellen. Ein späterer Versuch
zeigte, dass die Navigation unterhalb des scrollbaren Inhalts nicht zuverlässig
anklickbar war; Wechsel zu MKS deshalb in die sichtbare Kopfzeile verlegt.
Allein programmatische Befehlsprüfungen hätten diese Fehler nicht nachgewiesen.
Die echte Ziehprüfung deckte außerdem einen UI-Hänger auf: Ein Layer-Zugriff
innerhalb einer gehaltenen Input-Lesesperre war verschachtelt. Eingabezustand
und Layertransformation werden nun nacheinander gelesen; dieselbe Mausfolge
mit anschließender Bedienung dient als Regression. Der Zielanschluss wird über
die losgelassene Position im gezoomten Graphen erkannt, nicht über Hover-Fokus,
den während des Ziehens noch der Ausgang besitzt.

Nachweis: tools/verify_mks_cad_workflow.py erstellt gewöhnliche CAD-Komponenten als
Testfixture. Flächenklicks im Viewport, Werkzeugstart über die tatsächliche CAD-
Schaltfläche, Schnittstellenerfassung, Gelenkerstellung, Quelleneingabe, gezogene
Leitungen, Sensor-Trennen/Wiederverbinden, Nutzlast und Simulation erfolgen danach
über die Oberfläche. Unabhängig geprüft: CAD-Höhenänderung um 30 mm verschiebt
den gebundenen Flächenrahmen um 30 mm; der Kraftanschluss folgt demselben Bezug.
Eine Quintic-Vorgabe wird gegen ihre analytische Formel verglichen. Dateirundlauf
und CAD-/Quellen-/Gelenkfenster bei 1280×800 sind Bestandteil der Prüfung.
Der Roboter behält seine ursprüngliche native CAD-Baugruppe; sechs getrennte
Messleitungen werden geprüft. Abschlussbericht mit Fehlerzahlen, Modelle und echte
Screenshots: output/mks-cad-workflow/. Originaldatei-Hash unverändert.

Build und gezieltes UI-Clippy mit -D warnings bestanden. Alte große Testläufe
bleiben pausiert. Dies ist eine Bedien-/Integrationsprüfung mit kurzer numerischer
Referenz, keine experimentelle Validierung des Roboters. Bedienungsanleitung:
MKS_CAD_WORKFLOW.md.


## 29.10.2026 – Endlosgitter, assoziative Flächen und großer Roboterbewegungsablauf

Fragestellung: Können Berechnung, native CAD-Bemaßung, Lastschnittstelle und
MKS-Bewegung mit gemeinsamen Referenzen reproduzierbar nachgeführt werden?

Das MKS-Gitter zeichnet jetzt den sichtbaren Weltbereich des vorhandenen Scene-
Containers, einschließlich negativer Koordinaten. Blockausdehnung und sichtbarer
Hintergrund sind getrennt. Linke Maustaste bearbeitet Blöcke/Signalleitungen;
mittlere/rechte Maustaste navigiert auf freiem Hintergrund. Die Bedienprüfung
deckte auf, dass bisher jedes gedrückte Mausbutton-Ereignis einen Block bewegen
konnte. Blockbewegung und Leitungsbeginn wurden auf die primäre Taste begrenzt.

Neue optionale Portdaten: label, role und area_face. Kraftaufnahme/-abgabe ist
Lastpfadsemantik; der mechanische Anschluss bleibt reziprok. Das CAD-Werkzeug
erfasst die tatsächlich ausgewählte ebene Fläche als benannten nativen Ursprung.
Flächenrahmen und Flächeninhalt müssen denselben persistenten Flächennamen
verwenden. Kernel-Flächenauswertung mit 0,01 mm Toleranz; kein Rückgriff auf eine
alte skalare Fläche, wenn der Bezug fehlt oder mehrdeutig ist. Keine verteilte
Kontakt-/FEM-Last aus einer bloßen Starrkörper-Resultierenden ableiten.

Versuch 1: Im lokalen Calcpad R_Oberarm von 77 auf 80 mm ändern, Änderungen an
CAD-Referenzen anzeigen, worksheet.apply ausführen. Die native Querschnittsfläche
steigt von 18624.40109339 auf
20103.92427015 mm², konsistent zu (80/77)^2.
Danach Basisradius wiederherstellen. Unabhängige GUI-Prüfung mit Quader:
Fläche 8000 mm², nach Längenänderung 12000 mm²; Namen/Rollen über normale
Bedienelemente erfassen und ändern. Fehlender oder zum Rahmen unpassender
Flächenname wird abgewiesen. Bestehende Punktanschlüsse bleiben kompatibel.

Versuch 2: Unveränderte native Roboterbaugruppe, vorhandene 8-kg-Nutzlast,
sechs unabhängige Sinus-Bewegungsquellen mit amplitudenabhängigen Calcpad-
Referenzen. A = [90,35,50,90,45,120] Grad; f = [.25,.25,.375,.5,.25,.5] Hz.
q(t) = A (10u³ − 15u⁴ + 6u⁵) sin(2πft), u = min(t/0.8,1).
Lokaler Exudyn-Adapter, 8 s, 3960 Integrationsschritte, 67 Ausgabestützstellen.
Gemessene Gelenkkoordinaten gegen diese analytische Vorgabe vergleichen:
maximaler Fehler 1.22713685879e-08 rad.
Kraftsensoren liefern lokale Gelenk-Reaktionskräfte in N, keine nachgewiesenen
Antriebsmomente. Rohkurven und Spitzenwerte sind im Ergebnisordner gespeichert.

Ein erster Versuch zeigte historische Ergebnisse nach dem Speichern einer
neuen Variante. Korrektur im Erzeugungsablauf: Variante vor der Simulation
speichern, danach berechnen. Dateirundlauf prüft result_current und alle
CAD-Flächenbezüge. Die Originaldatei wurde anhand ihres SHA-256 unverändert
nachgewiesen. Keine Rückdatierung historischer Ergebnisse als aktuelle Werte.

Ein Screenshotvergleich deckte zusätzlich eine Abweichung zwischen Engine und
Oberfläche auf: Die Engine hatte ein aktuelles Ergebnis, die Oberfläche verglich
jedoch unbearbeitete Draft-Treiber mit den vom Quellgraphen eingesetzten
kanonischen Treibern und blendete 3D/Scope aus. Beim Anzeigevergleich werden
deshalb nur die vom Graphen überschriebenen lokalen Treiber ausgenommen. Die
Vergleichsdaten normalisieren außerdem gleichwertige JSON-Zahlen wie 8 und 8.0;
große IDs werden dabei nicht verlustbehaftet in Gleitkommazahlen umgewandelt. Die
Engine prüft weiterhin die vollständige Quelle. Regression: sichtbarer echter
3D-Viewport und Kraftkurven nach Dateirundlauf, anschließende Quellenänderung
blendet veraltete Wiedergabe aus, exakte Wiederherstellung stellt sie wieder her.

Eine zweite sichtbare Korrektur betraf den CAD-Dialog: Zusätzliche Eingaben waren
unterhalb der sichtbaren Höhe. Übertragungsarten stehen nun nebeneinander,
Beschriftung/Rolle kompakt in einer Zeile und der Erfassungsbefehl davor. Die
Hinweise sind aufklappbar. Der erneute Test verwendet den tatsächlichen CAD-
Flächenklick und die tatsächlichen Bedienelemente, keine direkten Ersatzbefehle
für die Erfassung oder das Ändern der Beschreibung.

Video: native CAD-Geometrie und tatsächliche gespeicherte Exudyn-Posen, feste
Kamera und identischer Ausschnitt aller Frames. 66 Intervalle / 8,25 Bilder/s
= 8 s, AVI/MJPEG; PNG-Endzustand und Rohbilder separat. Keine erfundene Bewegung
oder Bildgenerierung. JPEG-Bilder, RIFF-Größe, Bildanzahl und Abmessungen geprüft.

Berechnungsblatt: I = πR⁴/4, A = πR², m = ρAL und δ = FL³/(3EI)
als Vergleich eines einzelnen Vollrund-Kragträgers. Die CAD-Längen sind dafür
übernommene Auswertewerte; aktuell treiben die Radien die CAD-Geometrie und die
sechs Hübe die MKS. Eine Änderung beliebiger Längen samt Neuberechnung aller
Gelenkrahmen ist damit nicht bereits realisiert.

Zusätzlich vorhandene statische Balken-FEM erneut ausgeführt: lineare
Euler-Bernoulli-Elemente, verriegelte Antriebe, 100 N TCP-Last. Keine gekoppelte
flexible MKS; Schub, Kontakt, Gelenk-/Getriebesteifigkeit und dynamische
Spannungsrückgewinnung sind nicht Bestandteil dieses Nachweises. Die kurzen,
dicken Armquerschnitte begrenzen die Aussagekraft der Balkenidealisation.

Reproduzierbarkeit: tools/build_robot_endless_example.py,
tools/verify_robot_endless_ui.py, tools/export_robot_video.py; vollständige
Eingangs-/Ergebnisdateien und Screenshots in output/mks-endless-robot/.
Anleitung: ROBOTER_PARAMETRISCH_MKS.md. Gezielter nativer Build und UI-Clippy
mit -D warnings bestanden; alte umfangreiche Testläufe bleiben pausiert.
Für die Dissertation folgen Konvergenz-, Robustheits- und Änderungsaufwands-
Vergleiche gegen einen manuell gepflegten Referenzworkflow sowie reale Messdaten.


## 29.10.2026 – MKS-Wiedergabe: gemessene Geschwindigkeit und reproduzierbarer Start

Ausgangspunkt: Nutzerbeobachtung etwa ein Bild/s. Vorher-Messung am identischen
Roboter: 0,401–0,457 s nur für einzelne Posenaufbereitung und etwa 1,2 s pro
UI-Rundlauf während CPU-Wiedergabe; erneutes unverändertes Starten brauchte
34.336839 s. Statt den Solver ungenauer zu machen:
validierte Wiedergabedaten einmal vorbereiten, native CAD-Netze einmal auf die
GPU hochladen, pro Bild nur starre Platzierungen/Achsen übertragen. Kontinuierliche
Zeitsteuerung und gelenkrelative SLERP erhalten Anker in Zwischenposen. Rohkurven
bleiben bei 67 Stützstellen. Prüfen von vier Originalposen, Rotationsorthogonalität
und Zwischenpose-Ankerfehler verhindert eine bloß optisch schnellere falsche Bewegung.

Nachher in drei kurzen Wiederholungen: 34.316 neue GPU-Posen/s Median,
Spanne 15.208 bis 43.099; Start bis zur sichtbaren
Wiedergabe 0.703212 s. Unveränderter Engine-Lauf mit exakter
Quell-/Einstellungsprüfung: 0.058969 s Median; Einzelwerte und Messablauf in
verification.json. Neues Ergebnis wird nur bei geändertem Zustand oder explizitem
„Neu berechnen“ erzeugt. Ein Hintergrundlauf arbeitet auf Scratch-Daten und wird
über mks.job.commit erst nach erneuter vollständiger Eingabeprüfung übernommen.
Eine zwischenzeitliche Änderung der CAD-Schnittstelle weist das Ergebnis ab.

Zusätzlich begrenzter RAM-Cache der geometrischen Integrale; gehaltene native
Geometrie-/Netzidentitäten verhindern Adresswiederverwendungsfehler. Dichte und
Einbautransformation werden weiterhin neu angewendet. Warme Neuberechnung
4.410665 s; erster vollständiger Lauf
50.420124 s im Hintergrund. Dessen UI-Antwortzeiten stehen
einzeln im Bericht. Kräfte und Koordinaten bitgleich zur Referenz. Kein Nachweis
einer beschleunigten kalten Geometrieintegration und keine höhere physikalische
Messauflösung behauptet. GPU: RTX 4060 Laptop, DX12; CPU-Fallback unverändert.

Fehlergrenzen: Matrixvergleich 4.14729584008e-10,
Zwischenpose-Ankerfehler 1.13686837722e-13 mm.
Build und gezieltes UI-Clippy -D warnings bestanden. Numerische/UI-Prüfung:
tools/verify_mks_playback_speed.py; Messung: tools/profile_mks_playback.py.
Anleitung und Messinterpretation: MKS_WIEDERGABE_PERFORMANCE.md.


## 2026-10-29: Direktes mathematisches Arbeitsblatt und dimensionsgetreue Ausgabe

Fragestellung: Wie lassen sich freie, lesbare Berechnungen unmittelbar bearbeiten
und als kontrollierte CAD-Referenzen verwenden, ohne Anzeigeumrechnung und
physikalische Berechnung zu vermischen?

Vorgehen: Eine begrenzte zweidimensionale Formelansicht zerlegt Text in Brüche,
Potenzen und Indizes. Jeder sichtbare Cursor behält einen UTF-8-Quelltextoffset.
Damit bearbeitet ein Klick im Nenner denselben normalen Berechnungsausdruck;
es gibt keine zweite aus dem Bild zurückgelesene Rechenlogik. Auswertung über den
bereits lokal eingebundenen, MIT-lizenzierten Calcpad-Kern, weiterhin als einzelner
lokaler Prozess. Kein Berechnungsserver. Der grafische Editor ist eigenständig
implementiert; keine Übernahme von Mathcad-Quellcode oder Bedienelementgrafiken.

Darstellungsebenen: Originale volle skalare Gleitkommawerte werden vor einer
Ausgabeumrechnung exportiert. Einheitenwünsche, umgerechnete ungerundete
Anzeigewerte, Quellenfingerabdruck und Referenzmarkierungen liegen in der
Präsentation. CAD-Bindungen lesen weiterhin die Originalwerte. Signifikante
Stellen runden ausschließlich die Ausgabe; unnötige Nachkommanullen entfallen.
Eine Bezugseinheit für eine Komponente wird anhand der Dimension geprüft:
0,45 Nm mit gewünschtem mm ergibt 450 Nmm. Bei inkompatibler Dimension gilt die
vom Nutzer verlangte explizite Faktorisierung, beispielsweise 2 s mit m als
2 s * (1/m) * m. Dies ist keine Umrechnung von Zeit in Länge. Der zusätzliche
Faktor wird sichtbar erhalten, nicht algebraisch weggekürzt. Ungültige Syntax
wird atomar abgewiesen. Rohe Calcpad-Ergebnisse bleiben in beiden Fällen gleich.

Geprüfter Bedienablauf und Ergebnisse (tools/verify_inline_worksheet.py):
- b_1 direkt im Blatt von 15 auf 20 mm ändern, = berechnet und fokussiert die
  Ergebniseinheit; cm eingeben zeigt 2 cm, roh bleiben 20 mm.
- 0,45 Nm mit mm wird 450 Nmm. Den Nenner R_1 direkt per Maus anwählen und durch
  8 ersetzen ergibt 0,225 Nm; danach den normalen Ausdruck wiederherstellen.
- Sekunden mit gewünschtem m zeigen 2 (s)*(1/(m))*(m), auch nach Neuberechnung.
- Referenzwerkzeug auf Variablennamen klicken: Markierung und scrollbarere Liste;
  ref_b_1 treibt einen nativen CAD-Quader. Eine Änderung auf 25 mm bleibt als
  ausstehende CAD-Änderung sichtbar und wird über Konstruktion aktualisieren
  übernommen.
- Drei signifikante Anzeigestellen einstellen: third erscheint 0,333, der rohe
  Zahlenwert bleibt der volle Gleitkommawert von 1/3.
- Speichern/Laden erhält Einheitenwünsche, Referenzen und Präsentationsdaten.
- Eine neue Formel wird direkt ohne Bearbeitungsfenster eingefügt; 10^-3
  wird als 0,001 ausgewertet und mit hochgestelltem Exponenten angezeigt.
- Auch ein natives Längenparameterfeld kann cm bzw. den expliziten Faktor s
  erhalten, ohne seinen ursprünglichen CAD-Wert zu ändern. kg@@ wird ohne
  teilweise Übernahme abgewiesen.
- Kurzer realer Pendellauf mit 0,1 s, 100 Schritten und 11 Ausgabestützstellen:
  anschließende reine Einheitenänderung lässt die MKS gültig und alle gespeicherten
  Solverergebnisse identisch. Keine erneute Robotersimulation nötig.

Separater Einheitenvergleich (tools/verify_inline_unit_helper.py): Drehmoment,
Länge, Fläche, Druck, Winkel, 10^-3 und 1/3; genaue Vergleichstoleranzen und
Rohwerte in output/worksheet-inline/helper-verification.json. Bediennachweis und
Screenshots in output/worksheet-inline/inline-verification.json sowie den PNGs.
Build und gezieltes UI-Clippy bestanden; die früher pausierten breiten Testläufe
wurden nicht wieder gestartet.

Grenzen: Dies ist ein direkt editierbares skalares Berechnungsblatt, kein
vollständiger Mathcad-Nachbau mit symbolischem Algebraeditor. Für allgemeine
CAD-Maßbindungen gelten weiterhin die unterstützten CAD-Dimensionen; zusammengesetzte
Referenzen werden nicht als dimensionslose Länge ausgegeben. Der vorhandene
Diagrammtyp bleibt ein skalares Ergebnisdiagramm. Beim Laden des bisherigen Roboterbeispiels wurde schon vor einer
Einheitenänderung ein historisches Ergebnis festgestellt. Der konservative
Quellfingerabdruck umfasst auch Formatmarker; deren Änderung ist eine mögliche
Ursache, die hier nicht unabhängig isoliert wurde. Diese Gültigkeitsprüfung wurde nicht umgangen und
das Benutzerbeispiel nicht überschrieben. Die Einheitenanzeige selbst invalidiert
keinen zum aktuellen Eingabestand passenden Lauf.

Reproduzierbarkeit: Die geprüfte Desktop-Version, ihr Vorgänger sowie Hashes von
Quell- und Helferdateien stehen in output/worksheet-inline/provenance.json.

Installation: Die bisherige EXE war vom laufenden Benutzerfenster gesperrt.
Dieses Fenster wurde nicht geschlossen. Start-10derCraft.cmd wählt über eine
interne launch-target.txt die geprüfte benachbarte EXE; Build-10derCraft.ps1
entfernt diese Auswahl nach einem erfolgreichen regulären Neubau wieder.
Eine Sicherung des Vorgängers bleibt erhalten. Ein eigener kurzer Start der
installierten EXE bestätigt die Anzeige s*(1/m)*m und Nm->Nmm. Nachweis:
output/worksheet-inline/installed-verification.json; Screenshot unter
output/worksheet-inline-installed/Installierte-Version.png.

2026-10-29: Lokale Eingabehistorie mit maximal 100 Schritten für direkte Formeln ergänzt. Strg+Z, Strg+Y und Strg+Umschalt+Z bearbeiten die Eingabehistorie. Der aktive Formeleditor erhält Vorrang vor globalen CAD-Tastenkürzeln. Gezielter GUI-Test: 15 mm durch 20 mm ersetzen, zurücknehmen und auswerten ergibt 15 mm; 25 mm einsetzen, zurücknehmen, wiederherstellen und auswerten ergibt 25 mm. Nachweis: tools/verify_worksheet_undo.py und output/worksheet-undo/verification.json. Build bestanden; Screenshot im selben Ausgabeordner.

2026-10-29: Quellengebundene Materialtabellen im Berechnungsprojekt ergänzt. Zwei Dickenbereiche S355JR (Metinvest), drei Zustands-/Dickenkombinationen EN AW-6082 (Hydro) und definierte Normfallbeschleunigung (BIPM). Kennwerte sind benannte Mindestwerte bzw. explizite obere Bereichsgrenzen. Unbekannte E-Module, Dichten und zulässige Spannungen werden nicht erfunden. Einfügen erzeugt gewöhnliche Calcpad-Definitionen samt Quellenkommentar, keine automatische Änderung der CAD-Materialzuweisung. GUI-Test: Auswahl aufklappen, S355-Werte einfügen, erneutes Einfügen ohne Überschreiben blockieren. Alle sechs Datensätze einzeln lokal berechnet und jeden Ergebniswert gegen die Datenquelle geprüft; bestanden. Nachweise: tools/verify_worksheet_materials.py, output/worksheet-materials/verification.json und Materialtabelle.png. Umfang und Quellen in MATERIALTABELLEN.md. Erweiterung des bestehenden worksheet.read-Datenangebots; keine neue persistente Projektstruktur.

2026-10-29: Material-/Konstantensammlung als obere, ein-/ausblendbare Bibliothek mit Suchfeld angeordnet. Links bleiben Projekt und Referenzen. Kategorisierte Datensätze mit Quellen und Zustandsgrenzen; fünf definierte SI-Konstanten nach NIST ergänzt. Bei der Integration wurde e-Notation vom lokalen Parser als Einheit fehlinterpretiert; Einfügen erzeugt deshalb explizite Zehnerpotenzen und geklammerte Einheiten, auch für inverse Stoffmenge. Zwei eigene SVG-Master book_closed und book_open folgen direkt dem Sichtbarkeitszustand; keine automatische Abnahme als finales Artwork. Direkter Icon-Linter bestanden: 346 Master, Palette, Namens- und Elementregeln. Datentabelle und Kopfzeile bleiben über die bestehenden Befehle erreichbar; gewöhnliche Berechnungsdefinitionen statt neuer Projektstruktur. Gezielter Bediennachweis unter output/worksheet-materials; Testwerkzeug tools/verify_worksheet_materials.py.

Abschluss Bibliothek: Gezielte GUI-Prüfung bestanden: oberer Knopf, geschlossenes/offenes Buch, Suche nach Avogadro, normales Einfügen von Material und Naturkonstante, Blockade doppelter Namen sowie lokale Auswertung sämtlicher elf Datensätze gegen hinterlegte Werte. Screenshots Bibliothek-Geschlossen.png und Bibliothek-Suche.png unter output/worksheet-materials. Grenzen: erweiterbare Anfangssammlung, keine vollständige Werkstoffdatenbank und kein automatisches Umschalten des CAD-Materials.

2026-10-29: Berechnungsworkflow erweitert. Reversibles Abwählen erhält den zuletzt übernommenen CAD-Parameterwert und speichert die bisherige Bindung in suspended_references; Wiederanwählen stellt denselben Alias wieder her. Ergebniseinheiten werden mit normalisierten Divisionszeichen als echte Brüche gesetzt und mit der Zahl an einer gemeinsamen mathematischen Grundlinie ausgerichtet. Abstand verringert. Bibliothekskennwerte sind per Drag-and-drop als gewöhnliche Berechnung einfügbar. Blattpositionen werden nach Variablennamen statt Quellzeilennummern gespeichert, damit Tabellenwahl die Ablage nicht anderen Formeln zuordnet. Griechische Zeichen als obere Palette; Fehler beim Umbenennen ohne gespeichertes Layout gefunden: verändernder JSON-Zugriff hatte positions:null erzeugt. Optionalen Metadatenzugriff ohne Einfügen korrigiert. Eine isolierte Prüfung widerlegte die anfängliche Fokusvermutung als Ursache dieses verbleibenden Fehlers.
Allgemeine Familientabellen: bis 32 Familien, je bis 32 Spalten und 128 Zeilen mit skalaren Berechnungsausdrücken. Zwei typisierte Befehle worksheet.family.set/select, persistente Projektmetadaten und Matheblatt-Export/-Import. Zeilenwahl liefert stabile family_column-Namen und lässt CAD-Änderungen bis worksheet.apply ausstehen. Kein vollständiger Klassenparser, keine Vererbung, keine automatische Schraubenmontage. FlangeBolt-Vorlage hat englische Feldnamen, Nullwerte als Platzhalter; Testwerte sind handgewählte Daten für den Mechanismus, kein Normnachweis für Schrauben.
Gezielte Prüfung tools/verify_worksheet_flow.py bestanden: Abwählen bei existierendem CAD-Quader erhält 20 mm, erneutes Anwählen reaktiviert; Schwerkraftwert per Ziehen eingefügt; Familienwahl Small->Large erzeugt ausstehende Änderung 9->11 mm, explizit übernommen; Zellbearbeitung im normalen Tabelleneditor auf 12 mm berechnet; Export/Import erhält aktive Familienzeile; obere Symbolpalette setzt Alpha ein, anschließende Definition und Auswertung ergeben 3 mm. Numerische Prüfung, echte UI-Eingaben und Screenshots unter output/worksheet-flow. Build und gezieltes Clippy mit -D warnings bestanden. Frühere breite Testläufe weiterhin pausiert. Anleitung: BERECHNUNGSBLATT_FAMILIEN.md.
# 2026-10-30 – Dichte MKS-Ergebnisse und reproduzierbare Integration

Die Arbeitshypothese lautet: Eine grobe gespeicherte Zeitreihe verursacht kantige Diagramme; die 3D-Wiedergabe benötigt zusätzlich eine stetige, gelenktreue Interpolation. Beide Probleme wurden getrennt geprüft. 21 und 1001 Ausgabepunkte aus identischen 1000 Integrationsschritten stimmen an allen gemeinsamen Zeiten exakt überein (maximale Abweichung 0). Der Mehrbedarf in einer Einzelmessung betrug 2,47 s → 3,17 s. Dies ist kein allgemeiner Leistungsbenchmark und kein Genauigkeitsnachweis des physikalischen Robotermodells.

Zwischenergebnisse und Fehlversuche: Die höhere Ausgabe überschritt zunächst die Dokument-Metadatengrenze von 64 KiB. Ergebnisse wurden deshalb in ein eigenes auf 8 MiB begrenztes, gemeinsam genutztes Dokumentfeld verschoben. Danach fiel der reale Roboterlauf an der bisherigen 2-MiB-Antwortgrenze des Solverprozesses aus; mit begrenzten 12 MiB wurde derselbe Workflow erfolgreich abgeschlossen. Diese Grenzen werden ausdrücklich beibehalten, um unkontrollierte Speicherbelastung zu vermeiden. Keine globale Aufhebung von Eingabegrenzen.

Versuchsartefakte: `output/mks-dense-drag/verification.json`, `numeric-comparison.json`, `Ausgabeaufloesung-Vergleich.png` und `.pdf`; `output/robot-drag-chain/verification.json`; `output/pendulum-fine/verification.json`. Die Grafik stellt echte grobe und dichte Werte dar, ohne nachträgliche Kurvenglättung. Der interne Wiedergabezähler misst Zustandsaktualisierungen, keine verifizierte Bildschirmbildrate.

Methodische Trennung: Eingaben → unveränderlicher Auftrag → externe Berechnung → quellgeprüfte Übernahme → Ergebnisdarstellung. Präsentationslayout und Wiedergabegeschwindigkeit sind keine physikalischen Eingaben. Änderungen an Quellen oder Lasten machen gespeicherte Ergebnisse ungültig. Bei Videoexport werden nur Darstellungspositionen interpoliert; Kraftdaten werden dadurch nicht neu berechnet.

Die Integration des Originalprojekts auf SolidCraft 0.27.1 erfolgt aus einer gesicherten lokalen Kopie mittels Dreiwege-Zusammenführung. Herkunftscommits, Grenzen und Freigabeprüfungen stehen in `RELEASE_2026-10-30.md`. Eine Veröffentlichung ist erst nach der freigegebenen vollständigen Abschlussprüfung vorgesehen. Forschungsfragen bleiben die konsistente Rückführung zwischen Berechnung, CAD und Simulation, Variantenwechsel und überprüfbare Effizienzgewinne gegenüber einem getrennten Werkzeugworkflow. Der bloße Nachbau einer Benutzeroberfläche ist kein hinreichender Dissertationsbeitrag.



### Integrationsprüfung: Fehlertrennung und reproduzierbare Nachprüfung

2026-10-30: Die Upstream-Integration ist zusätzlich ein Fallbeispiel für die
Absicherung einer erweiterbaren Simulationsumgebung. Nach einem abgebrochenen
Gesamtlauf wurden Fehler einzeln klassifiziert: fehlende Abdeckung der eigenen
Featurevariante, geänderte Befehlsmenge, reale Platz-/Klickprobleme der Oberfläche
und zeitabhängige Benachrichtigung unter Last. Ein Fehlschlag wird nicht allein
durch Wiederholung oder Änderung des Sollwerts als behoben erklärt.

Für das Evolventenrad wurden nur die neuen Referenzzeilen ergänzt: sieben
benannte Größen mit Einheitenklassen, drei Mittelpunkt-Ausdrücke und die
Mittelpunkt-Transformation. Die drei bisherigen Golden-Dateien bleiben sonst
unverändert. Alle sieben Feature-Walk-Prüfungen bestehen anschließend auch
ohne Aufzeichnungsmodus.

Eine neue Interaktionsprüfung öffnet zwölf Dokumente bei 640 × 1000 Pixeln,
betätigt das Überlaufmenü und wechselt tatsächlich in MKS. Sie deckte zuerst
einen Fehler ihrer eigenen Instrumentierung und danach einen realen,
abgeschnittenen Klickbereich auf. Die finale Prüfung besteht nach einer
platzabhängigen Navigation; die breite Oberfläche behält ihre Reiter. Damit
wird Erreichbarkeit geprüft und nicht nur das Vorhandensein eines UI-Elements.

Die bisherige Benachrichtigungsprüfung besteht isoliert in 0,803 s ohne Änderung
der Produktionslogik. Das ist ein Hinweis auf eine lastabhängige Störung im
ersten Lauf, kein Beweis ihrer vollständigen Abwesenheit. Der erneute Gesamtlauf
verwendet Nextest mit Prozessisolation, vier Testplätzen, serialisierten
Windows-Grafiktests, allen schweren Tests und ohne automatische Wiederholungen.
Laufzeiten unter dieser Planung dürfen nicht direkt als Produktbeschleunigung
interpretiert werden. Der Gesamtlauf ist zum Zeitpunkt dieses Eintrags noch
offen. Protokolle: `integration/release-2026-10-30/` im lokalen Arbeitsordner;
veröffentlichbare Befunde im Integrationsbericht.


## Abschluss mit gezielten Prüfungen

Der Benutzer hat am 2026-10-30 ausdrücklich einen zügigen Abschluss mit weniger
gründlicher Prüfung verlangt. Der erneute vollständige Lauf wurde deshalb
während der großen CAD-Befehlsmatrix beendet. Er ist **nicht bestanden** und
wird nicht als vollständiger Prüfnachweis dargestellt. Vor dem Abbruch bestanden
Formatierung, Ressourcen, Symbole, Lizenzen, Listen, Skills, Schichten,
UI-Container, vollständige Clippy-Prüfung, Web-Kompilierung und Dokumentationstests.
Die vollständigen nativen Funktionstests bleiben offen.

Die sieben Feature-Walk-Tests ohne Aufzeichnungsmodus, die CLI-Empfehlung,
die isolierte Änderungsbenachrichtigung und der reale Klicktest bei schmalem
Fenster bestanden gezielt. Der abschließend gebaute Desktop wurde danach erneut
mit allen 18 Katalogprojekten, Roboter-Quellen-/Sensor-Drag-and-drop sowie
grafischem und programmatischem AVI-Export geprüft. Die Videobilder wurden
unabhängig dekodiert. Der frühere Nachweis mit 180 Getriebefällen und
17.280 skalaren Prüfungen bleibt separat erhalten.

Der genaue Umfang samt Binärhash steht in
`docs/10dercraft/evidence/focused-release.json`. Dieser Snapshot ist ein
Forschungsprototyp mit gezielter Freigabe; eine vollständig grüne Gesamtprüfung
wird nicht behauptet. Die FEM-Platte bleibt wie vereinbart ein späteres Beispiel.


## 02.11.2026 – Volumen-FEM, Flächenanschlüsse und flexible MKS

Fragestellung: Wie lassen sich assoziative CAD-Kraftübertragungsflächen als
arbeitskonjugierte Schnittstellen einer lokalen Modellreduktion verwenden,
ohne bei Änderungen oder starren Anbauteilen unbemerkt ein anderes Modell
zu simulieren?

Implementiert: ursprünglicher TET4/TET10-Solid-Solver mit konsistenter Masse,
RBE2-Flächenrahmen, Hurty–Craig–Bampton und erster Fläche als FFRF-Referenz.
Keine Übernahme fremden FEM-Quellcodes. NumPy/SciPy/Exudyn laufen lokal als
begrenzte optionale Worker. Transparente CAD-Ansicht mit tatsächlich benannten
Flächen in Rot; keine Ersatzkreise. Speicherung der FE-Netze und ROM-Basen
in einem eigenen, größenbegrenzten COW-Dokumentfeld statt kleiner Metadaten.

Methodische Entscheidung: Grobe lineare TET4-Modelle waren im Biegen zu steif.
Deshalb quadratische TET10 mit exakt integrierter konsistenter Massenmatrix
als Standard. Elementgeometrie bleibt geradkantig; keine Behauptung einer
exakten Zylindergeometrie. CAD-/FE-Flächenabdeckung muss 95–105 % sein.
Getrennte Baugruppennetze werden abgelehnt; Schrauben, Kontakt oder Klebung
müssen explizit modelliert werden. Ein Teil-ROM ersetzt keine fehlenden
starren Anbauteile seines CAD-Verbunds.

Numerische Nachweise: 1-m-Stahlquader, 100-N-Querlast; TET10-Verfeinerung
45/81/425 Knoten ergibt 0,016708/0,018145/0,018841 mm. Analytischer
Euler–Bernoulli-Vergleich 0,019048 mm; ca. 1,08 % Abweichung im feinsten
Referenznetz. Masse 78,5 kg, Kräftegleichgewicht, Nullenergie bei starrer
Translation und konsistente translatorische Masse geprüft. Statischer
ROM-/Voll-FEM-Verschiebungsfehler etwa 10⁻¹¹. Erste drei eingespannten
Eigenfrequenzen: ROM-Abweichung ca. 0,0033 %, 0,0097 %, 0,073 %.
Zusätzlicher Zylinderfall mit 1089 Knoten: polygonale Masse innerhalb 2 %.
Überlappende Anschlüsse, gebrochene Elementindizes und getrennte Netze
werden nicht als gültige Kopplung akzeptiert.

Flexible Dynamik: echte Exudyn-FFRF-Objekte und SuperElementRigid-Marker an
den reduzierten Flächennodes. Freier und getriebener Einzelkörper sowie zwei
flexible Körper geprüft. Gelenkrestfehler werden am verformten Anschluss
ausgewertet, nicht allein am starren Referenzrahmen. MKS-Replay rekonstruiert
die FE-Oberfläche aus modalen Zuständen; die bisherige starre Ankerkorrektur
wird bei flexiblen Ketten nicht angewendet. Die starre CAD-Geometrie wird
weiterhin einmal auf die GPU geladen; nur begrenzte flexible Netze ändern sich.

Native Integration: CAD → FEM → ROM → explizite flexible Blockbindung →
normale MKS-Ausführung mit Gelenkkräften → PNG/AVI → Speichern/Öffnen →
Längenänderung. Der vollständige Befehlsablauf bestand. Nach CAD-Änderung
sind FEM und MKS historisch. Getestetes Beispiel und Messdaten liegen unter
output/fem-rom-native; volle Solverreferenzen unter output/fem-rom.

Grenzen der Evidenz: Referenzfälle sind keine allgemeine Validierung beliebiger
Geometrien, Werkstoffe oder Anregungsbänder. Netz-, Moden- und Zeitschritt-
Konvergenz sind getrennte Untersuchungen. RBE2 versteift Flächen; RBE3 und
verteilte Drucklasten sind andere Modelle. Keine Plastizität, Kontakt,
Schraubenvorspannung, Beulen oder große elastische Deformation. Die
steifigkeitsproportionale Dämpfung entspricht 1 % an der ersten festgehaltenen
Schnittstelleneigenfrequenz. Die Energie enthält Antriebsarbeit nicht als
abgezogenen Bilanzterm; daher keine Energieerhaltungsbehauptung im Antriebsfall.

Für die Dissertation: Flächensemantik und Lastarbeit, vollständige
Bauteilabdeckung, Quellen-/Variantenkonsistenz, kontrollierte Invalidierung,
RBE2/RBE3-Vergleich, Netz-/Modenkonvergenz, dynamische Vergleichsmodelle,
experimentelle Modalanalyse und Kraftmessung untersuchen. Die reine
Integration vorhandener Solver ist noch kein wissenschaftlicher Beitrag.
Eine prüfbare Forschungsleistung kann die konsistente automatische Ableitung
und Aktualisierung inklusive messbarer Fehler- und Gültigkeitskriterien sein.

Reproduzierbare Skripte: tools/verify_fem_rom.py,
tools/verify_flexible_assembly.py, tools/verify_fem_native.py und
tools/verify_fem_rom_ui.py. Grafiken: output/fem-rom/ROM-Verifikation.png
und .pdf. Ausführliche Methodik und Primärquellen: FEM_ROM_WORKFLOW.md.


## 03.11.2026 - Zwei flexible Kranarme mit dynamischer Last

Native Kette aus Fundament, zwei TET10/HCB-ROM-Auslegern, zwei Sinusantrieben und freier 20-kg-Pendellast aufgebaut. 2 s / 4000 Schritte / 1001 Ausgaben. Maximaler Gelenkpositionsrest 3,04e-10 m; Achsrest 1,36e-9. CAD-Flächen als reduzierte Ports; reale FFRF-Dynamik. Geometrieänderungen nach Modellanlage invalidieren korrekt das ROM; deshalb erst alle Schnittstellen erfassen, dann Modelle anlegen/reduzieren. Video und natives Projekt gespeichert.

Frischer Prozess: Öffnen gespeicherter FEM-Matrizen, starre Rückfallberechnung und erneute flexible Bindung bestanden. Große SI-Steifigkeiten erhalten eigene begrenzte Zahlenvalidierung; CAD-Koordinatenschutz bleibt erhalten. Vier gezielte Dokumenttests bestanden. Lokaler Build installiert, keine vollständige CI behauptet.

Grenze: vereinfachter Kran, keine vollständige detaillierte Roboterbaugruppe. Last ist bereits gelenkig befestigt; kein Greifen, Loslassen, Reibkontakt oder Aufsetzen. Netz-/Moden-/Zeitschrittkonvergenz dieses Bewegungsfalls steht aus. Aus unbelastetem elastischem Anfangszustand folgt ein physikalischer Einschwingvorgang. Ausbau und Abnahmekriterien: KRAN_FEM_ROM_STATUS.md.


## 05.11.2026 - ROM-Bibliotheksblock und gemeinsame MKS-Bauteilhistorie

Gemeinsame Bedienung erweitert. Ein ROM-Bibliothekseintrag ordnet vorhandene CAD-Körper zu, zeigt Reduktionsgrad und Quellenstatus, bindet flexible Modelle und öffnet deren FEM-Ergebnis. Die zeitabhängige Bauteilansicht übernimmt bei aktuellem Baugruppenlauf dessen modale Koordinaten und Referenzbewegung. Interpolation der Koordinaten erfolgt linear und die Referenzrotation über SLERP. Historische MKS-Ergebnisse werden ausgeschlossen. Monotone endliche Zeiten und zum ROM passende Koordinatendimensionen werden geprüft. Die UI zeigt die gewählte Ergebnisquelle. Zusammenfassungen und verfügbare Historien werden nach Dokumentrevision zwischengespeichert.

Teilebibliothek um ein unabhängig konstruiertes Hersteller-Montagemodell 17HS19-2004S1 erweitert. Öffentliche Datenblattmaße, D-Welle und Montagebohrungen. 390 g über eine äquivalente homogene Dichte abgebildet, Hüllenträgheit als Modellannahme. Analytisches Volumen 85760,5489 mm³ gegenüber nativer Geometrie 85760,5006 mm³. Montage an einer Platte geprüft. Eine Motorfamilie in der CAD-Bibliothek ist von einer elektrischen bzw. drehmomentbegrenzten Antriebsmodellierung zu unterscheiden.

Sechsachsarm an den Fundamentbefestigungen mit vier ISO-Scheiben und M16×80-Bolzen samt Durchgängen aufgebaut. Acht neue Hardwarekörper ausdrücklich auf Stahl gesetzt, Gesamtmasse 706,813 g. Vorhandener achtsekündiger Bewegungsablauf erneut gerechnet. Größter Positionsrest 2,181e-9 m, Achsrest 1,386e-8. Diese Werte betreffen Zwangsbedingungen. Sie validieren keine Tragfähigkeit oder vollständige flexible Detailbaugruppe. Kurze freie/getriebene FFRF-Referenzen sowie zwei flexible Körper erneut bestanden.

Offene Forschungs-/Entwicklungsaufgabe bleibt die vollständige Domänenabdeckung der detaillierten Armgruppen einschließlich Verbindungen und starrer Anbauteile. Der bisherige automatische Volumenmesher verarbeitet primitive vollständige Einzelkörper. Die Datenübernahme darf keine fehlenden Motor-/Schraubenmassen verdecken. Reproduzierbare Skripte und Berichte stehen unter tools sowie docs/10dercraft/evidence/robot-rom. Methodische und technische Grenzen stehen in ROBOTER_ROM_WORKFLOW_STATUS.md.


### Greifermotor und Adaptermontage vom 05.11.2026

Hersteller-Montagemodell des NEMA17 auf nativer 4-mm-Aluminiumplatte am vorhandenen Greifer montiert. Wellenfreiraum, Zentrierbohrung, vier M3-Befestigungen und vier M4-Adapterbefestigungen mit Scheiben hergestellt. Freiräume für bestehende Führungen und Deckelschraubenköpfe ausgespart. Zwölf zusätzliche Normteilkomponenten aus der normalen Bibliothek, explizite Stahlzuweisung und starre CAD-Verbindungen. Geometrische Prüfung der nicht-gewindetragenden Passflächen bestanden. Der aktualisierte achtsekündige Sechsachslauf mit zusätzlicher starrer Motormasse ergab maximalen Positionsrest 2,168e-9 m. Fingertransmission, elektrische Motor-/Reglerdynamik, Festigkeit und flexible Detailarm-ROMs bleiben offen. Projekt und Berichte unter output/robot-gripper-motor sowie docs/10dercraft/evidence/robot-rom.


### Zusaetzliche Startpruefung vom 05.11.2026

Eine weitere isolierte Fensterstartpruefung blieb beim Grafikstart ohne erste UI-Antwort. Der Versuch wurde abgebrochen und die eigenen Pruefprozesse beendet. Dies ist eine offene Start-/Grafikdiagnose, kein bestandener Cold-Open-Nachweis. Vorherige Bibliotheks-, ROM- und native Montagepruefungen sind separat protokolliert. Eine vollstaendige CI wurde in diesem Durchlauf nicht ausgefuehrt.


## 05.11.2026 - Christbaum-Baugruppe und GitHub-Zwischenstand

34 vom Benutzer gelieferte STL-Druckteile anhand der Montagepositionen aus der zugehoerigen SCAD-Datei in native Komponenten eingeordnet. 23 radiale Schubgelenke, eine abhebbare Spitze, starre Etagen und eine Bewegungsstudie aufgebaut. Vor der spaeteren Testpause wurden 72 Positionen mit maximal 3,013e-12 mm Schwerpunkt-Verschiebungsfehler und 34 geschlossene Netze nachgewiesen. Das ist ein Montage-/Importnachweis. Parametrische BRep-Rekonstruktion, physischer Probedruck und vollstaendige Kollisionspruefung stehen aus.

Die vollstaendige Abschlusspruefung passierte die statischen Gates und stoppte bei zwei Clippy-Hinweisen zu konstanten Dreiergruppen. Die Aufrufe wurden auf as_chunks::<3> umgestellt. Der Benutzer ordnete vor Veroeffentlichung an, fuer diesen Zwischenstand keine weiteren Tests auszufuehren. Danach wurden keine weiteren Testlaeufe gestartet. Die korrigierten Stellen und portablen Skriptpfade wurden nicht erneut geprueft. Details in ZWISCHENSTAND_2026-11-05.md.


## 05.11.2026 - Rohrprofil-Entwurf und editierbare ROM-Einstellungen

Benutzer legt grossen Arm mit 8 kg Nutzlast und Metallfertigung fest. Separate native Variante mit 160x8-mm- und 140x6-mm-Rohrquerschnitten aufgebaut. Durchgehende Bohrungen und veroeffentlichte Calcpad-Wandstaerkenreferenzen treiben CAD. Bestehende Varianten erhalten Rohrformeln. STEP-Einzelrohre und native Stueckliste ausgegeben. Achsverbindungen, Lager-/Antriebsauslegung und weitere Ueberlappungen bleiben offen. Details in ROBOTER_FERTIGUNGSENTWURF.md.

ROM-Einstellungen lassen sich aus MKS und FEM erneut laden. Importnetz, Material, Elementordnung, Schnittstellenreihenfolge und Lasten werden uebernommen. Momenteneingabe hinzugefuegt. Die Balkenvorpruefung verwendet Kreisringflaeche und -traegheitsmoment bei vorhandenen Wandreferenzen und verweigert erkennbare CAD-Volumenabweichungen. Das ersetzt keine Volumen-FEM. Die Anwendung wurde gebaut. Gemaess Benutzeranweisung keine anschliessenden Testreihen.


### Normale Projektberechnung der Rohrprofilvariante

Achtsekundiger Starrkoerper-MKS-Lauf mit 501 gespeicherten Zeitpunkten und allen sechs Quellen berechnet. Maximaler Gelenkpositionsrest 1,971e-9 m. Der Wert betrifft Zwangsbedingungen und ist kein Festigkeitsnachweis. Anschliessend Balken-FEM-Vorberechnung mit Kreisringquerschnitten und gesperrten Antrieben ausgefuehrt. TCP-Durchbiegung 0,113024 mm bei 78,48 N, ohne Eigengewicht und Gelenknachgiebigkeit. Beide Rohre haben L/D <10, daher Schubverformung und Anschlusssteifigkeit fuer die Auslegung weiter erforderlich. Keine flexible Rueckkopplung dieser Balkenrechnung in MKS. Die realen Modellberechnungen sind von den pausierten Testreihen getrennt. Kraftverlaeufe als CSV exportiert. Aktuelle native Projektdatei enthaelt die Berechnungsergebnisse.
