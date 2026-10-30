# Zahnradwerkzeug: Recherche, Umsetzung und Ausbau

Stand: 21.10.2026. Dieser Bericht trennt vorhandene Funktionen von weiteren Entwicklungsaufgaben.

## Bedienung und aktueller Funktionsumfang

Erweiterung: Gespeicherte Entwürfe sind jetzt auswählbar und benennbar. Ein vorhandener
Entwurf wird mit **OK** aktualisiert; **Neuer Entwurf / Kopie** legt einen neuen Eintrag
an. **Ansicht → Vergleich** vergleicht gespeicherte Ergebnisse. Die Start-Variantenansicht
zeigt reale Berechnungsvarianten und den Stand der gespeicherten Zahnradentwürfe.
Bedienung, Unterschiede zwischen Entwurf und Projektvariante sowie nächste Schritte:
[SUITE_FAHRPLAN.md](SUITE_FAHRPLAN.md).

Im Reiter **CAD**, Register **Körper / SOLID**, Menü **Erstellen / CREATE** befindet sich
**Zahnradwerkzeug**. Alternativ in der Befehlssuche nach diesem Namen suchen.
Das Werkzeug verwendet den normalen SolidCraft-Eingabedialog; es gibt keine dauerhaft
eingeblendete zusätzliche Oberfläche.

1. Zahnradart auswählen. Die sichtbaren Eingabefelder passen sich an.
2. Eingaben als Zahl mit Einheit oder CAD-Ausdruck eingeben, beispielsweise
   `ref_modul`, `ref_z_s`, `ref_b` aus der integrierten Berechnung.
3. **Berechnen** zeigt Bezugsabmessungen, Übersetzung, Kräfte und Geltungsbereich.
   Über dem Ergebnis steht beim Überfahren mit der Maus die jeweilige Formel.
4. Bei Stirn-/Hohlrädern kann **CAD-Geometrie** aktiviert werden. **OK** speichert
   die Auslegung und erstellt gegebenenfalls ein natives Zahnrad-Feature.
5. Das Zahnrad wird durch Doppelklick auf sein Timeline-Feature bearbeitet.
   Seine gespeicherten Ausdrücke treiben bei Parameteränderung die Konstruktion.

Auslegungsprotokolle werden im `.solidcraft`-Projekt gespeichert. Beim erneuten Öffnen
des Werkzeugs stehen die zuletzt gespeicherten Eingaben bereit. Die Ergebnisprotokolle
sind ausdrücklich **Snapshots**; nach Parameteränderungen erneut berechnen. Sie ersetzen
das bereits vorhandene lokale Calcpad-Blatt und dessen bewusste Referenzübernahme nicht.
Das Projektblatt bleibt die zentrale, frei bearbeitbare Mathcad-artige Berechnung.

Für MCP, Skripte und den Kontrollkanal sind dieselben Befehle registriert:
`gear.catalog`, `gear.evaluate`, `gear.design` und `gear.designs`. Beispielsweise
berechnet `gear.evaluate` mit `{"family":"helical","module":"ref_modul",
"teeth":"ref_z_s","mate_teeth":60,"helix_angle":"20 deg"}` eine
Schrägradpaar-Auslegung ohne das Modell zu verändern. `gear.design` speichert
mit `create:false` nur die Auslegung; `create:true` verlangt einen verfügbaren
nativen Generator und wird für noch nicht unterstützte Geometrien abgewiesen.

| Zahnradart | Auslegung im Werkzeug | Native CAD-Geometrie |
| --- | --- | --- |
| Außenverzahntes Stirnrad | Bezugsabmessungen, Gegenrad, Übersetzung, Zahnkräfte | Ja: exakte analytische Evolventen |
| Geradverzahntes Hohlrad | Innenverzahnungsabmessungen, Achsabstand, Übersetzung | Ja: exakte analytische Evolventen |
| Schrägverzahntes Außen-/Hohlrad | Normal-/Stirnmodul, Winkelumrechnung, virtuelle Zahnzahl, Verdrehung, Sprungüberdeckung, Kräfte | Noch offen |
| Pfeilverzahnung ohne Mittelnut | Zwei symmetrische Hälften, wirksame Halbbreite, ideale Axialkraftaufhebung | Noch offen |
| Doppelschrägverzahnung mit Mittelnut | Wie Pfeil, mit Abzug der Nut von der wirksamen Breite | Noch offen |
| Schnecke / Schneckenrad | Axialmodul, Gänge, Durchmesserfaktor, Steigung, Bezugsdurchmesser, Achsabstand, Übersetzung | Noch offen |
| Gerades Kegelrad | Außen-Teilkreise, Teilkegel, Kegeldistanz, mittlerer Durchmesser, Übersetzung | Noch offen |
| Spiralkegelrad | Gleiche Teilkegel-/Kinematikbasis; keine Spiralauslegung | Noch offen |
| Zahnstange | Teilung, Vorschub pro Ritzelumdrehung und Vorschubgeschwindigkeit | Noch offen |

