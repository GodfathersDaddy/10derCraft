# Physikalische Eigenschaften der MKS-Blöcke

Stand: 22.10.2026. Im Reiter **MKS** auf **Masse und Trägheit aus CAD berechnen**
klicken, anschließend **Physikalische Eigenschaften · aktueller CAD-Stand** öffnen.
Die normale Blockansicht bleibt unabhängig von dieser aufwendigeren Auswertung.

## Berechnung

`mks.mass_properties` arbeitet lokal und lesend. Jeder Quellkörper wird je Aufruf
einmal geometrisch integriert; wiederholte Einbauten verwenden dieselben lokalen
Volumenmomente mit ihrer jeweiligen vollständigen Transformation. Körperversätze
und noch nicht übernommene Einbaupositionen sind berücksichtigt.

Das vorhandene `Mesh::mass_properties` integriert orientierte Tetraeder. Vorher
werden Netzpunkte nahe dem Körperschwerbereich zentriert und geometrisch identische
Randpunkte verschweißt. Das Netz muss geschlossen und konsistent orientiert sein.
Zwei Auflösungen prüfen die Änderung von Volumen und Trägheit; bei mehr als 0,5 %
Abweichung wird keine Blockmasse freigegeben. Diese Übereinstimmung ist eine
empirische Plausibilitätsprüfung, keine garantierte Fehlerschranke. Gekrümmte
Geometrie wird numerisch ausgewertet; der analytische CAD-Kurventyp bleibt erhalten.

Für jeden Körper gilt eine homogene Materialdichte aus dem CAD-Projekt.
Fehlende oder unbrauchbare Materialangaben verwenden den bestehenden CAD-Standard,
werden aber ausdrücklich als **angenommene Dichte** markiert. Es werden weder
fehlende Massen noch leere Komponenten durch künstliche Ersatzwerte ergänzt.

Für die Platzierung wird `I_W = R I_C Rᵀ` angewendet. Die Zusammenfassung erfolgt
mit massengewichtetem Schwerpunkt und Steinerschem Satz:

`I_Block = Σ (I_Körper,W + m · ((d·d) E − d dᵀ))`.

Dabei ist `d` der Vektor vom gemeinsamen Blockschwerpunkt zum Körperschwerpunkt.
Ergebnisse: **kg**, Schwerpunkt in **m**, Trägheit in **kg·m²**. Die Tensorachsen
sind parallel zum Weltkoordinatensystem; der Bezugspunkt ist der jeweilige
Blockschwerpunkt. Die gemischten Einträge sind die Einträge des Trägheitstensors,
nicht die positiven geometrischen Deviationsmomente.

## Gültigkeit und Grenzen

Nach CAD-, Material- oder Positionsänderungen wird die angezeigte Auswertung
als veraltet markiert; erneut über den Knopf berechnen. Beim Laden eines Projekts
wird aus seinen nativen Daten neu gerechnet. Die Ergebnisanzeige ist derzeit
sitzungsbezogen; es wird kein zusätzliches Massenergebnis ins CAD geschrieben.

Körper gelten als additive physische Teile. Überschneidungen werden nicht
automatisch vereinigt oder abgezogen. Es gibt keine Erkennung von Schüttgut,
inhomogenen Werkstoffen oder zusätzlich montierten, aber nicht modellierten Massen.
Offene bzw. nicht zuverlässig integrierbare Körper erzeugen Fehler je Block;
Teilergebnisse einzelner Körper sind Diagnoseinformationen, keine gültige Blockmasse.
Skalierungen und Scherungen sind für die starre Tensortransformation unzulässig.
Bei Parameter-/Featurefehlern oder als veraltet markierter CAD-Geometrie wird
die gesamte Auswertung abgelehnt; zuerst CAD reparieren und neu aufbauen.

Ein erster Exudyn-Adapter mit Zeitintegration, Projektpersistenz und
Eingabesnapshots ist inzwischen in [MKS_DYNAMIK.md](MKS_DYNAMIK.md) beschrieben.
Er sperrt unvollständige Masseneigenschaften. Noch offen sind flexible Körper,
allgemeinere Mechanismen und die nachfolgende FEM-/ROM-Kopplung.

## Reproduzierbare Prüfung

`python tools/verify_native_mks.py --mass-only` prüft unabhängige analytische
Quader-/Zylinderformeln, einen um 45° gedrehten Tensor mit gemischten Einträgen,
zwei Materialien, Steinerschen Satz, mehrfach verbaute Komponenten, Parameter-
und Dichteänderung, leere Komponenten, offene Netze, UI-Gültigkeit und Wiederladen.
Rohdaten, erwartete Werte, Startprojekt und echte UI-Aufnahme:
`output/native-mks-mass/`. Die breite alte Testsuite bleibt pausiert.
