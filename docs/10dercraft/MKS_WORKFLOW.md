# MKS: Gelenk, Bewegungsquelle und Messausgabe

## Bewegung vorgeben

Im MKS ein Gelenk öffnen (Doppelklick oder Klick auf seinen B-/F-Anschluss).
Unter Bewegung kann eine neue Bewegungsquelle für dieses Gelenk angelegt werden.
Ein Drehgelenk erhält eine Winkelquelle, ein Schubgelenk eine Wegquelle.
Name, Parameterausdruck und Bewegungsprofil sind im normalen Quelleneditor
bearbeitbar. Nach dem Speichern wird die Verbindung ausdrücklich bestätigt.
Eine nicht bestätigte Verbindung ersetzt keinen bestehenden Eingang.

Unter „Vorhandene Quelle verbinden“ werden Quellen der passenden Dimension
angeboten. Auch dieser Weg verwendet den normalen Austauschdialog. Eine Quelle
kann mehrere passende Gelenke antreiben. Verbundene Quellen lassen sich aus
dem Gelenkdialog öffnen und bearbeiten; sie sind keine Robotersonderlösung.

## Messungen vergleichen

Scope öffnen. „Alle Gelenkkräfte“ aktiviert die Kraftbeträge aller Gelenke und
ihre Messverbindung zum Scope. „Alle Gelenkwinkel“ aktiviert Winkel von
Drehgelenken, „Alle Schubwege“ die Positionen von Schubgelenken. Die bisherige
Auswahl einzelner Gelenke und Komponenten bleibt erhalten. „Auswahl leeren“
deaktiviert die ausgewählten Messkanäle, ohne Gelenke oder Ergebnisse zu löschen.
Die Auswahl wird wie bisher im Projektlayout gespeichert.

Ein Klick in das Diagramm pausiert die Wiedergabe, wählt die entsprechende
Zeit und öffnet die berechnete 3D-Stellung. Die Daten liegen weiterhin auf dem
Abtastgitter des Solvers; der Tooltip zeigt den jeweils nächstgelegenen Messwert,
keine neue Rechnung am Mauszeiger. Die Cursorlinie folgt der Wiedergabezeit.

## Wiedergabe

Oben stehen 0,1-fache bis 4-fache Wiedergabegeschwindigkeit und Wiederholung
bereit. Diese Einstellungen gelten für die aktuelle Ansicht, nicht als neue
physikalische Bewegungsparameter. Sie bleiben nicht über einen Programmneustart
erhalten. Die gesamte Werkzeugleiste scrollt horizontal, auch mit dem Mausrad;
sie benötigt keine zusätzliche umgebrochene Zeile.

Änderungen an Quellen oder Verbindungen machen alte Ergebnisse ungültig.
Für die neue Bewegung „Simulation starten“ verwenden. Reines Abspielen,
Ändern der Geschwindigkeit oder Auswählen von Messkanälen erfordert keinen
neuen Solverlauf. Es handelt sich weiterhin um vorgegebene Bewegungen und
Auswertung vorhandener Solverausgänge, nicht um einen neuen Reglerlöser.
