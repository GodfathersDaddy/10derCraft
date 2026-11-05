# Roboterarm für 8 kg Nutzlast

Der Benutzer hat den vorhandenen großen Roboterarm, 8 kg Nutzlast sowie Fräs-/Drehteile und verschraubte Profile als Entwicklungsrichtung bestätigt. Die neue Datei `examples/projects/Roboter-Rohrprofile-8kg-Entwurf.solidcraft` enthält den daraus abgeleiteten Strukturentwurf.

## Erstellte Geometrie und Berechnung

Oberarm und Unterarm erhalten durchgehende Innenbohrungen als normale native Zylinder-Schnittfeatures. Außenradien und Wandstärken sind veröffentlichte Referenzen der eingebetteten Calcpad-Berechnung. Der Oberarm hat nominal 160 mm Außendurchmesser und 8 mm Wandstärke bei 644,1 mm Länge. Der Unterarm hat 140 mm Außendurchmesser und 6 mm Wandstärke bei 505,2 mm Länge. Diese Abmessungen sind konstruktive Ausgangswerte. Materialgüte und lieferbares Halbzeug sind mit dem Lieferanten festzulegen.

Die Berechnung enthält für beide Rohrquerschnitte die Beziehungen A = π·(R² − r²) und I = π·(R⁴ − r⁴)/4 sowie eine idealisierte Masse und Einzelträgerdurchbiegung. Die gespeicherten Berechnungsvarianten wurden um die Wandstärken und Rohrformeln ergänzt. Die aktuelle Variante heißt „Metall-Rohrprofile 8kg - Entwurf“. Nach einer Berechnungsänderung erfolgt die CAD-Übernahme über „Konstruktion aktualisieren“.

Die native Balken-Vorprüfung berücksichtigt die inneren Radien, wenn die entsprechenden Wandstärkenreferenzen vorhanden sind. Sie vergleicht das aktuelle CAD-Volumen mit dem angenommenen Kreis- oder Kreisringquerschnitt und verweigert eine erkennbare Abweichung. Diese Volumenprüfung ersetzt keine lokale Geometrie- oder Festigkeitsprüfung. Vollständige Volumen-FEM/ROMs der detaillierten Armgruppen bleiben erforderlich.

Die analytischen Rohrteile stehen in `examples/manufacturing/robot-8kg` als STEP bereit. Die dortige CSV enthält die native Baugruppenstückliste. Die vorhandenen Achsgruppen enthalten weiterhin mehrere Körper pro Gruppe. Eine vollständige Fertigungsstückliste erfordert die noch auszuarbeitende Einzelteilzerlegung.

## Erforderliche mechanische Ausarbeitung

1. Tragende Rohranschlüsse und Achsgehäuse als separate bearbeitbare Einzelteile mit Zentrierungen, Schrauben, Werkzeugzugang und Montagefolge ausarbeiten. Die bisherigen geometrischen Überlappungen zwischen Armkörpern und Achshüllen brauchen reale Anschlusslösungen.
2. Für jede Achse Lastkollektiv, Drehzahl, Beschleunigung, Haltefall und Bremsanforderung bestimmen. Der 8-kg-Nennwert allein legt keinen Motor und kein Getriebe fest.
3. Lagerabstand, Wellen, Passungen, Vorspannung und Schraubverbindungen auslegen. Motorgehäuse und Anschlussmaße anhand ausgewählter Kaufteile konkretisieren.
4. Greifertransmission, elektrische Leitungsführung, Endanschläge und Wartungszugang ergänzen.
5. Reale Flächenverbindungen im CAD den FEM-Domänen zuordnen. Starr angenommene Motoren, Lager und Schrauben müssen mit ihren Massen und Trägheiten in der jeweiligen Gruppe enthalten bleiben.
6. Netze und ROM-Basen pro Armgruppe erzeugen. Danach flexible MKS mit aktuellen CAD-Massen, eindeutigen Flächenschnittstellen und Momenten-/Kraftausgaben aufbauen.

## Recherchegrundlagen

EN AW-6082 T6 ist ein zu prüfender Werkstoffkandidat für die Struktur. Das Herstellerdatenblatt unterscheidet Produkte und Zustände. In diesem Entwurf verwendet die Rechnung weiterhin angenommene Elastizitätswerte und eine Aluminiumdichte. [Hydro 6082](https://www.hydro.com/en/global/aluminium/products/extruded-profiles/alloys-for-extruded-profiles/6082-alloy/)

Für die spätere Lagerauswahl liegt der THK-Katalog als Primärquelle vor. Die Auswahl eines konkreten Kreuzrollenlagers erfordert die kombinierten Achslasten und die Anschlusskonstruktion. [THK Cross-Roller Ring](https://tech.thk.com/en/products/pdf_download.php?file=E_18_CrossRollerRing.pdf)

Die Harmonic-Drive-CSG-Reihe ist ein Recherchekandidat für die hoch übersetzten Achsen. Das Komponentengetriebe braucht eine eigene Anschluss- und Lagerkonstruktion. Eine Motor-/Getriebefreigabe wurde damit nicht getroffen. [Harmonic Drive CSG-40-100-2A-GR](https://www.harmonicdrive.net/products/component-sets/cup-type/csg-2a/csg-40-100-2a-gr)

## Prüfstatus

Die Variante wurde über normale CAD-, Berechnungs-, Referenz- und Exportbefehle erzeugt. Die Anwendung wurde mit den neuen ROM-Einstellungsfeldern und der Rohrquerschnittsbehandlung gebaut. Gemäß Benutzeranweisung wurden anschließend keine Testreihen gestartet. Physischer Prototyp, Anschlussauslegung, vollständige Kollisionsprüfung und flexible Sechsachs-Qualifikation stehen aus.
