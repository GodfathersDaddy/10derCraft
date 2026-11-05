# Christbaum-Adventkalender

Das Projekt `examples/projects/Christbaum-Adventkalender.solidcraft` enthält die 34 Druckteile aus dem vom Benutzer bereitgestellten Archiv `Christbaum_Adventkalender_CAD_STL.zip`. Die Montagepositionen folgen dessen OpenSCAD-Montagedefinition. Die Geometrie ist als Dreiecksnetz eingebettet. Die Komponenten und Gelenke sind native 10derCraft-Baugruppenelemente. Die ursprüngliche parametrische OpenSCAD-Geometrie wird beim STL-Import nicht rekonstruiert.

## Bedienung

Unter Start und Beispiele den Christbaum-Adventkalender öffnen. Die Komponenten im Browser einzeln auswählen. Die 23 Schubladen besitzen radiale Schubgelenke mit begrenztem Auszug. Die Sternspitze lässt sich nach oben abheben, der Stern ist an ihrem Deckel befestigt. Die vier Etagen und ihre Deckplatten sind starr verbunden. Die Bewegungsstudie „Schubladen und Sternspitze“ zeigt vier Auszüge und das Anheben der Spitze. Für eine manuelle Positionsvorgabe lässt sich das jeweilige Gelenk öffnen und sein Schubwert ändern.

## Prüfungen

Alle 34 importierten Netze sind geschlossen. Die Baugruppe besitzt 33 Gelenke und keine unverbundenen Komponenten oder Gelenkkonflikte. Für jede Schublade und die Spitze wurden halber Auszug, voller Auszug und Rückkehr geprüft, insgesamt 72 Positionen. Der maximale Fehler der Schwerpunktverschiebung beträgt 3,013e-12 mm. Die Volumina stimmen innerhalb von 1e-5 relativ mit den unabhängig aus den STL-Dreiecken integrierten Ausgangsvolumina überein. Kein Timeline-Feature meldet einen Fehler.

Der Bericht liegt in `docs/10dercraft/evidence/advent-calendar/verification.json`. Diese Prüfungen bestätigen Import und Montagebewegungen. Ein physischer Probedruck und eine vollständige Kollisionsprüfung stehen aus. Die Auszugbegrenzungen dienen der CAD-Bedienung und bilden keine mechanischen Anschläge ab.

## Herkunft

Die Ausgangsgeometrie wurde vom Benutzer als ZIP bereitgestellt. Die daraus aufgebaute Baugruppe wird auf dessen ausdrücklichen Wunsch zusammen mit 10derCraft veröffentlicht. Die Quelldateien im persönlichen Downloadordner werden für das Öffnen der gespeicherten Baugruppe nicht benötigt.
