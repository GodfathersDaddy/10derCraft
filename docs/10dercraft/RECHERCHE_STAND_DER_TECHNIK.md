# Stand der Technik und Komponentenwahl für 10derCraft

Recherche: 21.10.2026. Bezug: Masterarbeit Zehnder und Produktplan in `PLAN.md`. Dies ist eine vertiefte erste Literatur- und Werkzeugrecherche mit öffentlich zugänglichen Primärquellen, keine abgeschlossene systematische Literaturübersicht und kein Nachweis weltweiter Neuheit. Bibliothek, Originale, Downloadstatus und Prüfsummen stehen in [Literatur/INDEX.md](Literatur/INDEX.md) und [Manifest](Literatur/quellen_manifest.json).

## 1. Ergebnis für deine Idee

Eine Umgebung von parametrischem CAD über starre und flexible Mehrkörpersimulation bis zur Optimierung ist technisch plausibel. Viele ihrer Bausteine und auch vergleichbare durchgängige Abläufe existieren bereits. Die Dissertation sollte deshalb eine nachweisbare Verbesserung einer klar begrenzten Entwicklungsmethodik untersuchen. Eine Sammlung von Solvern mit zusätzlichen Reitern allein begründet noch keinen wissenschaftlichen Beitrag.

Der stärkste vorläufige Schwerpunkt ist **änderungsrobuste, variantenfähige Ableitung und Aktualisierung von Simulationsmodellen aus parametrischen Baugruppen mit expliziten mechanischen Schnittstellen**. Die Software dient als Forschungsinstrument. Zu beweisen wären geringerer Änderungsaufwand und weniger Übertragungsfehler bei kontrollierter Ergebnisgenauigkeit. Diese Aussage ist unser Forschungsvorschlag, kein Ergebnis der Quellen.

Die Bezeichnung „durchgängig“ muss messbar werden: Welche Informationen bleiben erhalten, welche werden neu erzeugt, welche Entscheidungen bleiben beim Anwender? Besonders interessant sind Änderungen von Flanschgeometrie, Werkzeugvariante, Material, Blechdicke und Modelltreue. Hydraulik bleibt außerhalb des ersten Nachweises.

## 2. Was nachweislich bereits existiert

