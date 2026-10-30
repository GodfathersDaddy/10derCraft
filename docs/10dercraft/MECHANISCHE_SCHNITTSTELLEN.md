# Projektschnittstellen und statische Lastpfade

Stand: 22.10.2026. Dieser Ausbau verbindet gespeicherte mechanische Schnittstellen
mit nativen CAD-Körpern und konkreten Baugruppeninstanzen. Er berechnet statische
Kraft-/Momentenresultanten auf einem ausdrücklich festgelegten starren Lastpfad.
Er löst noch keine dynamische Mehrkörpersimulation, Gelenkreaktionen oder Verformung.

## Bedienung

`Start-Mechanische-Schnittstellen.cmd` öffnet das geprüfte Referenzprojekt.
**MKS** zeigt dessen Körperblöcke, Ports, deklarierte Verbindungen und Lastfälle.
Die Eingabeabschnitte werden erst beim Aufklappen angezeigt.

1. Unter **Schnittstelle definieren / ändern** ID und CAD-Körper-Quellnamen eingeben.
   Für einen Komponentenkörper ist zusätzlich der genaue Occurrence-Pfad erforderlich,
   beispielsweise `3` oder `1,3`. Wiederholte Instanzen werden dadurch unterschieden.
2. Positionen werden im CAD-Koordinatensystem der Komponente angegeben. Sie akzeptieren
   Längenausdrücke wie `ref_L`, `L/2` oder `30 mm`. Quaternion `w,x,y,z` definiert
   die lokale Schnittstellenorientierung und wird beim Speichern normiert.
3. Übertragungsart wählen und die Schnittstelle speichern. Die ID wählt beim
   erneuten Speichern denselben Port; die Schaltflächen mit Port-IDs laden seine Eingaben.
4. Unter **Starren Lastpfad verbinden** Eltern-Port Richtung Fundament und Kind-Port
   Richtung Spitze festlegen. **Verbindung lösen** entfernt die Elternzuordnung des
   Kind-Ports. Ein Kind kann nur einen Eltern-Port haben; Schleifen werden abgelehnt.
5. Einen Einzellastfall mit Fundament-Port, Last-Port und lokalen Kraft-/Momentenwerten
   oder Normaldruck anlegen. **Statisch berechnen** speichert Resultat und Eingabestand.
   Mehrere Lasten sind über den Engine-/MCP-Befehl `mechanics.case` möglich. Der einfache
   UI-Editor überschreibt vorhandene Mehrlastfälle nicht mit einer einzigen Last.
6. Nach Änderungen an Parametern, relevanten Definitionen oder Baugruppenpositionen
   zeigt die Ergebnisliste historische bzw. ungültige Ergebnisse an. Erneut berechnen,
   bevor das Ergebnis weiterverwendet wird.

Die Darstellung fasst Ports desselben CAD-Körpers und desselben Occurrence-Pfads in
einem Block zusammen. Sie erkennt noch nicht automatisch alle gegeneinander starren
Teile einer Baugruppe. Die Verbindungen sind explizite statische Annahmen; sie
erzeugen oder übernehmen keine nativen CAD-Gelenke. Die alte Roboterdemo-Schnittstelle
bleibt als klar getrennter Sitzungsentwurf verfügbar.

## Verträge für die Lasteinleitung

| Typ | Zulässige Eingabe | Bedeutung / Grenze |
| --- | --- | --- |
| Punktkraft | Drei Kraftkomponenten in N, kein Moment | Ein Punktkraftanschluss darf auch als Übergang kein erforderliches resultierendes Moment tragen. |
| Rahmen: Kraft/Moment | Drei Kräfte in N und drei Momente in Nm | Abstrakter mechanischer Rahmen mit sechs Resultanten; keine automatisch vernetzte Punktlagerung. |
| Fläche: Kraft/Moment | Sechs Resultanten und deklarierte Fläche in mm² | Dokumentierter Flächenanschluss; noch keine Knotenlastverteilung oder FEM-Remote-Kopplung. |
| Fläche: Normaldruck | Ein Druckwert in MPa, deklarierte Fläche in mm² | Gleichmäßige vorzeichenbehaftete Normaltraktion; Resultante entlang lokaler +z-Achse, `F = p*A`. Der Portursprung muss als Flächenschwerpunkt angenommen werden. |

Die Fläche wird ausdrücklich eingegeben und noch nicht aus CAD-Flächen gemessen.
Ein Flächenwechsel oder eine Größenänderung muss fachlich nachgeführt werden.
Ein Druckport dient als belasteter Endpunkt, nicht als beliebiger Durchgang oder
Fundamentanschluss. Positiver Druck wirkt hier entlang lokaler +z, negative Werte
entgegengesetzt; es wird keine automatische Unterscheidung zwischen Außen-/Innennormale vorgenommen.

CAD-Zuordnung ist aktuell Körper-/Occurrence-Zuordnung, keine persistente Auswahl
einzelner topologischer Flächen. Ein normaler Geometrie-Move eines Root-Körpers
erzeugt kein eigenes bewegtes Körperkoordinatensystem für den Port. Dafür sind die
Positionsausdrücke anzupassen oder ein explizit platzierter Komponentenbezug zu nutzen.
Bei einer Komponentenbewegung wird dagegen deren tatsächliche native Platzierung
einschließlich noch nicht erfasster Positionen angewendet.

