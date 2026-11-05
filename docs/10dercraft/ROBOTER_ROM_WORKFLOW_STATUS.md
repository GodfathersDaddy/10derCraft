# Roboterworkflow

Stand 05.11.2026. Dieses Dokument unterscheidet nachgewiesene Funktionen und die noch erforderliche Integration des detaillierten Sechsachsarms.

## Gemeinsame Bedienung

Die Bibliothek im CAD verwendet parts.library und parts.insert. Schrauben, Muttern, Scheiben, Stifte und Lager entstehen als native editierbare Komponenten. Ergänzt ist die Familie stepper_motor mit 17HS19-2004S1. Das eigene Montagemodell enthält 42 × 42 × 48 mm Gehäuse, Zentrierbund Ø22 × 2 mm, Ø5-Welle mit 24 mm Überstand und 15-mm-D-Abflachung sowie 31-mm-Lochabstand und M3-Kernbohrungen mit 4,5 mm Tiefe. Seine Ersatzdichte liefert die Hersteller-Gesamtmasse 390 g. Die Trägheit folgt der homogenen Hülle. Elektrischer Antrieb, Rotorträgheit und Drehmoment-Drehzahl-Kennlinie erfordern eigene Modelle. Die Auswahl dieses Motors für die Bibliothek legt keine Motorisierung der großen Roboterachsen fest.

MKS enthält einen Bibliothekseintrag Flexibler Körper / ROM. Der Eintrag dient zur Zuordnung eines vorhandenen CAD-Körpers. Auf einen Körper gezogen öffnet er dessen Einstellungen. Dort lassen sich Bauteil-FEM/ROM anlegen, eine aktuelle Reduktion aktivieren und deren Ergebnisse öffnen. Der Dialog zeigt Quellenstatus und ursprüngliche/reduzierte Freiheitsgrade. Zusammenfassungen werden nach Dokumentrevision zwischengespeichert.

Ein ROM verändert die Verformungsdarstellung des vorhandenen mechanischen Körperblocks. Seine Gelenk- und Flächenanschlüsse bleiben zugeordnet. Mehrere aktive ROMs für denselben Block werden vom bestehenden Binder geprüft. Die Anzeige ersetzt das ausgewählte Bauteilnetz aus modalen Koordinaten und Referenzbewegung. Für ein aktuelles MKS-Ergebnis wird in der Bauteilansicht dessen Zeithistorie angeboten. Statische FEM und dynamische Ausgabe sind getrennt auswählbar. Änderungen an Parametern/CAD/Flächen machen zugehörige Ergebnisse historisch.

## Nachweise

Die analytische Volumenrechnung des Motormodells ergibt 85760,5489 mm³. Native Geometrie ergab 85760,5006 mm³. Das automatische Platzieren an einer Montageplatte wurde anhand der Welt-Bounding-Box und der Masse geprüft. Oberfläche und numerische Nachweise liegen in output/robot-rom-library. Die kurzen FFRF-Prüfungen für einen freien Körper, einen getriebenen Körper und zwei gekoppelte flexible Körper wurden erneut ausgeführt. Belege liegen in output/fem-rom/flexible-chain-verification.json.

Der vorhandene parametrisierte Sechsachsarm wird separat an den Fundamentbefestigungen auf echte ISO-7089-Scheiben, ISO-4017-M16×80-Bolzen und 17,5-mm-Durchgänge umgestellt. Die weiter vorhandenen Achsgehäuse, Leitungen und Armgeometrien stammen aus dem bisherigen Modell. Ausgabe und aktueller Prüfstatus dieses Aufbaus liegen in output/robot-standard-parts.

Am Greifer ist der Bibliotheksmotor auf einer 4-mm-Aluminiumadapterplatte montiert. Der neue Parameter t_greifer_motorplatte treibt deren Höhe. Adapterbohrungen und Freiräume berücksichtigen Zentrierbund, Welle, vorhandene Führungen und Deckelschrauben. Vier ISO-4762-M3×8 halten den Motor an der Platte, vier M4×12 mit vier Scheiben befestigen die Platte am Greifer. Die Hardware besteht aus nativen Bibliothekskomponenten mit Stahlzuweisung. Der Motor ist in dieser Stufe eine mitgeführte starre Masse. Das Fingergetriebe und die elektrische Ansteuerung bleiben eigene Aufgaben.

Aktuelles Roboterprojekt ist output/robot-gripper-motor/Roboter-Parameter-Motor-Normteile.solidcraft. Die vorhandene Parameterberechnung bleibt darin enthalten. Nicht-gewindetragende Passflächen wurden auf Freigängigkeit geprüft. Gewollter Gewindeeingriff in Kernbohrungen ist dabei separat zu behandeln. Der achtsekündige Lauf der sechs Achsen mit zusätzlicher Motor-/Adaptermasse ergab einen maximalen Gelenkpositionsrest von 2,168e-9 m. Es wurde kein Festigkeitsnachweis der Adapterverschraubung durchgeführt.

## Noch erforderliche Schritte für die vollständige Kette

Der detaillierte Arm besitzt mehrteilige starre Gruppen mit Motor-/Lagergehäusen und Befestigungsteilen. Der derzeitige automatische Volumenmesher verarbeitet vollständige Boxen oder Zylinder. Solche mehrteiligen Gruppen brauchen ein konformes importiertes Volumennetz mit expliziter Modellierung ihrer Verbindungen. Der Binder fordert die vollständige Massen- und Bauteilabdeckung. Das verhindert, dass eine reduzierte nackte Armstrebe stillschweigend die Massenträgheit einer ganzen Motor-/Schraubenbaugruppe ersetzt.

Für die vollständige Roboter-Umrüstung sind je Armgruppe die elastischen Domänen, als starr behandelten Anbauteile, Lageranschlüsse und Verbindungsannahmen festzulegen. Danach folgen Netzkonvergenz, Modenkonvergenz, Zeitschrittvergleich und Referenzläufe über die vorgesehene Bewegung. Hersteller-Motoren und Getriebe werden anhand dieser Lasten ausgewählt. Ein vollständiger flexibler Sechsachsarm mit allen Detailbauteilen ist durch die bisherigen Zweikörperprüfungen noch nicht qualifiziert.

## Abschließende Prüfgrenze

Build, Listen-, Schichtungs- und UI-Containerprüfung sowie die gezielten Bibliotheks-, ROM- und Bewegungsprüfungen wurden bestanden. Die zusätzliche erneute Fensteröffnung für eine Greifermotor-Nahaufnahme blieb beim Grafikstart ohne erste Antwort des UI-Kanals hängen. Auch isolierte Einstellungen und der sichere Startmodus lieferten im begrenzten Beobachtungszeitraum keine Antwort. Deshalb wird für diesen zusätzlichen Starttest kein Erfolg ausgewiesen. Die zuvor aufgenommenen Screenshots und die geprüfte gespeicherte Baugruppe sind verfügbar. Eine vollständige CI-Prüfung wurde in diesem Durchlauf nicht ausgeführt.

## Quellen

- https://www.omc-stepperonline.com/download/17HS19-2004S1.pdf
- https://www.omc-stepperonline.com/nema-17-bipolar-59ncm-84oz-in-2a-42x48mm-4-wires-w-1m-cable-connector-17hs19-2004s1
- https://exudyn.readthedocs.io/en/latest/docs/generated/items/ObjectFFRFreducedOrder.html
- https://jgerstmayr.github.io/EXUDYN/docs/RST/ModelOrderReductionAndComponentModeSynthesis.html
