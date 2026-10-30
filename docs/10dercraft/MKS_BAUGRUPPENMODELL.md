# Native Baugruppe als MKS-Blockmodell

Stand: 22.10.2026. Aufruf: `Start-MKS-Baugruppenmodell.cmd`, dann oben **MKS**
und die Systemansicht öffnen. Die SolidCraft-CAD-Oberfläche bleibt erhalten;
die Blockansicht erscheint im zusätzlichen Arbeitsbereich bzw. temporären Fenster.

## Umsetzung

`mks.model` liest die aktuelle native Baugruppe einschließlich noch nicht
übernommener Positionen. Es verändert weder CAD noch Parameter oder Gelenke.

- Alle Körper eines Komponenten-Einbaupfads gehören zum selben starren Block.
- Aktive starre Gelenke, auch aus „Rigid Group“, vereinigen diese Blöcke transitiv.
- Am Dokument-Root geerdete Komponenten gehören zum Fundamentblock.
- Unterbaugruppen werden nicht allein aufgrund der Baumhierarchie starr verbunden.
- Bewegliche Gelenke bleiben als Verbindungen zwischen den reduzierten Blöcken
  erhalten. Beide Welt-Bezugssysteme, Typ, native Definition, Werte, Grenzen,
  Ausdrücke sowie Flip, Winkel und Versatz bleiben verfügbar.
- Gültige Projekt-Kraftschnittstellen werden anhand ihres exakten Einbaupfads
  zugeordnet. Punkt-/Flächenvertrag und aktuelles Bezugssystem bleiben erhalten.
- Die Körperliste enthält Quellkörper, Einbaupfad und Welttransformation inklusive
  eventueller Körperversätze. Das ist die Übergabegrundlage für spätere Solveradapter.

Die Ansicht aktualisiert sich mit dem Dokumentstand. Es gibt kein zweites,
unabhängig zu pflegendes Baugruppenmodell. Block-IDs gelten innerhalb der jeweils
abgeleiteten Darstellung; dauerhafte Referenzen sind Quellkörper und Einbaupfade.
Die Ableitung wird nach dem Laden aus den vorhandenen Projektdaten rekonstruiert;
es wird kein zusätzlicher persistenter Dokumenttyp eingeführt.

## Bewusst sichtbare Diagnose

Ein natives Gelenk referenziert derzeit Occurrence-IDs, keine vollständigen
Instanzpfade. Wird dieselbe Unterbaugruppe mehrfach eingebaut, kann ein inneres
Gelenk dadurch mehrdeutig sein. Solche Gelenke werden mit Begründung ausgelassen;
die Darstellung wählt niemals stillschweigend die erste Instanz.

Verschachtelte Erdung wird nicht als globale Erdung interpretiert. Geometrische
Relationen, native Bewegungskopplungen und aktivierter CAD-Kontakt werden als noch
nicht übersetzte Simulationsbedingungen gekennzeichnet. Freie Blöcke sind sichtbar.
Eine geschlossene Graphkante ist eine Topologiediagnose, kein Nachweis eines
beweglichen oder widerspruchsfreien Mechanismus. Die lokalen Gelenkfreiheiten
werden nicht als globale Beweglichkeit addiert. Ein bewegliches Gelenk innerhalb
eines starren Blocks erhält einen gesonderten Hinweis.

## Grenze und nächste Umsetzung

Die Blockansicht ist eine strukturelle MKS-Vorstufe. Über den eigenen Knopf
**Masse und Trägheit aus CAD berechnen** stehen jetzt materialabhängige Massen,
Schwerpunkte und Trägheitstensoren zur Verfügung; Details in
[MKS_MASSENEIGENSCHAFTEN.md](MKS_MASSENEIGENSCHAFTEN.md).
Für offene Dreh-/Schubgelenkketten gibt es inzwischen den separaten
[Exudyn-Dynamikadapter](MKS_DYNAMIK.md), einschließlich vorgegebener Bewegungen
und lokaler Gelenkkräfte. Flexible Körper fehlen weiterhin. Die bestehende
statische Resultantentransformation bleibt separat: Ein Drehgelenk wird daraus
nicht automatisch zu einer starren, momententragenden Verbindung.

Nächste Schritte:

1. Massen und Trägheit sind als eigener Auswertungsschritt implementiert.
   Die Abnahme umfasst Quader, Zylinder, Drehung, Steinerschen Satz, unterschiedliche
   Dichten sowie Parameteränderungen. Für weitere Geometrieklassen folgen gezielte
   Nachweise und die spätere Sperre unvollständiger Modelle im Solveradapter.
2. Der erste Exudyn-Teilumfang ist umgesetzt: offene Kette mit Dreh-/Schubgelenken,
   eindeutigen Pfaden, Portlasten und idealen Rampenantrieben. Pendel, gekoppelte
   Kette, Schieber sowie Gültigkeit nach Änderungen sind gezielt geprüft.
3. Native Zahnradkopplung übertragen und die Planetenstufe gegen deren analytische
   Übersetzungs- und Drehrichtungsbeziehungen prüfen.
4. Lastzeitverlauf an Kraftschnittstellen speichern, FEM/ROM anbinden und für jede
   Variante Eingabestand, Gültigkeit und Optimierungsverlauf nachvollziehbar halten.

## Gezielte Prüfung

`tools/verify_native_mks.py` baut eine kleine native Baugruppe auf und prüft
starre Zusammenfassung, Fundament, freie Komponente, Gelenkbewegung, Portzuordnung,
Unterdrückung, Graphschleife, Wiederladen und Mehrdeutigkeit wiederholter Instanzen.
Rohdaten und tatsächliche UI-Aufnahmen liegen in `output/native-mks/`.
Die alte breite Testsuite bleibt pausiert.