| Teil deiner Idee | Vorhandener Ansatz | Konsequenz für uns |
|---|---|---|
| CAD-Baugruppe in mechanisches Blockmodell übertragen | Simscape `smimport` rekonstruiert Körper, Gelenke und Zwangsbedingungen aus geeigneten CAD-/URDF-Daten [S22](https://www.mathworks.com/help/sm/cad-import.html) | CAD → Blockmodell als solches ist keine neue Methode. |
| CAD → FEM → reduzierte flexible Körper | Simscape Reduced Order Flexible Solid verwendet Schnittstellenrahmen und reduzierte Massen-/Steifigkeitsmatrizen; Craig-Bampton wird als Erzeugungsverfahren beschrieben [S23](https://www.mathworks.com/help/sm/ref/reducedorderflexiblesolid.html) | Genauigkeit und Übertragbarkeit der Schnittstellen nach Änderungen untersuchen. |
| Skriptgesteuerte starre und flexible MKS | Exudyn enthält Körper, Gelenke, Lasten, Sensoren und numerische Lösungsverfahren [S01](https://doi.org/10.1007/s11044-023-09937-1) | Keinen eigenen allgemeinen Dynamiksolver entwickeln. |
| CAD-Anbindung eines offenen MKS-Kerns | Chrono::SolidWorks exportiert Mechanismen aus SolidWorks [S15](https://api.projectchrono.org/introduction_chrono_solidworks.html) | Direkter Vergleich für Modellableitung; die CAD-Anwendung selbst bleibt dabei kommerziell. |
| Parameteraustausch CAD ↔ Systemsimulation | Hällqvist et al. koppeln CATIA und Systemsimulation über SSP/FMI [S02](https://doi.org/10.3384/ecp2118137) | Sehr naher Vorläufer; im Exposé ausdrücklich abgrenzen. |
| Wiederholbare gekoppelte Engineering-Workflows | RCE integriert Disziplinwerkzeuge und speichert erzeugte Daten zur späteren Auswertung [S05](https://arxiv.org/abs/1908.03461) | Workflow-Automatisierung und Provenienz nicht als eigene Erfindung beanspruchen. |
| Zentrale parametrische Geometrie für mehrere Analysen | TiGL erzeugt aus CPACS Geometrien für Flugzeugentwurf und Optimierung [S29](https://arxiv.org/abs/1810.10795) | Zentrales Datenmodell hat Vorläufer; unser Anwendungsgebiet und Änderungsvertrag müssen konkreter sein. |
| Gekoppelte Optimierung | OpenMDAO behandelt gekoppelte Modelle und Gesamtgradienten [S03](https://doi.org/10.1007/s00158-019-02211-z) | Optimierungsframework übernehmen; wissenschaftliche Frage auf die Simulationskette richten. |
| Strukturelle Topologieoptimierung | DTU TopOpt dokumentiert ein etabliertes SIMP-Lehrbeispiel [S04](https://doi.org/10.1007/s00158-010-0594-7) | Von parametrischer Dimensionierung trennen; deutlich größerer zusätzlicher Forschungsumfang. |
| Austausch zusammengesetzter Simulationsmodelle | FMI und SSP beschreiben Modellaustausch bzw. Systemstruktur und Parametrierung [S08](https://fmi-standard.org/docs/3.0.2/), [S09](https://ssp-standard.org/docs/2.0.1/) | Standardadapter statt eines ausschließlich proprietären Austauschformats vorsehen. |
| Dokumentation von Simulationsentscheidungen | SSP Traceability und Arbeiten zu Simulation Credibility behandeln die Nachvollziehbarkeit von Entscheidungen und Simulationen [S27](https://modelica.github.io/ssp-ls-traceability/main/), [S28](https://doi.org/10.3384/ecp193171) | Evidenzmodell anschlussfähig gestalten; reine Protokollierung reicht als Neuheit nicht. |

### Wichtige Einordnung des nahen Vorläufers S02

Die Arbeit behandelt standardisierten Parametertransfer zwischen CAD und Systemsimulation. Im Schlussabschnitt benennt sie Grenzen beim Austausch strukturverändernder Parameter über die betrachteten FMUs. Das ist ein Ansatzpunkt für unseren Vergleich, jedoch keine allgemeine Aussage über alle heutigen FMI-Versionen oder Werkzeuge. Unser möglicher Zusatz liegt bei expliziten mechanischen Flächen-/Rahmenschnittstellen, Strukturvarianten, FEM-/ROM-Neuerzeugung und überprüfter Änderungsbehandlung. Ob diese Kombination wissenschaftlich neu genug ist, bleibt gezielt weiter zu recherchieren. [S02, PDF-S. 11](https://ecp.ep.liu.se/index.php/modelica/article/download/180/140/83)

## 3. Was diese Recherche noch nicht als fertige Lösung belegt

In der ausgewerteten Auswahl ist keine fertige Lösung nachgewiesen, die exakt die gewünschte SolidCraft-Oberfläche mit allen folgenden Eigenschaften verbindet:

- Montagevarianten schon im parametrischen CAD, einschließlich Kompatibilitätsregeln.
- Automatische Bildung starrer Bewegungseinheiten mit denselben äußeren Anschlüssen für starre und flexible Repräsentationen.
- Explizite Trennung von Anschlussrahmen, belasteter CAD-Geometrie und numerischer Lastverteilung.
- Geprüfte Aktualisierung dieser Zuordnungen nach Topologie- und Variantenänderungen.
- Abhängigkeitssensitive Neuerzeugung von Netz, FEM, ROM und MKS mit Rückführung der Entwurfsziele.
- Gemeinsame Bedienung, reproduzierbare Ausführung und Zugriff über MCP.

Das ist eine **offene Nachweislücke in dieser Recherche**, keine belegte Marktlücke. Es fehlen noch gezielte Vergleiche mit kommerziellen PLM-/SPDM-Systemen, CAD-CAE-Associativity, Simulation Intent und wissenschaftlichen Arbeiten zu persistenten geometrischen Referenzen und variantenbasierter Modellgenerierung.

## 4. Übertragung deiner Masterarbeit

Die lokale Masterarbeit ist S00. Die Seitenzahlen hier sind PDF-Seiten, nicht die abweichenden gedruckten Seitenzahlen.

| Ausgangspunkt der Masterarbeit | Erweiterung in der Software | Wissenschaftlich zu untersuchen |
|---|---|---|
| Method-Model-Tool-Kette, PDF 32–33 | Expliziter Abhängigkeitsgraph von Anforderung bis Ergebnis | Welche Änderungen erzwingen welche Neuerzeugungen? |
| Skelett und zentrales Notebook, PDF 41–44 | Einheitenbehaftete Parameter und Berechnungsabhängigkeiten | Widerspruchsfreiheit und zyklische Abhängigkeiten |
| CAD-Übertragung, PDF 47–50 | Körper, Massen, Trägheiten, Rahmen, Gelenke und Grenzen gemeinsam versionieren | Verlorene oder veraltete Gelenkgrenzen erkennen |
| Starre Zusammenfassung, benannte Anschlüsse, PDF 58–61 | Modul mit stabilen Anschluss-IDs | Erhalt der Anschlüsse bei Geometrie- und Variantenänderung |
| Remote Points auf Schraubflächen und Reduktion, PDF 63–70 | Separate Angaben für Rahmen, Flächenselektion, Kopplungsart und aktive Freiheitsgrade | Einfluss der Flächenkopplung auf Steifigkeit und Ergebnisfehler |
| Varianten starr/flexibel, PDF 87–95 | Gleiche äußere Anschlüsse, unterschiedliche interne Repräsentationen | Zulässiger Austausch bei festem Fehlerbudget |
| Experimentvergleich, PDF 72–75 | Wiederholbare Posen und Messdatenzuordnung | Validierung des TCP-Verformungsfehlers |

Die in der Masterarbeit verwendeten Netzweiten, Modenzahlen und Filterkonstanten sind fallspezifisch. Sie werden keine allgemeinen Produktstandards. Benannte „DH“-Anschlüsse übernehmen wir als explizite Rahmen; die dortige Achsenkonvention wird nicht als universelle DH-Konvention vorausgesetzt.

## 5. Empfohlene Komponenten und Alternativen

Dies ist eine technische Vorauswahl, keine Behauptung bereits integrierter oder validierter Backend-Funktionen. API, Distribution und Lizenznachweise werden für die tatsächlich eingesetzte Version festgehalten.

| Aufgabe | Erste Wahl | Warum passend | Grenze / nächster Eignungsnachweis |
|---|---|---|---|
| CAD, Oberfläche, Kommandos, MCP | Bestehender SolidCraft-Fork | Gewünschte native Oberfläche und bestehende parametrische Konstruktion | Persistenz der neuen Engineering-Daten und stabile Geometriereferenzen fehlen noch als belastbarer Nachweis. |
| Starre und flexible MKS | **Exudyn** | Python-Ansteuerung, flexible Körper, CMS und vorhandene FEM-Anbindung [S01, S11, S12] | Zunächst Einarm-/Zweigelenkmodell mit bekannten Reaktionen; danach ROM-Körper. Kein fertiger SolidCraft-Importer. |
| FEM und erste ROM-Erzeugung | **Netgen/NGSolve** | Skriptbare FEM, Elastizität und Anschluss an Exudyn [S12, S13] | CAD-/Flächenidentitäten und gewichtete Randkopplung explizit implementieren. Separater Prozess. |
| Parameteroptimierung | **SciPy** | Lokale beschränkte Optimierung und globale Verfahren vorhanden [S18] | Diskrete Montagevariante zunächst außen enumerieren; keine glatte Ableitung über Topologieänderungen voraussetzen. |
| Technisches Rechenblatt | **SymPy + Pint**, numerisch NumPy/SciPy | Symbolik und physikalische Einheiten [S19, S20] | Rechenblatt benötigt eigene Abhängigkeitsverwaltung, sichere Ausdruckssprache und Fehlerdarstellung. |
| Größere gekoppelte Optimierung | **OpenMDAO später** | Strukturierte Disziplinmodelle, Gesamtgradienten und gekoppelte Gleichungen [S03, S17] | CAD-Neuvernetzung ist oft nicht glatt; Gradientenkonsistenz muss nachgewiesen werden. |
| Co-Simulation / SIL | **FMI + SSP**, OMSimulator evaluieren | Austausch, Systemstruktur und FMU-Orchestrierung [S08, S09, S16] | Solverkopplung, Zeitsteuerung und Stabilität prüfen; kein automatischer Echtzeitnachweis. |
| Zweiter MKS-Solver | Chrono | CAD-Anbindung und alternative MKS-Plattform [S15] | Zusätzlicher Adapteraufwand; erst bei konkretem Funktionsbedarf oder für Kreuzvalidierung. |
| Alternativer Mesher | Gmsh | Gut dokumentierte Geometrie-/Netzschnittstellen [S14] | GPL-Software; in diesem Projekt als externer Werkzeugprozess behandeln. CAD-Kernel-Komponenten gesondert prüfen. |
| FEM-Workflow-Vergleich | PrePoMax / CalculiX | Praktische Vorbereitung von FEM-Modellen, Flächen-/Netzauswahl [S21] | Referenz für Bedienablauf und unabhängige Vergleichsrechnung; kein Ersatz für das gemeinsame Datenmodell. |
| Workflow-Vergleich | RCE | Bereits vorhandene Werkzeugorchestrierung [S05, S24] | Parallelbetrieb nur bei verteilten Jobs nötig; keinen zweiten vollständigen Workflow-Editor in die erste Version einbauen. |

**Reihenfolge:** Exudyn + SciPy zuerst; danach Netgen/NGSolve + ROM. OpenMDAO und FMI/SIL nach einem funktionierenden, gemessenen CAD→MKS→FEM→Rückführungsfall. Topologieoptimierung nach stabiler parametrischer Optimierung.

### Lizenz- und Architekturentscheidung

SolidCrafts lokale Projektregeln schließen die Übernahme bzw. Verlinkung von GPL-/LGPL-/AGPL-Quellcode in die Anwendung aus. Numerische Werkzeuge werden deshalb über einen dokumentierten Prozess- und Dateivertrag angebunden. Es wurde in dieser Recherche kein entsprechender Solver- oder CAD-Kernel-Quellcode übernommen.

NGSolve nennt LGPL, Gmsh GPL. Exudyn hat eine eigene BSD-artige Hauptlizenz und zusätzliche Fremdkomponentenhinweise; es sollte nicht pauschal als MIT oder Apache bezeichnet werden. Chronos Hauptlizenz ist BSD-artig. Diese Feststellungen ersetzen keine Prüfung der konkret ausgelieferten Pakete. Ein separater Prozess hebt Lizenzpflichten bei der Verteilung nicht automatisch auf. [NGSolve](https://ngsolve.org/), [Gmsh](https://gmsh.info/), [Exudyn-Lizenz S31](https://raw.githubusercontent.com/jgerstmayr/EXUDYN/master/LICENSE.txt), [Chrono-Lizenz S32](https://raw.githubusercontent.com/projectchrono/chrono/main/LICENSE)

## 6. Mechanische Schnittstellen als gemeinsamer Vertrag

Empfohlene eigene Spezifikation: `MechanicalPort` enthält eine stabile ID, Modul-ID, Rahmenpose, Einheitensystem, Flächenträger, Kopplungsmodell, aktive Freiheitsgrade und Herkunft. Die Pose wird relativ zum Modul gespeichert; Netznummern werden nicht als dauerhafte CAD-Identitäten verwendet.

| Ebene | Beispiele | Warum separat? |
|---|---|---|
| Referenzrahmen | Ursprung auf Gelenkachse, Orientierung, Transformation | Definiert den Ort für Gelenk, Messung und resultierendes Moment. |
| Geometrischer Träger | Punkt, Fläche, Schraubflächen-Gruppe | Beschreibt, wo die Last tatsächlich eingeleitet wird. |
| Numerische Kopplung | Einzelkraft, Rahmenwrench, starre oder verteilte Remote-Kopplung, Druck | Gleiche resultierende Kraft kann eine andere lokale Verformung erzeugen. |
| Modellrepräsentation | starr, Balken, FEM, ROM | Anschlussbedeutung soll erhalten bleiben, interner Zustandsraum darf wechseln. |

Diese Trennung passt zu den in Exudyn beschriebenen Randknotenmengen und Gewichtungen und zu den expliziten ROM-Schnittstellenrahmen bei Simscape. Eine belastbare Übersetzung zwischen beiden ist zusätzliche Arbeit. Nicht jede reduzierte Matrixdarstellung ist ohne Transformation austauschbar. [S12](https://exudyn.readthedocs.io/en/latest/docs/manual/theoryModal.html), [S23](https://www.mathworks.com/help/sm/ref/reducedorderflexiblesolid.html)

Eigene Akzeptanzregeln: Kräfte und Momente in denselben Rahmen transformieren; Resultierende und virtuelle Arbeit prüfen; unklare Flächenselektionen nach Geometrieänderungen sperren. Bei einem Momententransport gehört der Hebelarmterm dazu. Druck ist eine flächenbezogene Last, kein vollständiger Gelenkanschluss. Eine reine Punktkraft trägt kein frei vorgebbares Moment.

Im gemeinsamen Blockeditor bleiben **physikalische Verbindungen**, **gerichtete Regelsignale** und **Ableitungsabhängigkeiten** unterscheidbar. Eine FEM→ROM-Artefaktkante ist keine mechanische Kraftverbindung. FMI-/SSP-Signale ersetzen nicht automatisch die impliziten Gleichungen eines akausalen mechanischen Netzes; Modelica liefert hierfür ein relevantes semantisches Vorbild. [S07](https://specification.modelica.org/maint/3.7/MLS.pdf), [S08](https://fmi-standard.org/docs/3.0.2/)

## 7. Dissertation: tragfähige Fragestellung und Grenzen

Der bisherige Titel bleibt verwendbar:

**Methodik zur automatisierten, durchgängigen virtuellen Entwicklung mechatronischer Systeme auf Basis parametrischer Konstruktion und modularer Simulationsmodelle**

Für das Exposé empfehle ich eine konkrete Forschungsfrage:

> Wie lassen sich modulare Simulationsrepräsentationen aus parametrischen Baugruppen über mechanische Schnittstellen konsistent ableiten und nach Entwurfsänderungen aktualisieren, sodass der Modellierungsaufwand bei vorgegebener Ergebnisgenauigkeit sinkt?

Vorläufige Beiträge, jeweils noch als Hypothesen:

1. Ein formalisierter Schnittstellenvertrag für CAD, starre MKS und FEM-/ROM-Körper einschließlich Lastverteilung und Variantenkompatibilität.
2. Ein Algorithmus zur Erkennung betroffener Ableitungen und zur geprüften Wiederverwendung unveränderter Modelle nach Änderungen.
3. Ein reproduzierbar evaluierter Entwicklungsablauf, der Zeitaufwand, Übertragungsfehler und Ergebnisabweichungen gemeinsam bewertet.

Das könnte dissertationswürdig sein, wenn eine eigene Methode gegenüber ausreichend nahen Vorarbeiten und starken Baselines einen nachweisbaren Beitrag liefert. Die Betreuung und Promotionsordnung entscheiden letztlich über Annahme und Umfang. Eine neue allgemeine Topologieoptimierung, ein neuer Solver und vollständiges HIL sollten nicht zusätzlich verpflichtende Kernbeiträge werden.

### Evaluationsplan

| Benchmark | Änderung / Versuch | Referenz | Messgröße |
|---|---|---|---|
| B1 Kragträger / Flansch | Dicke, Höhe, Material; Punkt-/Flächenlast | Analytische Lösung im zulässigen Balkenbereich und verfeinerte FEM | Durchbiegung, Energie, Resultierende, Netzkonvergenz |
| B2 Zweiarmmechanismus | Gelenkpose, Werkzeugmasse, Längenänderung | Unabhängig manuell aufgebautes MKS-Modell | Position, Reaktionskräfte, Energie-/Zwangsfehler |
| B3 Flexibles Modul | Varianten von Netz und ROM-Modenzahl | Voll-FEM bzw. ausreichend verfeinerte Dynamikreferenz | Eigenfrequenzen, statische Nachgiebigkeit, TCP-Verlauf |
| B4 Modularer Roboter | Werkzeugwechsel, Flansch-/Topologieänderung | Manueller und vollständig neu erzeugter Ablauf | Aktive Bearbeitungszeit, Rechenzeit, Fehlerzahl, Wiederverwendungsquote |
| B5 Entwurfsoptimierung | Gewicht minimieren, Verformung beschränken | Unabhängige Nachrechnung der besten Kandidaten | Machbarkeit, Masse, Solveraufrufe, Gesamtzeit |
| B6 Physischer Aufbau | Wiederholte Posen und Lastfälle | Gemessene Verformung mit dokumentierter Messunsicherheit | Modellabweichung und Streuung |

Ablationsstudien: ohne stabile Anschluss-IDs; ohne inkrementelle Aktualisierung; ohne Modelltreuewechsel. Damit wird erkennbar, welcher Teil der Methodik den Vorteil erzeugt. Bedienzeit und automatische Rechenzeit getrennt erfassen. Wiederholungen und Stichprobengröße nach Pilotversuchen und Streuung festlegen, keine fiktive statistische Sicherheit behaupten.

Optimierung zunächst als `min Masse(x, Variante)` unter Verformungs-, Spannungs-, Montage- und Fertigungsgrenzen formulieren. Die Maximalverformung entlang einer Bewegung ist eine andere Anforderung als ein einzelner statischer Lastfall. Ein Optimum muss nicht auf der Verformungsgrenze liegen, wenn andere Grenzen aktiv sind. Rohdaten behalten; Filter und Dämpfung als Modellentscheidungen dokumentieren.

## 8. Softwarefahrplan aus der Recherche

1. **Vertrag und Persistenz:** Einheiten, Modul-/Port-IDs, Varianten, Rahmen und CAD-Träger speichern; ungültige Zuordnungen sichtbar machen. Gemeinsamer Blockeditor in der ursprünglichen SolidCraft-Oberfläche.
2. **Kleinster vollständiger Fall:** Ein parametrischer Träger mit Endlast und analytischer Kontrollrechnung; Parameter → Ergebnis → Entwurfsänderung nachvollziehbar speichern.
3. **Exudyn-Adapter:** Versionierter JSON-Vertrag, separater Worker, SI-Einheiten, Zustands-/Fehlermeldungen; ein starrer Mechanismus mit Last- und Bewegungsvorgabe. Keine gleichzeitige unabhängige Vorgabe von Bewegung und Kraft am selben Freiheitsgrad.
4. **FEM-/ROM-Adapter:** Netgen/NGSolve, markierte Anschlussflächen, K/M, Randkopplung und CMS; Modal-/Statikprüfung vor Nutzung in MKS.
5. **Varianten und Änderungen:** Geometrie-, Material-, Netz-, Randbedingungs- und Solverfingerprints. Veraltete ROMs invalidieren; nicht nur anhand gleicher Dateinamen wiederverwenden.
6. **SciPy-Optimierung:** Kleine Zahl kontinuierlicher Parameter, diskrete Varianten außen vergleichen. Abbruch, ungültige Geometrie und fehlgeschlagene Solves als definierte Ergebnisse behandeln.
7. **MCP und Forschungsdaten:** Dieselben registrierten Kommandos wie die GUI; Jobzustände, Resultate, Artefakte und Provenienz abfragen. Zuerst reine Simulationsaufgaben automatisieren.
8. **Später FMI/SIL:** Controller mit fester Abtastzeit und dokumentierter Kopplung. Reale Geräte und Echtzeitanforderungen erhalten einen eigenen Entwicklungsschritt.

Jeder Schritt braucht einen kleinen Ende-zu-Ende-Nachweis. Diese Recherche installiert keine Solver, startet keine pausierten Tests und behauptet keine funktionierende Gesamtintegration.

## 9. Dokumentation und spätere Grafiken

Für jeden Versuch einen Datensatz mit Frage/Hypothese, Quellbezug, Git-Stand, Varianten-ID, Parameterwerten und Einheiten, Geometrie-/Netz-/ROM-Hashes, Solverversionen, Einstellungen, Laufzeiten, Rohdaten, Auswertung und Entscheidung führen. Forschungsfortschritt und Softwarefortschritt getrennt berichten. PROV-O und FAIR sind Referenzen für Herkunft und Wiederverwendbarkeit; SSP Traceability ist für den Simulationsprozess besonders nah. [S10](https://www.w3.org/TR/prov-o/), [S06](https://doi.org/10.1038/sdata.2016.18), [S27](https://modelica.github.io/ssp-ls-traceability/main/)

Geplante eigene Abbildungen: Masterarbeitsprozess vs. automatisierter Prozess; Portvertrag mit Rahmen/Fläche/Kopplung; Varianten- und Abhängigkeitsgraph; Wiederverwendung nach Änderung; Netz-/Modenkonvergenz; Genauigkeit gegen Gesamtaufwand; Masse-Verformung-Paretofront. Fremde Abbildungen nicht ohne Prüfung ihrer Nutzungsrechte übernehmen.

## 10. Rechercheverfahren, Evidenz und offene Arbeit

Die Suche erfolgte über Websuche und direkte Projekt-/Verlagsseiten. Bevorzugt wurden Originalartikel, Standards, institutionelle Veröffentlichungen und offizielle Handbücher. Download und Extraktion liefen separat; ein erfolgreicher Download bedeutet nicht, dass jede Seite vollständig fachlich geprüft wurde. Kernaussagen wurden anhand der sichtbaren Originalquellen und ausgewählter Volltextabschnitte geprüft; große Handbücher sind Nachschlagekorpus.

Suchgruppen: „Exudyn flexible multibody model order reduction“, „CAD SSP FMI interoperability“, „Chrono SolidWorks“, „OpenMDAO derivatives“, „Modelica connector“, „SSP Traceability“, „RCE engineering workflows“, „CPACS TiGL“, „topology optimization 88 lines“, „NotebookLM supported sources“. Exakte Suchanfragen und Lesestatus: [Rechercheprotokoll](Literatur/RECHERCHEPROTOKOLL.md).

Noch gezielt zu ergänzen: persistente topologische Benennung; CAD-CAE-Assoziativität; automatische Starrkörpergruppierung bei Gelenkschleifen; Feature-/Variantenmodelle und Konfigurationsregeln; modellbasierte Simulation Intent; fehlergesteuerte Modelltreue; moderne CMS-Schnittstellenreduktion; Validierungsplanung und Messunsicherheit. Craig/Bampton 1968 ist ein Grundlagenhinweis (DOI 10.2514/3.4741), in dieser Runde wurde kein geprüfter Originalvolltext beschafft. Für diese Bereiche sind auch Datenbanksuchen und Rückwärts-/Vorwärtszitierungen nötig, bevor das Exposé Neuheit beansprucht.

Die Sammlung ist eine erweiterbare Ausgangsbibliothek, nicht „alle jemals benötigten Quellen“. Es wurden keine Paywalls oder Downloadschutzmaßnahmen umgangen. S22/S23 sind im Web geprüft, jedoch lokal nicht als Volltext vorhanden. Quellenangaben aus KI-Antworten vor Übernahme in die Dissertation am Original kontrollieren.
