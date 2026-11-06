# Entwicklungsschritt vom 06.11.2026

Die native Servo-Familie MSMF202L1H8, die motorseitige Montage-Unterbaugruppe und die Momentenausgänge für starre sowie FFRF-Modelle sind implementiert. Die Anwendung wurde gebaut und für den normalen lokalen Starter installiert. Die MKS-Oberfläche mit Scope und Gelenkausgängen wurde in einer isolierten OpenGL-Instanz dargestellt und aufgenommen.

Die neu berechnete Starrkörperbewegung des Servo-Einbauentwurfs liegt im nativen Projekt. Der Stator ist dem vorgelagerten Achsteil zugeordnet. CSV-Momentenreihen und eine vorläufige Antriebsabschätzung sind beigelegt. Andere Achsgehäuse und Antriebe enthalten weiterhin angenommene Geometrien und Massen.

Breite Tests und vollständige CI bleiben auf Benutzerwunsch pausiert. Normale CAD-Konstruktion, Anwendungsbuild und angeforderte Projektberechnungen wurden ausgeführt. Die reale lasttragende Halterung, Getriebe und Lagerung, vollständige flexible Detailgruppen-ROMs sowie deren Rückkopplung und Fertigungsfreigabe stehen aus. Die motorseitige Adapterplatte ist ein bearbeitbarer Entwurf mit kosmetischen Gewinden und nominalen Einbaumaßen.

Die frühere Startblockade der zusätzlichen Bildschirmaufnahme ließ sich für die aktuelle isolierte Darstellung mit dem OpenGL-Backend umgehen. Die normalen Benutzereinstellungen wurden dabei nicht angepasst.


## Nachfolgende flexible Kopplung

Zwei Hohlprofil-ROMs mit je 3.888 vollen und zwölf reduzierten elastischen Freiheitsgraden wurden an die native Sechsachsbaugruppe gekoppelt. Alle übrigen CAD-Körper der beiden Gruppen bleiben mit ihrer Massenträgheit als explizite starre Anbauteile erhalten. Die vollständige achtsekündige Bewegung mit 4.000 Integrationsschritten und 501 Ausgabepunkten wurde berechnet und nativ gespeichert. Maximale elastische Knotenverschiebung etwa 0,203 mm am Oberarm und 0,116 mm am Unterarm. Maximaler Gelenkpositionsrest 1,923e-9 m. Dieser Rest misst die Zwangsbedingungserfüllung.

Zwei vorangegangene Versuche wurden durch das bisherige 30-Sekunden-Prozesslimit beendet. Flexible Rechnungen besitzen jetzt ein begrenztes 180-Sekunden-Zeitfenster. Rechenfäden der Zahlenbibliotheken sind im Kindprozess standardmäßig auf einen begrenzt. Eine allgemeine Beschleunigung ist daraus noch nicht abgeleitet. Die vollständige Rechnung wurde anschließend erfolgreich über mks.job.run ausgeführt.

Die native Ergebniswiedergabe, gemeinsame Mz-Diagramme und die dynamische Bauteilansicht mit Netz wurden in isolierten OpenGL-Fenstern aufgenommen. Ein nativer AVI-Export mit 241 Bildern bei 30 FPS ist gespeichert. Der FEM-Screenshot verwendet einen 300-fachen Darstellungsmaßstab. Der neue Quellstand ist gebaut und die statische Listenprüfung ist bestanden. Breite Testreihen und CI bleiben pausiert. Tragende Achsanschlüsse, vollständige Motor-/Getriebe-/Lagerauslegung und Fertigungsfreigabe stehen weiterhin aus.


## Optionaler statischer ROM-Start

Ein weiterer nativer Lauf ergänzt den wählbaren statischen Anfangszustand. Der MKS-Reiter enthält dafür die Option Statisch vorspannen. Eigengewicht und konstante Anschlusslasten werden zunächst bei gehaltenen Gelenken äquilibriert. Das neue Beispiel Roboter-8kg-Statisch-vorgespannt.solidcraft enthält das aktuelle gespeicherte Ergebnis. Vergleich und Reaktionskonvention stehen in ROBOTER_STATISCHE_VORSPANNUNG.md. Die 8-s-Bewegung ergibt am unveränderten Vergleichsmodell A2-Mz-Spitze 597,267 Nm und RMS 373,040 Nm. Die vorherigen Werte beziehen sich weiterhin auf den unverformten Start.
