# Prüfbericht: Berechnung und Varianten der KUE-Planetenstufe

Stand: 2026-10-21T21:22:05. Geprüft wurde die lokale Desktop-Anwendung mit dem eingebetteten CalcpadCE-Rechenkern.

**Bestanden: 180/180 Rechenfälle mit 17280 unabhängigen Skalarkontrollen; 28 CAD-/Bewegungsfälle; 14 gezielt ungültige Eingaben.**

## Gefundene und behobene Fehler

- Windkraft- und Aufbauvariante waren gekoppelt. Die KUE-Aufgabe teilt sie getrennt zu. Eigenes Eingabefeld für Aufbauvariante 1–4 ergänzt; alle 16 Kombinationen geprüft.
- Drehzahlen und Momente wurden nur als Beträge ausgegeben. Die Berechnung führt jetzt die schaltungsabhängigen Vorzeichen durch beide Planetenstufen und die Stirnradstufe. Leistungsbilanz mit vorzeichenbehafteten Größen geprüft.
- Die Planetenzahnzahl war immer ein Vielfaches von drei. Der Montage-Rest wird jetzt passend zur Sonnenzahnzahl gewählt; auch Sonnenzahnzahlen 31 und 32 sind geprüft.

## Rechenprüfung

Die Referenz rechnet in SI außerhalb des Produkts. Zahnzahlen werden durch Enumeration montierbarer Kandidaten geprüft; die Produkt-Rundungsformel wird nicht wiederverwendet. Alle 96 exportierten fachlichen Skalare pro Fall werden einschließlich Dimensionen verglichen. Gleich gute Zahnzahlalternativen sind zulässig. Zusätzliche Kontrolle: P = T · 2π · n für jede Ausgangsstufe.

Abdeckung: 16 Grundkombinationen; je Kombination Drehmomente 100/1000/8000 Nm, Zielverhältnisse 3,2/3,8/4,6 und Sonnenzahnzahlen 31/32/36; zusätzlich Breitenfaktoren 10/20, manuelle Module, Drehzahländerungen und 25° Eingriffswinkel.

**Ein mathematisch korrekter Rechenfall ist nicht automatisch eine baubare Auslegung.** Die CAD-Prüfung und die Ablehnung unzulässiger Eingaben sind getrennte Nachweise.

## Alle 16 Grundkombinationen

| Windkraft | Aufbau | n Stufe 1 [1/min] | n Stufe 2 [1/min] | n Generator [1/min] | Abweichung Betrag [%] |
|---|---|---:|---:|---:|---:|
| 1 | 1 | 74.800 | 344.080 | 1012.000 | +1.200 |
| 1 | 2 | 74.800 | -344.080 | -1012.000 | +1.200 |
| 1 | 3 | -74.800 | -344.080 | -1012.000 | +1.200 |
| 1 | 4 | -74.800 | 344.080 | 1012.000 | +1.200 |
| 2 | 1 | 102.000 | 428.400 | 1224.000 | +2.000 |
| 2 | 2 | 102.000 | -428.400 | -1224.000 | +2.000 |
| 2 | 3 | -102.000 | -428.400 | -1224.000 | +2.000 |
| 2 | 4 | -102.000 | 428.400 | 1224.000 | +2.000 |
| 3 | 1 | 136.000 | 516.800 | 1435.556 | -4.296 |
| 3 | 2 | 136.000 | -516.800 | -1435.556 | -4.296 |
| 3 | 3 | -136.000 | -516.800 | -1435.556 | -4.296 |
| 3 | 4 | -136.000 | 516.800 | 1435.556 | -4.296 |
| 4 | 1 | 204.000 | 693.600 | 1825.263 | +1.404 |
| 4 | 2 | 204.000 | -693.600 | -1825.263 | +1.404 |
| 4 | 3 | -204.000 | -693.600 | -1825.263 | +1.404 |
| 4 | 4 | -204.000 | 693.600 | 1825.263 | +1.404 |

## CAD und Bewegung

