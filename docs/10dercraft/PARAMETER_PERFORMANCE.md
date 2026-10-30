# Parameteränderungen und Werkzeugleiste

Stand: 29. Oktober 2026.

## Befund

Ein einzelner Wechsel des Besenstieldurchmessers von 24 auf 25 mm wurde in einer
eigenen Desktopinstanz gemessen. Das native Modell ist
`output/Besenhalter/Innenmutter/Besenhalter_D24_M4_Innenmutter.solidcraft`.
Die Originaldatei wurde bei der Messung nicht gespeichert oder überschrieben.

| Version | Parameteränderung |
|---|---:|
| Schnellbau, eigene Crates ohne Optimierung | 55,55 s |
| Optimierung Stufe 2, Oberfläche weiterhin Stufe 0 | 53,01 s |

Das entspricht etwa 4,6 Prozent kürzerer Zeit. Zwei Ausgangsläufe lagen bei
54,47 und 55,55 Sekunden; nur der zweite wurde vollständig als Bericht gesichert.
Das ist **kein belastbarer allgemeiner Geschwindigkeitsgewinn** und keine Lösung
der langen Geometrieberechnung. Es wurde kein großer Varianten-Testlauf gestartet.

Die vollständigen Messgrößen beider Läufe stimmen mit relativer Toleranz 1e-8
und absoluter Toleranz 1e-6 überein. Ergebnis: ein geschlossener Körper,
19 Flächen, 32 Kanten, 26 Ecken, Volumen 17905,157692 mm³. Alle Timelinefehler
waren leer. Die im Bericht enthaltene Masse ist keine Filamentauslegung.

Die sechs von der Durchmesseränderung betroffenen Features umfassen unter
anderem Körpervereinigungen und Bohrungs-/Muttertaschen-Subtraktionen. Ihre
Booleschen Berechnungen bestimmen die Laufzeit. Die Formelberechnung selbst
erklärt die lange Wartezeit nicht.

## Änderungen

- Eigene Geometrie- und Berechnungscrates werden im Start-Build mit Stufe 2
  kompiliert. Die Oberfläche behält Stufe 0 für überschaubare Bauzeiten.
- Native Formeländerungen, Parameterdialogänderungen und das Übernehmen von
  Berechnungsergebnissen verwenden die vorhandene Hintergrundberechnung
  `run_heavy`. Die Geometrie benötigt weiterhin ihre Rechenzeit.
- Die obere Berechnungsleiste bleibt in einer Zeile. Ihr Inhalt einschließlich
  Symbolpalette kann horizontal gescrollt werden. Der Scrollbalken ist sichtbar;
  das Mausrad soll über dieser Leiste ebenfalls horizontal scrollen.
- Das Prüfwerkzeug wartet bei einer Server-Wartezeitüberschreitung auf dieselbe
  laufende Anfrage mittels `control.pending`. Änderungen werden nicht erneut gesendet.

Beim ersten Öffnen nach einem geänderten Build kann der gespeicherte Modellcache
ungültig sein. Der optimierte Vergleich benötigte deshalb 83,17 Sekunden zum
kalten Öffnen; der alte Build konnte seinen gespeicherten Cache verwenden und
benötigte 0,89 Sekunden. Diese Ladezeiten sind nicht direkt vergleichbar. Das
Vertrauen in alte Caches wurde nicht aufgeweicht.

## Nachweise und Grenzen

Messungen: `output/parameter-performance/before/benchmark.json`,
`after/benchmark.json` und `comparison.json`. Die SHA-256-Werte identifizieren
die tatsächlich gemessenen Programme. Die abschließende Scrollkorrektur betrifft
nur die Oberfläche; die Veröffentlichungsdatei führt ihre Programmidentität separat.

UI-Prüfung und Screenshots unter `output/parameter-performance/ui`: bestanden.
Getestet wurden die einzelne Werkzeugzeile, horizontales Mausradscrollen und
eine echte Formeländerung von 10 auf 12 mm, die das Quader-Volumen von 6000 auf
7200 mm³ änderte. Build und gezieltes Clippy mit `-D warnings` bestanden.
Es wurden weder alle Modelle geprüft noch allgemeine Echtzeit-Änderungen bewiesen.
Die weitere Beschleunigung der Booleschen Geometrie bleibt offen. Der
Projektleitfaden `solidcraft/skills/solidcraft-performance/SKILL.md` legt fest:
“Do not rewrite kernel code for speed”; Arbeiten am Geometriekern sind pausiert.

Inventor-Importvorbereitung: `output/body-parts-import/README.md` und
`assembly-manifest.json`. Die neun CSV-Platzierungen sind geprüft; echte
Inventor-Geometrie wurde noch nicht importiert.
