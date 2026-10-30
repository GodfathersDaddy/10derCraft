# MKS-Bibliothek: tatsächlicher Funktionsumfang

Ergebnisdaten werden in einem separaten Dokumentfeld bis 8 MiB gespeichert und
bei Dokumentkopien gemeinsam genutzt. Alte inline gespeicherte Resultate bleiben
lesbar. Sie gehören nicht zum CAD-Eingabesnapshot. Nach Änderungen an Bewegungen
oder Lasten muss neu gerechnet werden. Der lokale Solverprozess akzeptiert
begrenzt 12 MiB Ergebnisdaten, bei weiterhin 30 s Laufzeitgrenze.

Der neue Videoexport ist AVI/MJPEG mit wählbarer Bildrate. Er verwendet dieselbe
gelenktreue Interpolation wie die 3D-Wiedergabe; Zeitreihenwerte werden nicht durch
Videobilder ersetzt. Die tatsächliche Abschlussprüfung wird im Integrationsbericht
protokolliert.

## Vorhandene, wiederverwendbare Funktionen

| Kategorie | Einträge | Berechnung / Daten |
|---|---|---|
| Körper | World/Ground, starrer Körper, Nutzlast | Native CAD-Körper, Massen und Trägheiten; Nutzlast am CAD-Port |
| Verbindungen | Starr, Revolute, Prismatic | CAD-Gelenke; offener starrer Gelenkbaum |
| Bewegung | Freie Quelle, gemeinsame Steuerung, Bewegungseingang, Kosinus, Quintic, Sinus | Winkel- oder Wegvorgaben mit Einheiten und Parameterausdrücken |
| Sensorik | Gelenkkraft, Position/Winkel, Geschwindigkeit | Solverausgänge; keine erfundenen Messwerte |
| Lasten | Konstante Kraft; Porteditor auch mit Momenten | Angriff am benannten CAD-Interface |
| Ausgabe | Scope, interaktive 3D-Wiedergabe | Mehrkanalvergleich, Zeitauswahl und Wiedergabeoptionen |

Die Bibliothek zeigt Symbole aus dem vorhandenen eigenen Iconbestand. Ein
Bewegungsprofil auf einem Gelenk erzeugt eine bearbeitbare, typisierte Quelle
und bietet die explizite Verbindung an. Das vermeidet lokale Vorgaben, die bei
extern beschalteten Gelenken wirkungslos wären. Die Nutzlast kann auf einen
Körper mit CAD-Port gezogen werden; ihre Masse bleibt bearbeitbar.

## Noch keine vollständige Simscape-/Modelica-Bibliothek

Kugel-, Universal- und planare Gelenke, geschlossene Zwangsbedingungsschleifen,
Kontakt/Reibung, Federn/Dämpfer und allgemeine rückgekoppelte Signalnetze sind
in diesem Adapter nicht vollständig implementiert. Dafür werden keine scheinbar
funktionsfähigen Bibliotheksblöcke angelegt. Der Exudyn-Adapter bleibt auf acht
bewegliche starre Blöcke und offene Revolute-/Prismatic-Bäume begrenzt.

Die nächsten fachlichen Erweiterungen benötigen jeweils ein Datenmodell,
CAD-Zuordnung, Solverübersetzung, Einheitenvertrag, Dialog, Drag-and-drop und
einen unabhängigen Referenzfall. Eine reine Symbolsammlung erfüllt das nicht.

## Ausgabeauflösung

Neue grafische Projekte verwenden 201 Ausgabepunkte. Bestehende Projekte behalten
ihre gespeicherten Einstellungen. „Feine Ausgabe (neu berechnen)“ wählt 1001
Ausgabepunkte und rundet die Integrationsschritte nach oben auf ein Vielfaches
von 1000. Anschließend neu berechnen. Der Adapter begrenzt die Daten auf acht
Körper und 8000 Körper-/Ausgabeintervalle. Die Solverzeit und die Ausgabezahl
sind verschiedene Größen; mehr Datenpunkte garantieren keine höhere
Integrationsgenauigkeit. Messkurven werden nicht mit erfundenen Punkten geglättet.

Die GPU-Wiedergabe interpoliert Gelenkrotationen zwischen vorhandenen Ergebnissen
und erhält Gelenkanker. Eine höhere Ausgabeauflösung verkleinert die Intervalle,
innerhalb derer diese Anzeigeinterpolation verwendet wird. Die CPU-Ersatzanzeige
bleibt von ihrer Rendergeschwindigkeit abhängig. GIF-Ausgaben werden innerhalb
des bestehenden Pixelbudgets auf höchstens 101 Bilder reduziert; sie sind kein
Ersatz für flüssige Live-Wiedergabe.

## Parameterbibliothek

Werkstoffzustände und Kennwerte, Naturkonstanten, Schraubenkennwerte und eigene
Familientabellen bilden den vorhandenen Umfang. Ergänzt wurden M2, M3, M4, M5,
M12, M16, M20 und M24 mit einzeln überprüften Bohrungsempfehlungen aus der
Herstellertabelle von TR Fastenings. Zusammen mit M6, M8 und M10 sind elf Größen
vorhanden. Dies sind keine vollständigen Schraubengeometrien oder Festigkeits-
zertifikate. Quellen und Bedingungen stehen an jedem Datensatz.

Eine universell vollständige Parameterbibliothek ist damit nicht erreicht.
Insbesondere Lagerreihen, Profile, Passungen und Schraubenfestigkeitsklassen
benötigen noch getrennte, belegte Datensätze. Bestehende Familien lassen sich
bearbeiten, auswählen und mit Referenzen an CAD-Maße binden.