28 Fälle mit jeweils 121 Eingangsstellungen, drei Planeten und zwei Eingriffspaaren: 20328 untersuchte Paarstellungen. Keine gemessene Durchdringung über der Toleranz von 0,0002 · Modul. Analytische Kontaktbedingungen und Willis-Phaseninvarianten separat kontrolliert.

Gemessen wurden echte analytische B-rep-Kanten und Extrusionsflächen, native Vorkommenstransformationen und Gelenkwinkel. Änderungen der Berechnung wirken erst nach bewusster Übernahme. Geprüft: Zielpunktbewegung des Trägers um 5°, Speichern/Wiederladen, echte Eingabe und Übernahme per Button sowie Eingaben und Scrollen bei 900×700.

Die Prüfung verwendet native Konturmessungen von Sonne, Planet 1 und Hohlrad und prüft die Einbauphasen aller drei Planeten. Die tatsächlichen Transformationen aller drei Planetenvorkommen werden separat gemessen. Endliche Konturproben ersetzen keinen kontinuierlichen Kollisions- oder Kontaktkraftnachweis.

## Fehlerfälle

| Parameter | Eingabe | Ergebnis |
|---|---|---|
| `modul` | `0mm` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `modul` | `1mm` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `modul` | `3s` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `z_s` | `0` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `z_s` | `30.5` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `T_in` | `-1000N*m` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `T_in` | `0N*m` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `T_in` | `1000000000000N*m` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `n_in` | `0/min` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `breitenfaktor` | `0` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `Aufbauvariante` | `5` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `R_Ziel` | `1` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `alpha_grad` | `15` | Abgewiesen; Referenzen und gültiges CAD erhalten |
| `T_in` | `unknown_symbol` | Abgewiesen; Referenzen und gültiges CAD erhalten |

## Grenzen und nicht implementierte Nachweise

- Die Zahnfuß- und Wellen-/Bolzenrechnung ist eine vereinfachte Vorauslegung. Das ist keine Bestätigung der vollständigen, lehrveranstaltungsspezifischen Modul-Minimum-/Maximum-Auslegung.
- Katalogauswahl und statischer Tragfähigkeitsnachweis der Planetenlager sowie zwei detaillierte Welle-Nabe-Nachweise fehlen.
- Zweite Planetenstufe und Stirnradstufe sind im Leistungs-/Drehzahlpfad enthalten; konstruiert wird nur Stufe 1. Die KUE-Gesamtkonstruktion mit Gehäusen, Montage, Zeichnungen und Lastausgleich ist nicht vollständig.
- Zahnfuß mit radialer Tangente/Kreisverrundung, noch keine reale Werkzeughüllkurve. Kein dynamischer Zahnkontakt, keine elastischen Körper und keine Fertigungsfreigabe.
- Die Vorauslegung setzt positive Eingangslasten voraus. Negative Eingangsmomente werden abgewiesen; ein rückspeisender Leistungsfluss ist nicht implementiert.
- Analytischer STEP-Export dieses neuen Kurventyps wird noch abgewiesen. Native Projektdateien bleiben analytisch.
- Keine breite SolidCraft-Gesamttestsuite ausgeführt. Diese Prüfung betrifft die Berechnungs-/CAD-/Variantenkette.

## Reproduktion und Rohdaten

`python tools/verify_kue_calculations.py` und danach `python tools/verify_kue_planetary.py --comprehensive`; Bericht mit `python tools/write_kue_audit_report.py` erzeugen.

Rohdaten: `output/kue-comprehensive/calculation-audit.json`, `desktop/verification.json`; CSV-Dateien im selben Ordner für eigene Auswertung und Grafiken. Bildschirmaufnahmen und Testprojekt unter `desktop/`. Build-/Quellhashes: `build-verification.json`.

Aufgabenbezug: `Angabe_KUE_Teil_II_WS2023_2024.pdf`, S. 6–9. Öffentliche Geometriegrundlage: [KHK, Internal Gears](https://khkgears.net/pdf/internal-tech.pdf). Die Quelle unterscheidet Evolventen-, Trochoiden- und Fertigungsinterferenz; der vorliegende Test deckt diese nicht vollständig ab.
