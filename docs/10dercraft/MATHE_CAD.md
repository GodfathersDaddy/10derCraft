# Berechnungsdokumente und CAD

## Neu: lokales Calcpad und Varianten im integrierten Reiter

`Start-Calcpad-CAD.cmd` öffnet das allgemeine Referenzbeispiel direkt im Reiter **Parameter**.
Der lokale CalcpadCE-Rechenkern ist ein mitgelieferter Prozess, der nach jeder Berechnung
wieder endet. Ein Berechnungsserver ist nicht erforderlich. Aufbau und Grenzen:
[Lokale Anbindung](integrations/calcpad-local/README.md).

1. Formeln im integrierten Calcpad-Blatt bearbeiten und **Berechnen** klicken.
2. Benötigte Ergebnisse als **CAD-Referenz** markieren. Aus `B` entsteht `ref_B`.
3. In Skizzen und Featuremaßen `ref_B` oder etwa `2 * ref_B + 5 mm` eingeben.
4. Die Berechnung unter einem Namen **als Variante speichern**. Weitere Varianten
   mit anderen Eingaben speichern und im Feld **Variante** auswählen.
5. Auswahl berechnet die gespeicherte Variante neu. Geänderte Referenzen werden mit
   altem/neuem Wert und ihren Verwendungen angezeigt; im CAD erscheint dafür ein Hinweis.
6. **Konstruktion aktualisieren** übernimmt die Ergebnisse in den bestehenden CAD-Aufbau.

Quelltextänderungen sperren die Übernahme alter Ergebnisse, bis erfolgreich neu gerechnet
wurde. Referenzen, Varianten und Berechnung werden im CAD-Projekt gespeichert.
Die Mathedatei exportiert Berechnungsdefinitionen und Varianten, überträgt aber keine
CAD-Referenzbindungen auf ein anderes Projekt. Native direkte Parameter bleiben wie
bisher sofort wirksam; für die bewusste Übernahme müssen Maße `ref_…` verwenden.

Die erste Oberfläche zeigt Calcpad-Quelltext und Ergebnisse im SolidCraft-Stil. Ein
typografischer Mathcad-Formeleditor und vollständige Calcpad-Dokumentfunktionen sind
noch nicht umgesetzt. Varianten ändern aktuell Berechnungen und referenzierte Maße
eines vorhandenen Modells; sie erzeugen noch keine beliebig andere Baugruppentopologie.
Die KUE-Planetenstufe besitzt jetzt ein eigenes analytisches Evolventen-Feature
und einen lokalen Calcpad-Variantenaufbau. Zahnzahlwechsel bauen die Zahnr?der neu
auf; Schaltungswechsel aktualisieren die Gelenke. Bedienung, Nachweise und Grenzen:
[Planetengetriebe](PLANETENGETRIEBE.md). Eine vollst?ndige Tragf?higkeits- und
Fertigungsfreigabe bleibt offen.

## Bestehende native CAD-Formeln

Die native Projektseite **Parameter** neben **Start** enthält die Berechnung. Jede Definition besitzt einen Namen, einen Ausdruck, eine Einheit und eine Beschreibung. Die mathematischen Definitionen und die CAD-Parameter verwenden denselben Abhängigkeitsgraphen. Es gibt keine zweite, möglicherweise veraltete Ergebnistabelle.

Beispiel:

| Name | Ausdruck | Einheit |
| --- | --- | --- |
| Laenge | 120 mm | mm |
| Breite | Laenge / 3 | mm |
| Dicke | 5 mm | mm |

Im Skizzenmaß oder einer Extrusion kann unmittelbar `Laenge`, `Breite` oder beispielsweise `2 * Dicke` stehen. Eine Formeländerung wird mit Enter oder beim Verlassen ihres Eingabefeldes übernommen. Abhängige Berechnungen, Skizzen und nachgelagerte CAD-Features werden neu berechnet. Zyklische und fehlerhafte Berechnungen werden abgewiesen; ebenso Änderungen, die ein zuvor gültiges CAD-Feature ungültig machen. Der letzte gültige Projektzustand bleibt dabei erhalten. Die Übernahme ist ein rückgängig machbarer Projektschritt.

