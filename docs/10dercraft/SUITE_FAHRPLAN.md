# Nächste Entwicklungsschritte der Engineering-Suite

Ausbau vom 22.10.2026: Projektschnittstellen mit Körper-/Occurrence-Bezug,
parametrischen Positionen, Punkt-/Rahmen-/Flächenverträgen und statischer
Resultantenweiterleitung sind implementiert. Definitionen und Ergebnisstände
werden gespeichert; die Blockansicht zeigt die deklarierten Lastpfade.
[Bedienung, Rechenmodell und Grenzen](MECHANISCHE_SCHNITTSTELLEN.md).
Automatische Starrgruppen, dynamische MKS und FEM-Lastverteilung sind weiterhin offen.

Stand 21.10.2026. Ergänzt den Gesamtplan in PLAN.md; ersetzt ihn nicht.
Leitlinie: dieselbe Konstruktion, benannte Varianten und nachvollziehbare Übergaben
zwischen lokaler Berechnung, CAD, Simulation und Rückführung.

## Aktueller Ausbau: Varianten und Aktualisierung

Die Start-Variantenansicht zeigt für Projekte mit gespeicherten Berechnungsvarianten
oder Zahnradentwürfen jetzt deren reale Projektdaten. Sie bietet dieselbe Auswahl
und explizite Referenzübernahme wie die Parameterseite. Die bisherigen Roboterdemo-
Optionen bleiben für Projekte ohne diese Daten verfügbar.

Im Zahnradwerkzeug:

1. **Entwurf** wählt einen gespeicherten Eintrag oder **Neuer Entwurf / Kopie**.
2. **Name** benennt den Entwurf. Alte Projekte bekommen passende Anzeigenamen.
3. **Gespeicherter Stand** zeigt geänderte Berechnungsergebnisse; die CAD-Verbindung
   meldet unter anderem abweichende Definitionen oder ein gelöschtes Zahnrad.
4. **Berechnen** bewertet die eingegebenen Ausdrücke. **OK** aktualisiert bei einem
   ausgewählten Entwurf dessen vorhandenen Datensatz. Mit aktivierter CAD-Geometrie
   wird das verknüpfte Feature an derselben Stelle der Timeline neu aufgebaut.
5. **Ansicht → Vergleich** vergleicht die gespeicherten Ergebnisstände zweier Entwürfe.
   Unterschiedliche Einheiten werden nicht als gleichartige Zahlen subtrahiert.

Projektvarianten und Zahnradentwürfe haben verschiedene Aufgaben: Eine Projektvariante
speichert das lokale Berechnungsblatt und seine Definitionen. Ein Zahnradentwurf
speichert seine Eingabeausdrücke, das Auslegungsprotokoll und optional die CAD-Verknüpfung.
Gemeinsam genutzte `ref_...`-Parameter bleiben gemeinsam: Zwei Entwürfe sind dadurch
keine voneinander isolierten Kopien des gesamten Parametergraphen. Der Standardvergleich
benutzt deshalb die gespeicherten Ergebnisstände; ein Vergleich im aktuellen
Parameterkontext ist über den Engine-Befehl zusätzlich möglich.

Neue registrierte Befehle: `gear.review`, `gear.update`, `gear.compare`.
`gear.update` arbeitet auf einem temporären Dokument und übernimmt den erfolgreichen
Stand gemeinsam. Ein gelöschtes verknüpftes Feature wird nicht stillschweigend ersetzt.
Eine ausdrücklich angeforderte Aktualisierung kann manuelle CAD-Eingaben überschreiben;
der Dialog zeigt vorher eine abweichende Definition an. Eine unveränderte Feature-ID
ist noch keine Zusage, dass jede einzelne Fläche bei Zahnzahlwechsel identisch bleibt.

## Ausbaufolge mit überprüfbaren Ergebnissen