Das ist damit ein nutzbarer Anfang des Zahnradwerkzeugs, **kein vollständiger Generator
aller Verzahnungsarten**. Profilverschiebung, Sonderprofile, Balligkeit, Flankenmodifikationen,
Werkzeugdaten, Toleranzklassen, Werkstoff-/Wärmebehandlungswahl und normgerechte
Tragfähigkeitsnachweise sind noch nicht umgesetzt. Die Drehmomentangabe ändert hier
die berechneten Kräfte; sie dimensioniert nicht automatisch den Modul. Diese automatische
Vorbemessung existiert bislang im besonderen KUE-Calcpad-Blatt.

Die Evolventen werden aus ihrer Funktion gespeichert, ohne Linien- oder Splineersatz.
Die Fußübergänge sind Kreisbögen und keine durch ein reales Werkzeug erzeugte Trochoide.
Die native Geometrie unterstützt aktuell 18–240 Zähne, Modul 0,1–200 mm und
Eingriffswinkel 15–30°. Die Auslegungsrechnung erlaubt einen größeren Bereich und
ist deshalb keine automatische Freigabe für die Geometrieerzeugung oder Fertigung.
Das Werkzeug erstellt einzelne Räder; ein vollständiges, automatisch montiertes Paar
mit Eingriffsprüfung ist außerhalb der vorhandenen KUE-Planetenbaugruppe noch offen.

## Primärquellen und fachliche Entscheidungen