Die Zuordnung geschieht über den Namen. Vorhandene Geometrie mit festen Zahlen wird nicht automatisch mit gleichnamigen Berechnungen verknüpft: Im jeweiligen Maß muss die Formel oder der Berechnungsname eingetragen sein. Frei eingegebene Zähnezahlen können ebenfalls nur dann neue Zähne erzeugen, wenn das entsprechende CAD-Feature diese Topologieänderung unterstützt. Das neue Feature `solid.involute_gear` verwendet diese Referenzen und unterstützt den Zahnzahlwechsel unmittelbar.

## Mathedatei

Die Berechnungen sind mit ihren Formeln im `.solidcraft`-Projekt gespeichert. **Speichern (.scmath)** auf der Parameterseite exportiert zusätzlich eine eigenständige, lesbare JSON-Mathedatei. **Laden und CAD aktualisieren** lädt solche Berechnungen in das aktuelle Projekt. Vorhandene gleichnamige Benutzerparameter werden dabei aktualisiert; Modellmaße können nicht durch den Import überschrieben werden. Der Import ist atomar. Dateien führen keinen Code aus.

Die exportierte Datei enthält Definitionen und keinen eingefrorenen Ergebnisstand:

```json
{
  "format": "solidcraft-math/1",
  "title": "Bauteilberechnung",
  "notes": "Laenge treibt Breite und die CAD-Skizze.",
  "definitions": [
    {"name": "Laenge", "expression": "120 mm", "unit": "mm"},
    {"name": "Breite", "expression": "Laenge / 3", "unit": "mm"}
  ]
}
```

Eine extern geänderte `.scmath`-Datei muss bewusst erneut geladen werden; es gibt derzeit keine Dateisystemüberwachung. Nach dem Laden lebt die Berechnung im Projekt. Ein Export wird nicht bei jedem CAD-Speichern automatisch überschrieben. Die Datei ist ein eigenes offenes Format, kein proprietäres Mathcad-Dokument.

## Allgemeine Schnittstellen

Alle Funktionen verwenden registrierte Engine-Befehle und sind damit über denselben Befehlsweg wie CAD, Skripte und MCP steuerbar:

- `worksheet.read`: Definitionen, Ergebnisse und Projektbeschreibung lesen.
- `worksheet.set`: benannte Berechnung anlegen oder ändern und CAD neu berechnen.
- `worksheet.describe`: Titel und Notizen im Projekt speichern.
- `worksheet.save`: Mathedatei schreiben oder Dokument als JSON zurückgeben.
- `worksheet.load`: Mathedatei oder JSON-Dokument laden und CAD aktualisieren.
- `worksheet.open`: native Bearbeitungsseite öffnen, ausschließlich beim interaktiven Aufruf.

Die Parameterbefehle bleiben ebenfalls verwendbar. Einheiten, Funktionen und Abhängigkeiten werden durch den vorhandenen CAD-Auswerter geprüft.

## Umfang

Der native CAD-Auswerter verarbeitet Skalare, Laengen und Winkel. CalcpadCE
verarbeitet zusaetzlich physikalische Verbundeinheiten und eigene Funktionen;
veroeffentlichte CAD-Referenzen bleiben auf Laenge, Winkel und dimensionslose
Groessen beschraenkt. Die KUE-Rechnung fuehrt Drehzahlen mit 1/min und Drehmomente
mit N*m, und exportiert fuer die CAD-Qualitaetspruefung normalisierte Skalare.
Noch offen sind ein typografischer 2D-Formeleditor und die Integration der
Calcpad-Solver-/Import-Makros.

Die neue Planetenstufe verwendet analytische Evolventen im Geometriekern.
Das alte Polygonbeispiel unter output/planetary bleibt als Entwicklungsstand
vorhanden; die aktualisierten Startdateien oeffnen das vollstaendig gepruefte Projekt unter
output/kue-comprehensive/desktop. Der neue Bericht steht in PRUEFBERICHT_KUE.md.
Werkzeuggenerierte Fusskurven und die vollstaendige Tragfaehigkeitsfreigabe
bleiben offen. Details und reproduzierbare Nachweise: PLANETENGETRIEBE.md.
