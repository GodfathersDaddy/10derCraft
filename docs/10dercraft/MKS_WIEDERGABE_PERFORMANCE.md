# MKS-Wiedergabe und Startgeschwindigkeit

„Simulation starten“ startet die 3D-Wiedergabe automatisch. Ein exakt zum CAD,
Material, Einbauzustand, Quellgraphen und Einstellungen passendes Ergebnis wird
wiederverwendet. „Neu berechnen“ erzwingt einen echten neuen Solverlauf.

Die Live-Wiedergabe verwendet den CAD-GPU-Zeichenweg. CAD-Dreiecke werden einmal
hochgeladen; danach ändern sich Körperplatzierungen und Schnittstellenachsen.
Der Zeitzeiger läuft unabhängig von der Anzahl der gespeicherten Solverzustände.
Zwischenzustände entstehen aus SLERP der relativen Gelenkrotationen, unter
Erhaltung der Gelenkanker; Schubgelenkwege werden zwischen den gemessenen
Zuständen interpoliert. Einzelne gewählte Solverstützstellen bleiben exakt.
Das ändert keine berechnete Kraft, keinen Rohwert und keine CAD-Geometrie.

Eine echte Neuberechnung läuft auf einer Kopie im Hintergrund. Der normale
Übernahmebefehl prüft den aktuellen Eingabestand und das vollständige vorbereitete
Modell erneut. Zwischenzeitliche Änderungen führen zum Verwerfen des Ergebnisses.
„Ergebnis verwerfen“ verhindert die Übernahme; die bereits gestartete, begrenzte
Berechnung kann noch fertiglaufen. Das Benutzerprojekt wird dabei nicht ersetzt.

Geometrische Integrale werden mit zwei Auflösungen wie zuvor berechnet und
begrenzt im Arbeitsspeicher gehalten. Die Identität der unveränderlichen
Geometrie und ihres Darstellungsnetzes schützt gegen falsche Wiederverwendung.
Dichte und Einbauposition werden bei jeder Zusammenfassung neu angewendet.
Ein geänderter Körper wird neu integriert. Kein Cache wird in die Projektdatei
geschrieben.

Ein bereits zum vollständigen Eingabestand passender Modellaufbau darf auch bei
„Neu berechnen“ erneut als Solver-Eingabe dienen. Der Solver wird dabei wirklich
neu ausgeführt. Diese Abkürzung ist von einer bloßen Ergebniswiedergabe getrennt.
Gemessen: 4.978 s für diesen vorbereiteten neuen Lauf.

## Gezielte Messung am Roboter

Gerät: Windows, NVIDIA GeForce RTX 4060 Laptop GPU, DX12. Dasselbe gespeicherte
Robotermodell, 8 s, 67 Solverzustände, sechs Achsen; kein veränderter Solver.
Die Bildrate zählt neu eingereichte GPU-Posen während laufender Wiedergabe.
Sie ist eine Messung dieses Rechners, kein garantierter Mindestwert.

| Prüfung | Vorher | Nachher |
|---|---:|---:|
| Unveränderter Lauf, Engine-Aufruf | 34.337 s | 0.059 s Median |
| Sichtbare automatische Wiedergabe nach Start | – | 0.703 s |
| Laufende Darstellungsbilder, drei kurze Wiederholungen | ungefähr 1/s | 34.3/s Median, 15.2–43.1/s |
| UI-Abfrage während Wiedergabe | ungefähr 1,2 s | 0.0275 s Median |
| Echter Solverlauf mit exakt gültigem Modellaufbau | – | 4.978 s |
| Neuer Eingabestand: erster Lauf mit Geometrieintegration | – | 50.420 s, im Hintergrund |
| Weitere erzwungene Berechnung des gültigen Modellaufbaus | – | 4.411 s |

Cache-Laufzeiten: [1.8160399999469519, 0.058968500001356006, 0.03709340002387762]. Median und Einzelwerte
werden getrennt berichtet; kalte Vorbereitung und warme Wiederholung sind keine
gleichartigen Aufgaben. Die erste vollständige Integration wurde nicht durch
eine ungenauere Berechnung ersetzt.

Die separate Prüfung des Geometriecaches verwendet einen nativen Quader: 5 kg,
nach Dichteänderung 10 kg und nach Verdopplung einer CAD-Länge 20 kg. Damit wird
geprüft, dass Dichteänderungen neu angewendet und Geometrieänderungen neu
integriert werden.

Matrixabweichung zu vier Originalstützstellen: 4.14729584008e-10.
Gelenkankerfehler in einer Zwischenpose: 1.13686837722e-13 mm.
Kraft- und Koordinatenkurven der warmen Berechnung sind bitgleich zur Referenz.
Zwischenposen sind eine Darstellung, keine zusätzlichen Solvermessungen.

Ohne GPU bleibt der vorhandene CPU-Anzeigeweg verfügbar; die hier gemessene
Beschleunigung betrifft die GPU-Wiedergabe. Die frühere AVI-Datei behält ihre
ursprüngliche Bildrate. Die neue flüssige Darstellung gilt für die Live-Ansicht.

Nachweise: `output/mks-playback-performance/verification.json`, `before.json`,
Screenshots und `provenance.json`. Alte umfangreiche Testläufe bleiben pausiert.
