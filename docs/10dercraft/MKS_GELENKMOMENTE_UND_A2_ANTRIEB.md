# Gelenkmomente und erster realer Achsantrieb

Stand 06.11.2026. Dieser Entwicklungsschritt ergänzt native Gelenk-/Sensorfunktionen und eine eigenständig modellierte Kaufteil-Montagegeometrie. Die fertigbare Gesamtkonstruktion und die flexible Reduktion der detaillierten Armgruppen bleiben in Ausarbeitung.

## Normale Bedienung

Im MKS-Gelenkblock unter Sensing lassen sich zusätzlich Mx, My, Mz und der Betrag |M| aktivieren und mit Scope verbinden. „Alle Gelenkmomente“ zeigt die Momentenbeträge mehrerer Gelenke. „Alle Drehachsen Mz“ wählt deren lokale Z-Komponenten. Vorhandene Ergebnisse ohne diese Daten benötigen „Neu berechnen“. Die bisherigen Kraft- und Koordinatenkanäle behalten ihre gespeicherten Nummern.

Starre und gemischte FFRF-Baugruppen verwenden hierfür TorqueLocal-Sensoren am tatsächlichen Exudyn-GenericJoint. Die Engine prüft Anzahl, Länge und endliche Werte der Momentenreihen, wenn sie vorhanden sind. Historische Dateien ohne Momentenreihen bleiben lesbar. Der erste Reaktionswert wird in den Diagrammen ausgelassen.

Die lokale Komponenten-Konvention des GenericJoint ist bei der Interpretation maßgebend. Bei den hier vorgegebenen einachsigen Drehbewegungen wird Mz für die vorläufige Antriebsabschätzung verwendet. Der Gesamtbetrag |M| enthält auch die übrigen Reaktionsmomente. [Exudyn GenericJoint](https://exudyn.readthedocs.io/en/v1.13.6/docs/generated/items/ObjectJointGeneric.html)

## Vorläufige Lastableitung

Der achtsekündige Rechenlauf enthält 501 Ausgabezeitpunkte. Für Achse 2 ergeben sich im aktuellen Servoeinbau-Entwurf rund 590 Nm Spitze und 373 Nm zeitgewichteter RMS-Wert der Mz-Komponente. Bei einer angenommenen Übersetzung von 120, einem Wirkungsgrad von 0,75 und einem Zuschlagsfaktor von 1,5 entstehen etwa 6,22 Nm RMS- und 9,83 Nm Spitzenanforderung auf der Motorseite. Die maximale aus der Gelenkgeschwindigkeit abgeleitete Motordrehzahl beträgt rund 1584 U/min.

Diese Zahlen gelten für das vorhandene Bewegungsfenster und die aktuellen CAD-Massen. Die übrigen Achsantriebe und Gehäuse enthalten weiterhin Modellannahmen. Wirkungsgrad und Zuschlag sind Annahmen. Rotorträgheit, thermischer Dauerbetrieb, Reibung, Bremsfall und ein vollständiges Lastkollektiv müssen gesondert bewertet werden. Ein RMS-Wert über acht Sekunden ist keine thermische Dauerbetriebsfreigabe.

## Bibliothek und Montagebaugruppe

Unter Einfügen und Bauteile steht die neue Familie servo_motor mit MSMF202L1H8 bereit. Sie erzeugt normale native Box-, Zylinder- und Schnittfeatures. Die Montagegeometrie verwendet den öffentlichen Panasonic-Katalog, gedruckte Seiten 129–130, PDF-Seite 66. Wesentliche Werte sind 100-mm-Flansch, 115-mm-Lochkreis, vier 9-mm-Bohrungen, 95-mm-Zentrierbund und 19-mm-Welle mit 55 mm Überstand. Die Variante besitzt eine Haltebremse und eine Katalogmasse von 6,6 kg. Das Gehäuse wird als vereinfachte zylindrische Einbauhülle behandelt. Eine Ersatzdichte bildet die Gesamtmasse ab, die Massenträgheit folgt dieser Hülle. Elektrischer Antrieb und Rotorbewegung erfordern eigene Modelle. [Panasonic MINAS A6](https://mediap.industry.panasonic.eu/assets/imported/industrial.panasonic.com/ac/cdn/e/motor/fa-motor/ac-servo/catalog/minas-a6_ctlg_e.pdf)

Die native Unterbaugruppe A2-Servomotor-Montage enthält den Motor, eine 15-mm-Adapterplatte mit Zentrieraufnahme, vier native M8-Innengewindefeatures, vier ISO-4762-M8×25-Schrauben und vier ISO-7089-Scheiben. Die Hardware ist mit Stahl belegt. Die Innengewinde sind kosmetisch dargestellt. STEP enthält deren Kernlochgeometrie, die Gewindebezeichnung bleibt im nativen CAD-Projekt und den Fertigungsangaben erhalten. Die Platte besitzt einen parametrischen äußeren Lochkreis. Lieferantentoleranzen und endgültige Anschlussabmessungen sind weiter festzulegen.

Im separaten Roboterentwurf ist der Motor samt Platte als Einbauanordnung platziert und der Stator starr mit dem vorgelagerten Achsteil axis1 verbunden. Die bisherige illustrative A2-Motorhülle wurde entfernt. Der Welleneinbauraum im Achsgehäuse ist freigestellt. Eine ideal starre CAD-Verbindung beschreibt den vorgesehenen Anschluss. Die dazugehörige lasttragende Halterung, Getriebeanbindung und Lagerung müssen noch konstruiert werden.

Als Getriebekandidat wird die CSG-50-Familie weiter untersucht. Die Übersetzungsvariante und ihre Anschlusszeichnung müssen zur gewählten Motorkombination passen. [Harmonic Drive CSG-50](https://global.harmonicdrive.net/products/component-sets/cup-type/csg-2a/csg-50-100-2a-gr)

## Nachvollziehbarkeit

Der neue Roboterentwurf enthält eine erneut berechnete Starrkörperbewegung und die separate Balken-Vorberechnung. Momentenverläufe und vorläufige Antriebsanforderungen liegen unter examples/manufacturing/robot-8kg und docs/10dercraft/evidence/robot-axis-drive. Der lokale Herstellerkatalog wird als Literaturquelle außerhalb des Repositorys aufbewahrt.

Die Anwendung wurde gebaut und die normalen CAD-/Berechnungsbefehle ausgeführt. Die neuen Scope- und Gelenkansichten wurden in einer isolierten nativen Instanz mit OpenGL dargestellt und aufgenommen. Breite Tests bleiben auf Benutzerwunsch pausiert. Damit sind weder die Fertigungsfreigabe des Gesamtarms noch die flexible FEM–ROM–MKS-Kopplung der detaillierten Achsgruppen nachgewiesen.


## Fortschreibung mit flexiblen Rohr-ROMs

Das später am 06.11.2026 berechnete flexible Modell ergibt für A2 etwa 403,65 Nm RMS über die acht Sekunden und 875,48 Nm maximale Mz-Reaktion. Unter denselben vorläufigen Annahmen werden motorseitig 6,73 Nm RMS und 14,59 Nm Spitze benötigt. Der 2-kW-Montagekandidat mit 6,37 Nm Nennmoment deckt diese RMS-Abschätzung nicht ab. Eine weitere Dimensionierung erfordert Netz-, Moden- und Zeitschrittbewertung, Anfangsgleichgewicht und den realen Betriebszyklus. [Aktueller flexibler Workflow](FLEXIBLER_ROBOTER_WORKFLOW.md)
