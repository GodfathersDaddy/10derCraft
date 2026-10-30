# Berechnungsgetriebene KUE-Planetenstufe

Aktualisierung 21.10.2026: Das neue Startprojekt unter `output/kue-editable/` verwendet
16 normale Kreisskizzen mit echten treibenden `ref_...`-Bemaßungen und zugehörige
Extrusionen/Ausschnitte. Skizzen und Extrusionen lassen sich regulär bearbeiten.
Die analytischen Zahnräder besitzen jetzt einen eigenen normalen Feature-Dialog.
Das allgemeine [Zahnradwerkzeug](ZAHNRADWERKZEUG_RECHERCHE.md) ist zusätzlich im
CAD-Erstellen-Menü verfügbar; seine noch offenen Zahnradfamilien sind dort dokumentiert.

Das Projekt enthält das lokale Calcpad-Blatt, vier KUE-Angaben, veröffentlichte
CAD-Referenzen und eine native Baugruppe der ersten Stufe. Beide Planetenstufen
und die nachgeschaltete Stirnradstufe werden berechnet; nur Stufe 1 wird konstruiert.

## Bedienung

1. `Start-Getriebe-Berechnung.cmd` öffnet den Reiter **Parameter**.
2. Im Variantenfeld **KUE Variante 1…4** wählen. Die Angabe einschließlich
   Leistungen, Drehzahlen, Wirkungsgraden und Schaltungsart wird neu berechnet.
   Diese gespeicherten Vorlagen setzen zunächst Windkraft- und Aufbauvariante
   auf dieselbe Nummer. Das eigene Feld **Aufbauvariante (1–4)** lässt sich danach
   unabhängig ändern: Die Angabe teilt beide Varianten getrennt zu (S. 6–7).
   Dadurch sind alle 16 Kombinationen möglich. Eigene Kombinationen können
   anschließend unter einem neuen Variantennamen gespeichert werden.
3. Eingaben wie `T_in`, `R_Ziel`, `z_s` oder `modul` bearbeiten.
   Beispiel: `T_in = 1000N*m`, `R_Ziel = 3.8`, `z_s = 30`.
   Für automatische Modulwahl lautet der Ausdruck `modul_auto`; ein manueller
   Wert ist beispielsweise `4 mm`. Die Auswahl nutzt eine Projekt-Vorzugsreihe.
4. **Berechnen** zeigt neue Ergebnisse. Die Konstruktion bleibt bis zur Übernahme
   auf ihrem letzten Stand. Änderungen und betroffene Verwendungen sind sichtbar.
5. **Konstruktion aktualisieren** übernimmt gültige Referenzen gemeinsam mit
   Zahnzahlen, Abmessungen, Achsabständen, Gelenkankern und Bewegungskopplungen.
   Unzulässige Auslegungen werden abgewiesen; der gültige CAD-Stand bleibt erhalten.
6. **CAD** zeigt die Konstruktion. Das Drehgelenk **Input** treibt Sonne und Planeten
   über native Bewegungskopplungen. Ein Parameterwechsel setzt die Gelenke auf null.

`R_Ziel` bezeichnet den Drehzahlerhöhungsfaktor der ersten Stufe, also
`|n_aus/n_ein|`. Die ganzzahligen Zahnzahlen bestimmen den tatsächlich erreichten
Wert. Drei gleichmäßig verteilte Planeten setzen eine passende Montagebedingung
voraus. Beliebige Eingaben sind deshalb nicht zwangsläufig baubar.

Bei Aufbauvariante 1/2 steht der Ring fest, der Träger treibt an. Bei Aufbauvariante 3/4 steht
der Träger fest, der Ring treibt an und die Sonne dreht entgegengesetzt.
Die berechneten Stufendrehzahlen und Drehmomente tragen diese Vorzeichen;
die Drehzahltoleranz bezieht sich auf den Betrag der Generatordrehzahl.
Auch Sonnenzahnzahlen wie 31 oder 32 werden durch eine passende Restklasse der
Planetenzahnzahl auf die Montagebedingung für drei Planeten abgestimmt.
Die Bewegung ist eine ideale kinematische Willis-Kopplung. Zahnkontaktkräfte,
elastische Verformung und ein dynamisches Durchlaufen des Flankenspiels werden
dadurch noch nicht simuliert. Die Geometrie enthält bewusst Flankenspiel.