| Schritt | Ergebnis | Abnahme |
| --- | --- | --- |
| 1. Varianten durchgängig machen | Reale Projektauswahl, Entwurfsvergleich, Änderungsprüfung und Aktualisierung im vorhandenen Feature | Lokales Calcpad → Referenzübernahme → CAD-Neuaufbau → Vergleich → Speichern/Laden; Fehler lassen Ausgangsstand bestehen |
| 2. Exakte Schräg-/Pfeilverzahnung | Analytische Schraubkurven und Schraubflächen, korrekte Normalmodul-Zahnhöhen, persistente native Körper | Funktions-/Ableitungsvergleich, geschlossene Topologie, Wiederladen, Parameterwechsel; keine angenäherten Ersatzflanken |
| 3. Wiederverwendbares Zahnradpaar | Gegenrad, Montage, Phasenlage und Bewegungsbeziehung als normale Baugruppe | Eingriff und Interferenz über eine Periode; Drehrichtungs-/Übersetzungsprüfung; Varianten ohne doppelte Körper |
| 4. Mechanische Schnittstellen | Punkt, Achse und Fläche mit Bezugssystem und definierter Last-/Momentenübertragung | Gemeinsamer Lastangriff nach Geometrieänderung; Kraft- und Momentenbilanz an einfachem Referenzfall |
| 5. Baugruppe → starre MKS | Starre Verbände, Gelenke, Masse/Trägheit und Antrieb als verbundenes Blockmodell | Zunächst Pendel bzw. Zweikörpermechanismus, dann Planetenstufe; Vergleich mit analytischem Fall und Erhaltungssätzen |
| 6. MKS → FEM/ROM → Rückführung | Gespeicherter Lastfall, Randbedingungen, Verformung und Gültigkeitsbereich des Ergebnisses | Erst statischer Referenzkörper; anschließend nachvollziehbar reduzierte flexible Komponente; keine Gleichsetzung mit dem heutigen Endmassenschwinger |
| 7. Variantenstudie und Optimierung | Ergebnisvergleich je Geometrie-/Solverstand, Nebenbedingungen und nachvollziehbarer Iterationsverlauf | Reproduzierbarer kleiner Entwurfsraum gegen vollständige Enumeration, danach Optimierer; veraltete Ergebnisse sperren |
| 8. Regler/SIL und reale Geräte | Getrennte Signal- und mechanische Schnittstellen, wiederholbare Controllerläufe | Erst Simulation, anschließend klar abgegrenzter Hardwareadapter mit Gerätefreigabe und Abschaltkonzept |

Die bestehende Forschungsumgebung unter `simulation/` liefert bereits unabhängige
Referenzmodelle für Trägerdimensionierung und einen Exudyn-Endmassenschwinger.
Sie ist noch keine automatisch aus der CAD-Baugruppe abgeleitete MKS oder FEM-ROM.
Die neuen Variantenfunktionen verändern das nicht. Schnecken-/Kegelradflanken,
Tragfähigkeitsnachweise und Fertigungsdetails folgen eigenen Aufgaben gemäß
ZAHNRADWERKZEUG_RECHERCHE.md.

## Nachweise und Dissertation

### Ergänzung 22.10.2026: native Baugruppe als MKS-Struktur

Schritt 5 ist jetzt strukturell begonnen: `mks.model` reduziert native starre
Gelenke und Komponenten zu Blöcken, übernimmt bewegliche Gelenke sowie exakte
Einbaupfade und ordnet Projekt-Kraftschnittstellen zu. Die MKS-Systemansicht
zeigt Fundament, freie Blöcke und Gelenkverbindungen. Mehrdeutige Instanzen und
noch nicht übersetzte Kopplungen erhalten Diagnosehinweise.

Noch offen in Schritt 5: Trägheitstensoren, SI-Solverübergabe, Zeitintegration
und dynamische Referenzfälle. Anleitung und nächste Abnahmebedingungen:
[MKS_BAUGRUPPENMODELL.md](MKS_BAUGRUPPENMODELL.md).
Gezielter Nachweis: `tools/verify_native_mks.py`, Rohdaten `output/native-mks/`.

