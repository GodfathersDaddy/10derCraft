# Direktes Berechnungsblatt

Die Formel wird auf dem Blatt per Doppelklick geöffnet und dort mit der Tastatur bearbeitet.
Ein separates Fenster ist dafür nicht nötig. Mit `:=` wird eine Variable definiert;
mit `=` wird die aktuelle Eingabe berechnet. Danach ist die Ergebniseinheit zur
Eingabe ausgewählt. Enter übernimmt eine Eingabe, Escape verwirft die aktuelle
noch nicht übernommene Eingabe. Strg+A markiert den Inhalt des aktiven Felds.

Brüche, Potenzen und Indizes werden zweidimensional dargestellt. In der Eingabe
bezeichnet `/` einen Bruch, `^` eine Potenz und `_` einen Index. Beispielsweise
erscheinen `b_1` mit tiefgestellter 1 und `10^-3` mit hochgestelltem Exponenten.
Der Cursor kann auch direkt in Zähler, Nenner oder Exponent gesetzt werden.

Die Einheit rechts vom Ergebnis ist direkt bearbeitbar. Bei kompatiblen Einheiten
wird der Zahlenwert umgerechnet, beispielsweise 15 mm in 1,5 cm. Bei einer
Längeneingabe zu einer zusammengesetzten mechanischen Einheit wird eine passende
Komponente gesucht: 2 Nm mit `mm` wird 2000 Nmm. Bei unverträglichen Dimensionen
bleibt die gewünschte Einheit ausdrücklich als Faktor erhalten: 2 s mit `m`
erscheint als `2 s · (1/m) · m`. Die beiden Meterfaktoren werden absichtlich nicht
weggekürzt. Unlesbare Einheitensyntax wird weiterhin als Fehler gemeldet.

Diese Ausgabeumrechnung verändert keine ursprünglichen Rechenergebnisse. CAD
verwendet weiterhin die ursprünglichen Werte und die expliziten CAD-Referenzen.
Auch die Anzahl signifikanter Stellen ist nur eine Anzeigeeinstellung: intern
wird mit der vollen verfügbaren Gleitkommagenauigkeit gerechnet. Unnötige
Nachkommanullen werden weggelassen.

Oben „Referenz auswählen“ einschalten und auf den Variablennamen klicken. Der
Name wird hervorgehoben und unten links in der scrollbaren Referenzliste
angezeigt. Für CAD-kompatible Einheiten wird eine CAD-Referenz erzeugt; eine
zusammengesetzte Größe wird ebenfalls gelistet, aber ihre Dimension nicht
stillschweigend entfernt, um sie als Längenmaß zu verwenden.

Normales Ziehen verschiebt ein Feld. Erst nach dem Doppelklick bearbeitet oder
markiert Ziehen den Text. Anordnen beziehungsweise Alt erzwingen das Verschieben. Das Blatt
verwendet weiterhin normale lokale Calcpad-Berechnungen ohne Berechnungsserver.

Gezielte Nachweise liegen unter `output/worksheet-inline/`. Die Geschwindigkeits-
änderungen der MKS sind getrennt in `MKS_WIEDERGABE_PERFORMANCE.md` dokumentiert.

Innerhalb einer aktiven Formeleingabe nimmt Strg+Z die letzte Eingabe zurück. Strg+Y oder Strg+Umschalt+Z stellt sie wieder her. Die Eingabehistorie umfasst bis zu 100 Schritte und wird nicht im Projekt gespeichert.


Die Bibliothek besitzt Reiter Werkstoffe, Schrauben, Konstanten / Standardwerte und
Eigene Tabellen. Datensätze zeigen Eigenschaften, Zahlen und Einheiten in Spalten.
Die Punkt-Vervollständigung gilt für alle Katalogeinträge und gespeicherten Familien:
S355. zeigt Varianten, M10. Bohrungskennwerte, SI_c. die Lichtgeschwindigkeit.
EigeneFamilie. zeigt deren Spalten; dafür zuerst eine Familienzeile auswählen.
Pfeil auf/ab wechselt den Vorschlag, Tab übernimmt. Dabei wird der stabile
Berechnungsname eingesetzt. Noch nicht vorhandene Kataloggrößen werden als normale
Definition mit Quellenkommentar eingefügt. Das erzeugt keine CAD-Referenz.
Für Schraubenbohrungen liegen Empfehlungen M6/M8/M10 vor:
https://www.trfastenings.com/Knowledge-Base/Engineering-Data/tapping-sizes-and-clearance-holes