## Geometrie und Grenzen

Die Zahnflanken sind native analytische B-rep-Kurven mit
`x(t)=rb(cos(t)+t sin(t))`, `y(t)=rb(sin(t)-t cos(t))`, einschließlich ihrer
analytischen Extrusionsflächen. Sie sind keine angepassten Splines oder Polygone.
Nur die Bildschirmdarstellung und geometrische Messproben werden tesselliert.
Die native Projektdatei bewahrt die Kurventypen beim Speichern und Laden.
Das Feature unterstützt Zahnzahländerungen mit neu aufgebauter Topologie.

Unterhalb des Grundkreises ist keine Evolvente definiert. Dort verbindet eine
radiale Tangente mit Kreisverrundung den Zahnfuß. Dies ist noch keine durch ein
reales Werkzeug erzeugte Trochoide. Profilverschiebung, Schrägverzahnung und eine
Werkzeug-/Fertigungsfreigabe fehlen. Der analytische STEP-Export dieses neuen
Kurventyps ist noch nicht implementiert und wird ausdrücklich abgewiesen.

Die Rechnung enthält eine vereinfachte Zahnfuß-Vorauslegung, Wellentorsion,
beidseitig abgestützte Planetenbolzen, Durchbiegung/Neigung, Wandstärke,
Montierbarkeit und die Gesamtdrehzahlabweichung. Sie ersetzt keine ISO-6336-
Tragfähigkeitsrechnung, Hertz-/Ermüdungsrechnung oder Kataloglagerauswahl.
Die gezeichneten Buchsen stellen Gleitlagergeometrie dar. Die KUE-Aufgabe ist
damit noch nicht als vollständiger Fertigungs- und Festigkeitsnachweis abgegeben.

## Reproduzierbare Prüfung

`tools/verify_kue_planetary.py` prüft gezielt die Desktop-Anwendung: vier Angaben,
weitere Übersetzungen, Drehmomente und Sonnenzahnzahl, tatsächliche analytische
B-rep-Kanten, Bewegungsrelationen, atomare Übernahme, Bedienung im Parameterreiter
und Wiederladen. Ergebnisse und Bildschirmaufnahmen liegen in `output/kue-stage`.
Die Prüfung unterscheidet analytische Eingriffsbedingungen von endlichen
Durchdringungsproben über mehrere Bewegungsstellungen. Letztere sind kein
kontinuierlicher Kollisionsbeweis für alle denkbaren Geometrien.

## Fachliche Grundlagen

- Eigene Aufgabenstellung: `Angabe_KUE_Teil_II_WS2023_2024.pdf`, Angaben S. 6–7.
- [KHK: technische Grundlagen Innenverzahnungen](https://khkgears.net/pdf/internal-tech.pdf):
  Standardgeometrie und Interferenzarten.
- [KHK: Zahnradabmessungen](https://khkgears.net/gear-knowledge/gear-technical-reference/calculation-gear-dimensions/):
  Bezugs-, Grund-, Kopf- und Fußkreise.
- [DrivetrainHub: Planetary Gears](https://drivetrainhub.com/notebooks/gears/geometry/Chapter%204%20-%20Planetary%20Gears.html):
  Montage- und Nachbarschaftsbedingungen gleichmäßig verteilter Planeten.
- [DrivetrainHub: Kinematics](https://www.drivetrainhub.com/notebooks/fundamentals/rotational_mechanics/Chapter%202%20-%20Kinematics.html):
  Relativdrehzahlen und Willis-Beziehung.
