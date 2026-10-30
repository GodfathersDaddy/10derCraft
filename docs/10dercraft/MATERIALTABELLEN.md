# Materialtabellen und Standardwerte

Stand 30.10.2026: Die Schraubentabelle enthält M2, M3, M4, M5, M6, M8, M10,
M12, M16, M20 und M24. `thread_diameter`, `tap_drill_diameter` und
`clearance_hole_diameter` sind Werte in mm nach den veröffentlichten
Herstellerrichtwerten von [TR Fastenings](https://www.trfastenings.com/Knowledge-Base/Engineering-Data/tapping-sizes-and-clearance-holes).
Der Knopf „Bearbeitbare Fastener-Familie anlegen“ übernimmt alle elf Zeilen
als normale Familientabelle; M6 ist zunächst aktiv. Bestehende Familien werden
nicht überschrieben, CAD-Referenzen werden weiterhin ausdrücklich ausgewählt.
Das ist keine vollständige Schrauben-, Toleranz- oder Festigkeitsnormdatenbank.

Im Parameter-Reiter oben Bibliothek anklicken. Die Bibliotheksleiste lässt sich
über denselben Knopf wieder schließen. Über die Suche nach Werkstoff, Kennwert
oder Konstante filtern, dann den Datensatz öffnen und Werte ins Blatt übernehmen. Einfügen erzeugt
normale direkt editierbare Berechnungen mit eindeutigen Namen. Vorhandene Namen
werden nicht überschrieben. Die Quelladresse und der Geltungsbereich werden als
Kommentar in der Berechnung gespeichert. Diese Werte können wie andere Ergebnisse
als Referenz markiert werden. Die Einfügung ändert keine CAD-Materialzuweisung.

Erster Umfang: S355JR für 3–16 mm und über 16–40 mm; EN AW-6082 als stranggepresstes
Profil in T4 bis 25 mm sowie T6 bis 5 mm und über 5–15 mm; Normfallbeschleunigung
9,80665 m/s². Streck-/Dehngrenzen und untere Zugfestigkeiten sind Mindestwerte;
Rm_max bei Stahl ist eine obere Bereichsgrenze. Für Aluminium ist keine unbekannte
obere Grenze erfunden. Die dickeren T6-Daten sind bewusst auf den gemeinsamen
Bereich offener und geschlossener Profile begrenzt.

Quellen, geprüft 29.10.2026:
- https://www.metinvestholding.com/en/products/steel-grades/s355jr
- https://www.hydro.com/Document/Index?id=7824&name=Hydro+EN+AW+6082.PDF
- https://www.bipm.org/en/committees/cg/cgpm/3-1901/resolution-2

Die Daten sind nachprüfbare Ausgangswerte für den angegebenen Werkstoffzustand,
keine pauschalen zulässigen Spannungen. Sicherheitsbeiwerte, Ermüdungsfestigkeit,
Temperaturabhängigkeit und Fertigungseinflüsse müssen zur Auslegungsmethode passen.
E-Modul, Dichte und unbekannte Kennwerte werden mit diesem ersten Datensatz nicht
automatisch hinzugefügt. Die vorhandene CAD-Materialdatenbank bleibt getrennt.

Die editierbare Datensammlung liegt unter solidcraft/assets/data/worksheet-materials.json.
Ihre Fassung wird mit der Anwendung ausgeliefert; eingefügte Berechnungen bleiben
als Projektdaten erhalten. Keine neue eigene Projektdateistruktur.


Erweiterung: Die zentrale Bibliothek gruppiert Werkstoffe, technische Standardwerte
und Naturkonstanten. Hinzu kommen Lichtgeschwindigkeit, Planck-Konstante,
Elementarladung, Boltzmann-Konstante und Avogadro-Konstante mit ihren definierten
SI-Werten. Quelle: https://www.nist.gov/pml/special-publication-330/sp-330-section-2
Diese Definitionen werden in der lokalen Berechnung mit der verfügbaren
Gleitkommagenauigkeit dargestellt. Die Bibliothek ist erweiterbar; dies ist kein
Anspruch auf eine vollständige Sammlung aller technischen Kennwerte.