[KHK: Calculation of Gear Dimensions](https://khkgears.net/new/gear_knowledge/gear_technical_reference/calculation_gear_dimensions.html)
behandelt unterschiedliche Zahnradfamilien mit getrennten Berechnungsansätzen.
Für das Werkzeug folgt daraus: Modulbezug, Gegenrad und Art der Verzahnung gehören
explizit zur Eingabe. Eine universelle, anschließend bloß verformte Stirnradkontur
reicht als fachliche Grundlage nicht aus.

[KG Stock Gears: technische Unterlagen](https://www.kggear.co.jp/en/gear-technical-data)
stellt unter anderem getrennte Kapitel für Schrägverzahnungen, Schneckenpaare,
Kegelräder, Planetenmechanismen und Festigkeit bereit. Diese Trennung wird in der
Software zwischen Bezugsgeometrie, Flankenerzeugung, Paarprüfung und Festigkeit übernommen.

Direkt nutzbare Literaturkopien:

- [KG 9.3: Schrägverzahnung](Literatur/Planetengetriebe/Zahnradwerkzeug/KG_9_3_Schraegverzahnung.pdf):
  Normal-/Stirnmodulumrechnung und Winkelbezüge. Implementiert sind unter anderem
  `mt = mn/cos(beta)`, `alpha_t = atan(tan(alpha_n)/cos(beta))` und `d = mt*z`.
- [KG 9.5: Schneckenpaar](Literatur/Planetengetriebe/Zahnradwerkzeug/KG_9_5_Schneckenpaar.pdf):
  Der Modulbezug muss eindeutig sein. Der aktuelle Vorentwurf verwendet Axialmodul
  `mx`, `d1=q*mx`, `lead=pi*mx*starts` und `gamma=atan(starts/q)`.
- [KG 9.7: gerades Kegelrad](Literatur/Planetengetriebe/Zahnradwerkzeug/KG_9_7_Kegelrad.pdf):
  Teilkegel und Kegeldistanz bilden eine eigene Geometriegrundlage. Das Werkzeug
  berechnet zunächst diese Bezugsgrößen, nicht die fertigungsspezifische Zahnflanke.

[KHK: Double Helical Gears](https://khkgears.net/new/gear_knowledge/gear-nomenclature/double-helical-gears.html)
unterscheidet Anordnungen mit und ohne Mittelnut. Im Werkzeug reduziert die Nut
die wirksame Breite. Eine ideale Axialkraftaufhebung setzt symmetrische Belastung voraus.

[KHK: Worm Gear Pair](https://khkgears.net/product-category/worm-gear-pair/)
beschreibt ein zusammengehöriges Schneckenpaar. Für die weitere Implementierung ist
deshalb die passende erzeugende Geometrie des Rads erforderlich. Selbsthemmung wird
aus einer Übersetzung allein nicht zugesagt.

Die PDFs sind Herstellerliteratur für die lokale Recherche. Daraus wird keine freie
Lizenz für Quellcode oder Weiterveröffentlichung der Dokumente abgeleitet.
Abrufdatum, Original-URLs, Dateigrößen und SHA-256 stehen in
[quellen.json](Literatur/Planetengetriebe/Zahnradwerkzeug/quellen.json).
Die drei PDFs können als Quellen in NotebookLM importiert werden; keine Anmeldung
oder automatische Übermittlung an Google wurde vorgenommen.

## Open-Source-Komponenten: Eignung statt bloßer Funktionsliste

| Projekt | Beobachteter Umfang | Entscheidung |
| --- | --- | --- |
| [for-the-love-of-gears](https://github.com/MovAh13h/for-the-love-of-gears), MIT | Rust-Bibliothek für unter anderem Stirn-/Schrägradabmessungen und Getrieberechnung | Kandidat für spätere unabhängige Zahlenvergleiche. Kein Ersatz für native parametrische B-rep-Flanken. Keine neue Abhängigkeit übernommen. |
| [gearGeneration](https://github.com/mofopeadegoke/gearGeneration), MIT | C++-Werkzeug mit mehreren Zahnradarten und STL-Ausgabe | Eine STL-Ausgabe erfüllt die geforderte analytische, editierbare CAD-Geometrie nicht. Keine Integration oder Codeübernahme. |
| Vorhandener eigener Evolventenkern auf Truck, Apache-2.0 | Exakte Kurven und gerade Extrusionsflächen im nativen Projekt | Für Stirn-/Hohlräder weiterverwenden; gezielt erweitern, nicht durch Meshimport ersetzen. |

Die Lizenzangaben wurden in den öffentlichen Projektbeschreibungen geprüft; vor einer
späteren tatsächlichen Übernahme muss zusätzlich die konkrete Revision geprüft werden.
Die Bewertung behauptet keine vollständige technische Validierung dieser fremden Projekte.

## Nächste Entwicklungsstufen

1. **Exakte Schraubflächen:** analytische Schraublinien und Schraubextrusionsflächen
   im Geometrieschema, Kernel, Mesher und in der Persistenz ergänzen. Die Stirnevolvente
   bleibt erhalten; ihre Drehung wird eine Funktion der axialen Position.
   Normalmodul bestimmt weiterhin die Zahnhöhen, der Stirnmodul die Teilkreise.
2. **Pfeil-/Doppelschrägkörper:** Gegenhändigkeit, Mittelanschluss und Nut geometrisch
   korrekt behandeln. Geschlossenheit und Topologie der zusammengesetzten Körper prüfen.
3. **Werkzeugerzeugte Fußkurven:** Zahnstangen-/Wälzfräsererzeugung einschließlich
   Unterschnitt, Profilverschiebung und definierter Werkzeugkopfrundung.
4. **Schnecke und Schneckenrad:** konkrete Profilfamilie und Werkzeug festlegen;
   Schnecke analytisch beschreiben, passende Radflanke als Hüllfläche erzeugen.
   Kontakt über mehrere Stellungen prüfen. Ein gewöhnliches schräges Stirnrad genügt nicht.
5. **Kegelradfamilien:** gerade Standardverzahnung zuerst; Herstellverfahren und
   Flankendefinition festlegen. Spiralkegel- und Hypoidvarianten als eigene Aufgaben
   mit zusätzlicher Fertigungs-/Kontaktgeometrie behandeln.
6. **Paar-/Baugruppenwerkzeug:** Montage, Phasenlage, Drehrichtung, Schnittstellen,
   Kollision und tragende Kraftkette. Ergebnisse an FEM/MKS und Variantenstudien anbinden.
7. **Nachweise:** Zahnfuß-/Flankenfestigkeit, Lebensdauer, Lager und Wärmehaushalt
   mit nachvollziehbaren Annahmen und dafür geeigneten Norm-/Literaturgrundlagen.

Jede Stufe braucht eigene Akzeptanzkriterien: analytische Vergleichspunkte und Ableitungen,
geschlossene native Körper, Speichern/Wiederladen, Parameter-Neuaufbau, Eingriffsprüfung
über mehrere Stellungen, UI-Bearbeitung und Ablehnung ungültiger Eingaben.
Eine optisch plausible Vorschau allein gilt nicht als geometrischer Nachweis.

## Reproduzierbare Prüfungen

- `tools/verify_editable_planetary.py`: echte Kreisskizzen-Bemaßungen, normales
  Bearbeiten von Extrusion/Zahnrad, Referenzmaß per Doppelklick, Varianten und Wiederladen.
- `tools/verify_gear_tool.py`: 30 Auslegungen, negative Fälle, native Stirn-/Hohlräder,
  Parameter-Neuaufbau, tatsächlicher Dialog und Speicherung der Auslegungsprotokolle.
- Rohdaten und Screenshots: `output/kue-editable/` und `output/gear-tool/`.

Den tatsächlichen Stand des letzten Testlaufs dokumentiert das Forschungslog.
Die ältere umfassende KUE-Prüfung bleibt ein historischer Nachweis ihres damaligen
Softwarestands und darf nicht ohne erneute Prüfung auf neue Generatoren übertragen werden.
