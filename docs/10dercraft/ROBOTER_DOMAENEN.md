# Roboterarm: gemeinsames Modell in den Domänen

Start: `Start-10derCraft.cmd`, dann **Beispiele → Roboterarm – alle Domänen**.
Die alten Einzelstarter sind archiviert. Das ursprüngliche
`output/Modular-Robot-Realistic.solidcraft` bleibt als Ausgangsmodell erhalten.

Die Oberfläche bleibt SolidCraft: Start, Parameter, CAD, FEM und MKS.
Werkzeugfenster öffnen sich vorübergehend. Die neue Datei enthält die vorhandene
realistische Geometrie, native Komponenten und sechs bewegliche Achsen.

## Bedienung

1. Unter **Parameter** die eingebettete lokale Calcpad-Berechnung bearbeiten.
   `R_Oberarm`, `R_Unterarm`, `Hub_A1` bis `Hub_A6`, `F_TCP_N`, `E_MPa`,
   `nu` und `Grenze_TCP` sind die Eingaben. Zusammengesetzte physikalische
   Einheiten sind im CAD-Ausdruckssystem noch nicht allgemein unterstützt:
   `E_MPa` und `F_TCP_N` sind Zahlen mit ausdrücklich benannter Einheit.
2. Lokal berechnen, geänderte Referenzen prüfen und **Konstruktion aktualisieren**.
   Die nativen Radiusbemaßungen verwenden `ref_R_Oberarm` und
   `ref_R_Unterarm`. Die Körper bleiben normale bearbeitbare CAD-Zylinder.
3. Unter **Start** bzw. im Variantenfeld `Roboter Basis` oder
   `Roboter Leichtbau` wählen und Referenzen übernehmen. Beide Varianten
   besitzen dieselben Gelenke und Anschluss-IDs. Der Leichtbau ändert die
   beiden Vollquerschnittradien, nicht die Reichweite.
4. Unter **MKS** „Sechsachsige Bewegung berechnen“ starten. Alle sechs Achsen
   werden über eine halbe Kosinusrampe in 0,5 s angetrieben. Es werden
   CAD-Massen und Trägheiten, Schwerkraft und die TCP-Portlast verwendet.
   Anschließend Zeitschritte bzw. 3D-Ergebnisansicht öffnen; PNG/GIF ist dort
   exportierbar. Die CAD-Gelenkstellung bleibt bei der Ergebniswiedergabe gleich.
5. Unter **FEM** „Struktur berechnen“ wählen. Der aktuelle CAD-Stand und die
   Gelenkstellung liefern zwei idealisierte Kreisvollquerschnitt-Balken.
   Ergebnis: Verschiebungen, Rotationen, nominelle Normalspannungen und
   TCP-Nachgiebigkeit einschließlich Hebelwirkung der starren Anschlussmodule.

## Kraftschnittstellen

Sechs persistente, exakt an Körper und Vorkommen gebundene Ports:
`robot-ground`, `robot-upper-root`, `robot-upper-tip`,
`robot-forearm-root`, `robot-forearm-tip`, `robot-tcp`.
`frame_wrench` überträgt Kraft und Moment. Der allgemeine Portmechanismus
unterscheidet außerdem Punktkräfte und deklarierte Druckflächen.
Hier wird keine nur behauptete FEM-Flächenkopplung erzeugt.

`robot.loads.solve` wertet den ausdrücklich für **gesperrte Aktuatoren**
deklarierten statischen Lastpfad vom TCP zum Fundament aus. Der Auftrag
bleibt im Projekt; der ausführliche Bericht wird zurückgegeben. Kräfte wirken
in TCP −Z. In der Ausgangsstellung entspricht das Welt −Z. Bei einer Bewegung
dreht sich die MKS-Portlast mit dem Werkzeug. Statische Lastpfade sind keine
zusätzlichen starren MKS-Gelenke.

## Methodik und Grenzen

Die Dichten 2700/7850 kg/m³ und E = 70000 MPa, ν = 0,33 sind angenommene
Forschungsdaten, keine identifizierten Herstellerwerte. Körpervolumina werden
additiv ausgewertet; die anschaulichen Gehäuse sind überwiegend Vollkörper,
keine fertig ausgelegten Gussteile mit Motor-/Getriebeinnenleben.
Die bisher gleichzeitig modellierten Alternativwerkzeuge Schweißkopf und
Nutzlastblock werden aus der neuen Arbeitskopie entfernt. Nur der Greifer
gehört zu diesem physikalischen Beispiel. Ein echter Werkzeugwechsel bleibt
ein weiterer Schritt des Variantenmanagements.