## Rechenmodell und Einheiten

Der gerichtete Baum kann sich verzweigen. Alle angegebenen Verbindungen werden für
diesen Rechenschritt als starr angenommen. Jede Last muss genau einen Weg zum
angegebenen Fundament haben; parallele Lastpfade und statisch unbestimmte Schleifen
werden nicht aufgeteilt. Zusätzliche Kraft- und Momentenkomponenten sind pro Last erlaubt.

Für jede Last werden lokale Komponenten zuerst in Weltkoordinaten gedreht. Danach
wird ihr Moment zu jedem Port auf dem Weg zum Fundament umgerechnet:

```text
F_world = R * F_local
M_target = R * M_local + ((x_source_mm - x_target_mm)/1000) × F_world
W_reaction_at_root = -sum(W_external_at_root)
```

Verzweigungen werden linear überlagert. Ein Punktkraft-Übergang wird erst anhand
seiner gesamten resultierenden Momentenbelastung geprüft. Die numerische
Momententoleranz beträgt `1e-9 * max(1, |F_N|)` in Nm. Die Gleichgewichtsresiduen
geben die verbleibende Differenz zwischen externen Lasten und der Fundamentbilanz an.

Damit ist die Resultantentransformation nachprüfbar, aber die Annahme eines starren
Lastpfads noch nicht validiert. Eigengewicht, Trägheitslasten, Kontakt, Reibung,
Gelenkfreiheiten, Nachgiebigkeit und Spannungsverteilung werden nicht berechnet.
Eine passende Momentenbilanz beweist deshalb weder Tragfähigkeit noch korrekte
Gelenkreaktionen eines beweglichen Mechanismus.

## Speicherung, Gültigkeit und MCP

Ports, Verbindungen, Lastfälle und Ergebnisse liegen gemeinsam im `.solidcraft`-Projekt.
Ein Bericht enthält Einheiten, Annahmen, durchlaufene Portfolgen, Resultanten,
Fundamentreaktion in Welt- und lokalen Koordinaten sowie einen exakten Eingabesnapshot.
Dieser umfasst auch die ausgewerteten Rahmen, CAD-Features und CAD-Parameter.

Ändert sich der Snapshot, bleibt das alte Ergebnis als historischer Stand erhalten.
Die Prüfung ist konservativ: Auch Änderungen an anderen CAD-Features oder Ports
können einen Bericht als veraltet markieren. Fehlende Körper oder ungültige
Lastpfade führen zur Ungültigkeitsmeldung. Die bloße Erfassung einer bereits
berücksichtigten Baugruppenposition soll keinen physisch neuen Lastfall erzeugen.

Registrierte Befehle:

- `mechanics.port`: Port anlegen oder mit gleicher ID ersetzen.
- `mechanics.connect` / `mechanics.disconnect`: starre Elternzuordnung anlegen bzw. lösen.
- `mechanics.case`: Lastfall anlegen oder mit gleicher ID ersetzen; dabei aktuellen
  Weg, Übertragungsverträge und Geometriebezug prüfen.
- `mechanics.evaluate`: aktuellen Lastfall auswerten, ohne ein Ergebnis zu speichern.
- `mechanics.solve`: Bericht mit Snapshot speichern, CAD dabei unverändert lassen.
- `mechanics.state`: Definitionen, aufgelöste Rahmen und Gültigkeit gespeicherter Berichte lesen.

Grenzen des ersten Schemas: 128 Ports, 32 Lastfälle, 128 Lasten pro Fall,
2 MiB je Eingabesnapshot. ID: 1–64 ASCII-Buchstaben, Ziffern, `_` oder `-`.
Die in der UI sichtbaren Aktionen verwenden dieselben Befehle wie MCP und Skripte.

## Gezielte Verifikation und nächste Schritte

`tools/verify_mechanical_project.py` verwendet native CAD-Körper und eine echte
Baugruppeninstanz. Die Referenzwerte werden unabhängig in SI-Einheiten angegeben:
100 N am Hebelarm 1 m ergeben 100 Nm; 2 MPa auf 200 mm² ergeben 400 N.
Weitere Prüfungen betreffen Verzweigungen, zwei zusammengesetzte Orientierungen,
lokale Fundamentkomponenten, Parameteränderungen, vorläufige/native Platzierungen,
abgewiesene Eingaben, Wiederladen und gelöschte CAD-Geometrie.
Rohdaten und UI-Aufnahmen liegen unter `output/mechanical-project/`.

Als Nächstes sind topologische Flächenbindungen mit überprüfbarer Schwerpunkt-/Flächen-
ableitung, eine explizite Starrgruppenbildung aus nativen Baugruppenbindungen und
die Übersetzung geeigneter Gelenke in einen separaten MKS-Solver nötig. FEM-Kopplungen
benötigen zusätzlich eine definierte Lastverteilung, deren resultierende Kräfte und
Momente erhalten bleiben. Der heutige statische Baum ist dafür eine Referenz, kein Ersatz.
