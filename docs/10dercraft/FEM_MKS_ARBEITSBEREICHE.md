# FEM und MKS im Projekt

Start mit `Start-10derCraft.cmd`, dann **Beispiele → Roboterarm – alle Domänen**.

Neu: eigener Gelenk-/Signalgraph und das einfache Pendel unter Beispiele.
Siehe [MKS-Blockbedienung](MKS_BLOCKBEDIENUNG.md).

## MKS

Der obere Reiter öffnet eine große Blockansicht statt eines Werkzeugfensters.
Jede geometrisch befüllte Blockkarte erhält ein aus den tatsächlich zugeordneten
CAD-Körpern und ihren Einbaupositionen gerendertes Bild. Mehrere starr verbundene
Teile erscheinen gemeinsam. Die Bilder werden im Hintergrund erzeugt und nach
Änderung des Modellstands erneuert. Namen und Einbaupfade bleiben zusätzlich
verfügbar. Sehr große Geometrien behalten die beschriftete Karte als Ersatz.

Antriebe und Berechnung sind aufklappbar. Schnittstellen lassen sich ausdrücklich
über einen vorübergehenden Editor öffnen. MKS-Ergebnisse und die Konstruktion
bleiben unterschiedliche Zustände; die bestehende Bewegungsauswertung bleibt nutzbar.

## FEM

Der obere Reiter öffnet die FEM-Werkzeugleiste und lässt zunächst das normale CAD
darunter sichtbar und auswählbar. **Arm berechnen** führt beim vorbereiteten
Roboter die vorhandene Balken-FEM mit 16 Elementen pro Armglied aus.
**Ergebnisse** wechselt zur integrierten Verformungsansicht, **CAD / Vorbereitung**
zurück zur Konstruktion. Der Reiterwechsel selbst öffnet kein Fenster.

Die Ergebnisansicht zeigt Verschiebungsbeträge in Millimetern mit einer gemeinsamen
blau–grün–roten Skala. Der Faktor, beispielsweise x1000, überhöht nur die Geometrie.
Farben und Zahlen bleiben unskalierte berechnete Verschiebungen. Ober- und Unterarm
verwenden die kubischen Hermite-Verschiebungen, axiale Verschiebung und
Querschnittsrotation; die Nachgiebigkeit des vorgelagerten Armglieds wird addiert.
Die restliche CAD-Geometrie ist eine transparente, unverformte Referenz. Das
Darstellungsnetz der Kreisquerschnitte ist axial unterteilt, unabhängig vom CAD-Netz.
Kamerabedienung erfolgt wie im CAD. Historische Ergebnisse werden nicht gerendert.

Der vorhandene Schnittstelleneditor unterstützt lokale Punkt-/Rahmenlasten und
deklarierte Flächen. Das Anklicken einer CAD-Fläche erzeugt bisher noch keine
allgemeine FEM-Flächenlast oder Einspannung. Der FEM-Reiter allein ergänzt das nicht.

## Bildexport und Nachrechnung

Der registrierte Befehl `robot.fem.snapshot` ist auch per MCP/Control verfügbar:

```json
{
  "path": "Arm-Verformung.png",
  "scale": 1000,
  "width": 800,
  "height": 500,
  "view": "iso",
  "overwrite": false
}
```

Er verlangt einen aktuellen FEM-Stand und ändert keine CAD-Geometrie. Der
Rückgabewert dokumentiert Darstellungsfaktor, Farbgröße, Maximalwert und Kamera.
Eine explizite Kamera ermöglicht Bildvergleiche mit unverändertem Ausschnitt.

`tools/verify_analysis_workspaces.py` prüft die Bedienung und erzeugt unter
`output/analysis-workspaces` eine separate Projektkopie, Referenz-/Verformungsbilder,
fünf quasistatisch gerechnete Bewegungsstellungen, eine GIF-Posenfolge und den
numerischen Bericht. Die Originalprojektdatei wird über ihren SHA256 geprüft.
Die Posenfolge enthält keine dynamischen Spannungen, kein FEM-Eigengewicht und
keine flexible Rückkopplung in die MKS. Sie ist eine erste Strukturauswertung
entlang der vorhandenen Bewegung und keine vollständige elastische Armsimulation.

Vergleich und Quellen: [PrePoMax-Workflow](Literatur/PrePoMax_Workflow_2026-10-28.md).