MKS: sechs starre bewegliche Blöcke; Flansch und Greifer sind nativ starr
verbunden. Ideale vorgeschriebene Antriebe, keine Regelung, Reibung oder
Getriebelastregelung. Native Achsgrenzen werden nur für vollständig
vorgeschriebene monotone Rampen zugelassen, die innerhalb ihres gesamten
Weges bleiben. Freie Bewegung gegen Anschläge bleibt abgewiesen.

FEM: selbst implementierte, assemblierte kubische Hermite-Biegeelemente,
1–32 Elemente je Arm; direkte Lösung des eingespannten Steifigkeitssystems
mit Pivotierung. Axial- und Torsionsnachgiebigkeit werden analytisch ergänzt.
Beide Armbeiträge werden über die Starrkörperhebel bis zum TCP transportiert.
Material und Querschnitt sind je Balken konstant, Verformungen klein.
Eigengewicht, Schubnachgiebigkeit, Gehäuseversteifung, Lager-/Getriebespiel,
Kontakt, lokale Kerben und dynamische Spannungsrückrechnung fehlen hier.
Die kurzen/dicken Armglieder erfüllen die Schlankheitsannahme nicht gut;
das Ergebnis zeigt die Softwarekette, keine Bauteilfreigabe.
Es gibt noch keine Volumenvernetzung, CMS/FFRF-Kopplung oder flexible
Rückwirkung in die MKS. SIL und reale Geräteansteuerung sind nicht implementiert.

## Änderungsverfolgung

FEM-Bericht und MKS-Ergebnis werden im CAD-Projekt mit ihrem Eingabestand
gespeichert. Änderungen an Berechnung, CAD, Material, Ports oder Gelenkstellung
machen Ergebnisse historisch. Reine neue Solverergebnisse invalidieren die
jeweils andere Domäne nicht. Bei großen CAD-Dokumenten identifiziert SHA-256
den normalisierten nativen JSON-Stand; Titel und reine Ergebnis-Metadaten
sind ausgenommen. Die CAD-Datei und die archivierten Solveraufträge gehören
zusammen zur Reproduktion. Das ist keine kryptografische Beglaubigung eines
externen Ergebnisproduzenten.

## Engine / MCP

Alle Aktionen verwenden registrierte Engine-Kommandos, ohne zusätzlichen Server:

```json
{"command":"robot.inspect","params":{}}
{"command":"robot.motion.run","params":{}}
{"command":"robot.fem.solve","params":{"elements":8}}
{"command":"robot.loads.solve","params":{}}
```

`robot.prepare` erweitert atomar eine Kopie der ursprünglichen nativen
Roboterbaugruppe. Es verweigert bestehende Berechnungsdokumente und vermischte
Werkzeugtransfers. Der normale Benutzer startet direkt das erzeugte Projekt.

Gezielte Abnahme: `python -B tools/verify_robot_domains.py`.
Ergebnisse, CAD-Dateien, Aufträge und Screenshots unter `output/robot-domains`;
Rohläufe unter `output/mks-runs`. Der Test prüft zwei Varianten, zwei Kräfte,
Netzdiskretisierung, geschlossene Balkenlösungen, sechsachsige Bewegung gegen
native Gelenkkinematik, Speichern/Laden und ungültige Eingaben.

Methodische Grundlagen: [TU Delft – Euler-Bernoulli-Steifigkeitsmatrix](https://oit.tudelft.nl/CIEM5000/2026/lecture1/other_elements.html)
und [Exudyn GenericJoint](https://exudyn.readthedocs.io/en/latest/docs/generated/items/ObjectJointGeneric.html).
Die Formeln und Adapter sind eigenständig implementiert; kein fremder Solvercode
wurde übernommen.

## Gemessener Softwarestand der gezielten Abnahme

Beide Varianten bestehen den durchgängigen Test. Bewegte illustrative CAD-Masse:
149,3272 kg (Basis), 134,8760 kg (Leichtbau). TCP-Verschiebung bei 100 N:
0,0579006 mm bzw. 0,1136191 mm. TCP-Weg der sechsachsigen Bewegung:
174,3627 mm; Endpunktabweichung zur nativen CAD-Kinematik 5,58e-10 mm.
Das sind numerische Modellversuche mit den oben genannten Annahmen.

Desktop-Build, Clippy Engine/UI und WASM-Engine-Check bestanden.
Zusätzlich bestehen acht analytische Massenreferenzen einschließlich versetzter
kleiner Schraubengeometrie und Ablehnung offener Netze. PNG/GIF des Roboters
einschließlich Originalfarben, reproduzierbarer Kamera und unveränderter
CAD-/Ergebnisgültigkeit separat geprüft.
