# CAD → MKS ohne Programmierung

## Zuständigkeit der Modelle

Die CAD-Baugruppe definiert die Bauteile, ihre Geometrie, Materialien, Lage und Gelenke. MKS liest diese Daten erneut ein, sobald sich das CAD-Projekt ändert. Starr miteinander verbundene Teile werden zu einem starren MKS-Körper mit gemeinsamer Masse und Trägheit zusammengefasst. Ein MKS-Körperblock ist deshalb nicht zwingend ein einzelnes CAD-Teil. Die Bilder und Bauteilnamen im Block zeigen seine Herkunft.

Mechanische Verbindungen zwischen Körpern werden im CAD angelegt. Die blauen Leitungen im MKS sind Bewegungsvorgaben; die grünen Leitungen sind Messsignale. Eine grüne Leitung zwischen Sensor und Scope ist keine mechanische Verbindung und überträgt keine physikalische Kraft auf ein anderes Bauteil.

## CAD vorbereiten

1. Eine Baugruppe mit separaten beweglichen Komponenten konstruieren. Das Fundament festsetzen und Materialien zuweisen. Der lokale Dynamikadapter benötigt daraus positive Massen und Trägheiten.
2. Eine ebene Anschlussfläche auswählen.
3. Unter **CAD → Zusammenbau → CAD-Simulationsschnittstellen** den Reiter **Fläche erfassen** öffnen.
4. Eine eindeutige ID vergeben, z. B. `IF_Schulter_B`. Bei mehreren Instanzen desselben Bauteils die richtige Instanz wählen.
5. **Flächenrahmen: Kraft und Moment** für einen mechanischen Anschluss oder **Punktkraft im Flächenzentrum** für einen reinen Kraftangriff wählen. **Ausgewählte Fläche erfassen** legt den Bezug im Projekt an.
6. Den Gegenanschluss auf dem zweiten Bauteil ebenso erfassen.
7. Im Reiter **CAD-Gelenk** B- und F-Anschluss sowie Starr-, Dreh- oder Schubgelenk auswählen. **CAD-Gelenk erstellen** verwendet den normalen CAD-Baugruppenlöser. Dabei können Teile an den Anschlussrahmen ausgerichtet und bewegt werden.
8. **Abgeleitetes MKS-Modell öffnen** wählen. Vorhandene native CAD-Gelenke werden ebenfalls übernommen; die neue Erfassung ist kein zweiter Baugruppenlöser.

Der Bezug wird über den nativen benannten Gelenkursprung an der Fläche gespeichert. Er wird nach CAD-Änderungen neu aufgelöst. Die Kraftschnittstelle verwendet denselben Bezug. Eine verlorene Fläche muss repariert werden; es wird nicht still auf eine andere Koordinate ausgewichen.

Die einfache Erfassung unterstützt zunächst ebene Flächen direkt eingesetzter Komponenten. Für zylindrische Ursprünge und komplexere Baugruppen stehen die vorhandenen nativen CAD-Gelenkursprungswerkzeuge zur Verfügung. Der neue Dialog zeigt auch solche benannten Ursprünge. Der MKS-Adapter unterstützt weiterhin offene starre Dreh-/Schubgelenkketten.

## Bewegung und Messung im MKS

1. **Freie Bewegungsquelle** aus der linken Bibliothek einfügen. Name, Winkel-/Wegausdruck, Profil und Rampenzeit einstellen und speichern.
2. Den blauen Ausgang mit gedrückter linker Maustaste auf den q-Eingang des Gelenks ziehen. Alternativ bleiben Ausgangsklick und Eingangsklick möglich. Das Ersetzen eines bisherigen Antriebs im Dialog bestätigen.
3. Einen Gelenksensor über die Bibliothek am gewünschten Gelenk ergänzen oder im Gelenkdialog **Sensorik / Ausgänge** aktivieren. Winkel, Geschwindigkeit und/oder Kraftkomponenten auswählen.
4. Den grünen Ausgang zum Scope-Eingang ziehen. Doppelklick auf den Sensor öffnet seine Messauswahl; Doppelklick auf die Quelle ihre Berechnungsvorgabe.
5. **Simulation starten** berechnet die Bewegung. Die rechte Ansicht zeigt die berechnete CAD-Bewegung und benutzt die CAD-Navigation. Im Scope die gewünschte physikalische Größe auswählen.
6. Für eine Endlast **Nutzlast / Gewicht** wählen und den im CAD erfassten Anschluss des beweglichen Bauteils auswählen. Es entsteht ein normales CAD-Bauteil mit Material und einer nativen starren Verbindung. Danach neu simulieren.
7. Das Projekt über die normale Speicherfunktion sichern. Schnittstellen, Gelenke, Quellen und Darstellung bleiben gemeinsam erhalten.

Eine grüne Leitung kann ausgewählt und über **Messleitung trennen** entfernt werden. Der Sensorblock und seine eingestellten Messgrößen bleiben erhalten. Sein grüner Ausgang lässt sich anschließend wieder zum Scope ziehen. Neu aus der Bibliothek ergänzte Sensoren müssen auf diese Weise mit einer Ausgabe verbunden werden; ältere Projekte behalten ihre bisherigen Verbindungen.

Die Messleitungen werden auf getrennten Spuren außerhalb der Bauteilblöcke zum Scope geführt. Der Scope wird beim automatischen Anordnen neben diesem Leitungsbereich platziert. Pfeile zeigen die Signalrichtung; eine Kreuzung ohne Verbindungspunkt ist keine Signaladdition. Die neue automatische Linienführung berücksichtigt Blockrechtecke und bevorzugt wenige Knicke. Sie ersetzt keine allgemeine Schaltplan-Optimierung.

## Bedeutung von Punkt und Fläche

Im aktuellen starren MKS-Modell wird ein Flächenanschluss durch einen Rahmen und die resultierende Kraft bzw. das Moment beschrieben. Eine Punktkraft darf kein freies Moment übertragen. Die erfasste Fläche ist ein geometrischer Bezug; diese Ausbaustufe verteilt Lasten noch nicht auf ein FEM-Netz und implementiert keine verteilte Kontakt- oder Druckrechnung. Für FEM sind Auswahl der tatsächlich belasteten Flächen, Netzabbildung und eine passende Kopplung zusätzlich nötig.

## Prüfnachweise

`tools/verify_mks_cad_workflow.py` benutzt normale CAD-Komponenten als Ausgangsmodell und führt Flächenauswahl, Schnittstellenerfassung, Gelenkerstellung, Quelle, Ziehverbindungen, Sensorfenster, Nutzlast und Simulationsstart über die Oberfläche aus. Ein separater Parametervergleich prüft die Assoziativität des Flächenrahmens. Der numerische Sollverlauf wird unabhängig berechnet. Zusätzlich werden Dateirundlauf, kleine Fenstergröße und die einzelnen Messleitungen des Roboters geprüft.

Abschlussstatus und Screenshots stehen unter `output/mks-cad-workflow/`. Nur ein Bericht mit `passed: true` bestätigt die vollständige beschriebene Prüfung.