Für jede Übergabe werden künftig Eingabesnapshot, Einheiten, Geometrie-/Variantenstand,
Solverkonfiguration und Herkunft festgehalten. Fehlgeschlagene Versuche und Gründe
für eine Ablehnung gehören ebenso zum Versuchsprotokoll wie erfolgreiche Ergebnisse.
Der heutige Ausbau prüft die erste Strecke dieser Kette; eine allgemeine Gültigkeits-
verwaltung für sämtliche FEM-/MKS-Ergebnisse ist damit noch nicht implementiert.

Gezielte Prüfung dieses Ausbaus: `tools/verify_suite_variants.py`.
Rohdaten und tatsächliche UI-Aufnahmen: `output/suite-variants/`.
Die breite alte Testsuite bleibt pausiert; weitere kurze Prüfungen erfolgen je
neuem Funktionsschritt. Forschungsnotizen stehen in DISSERTATION_FORSCHUNGSLOG.md.


### Ergaenzung 22.10.2026: Masse, Schwerpunkt und Traegheit

Schritt 5 besitzt jetzt einen separaten nativen Auswertungsschritt
mks.mass_properties. Materialabhaengige Masseneigenschaften werden in SI
je reduziertem Block berechnet; unvollstaendige Koerper und angenommene
Dichten bleiben sichtbar. Quader-/Zylinder-, Rotations- und Steinerreferenzen
sowie Aenderungsfaelle sind dokumentiert. Anleitung: MKS_MASSENEIGENSCHAFTEN.md.
Offen bleiben Exudyn-Uebergabe, Zeitintegration und dynamische Referenzrechnung.


### Ergaenzung 22.10.2026: CAD-abgeleitete Zeitintegration und Bewegungsantrieb

Exudyn-Adapter fuer offene starre Dreh-/Schubgelenkbaeume umgesetzt.
Masseneigenschaften aus CAD, konstante Portlasten, deklarierter Druck und
ideale Kosinusrampen als Winkel-/Laengen-Parameterausdruecke. Gespeicherte
Jobs/Ergebnisse mit CAD-Eingabestand, Historisch-Anzeige und lesende
Replay-Transformationen. Schwerpunktbahn mit Zeitregler im MKS-Fenster.
Abnahme: nichtlineares physikalisches Pendel, gekoppelte Kette, Schieber,
Portlasten, Berechnungsvarianten Basis/Lang, vorgegebene Bewegung und
Manipulations-/Gueltigkeitspruefungen. Anleitung: MKS_DYNAMIK.md.

Naechste noch offene Produktbausteine: native Zahnradkopplung, geschlossene
Mechanismen, echte 3D-Ergebniswiedergabe, flexible FEM/ROM und Antriebs-/
Reglermodelle fuer SIL. Kein automatisches Entfernen ununterstuetzter Bedingungen.


### Fortsetzung 22.10.2026: 3D-Ergebnisfenster und MCP-Bildexport

Die erste 3D-Ergebnisdarstellung ist inzwischen umgesetzt. Native Koerper und
Portachsen werden anhand der berechneten Transformationen dargestellt;
Konstruktion und Gelenke bleiben unveraendert. PNG/GIF-Export benutzt
denselben Engine-Szenenaufbau. Physikalischer Zeitregler mit Tempofaktor,
gezielte UI- und Dateidecoderpruefungen vorhanden. Abbildungen unter
output/assembly-dynamics/ und output/assembly-dynamics-export/.

23.10.2026: Gemeinsames Roboterbeispiel für Parameter/CAD/MKS/Balken-FEM
mit zwei echten Radiusvarianten umgesetzt und gezielt abgenommen.
MKS-Endpunkt gegen native Gelenkkinematik und Balken-FEM gegen geschlossene
Lösungen geprüft. Ergebnisgültigkeit über Änderungen und Wiederladen geprüft.
Keine Volumen-FEM, flexible MKS oder SIL behauptet. Anleitung und nächste
Entwicklungsschritte: ROBOTER_DOMAENEN.md; Forschungsdaten output/robot-domains.
