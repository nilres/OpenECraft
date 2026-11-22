# Schaltschrankaufbau, Räume und 3D-Verlegung

### Importierte Karten auf waagerechten Hutschienen

OpenECraft richtet den Clip des Betriebsmittels auf die Mittellinie der Schiene aus. Bei den EL-E/A-Gehäusen von Beckhoff mit 12 × 100 × 68 mm und 8 oder 16 Klemmstellen sowie dem passenden Koppler EK1100 liegt die Schienenmitte 63,8 mm unter der Oberkante. Eine Schiene mit Mitte bei Y = 100 mm legt die Oberkante zum Beispiel auf Y = 36,2 mm. Das Maß 35,4 mm ist die Gehäuseöffnung für die Schiene mit 35 mm Nennmaß. Der Import behält einen von null verschiedenen Mittenversatz und eine Aufrasthöhe des Artikels bei. Den aus der Zeichnung abgeleiteten Mittenversatz wendet er nur auf passende Gehäuse an, wenn der Mittenversatz des Katalogs null ist. [8-polige Klemme](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_8_pin.pdf) · [16-polige Klemme](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_16_pin.pdf) · [Gehäuse EK1100](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Coupler/pdf/ethercat-coupler_44mm_2xrj45_8pin.pdf)

Eine Karte, die innerhalb von 4 mm neben einer freien Nachbarkante platziert oder dorthin verschoben wird, rastet auf derselben Montageplatte bündig an ihr ein. Die Platzierungsvorschau zeigt das Ergebnis. Die Darstellung im Schrank verwendet zuerst die zugewiesene Frontansicht, dann eine importierte Montagedarstellung. Fehlen beide, liefert eine Herstellerübersicht eine Abbildung, die in die physische Grundfläche eingepasst wird.

Gehäuse mit ihren Betriebsmitteln in Originalgröße in 2D und 3D (im Sinne von EPLAN Pro
Panel), virtuelle Verdrahtung durch die Kanäle, Bohr- und NC-Daten, Gehäuseklima,
Anreihungen, Räume des Gebäudes mit ihren bodenstehenden Geräten und Rohrtrassen,
19-Zoll-Racks und 3D-Verlegung von Kabeln, Schläuchen und Rohren. Die Beispielprojekte
`conveyor`, `cabinet_lineup`, `electrical_room` und `pump_room` zeigen sie.

![Die Anreihung des Beispiels cabinet_lineup in 3D, Tür offen, mit Schienen, Kanälen, Betriebsmitteln und verlegten Drähten](../../images/panel-3d.png)

## Gehäuse und Betriebsmittel

- Gehäuse (aus einem Artikel wie `GEN.ENC-600X800` oder mit eigener Größe) mit
  Montageplatte.
- Tragschienen und Kabelkanäle werden in ihrer Länge gezeichnet; eine(n) auswählen und
  an einem der Enden ziehen, um die Länge zu ändern, oder verschieben (Betriebsmittel
  wandern mit ihrer Schiene mit). Betriebsmittel werden aus einer Liste „Zu
  platzierende Betriebsmittel“ mit ihren realen Abmessungen aus den Artikeldaten
  platziert (Schütz + Hilfsschalterblock addieren sich in der Tiefe, Klemmenleisten
  sind so lang wie ihre Klemmen und Reserven) und rasten auf Schienen ein; drehen,
  löschen.
- Ausschneiden, Kopieren und Einfügen in der Frontansicht: Rechtsklick auf ein Element
  für *Ausschneiden*, *Kopieren*, *Einfügen*, *Eigenschaften* und *Entfernen*
  (Betriebsmittel kommen zurück zu den zu platzierenden Betriebsmitteln), oder
  `Strg+X` / `Strg+C` / `Strg+V`. Schienen, Kanäle, Sammelschienen, Ausschnitte und
  Zubehör werden mit den darauf montierten Elementen kopiert, Betriebsmittel werden
  ausgelassen (ein Betriebsmittel wird nur einmal platziert); ausgeschnittene Elemente
  verlassen den Aufbau, ausgeschnittene Betriebsmittel kommen zurück zu den zu
  platzierenden Betriebsmitteln. Die eingefügten Elemente folgen dem Mauszeiger, bis
  ein Klick sie platziert, auf jeder Fläche jedes Gehäuses; `Esc` oder ein Rechtsklick
  bricht ab. Der Befehl `copy_panel_items` leistet dasselbe von der Kommandozeile und
  für KI-Agenten.
- 2D-Frontansicht und eine 3D-Ansicht: Ziehen dreht die Ansicht, Ziehen mit der
  rechten / mittleren Taste oder mit Umschalt verschiebt sie, das Mausrad zoomt am
  Mauszeiger, immer näher an die feste Fläche dort, aber nicht durch sie hindurch (im
  leeren Raum geht es weiter in einen Raum hinein), ein Doppelklick dreht zum
  Angeklickten und geht näher heran, `W` `A` `S` `D` / Pfeiltasten und `Q` `E` bewegen
  durch die Ansicht (Umschalt: schneller), die Leiste in der Ansicht enthält die
  Standardansichten und *Einpassen* (`Pos1`); Klick
  zum Auswählen. Wände, Decken und
  Türen sind von außen gesehen durchsichtig (*Deckkraft der Wände* in den Einstellungen,
  unter Erscheinungsbild); *Wände davor ausblenden* lässt die Wände zwischen dem Auge
  und dem Inneren weg, *Decke ausblenden* die Decken der Räume. Drähte werden in ihrer
  Farbe gezeichnet (zweifarbige Drähte wie Grün-Gelb mit einem Streifen) und, wie Kabel
  und Schläuche, so dick wie ihr Außendurchmesser. Ein Kabel endet kurz vor dem
  Betriebsmittel, an das es angeschlossen ist, und seine Adern trennen sich von dort in
  weichen Bögen, jede in ihren Anschlusspunkt von der Seite, auf der dessen Drähte ihn
  verlassen. An seinem Anschlusspunkt endet ein Draht oder eine Ader in seiner
  Endenbehandlung: einer Aderendhülse (Metallhülse und Kragen in der Farbe nach
  DIN 46228-4) oder einer Zwillingsaderendhülse, einem Ringkabelschuh (sein Ring um den
  Anschlusspunkt), einem Gabel- oder Stiftkabelschuh, einem Flachstecker oder einer
  Flachsteckhülse (mit roter, blauer oder gelber Isolierhülse; unisolierte
  Rohrkabelschuhe ab 10 mm²) oder dem blanken Kupferleiter, so lang, wie er abisoliert
  ist. Jedes Drahtende kann in den Eigenschaften des Drahts eine eigene
  Endenbehandlung erhalten (*Verbindungsenden*), alle Adern eines Kabels an einem
  seiner Betriebsmittel in den Eigenschaften des Kabels (*Kabelenden*) und auf der
  Kommandozeile alle Enden an einem Anschlusspunkt oder einem Betriebsmittel
  (`set_termination`); Enden ohne eine erhalten eine Aderendhülse, wo *Leitungsenden
  mit Aderendhülsen* gesetzt ist, sonst den blanken Leiter. Eine Zwillingsaderendhülse
  teilen sich die zwei Drähte, die an einem Anschlusspunkt enden: Sie wird nur dort
  angeboten, für beide zugleich gewählt, für den größeren Querschnitt bemessen und
  einmal gezeichnet, beim ersten der beiden; mit
  *Zwillingsaderendhülsen, wo zwei Leitungen einen Anschlusspunkt teilen* (oder
  `set_ferrules`) erhält jedes solche Paar ohne eigene Endenbehandlung eine. Die
  Prüfung *Zwillingsaderendhülsen* warnt vor einer Zwillingsaderendhülse mit einem
  Draht, an einem Punkt mit mehr als zwei Drähten oder für zwei Querschnitte. Eine
  Aderendhülse oder ein gecrimpter Kabelschuh biegt sich nicht: Der Draht oder die Ader
  läuft gerade aus dem Anschlusspunkt heraus, so weit die Hülse reicht (mit Kragen oder
  Isolierhülse), und biegt erst dahinter. Die 3D-Verlegung plant diesen geraden Weg
  heraus, wo der Raum vor dem Anschlusspunkt frei ist; wo er es nicht ist (ein
  benachbarter Draht, ein Kabelkanal dicht daneben), biegt der Draht direkt hinter der
  Aderendhülse oder dem Kabelschuh enger.
  Die Fertigungs-Leitungsliste, der
  Verdrahtungsassistent, die Maschinenexporte und die Stückliste übernehmen dieselben
  Endenbehandlungen; durch Kabelverschraubungen, Wanddurchbrüche und die
  Kabeldurchführungen zwischen Schränken werden Kabel und Drähte in einem Stück
  gezeichnet, dort herumgebogen, mehrere durch einen Kabeleinführungsflansch oder
  Wanddurchbruch nebeneinander. Kabelverschraubungen werden mit den Modellen ihrer
  Artikel gezeichnet: außen der Sechskantkörper, der Hals und die Hutmutter auf der
  Außenfläche ihrer Wand, innen die Gegenmutter und das Ende des Gewindes auf ihrer
  Innenfläche, in vernickeltem Messing oder Polyamid; Verlegewege halten Abstand zur Gegenmutter. In einem Schrank ohne Kabelkanäle auf seiner
  Montageplatte werden die Drähte von der 3D-Verlegung gelegt (siehe unten), von
  Anschlusspunkt zu Anschlusspunkt.
- **Sammelschienensysteme** aus Kupfer (Schienen gezeichnet, Meter und Halter in der
  Stückliste) und rechteckige oder runde **Ausschnitte** in der Montageplatte. Ein über
  einem Sammelschienensystem platziertes Betriebsmittel wird auf dessen Schienen
  montiert; Betriebsmittel auf demselben Sammelschienensystem sind durch die Schienen
  verbunden, nicht durch Drähte. Die Sammelschienensysteme angereihter Gehäuse, die
  sich an ihrer gemeinsamen Seite treffen (derselbe Artikel auf derselben Höhe, ihre
  Enden höchstens 100 mm auseinander), sind ein System, durch die Seite verbunden. Die
  Schienen liegen im `Centre distance` des Artikels (ohne einen gleichmäßig verteilt)
  entlang des Systems, wie kurz ein Abschnitt davon auch ist; die 3D-Ansicht zeichnet
  jede Schiene für sich, auf isolierenden Haltern, die gerade über sie reichen, an den
  Bohrpositionen der Platte, aber neben den Betriebsmitteln, die in den Schienen sitzen
  (die Wandler, durch die die Schienen laufen), niemals durch sie hindurch
  (`busbars::holder_places`). Eine quer über die Schienen platzierte Sammelschienen-Klemmenleiste (`busbars::place_on_bars`)
  setzt jede Klemme aufrecht auf ihre eigene Schiene, eine Leiste je Zuleitungskabel;
  in der 3D-Verlegung ist jede dieser Klemmen ein eigener Körper (ein Draht zu einer
  geht um die anderen herum), und Klemmen, die entlang der Schienen näher als 30 mm
  beieinander liegen, werden von vorn angefahren.
  Jede Schiene
  ist ein Leiter: Die Prüfung `busbars` meldet ein System, dessen Betriebsmittel mehr
  Leiter verbinden, als es Schienen hat (Artikeldaten `Bars`), als Kurzschluss, etwa
  eine PEN-Klemme auf den Schienen der Außenleiter oder die Abgänge hinter einem
  Kuppelschalter auf der Sammelschiene davor. Sie nimmt die Einspeiseseite von
  Schaltern und Sicherungen (die Abgangsseite, wo das diejenige ist, die die anderen
  Betriebsmittel gemeinsam haben: eine Einspeiseleiste), jede Klemme als einen Leiter,
  und folgt den Leitern durch die Primärseiten von Stromwandlern.
- Montageplan-Seiten (maßstäbliche 2D-Modellansicht mit Bemaßungen) zum Drucken / für
  SVG; Kabelkanäle, Tragschienen und Sammelschienensysteme tragen eine Beschriftung mit
  ihrem Querschnitt und ihrer Zuschnittlänge (*60 × 100 mm, L = 940 mm*) auf dem
  längsten Stück, das kein Betriebsmittel verdeckt, passend verkleinert, nur die Länge
  (*L = 430 mm*), wo die ganze Beschriftung nicht passt. Ein voll belegtes Element
  erhält sie neben seinem Ende (sonst seinem Anfang), in seiner Flucht, bis zum
  nächsten Element oder zur Plattenkante; sie entfällt, wo sie selbst dort unlesbar
  wäre. Schienen, Kanäle und Sammelschienen
  erscheinen in der Stückliste mit ihrer Gesamtlänge (oder in Lagerlängen), mit den
  zuzuschneidenden Stücken je Gehäuse und Länge unter jedem Artikel (ohne eigene
  Artikelnummer, damit ERP-Exporte das Material nur einmal zählen); Gehäuse je Stück.
  Die Schaltschranklegende listet nach den Betriebsmitteln jede Schiene, jeden Kanal
  und jedes Sammelschienensystem mit Artikelnummer, Querschnitt (Breite × Höhe) und
  Zuschnittlänge auf.
- Sprung zwischen einem Betriebsmittel im Schaltplan und seinem Platz im
  Schaltschrankaufbau.

## Kollisions- und Grenzprüfungen

Überlappungen, außerhalb der Platte, zu tief für das Gehäuse einschließlich eines
Türabstands, über einem Ausschnitt montiert, innerhalb des Abstands, den ein
Betriebsmittel laut seinen Artikeldaten oben und unten benötigt (`Clearance: 25 mm`);
wo ein Betriebsmittel ein 3D-Modell hat, werden die **Modelle** auf Durchdringung
geprüft, nicht nur ihre Hüllquader.

## Gehäusetür

Befehls- und Meldegeräte (Drucktaster, Leuchtmelder …) werden auf der Tür platziert
(Frontansicht *Tür*); jedes erhält seine **Türausschnitte** (standardmäßig Ø22,5 oder
aus den Artikeldaten `Door cutout: Ø30.5` / `92 x 68`, sonst dessen Ausschnitte), dort
platziert, wo der Artikel sie hat, und mit dem Betriebsmittel gedreht, geprüft gegen
den freien Rand der Tür (Rahmen, Scharniere) und die Front des Betriebsmittels (ein
darüber hinausragender Ausschnitt wird gemeldet) und in 3D gegen die Bestückung der
Montageplatte; ihre Drähte werden über das Scharnier geführt; die Montageplan-Seite
zeigt die Platte und jede Fläche mit Elementen (Tür, Seitenwände, Rückwand, Dach,
Boden) nebeneinander.

Die Tür ist links oder rechts angeschlagen (*Türanschlag* in den Eigenschaften des
Gehäuses, `door_hinge` von `add_enclosure` / `set_enclosure`, Artikeldaten
`Door hinge: right` eines Hersteller-Gehäuseartikels): Die Türansicht zeigt die
Scharniere und den Griff auf ihren Seiten, die 3D-Ansicht öffnet die Tür um diese
Kante, und die Türdrähte laufen an der Tür entlang zu dieser Kante, über die
Scharnierschlaufe und auf dieser Seite in den Rand der Platte.

## Sperrflächen

Ein Gehäuseartikel kann Bereiche seiner Flächen freihalten (*Sperrflächen* in den
Montagedaten des Artikels, importiert mit den Gehäusepaketen des Rittal-Konfigurators):
Flächen der Art *Platzierung* nehmen keine Betriebsmittel, Schienen, Kanäle oder
Sammelschienen auf, Flächen der Art *Ausschnitt* keine Ausschnitte oder Bohrungen, z. B.
die Scharniere, das Schloss und die Versteifung der Tür oder die Befestigungspunkte der
Montageplatte. Die Flächen zeigen sie schraffiert (Platzierungsflächen steigend,
Ausschnittflächen fallend), im Editor und auf den Montageplan-Seiten, und die
3D-Ansicht als durchscheinende Flächen (Platzierung blau, Ausschnitt orange), die mit
ihrer Wand oder der Tür mitgehen. Die Prüfung meldet ein Element, das in einer
Platzierungsfläche liegt (*Außerhalb*), und die Ausschnitte und Bohrungen in einer
Ausschnittfläche (*Fertigung*); das Berühren des Rands einer Fläche ist erlaubt. Der
Artikeleditor listet sie unter *Montagedaten › Sperrflächen* auf (Art, Fläche, Platz
und Größe), wo sie hinzugefügt, geändert und entfernt werden.

## Gehäusemodelle des Herstellers

Ein Gehäuse, dessen Artikel ein Herstellerartikel ist (keiner des eingebauten Katalogs)
mit einem 3D-Modell in der Breite und Höhe des Gehäuses, wie es die Gehäusepakete des
Rittal-Konfigurators mitbringen, zeigt in der 3D-Ansicht und den 3D-Exporten dieses
Modell statt des generischen Korpus, der Wände, der Tür und der Platten. Die Tür und
das, was auf ihr montiert ist (Scharnierflügel, Schlösser, Dichtungen), wie das Makro
des Herstellers sie kennzeichnet, sind durchscheinend und öffnen sich mit der Tür; ein
Modell ohne diese Kennzeichnung öffnet seine Front um 20 mm. Die übrigen Dreiecke gehen
mit der Wand mit, in deren Ebene sie liegen (mit ihr ausgeblendet, wenn die Wände davor
ausgeblendet werden), oder bleiben im Inneren (Montageplatte, Flanschplatte, Flansche).
Sockel, Anreihsätze und 19-Zoll-Schienen bleiben generisch. Die Tür öffnet auf ihrer
Anschlagseite (der Rittal AX ist rechts angeschlagen, siehe
[Gehäusetür](#gehäusetür)).

## Bohrbild und NC-Daten

Schienen, Kanäle und Sammelschienenhalter erhalten Gewindebohrungen entlang ihrer
Länge, auf die Platte geschraubte Betriebsmittel erhalten Bohrungen aus ihren
Artikeldaten (`Mounting holes: M5: 9,12; 69,72`) oder zwei Standardbohrungen;
**NC-Export** je Gehäuse und Fläche als DXF oder DWG (Kontur, ein Layer je
Bohrungsdurchmesser, Ausschnitte, gedrehte oder gerundete als ihre Kontur) oder CSV
(mit der Drehung und dem Eckenradius jedes Ausschnitts).

Eine Tragschiene wird immer geschraubt oder genietet (*Befestigung* in ihren
Eigenschaften: Gewindebohrung, Durchgangsbohrung mit Mutter, Einnietmutter, Blindniet
oder Schweißbolzen; `set_panel_item fastening=rivet`): ihre Befestigungspunkte liegen in
den Langlöchern einer gelochten Schiene, die 3D-Ansicht zeigt die Schrauben- oder
Nietköpfe darin und die Stückliste die Schrauben, Muttern oder Blindniete. Die
Auswertung *Bohrbild* listet jede Bohrung und jeden Ausschnitt jeder Fläche mit seinem
Buchstaben, seinen Maßen von der linken unteren Ecke der Fläche (wie die NC-Daten),
Größe, Gewinde und Bearbeitung. Eine Seite *Bohrbild* eines Gehäuses (*Neue
Bohrbild-Seite*, `add_page kind=drilling enclosure=+CAB1`) zeichnet seine Flächen mit
ihren Bohrungen und Ausschnitten in einem Maßstab, bemaßt von dieser Ecke, daneben die
Arten der Bohrungen und Ausschnitte mit ihrer Anzahl.

## Virtuelle Verdrahtung und Füllgrade der Kabelkanäle

Jede Schaltplanverbindung zwischen platzierten Betriebsmitteln wird von ihrem
Anschlusspunkt (gedrehte Betriebsmittel, Hilfsschalterblöcke, Klemme für Klemme) in den
Kanal geführt, dem die Klemme zugewandt ist, und durch das Kabelkanalnetz (kürzester
Weg) → Leitungsliste mit Längen, Drähte in 2D / 3D dargestellt.

**Füllgrade der Kabelkanäle**: Querschnitte der verlegten Drähte (mit Isolierung),
Kabel und Schläuche gegenüber dem Querschnitt im Inneren des Kanals (zwischen seinen
Wänden, dem Boden und dem Deckel) an seinem vollsten Stück, auf der Montageplatte wie
an den Wänden, der Decke und dem Boden eines Raums;
Kanäle in der Frontansicht grün / orange / rot eingefärbt, Füllgrad in den
Kanaleigenschaften, Grenzwert in den Gehäuseeigenschaften (standardmäßig 70 %);
Auswertung *Füllgrade der Kabelkanäle*, die Prüfung warnt oberhalb des Grenzwerts und
gibt einen Hinweis oberhalb von 80 % davon; wenn ein Kanal überfüllt ist, wird die
Verdrahtung neu verlegt, die dicksten Drähte zuerst, und Drähte nehmen einen Umweg (bis
zu 10 × länger) um Stücke, die sie über den Grenzwert füllen würden; in 3D verlegte
Kabel und Schläuche tun dasselbe um die Stücke, die die Drähte und die Kabel vor ihnen
bereits füllen; das ausgewählte Betriebsmittel zeigt seine Anschlusspunkte.

## Klima

IEC/TR 60890, in den Gehäuseeigenschaften: Verlustleistung der platzierten
Betriebsmittel aus den technischen Daten ihrer Artikel (`Power loss: 7.5 W`) plus eine
zusätzliche Verlustleistung, wirksame Oberfläche nach Aufstellungsart (einzeln, in
einer Reihe, frei stehend, an der Wand, mit abgedecktem Dach), Werkstoff
(Wärmedurchgangskoeffizient), Umgebungs- und zulässige Innentemperatur
→ Innentemperatur, Luftleistung des Filterlüfters (korrigiert um die Höhe des
Aufstellorts), Luft/Luft-Wärmetauscher (W/K, wenn die Schutzart erhalten bleiben muss)
oder Leistung des Kühlgeräts, Heizleistung, die das Innere auch über dem Taupunkt der
Umgebungsluft hält (relative Luftfeuchte); Auswertung *Gehäuseklimatisierung*, die
Prüfung warnt, wenn Kühlung nötig ist.

Die Wärmekarte der Montageplatte schichtet die mittlere Temperatur und fügt über jedem
Betriebsmittel mit Verlustleistungsdaten eine Wärmefahne hinzu; waagerechte Kanäle
halten Wärmefahnen zurück (warme Luft staut sich unter ihnen), Lüfter und Kühlgeräte
schwächen sie ab.

![Wärmekarte einer Montageplatte neben der Klimaberechnung des Gehäuses](../../images/heat-map.png)

## Anreihungen

Gehäuse, nebeneinander angereiht, mit Kabeldurchführungen dazwischen (Ausschnitte in den
einander zugewandten Seitenwänden oder zueinander offene Seiten); die 3D-Ansicht zeigt
die ganze Anreihung, Drähte laufen von einem Gehäuse in das nächste. Die Ansicht *Raum*
zeigt alle Gehäuse nebeneinander an ihren Plätzen, jede Anreihung an ihrer Position und
mit ihrer Drehung, von Hand gesetzt oder aus dem Anlagenlayout übernommen.

## Räume

*Layoutbereich › Neuer Raum* (Plugin `rooms`): ein Raum des Gebäudes mit seinem
Grundriss, seiner Höhe und seinem Platz in der Anlage. Der Grundriss ist ein Rechteck
oder eine Kontur aus rechtwinkligen Wänden (L-, U- oder T-förmige Räume): *Grundriss*
zeigt ihn von oben, *Wände zeichnen* zeichnet ihn durch Anklicken der Ecken (auf einem
50-mm-Raster, geschlossen durch Klick auf die erste Ecke oder mit der Eingabetaste),
Wände werden gezogen, um sie zu verschieben, oder erhalten eine Länge in den
Eigenschaften des Raums (Befehle `set_room_outline`, `set_room_wall`).

- **Innenwände** (*Layoutbereich › Zeichnen › Innenwände*, Befehle `add_inner_wall`,
  `set_inner_wall`) werden im Grundriss durch Anklicken ihrer Enden gezeichnet (eine
  Kette, rechtwinklig; ein Ende nahe einer Wand rastet auf ihr ein), quer gezogen, um
  sie zu verschieben, erhalten Länge und Dicke in den Eigenschaften des Raums und
  werden mit Entf entfernt; beide Seiten (`inner1a`, `inner1b`) sind Flächen wie
  die Wände.
- **Türen und Fenster** (*Layoutbereich › Zeichnen › Tür / Fenster*, Befehle
  `add_wall_opening`, `set_wall_opening`) sind Öffnungen in einer Wand, platziert durch
  Klick auf die Wand im Grundriss oder in der Ansicht der Wand: Eine Tür steht auf dem
  Boden, ein Fenster hat seine Brüstungshöhe; Größe, Position, Brüstung und
  Öffnungsart stehen in ihren Eigenschaften. *Öffnet* (`opens`) wählt die Scharniere an
  der linken oder rechten Kante (vom Rauminneren gesehen) mit Aufschlag in den Raum
  oder aus ihm heraus, oder einen Schiebeflügel, der sich nach links oder rechts an der
  Wand entlang bewegt. Der Grundriss zeichnet den Flügel mit seinem Aufschlag (von der
  Außenfläche der Wand, wenn er nach außen öffnet) oder den Schiebeflügel mit seinem
  Weg, die Ansicht der Wand das Öffnungssymbol (seine Spitze an den Scharnieren,
  gestrichelt, wenn der Flügel vom Betrachter weg öffnet) oder einen Pfeil. Die Prüfung
  hält den Aufschlag einer nach innen öffnenden Tür und den Weg einer Schiebetür vor
  der Wand frei von Schränken.
- **Schränke** (ihre Anreihungen) stehen darin an einem Platz des Grundrisses, werden
  im Grundriss dorthin gezogen (R dreht den ausgewählten um seine Mitte) und bewegen
  sich mit dem Raum. Ein Schrank außerhalb des Raums, quer über einer Wand, in einer
  Innenwand oder im Aufschlag einer Tür wird in der Warnfarbe gezeichnet, und der
  Mauszeiger darüber nennt den Grund.
- Jede Wand (vom Rauminneren gesehen) und die Decke tragen Tragschienen, Kabelkanäle
  und Betriebsmittel: Ein Betriebsmittel an einer Wand sitzt auf einer Schiene, es sei
  denn, sein Artikel ist mit *Wandmontage* gekennzeichnet (Artikeleditor ›
  Montagedaten, ein Kennzeichen `WALL` in den Artikelbäumen, Filter *Nur Artikel für
  direkte Wandmontage*), dann kommt es direkt auf die Wand
  (*Einfügen › Wandmontage-Artikel*, auch ohne Betriebsmittelkennzeichen).
- **Dosen, Schalter und Heizung** (*Einfügen › Dosen, Schalter, Heizung*): Steckdosen,
  Daten- und Antennendosen, Schalter, Taster, Dimmer, Raumthermostate,
  Heizkreisverteiler und Einzelraumregelboxen (RTL / TH), in der 3D-Ansicht mit ihren
  Modellen gezeichnet. Das Menü hat je eine Registerkarte für Dosen, Schalter und
  Heizung, eine Suche, Filter nach Hersteller und Typ, die Liste und den gewählten
  Artikel mit seinem 3D-Modell (Ziehen dreht es), seiner Größe und Montagehöhe (*An
  die Wand setzen* oder ein Doppelklick platziert
  ihn). Es listet
  die generischen Artikel des Katalogs (`GEN.SOCKET-F-UP`, `GEN.SW-2WAY-UP`, …,
  Verteiler `GEN.HCM-2` … `GEN.HCM-12`, die Anzahl der Kreise im Menü gewählt,
  `GEN.RTL-BOX`) und die der Artikeldatenbank und der mitgelieferten Kataloge `gira`
  (System 55), `mdt` (KNX-Taster, Glastaster, Raumtemperaturregler) und `cosmo`
  (Verteiler Standard und Balance, Regelboxen); ein Artikel eines mitgelieferten
  Katalogs kommt in das Projekt, wenn er gewählt wird. Ein Artikel mit einer
  *Montagehöhe* in seinen technischen Daten kommt in dieser Höhe an die Wand (seine
  Mitte über dem Boden: Steck- und Datendosen 300 mm, Schalter 1050 mm, Thermostate
  1500 mm, Regelboxen 250 mm), wo die Wand angeklickt wird. Die Verteiler haben einen
  Stutzen für Vorlauf und Rücklauf und ein Paar je Kreis, nach unten, für die
  Rohrverlegung.
- Der **Boden** (Fläche `floor`) trägt die mit *Bodenmontage* gekennzeichneten Artikel
  (Pumpen, Behälter, Kompressoren; ein Kennzeichen `FLOOR`, Filter *Nur bodenstehende
  Artikel*): *Einfügen › Bodenstehender Artikel*, zu platzierende Betriebsmittel in der
  Ansicht des Bodens, Befehl `mount_on_wall` mit `face` `floor`. Sie werden im
  Grundriss gezeichnet, dort gezogen, um sie zu verschieben, und mit Entf entfernt.
- **Wanddurchbrüche** (*Layoutbereich › Zeichnen › Wanddurchbruch*, Befehl
  `add_wall_opening` mit `kind` `penetration`; 300 x 200 mm, 300 mm unter der Decke,
  sofern nicht anders eingestellt) kommen in die Außenwände und führen in den Raum
  hinter der Wand (bis zu 1 m entfernt, auch durch gegeneinander gedrehte Räume):
  Kabel, Schläuche und Rohre werden durch sie von Raum zu Raum verlegt. Der Grundriss
  zeichnet sie und den Weg durch die Wand gestrichelt; `list_rooms` nennt, wohin jeder
  führt.
- **Hauseinführungen** (*Layoutbereich › Zeichnen › Hauseinführung*, Befehl
  `add_wall_opening` mit `kind` `entry`; 200 x 200 mm, 600 mm über dem Boden, sofern
  nicht anders eingestellt) sind die Einführungen der Außenwände: Das Zuleitungskabel,
  die Wasserhauptleitung und andere Leitungen von außerhalb des Gebäudes kommen durch
  sie herein. Ein Kabel oder Rohr von einem Betriebsmittel im Raum zu einem
  Betriebsmittel außerhalb des Gebäudes (sein Ort ist keiner der Räume und Gehäuse des
  Layouts, etwa der Kabelverteilerschrank des Netzbetreibers) wird zur nächsten
  Hauseinführung verlegt.
- **Rohrtrassen** (*Layoutbereich › Zeichnen › Rohrtrasse*, Befehl `add_pipe_run`)
  werden an einer Wand, der Decke oder dem Boden durch Anklicken ihrer Ecken gezeichnet
  (rechtwinklig), 100 mm vor einer Wand, 300 mm unter der Decke oder 200 mm über dem
  Boden, sofern nicht anders eingestellt: Die Schläuche, Rohre und Rohrleitungen des
  Schaltplans werden an ihnen entlang verlegt wie Drähte an Kabelkanälen. Der
  Grundriss zeigt sie gestrichelt.
- **Die Außenseite des Gebäudes:** Jede Wand eines Raums hat auch ihre Außenseite
  (Flächen `outside1`, `outside2`, …, von außen gesehen, *Außenseite Wand n* im Menü
  der Wände) und die Decke ihre Oberseite (Fläche `rooftop`): Fassadenleuchten,
  Sensoren und Kameras kommen auf die Außenseite einer Außenwand wie auf eine Wand
  innen (auf eine Schiene oder, mit *Wandmontage* gekennzeichnet, direkt), eine
  Wetterstation oder andere mit *Bodenmontage* gekennzeichnete Artikel stehen auf dem
  Dach. Ihre Kabel und Rohre laufen direkt hinter ihnen durch die Wand (oder die Decke)
  in den Raum. Die Prüfung meldet solche Elemente, wo direkt hinter der Wand oder über
  der Decke ein anderer Raum liegt (innerhalb von 1 m).

Die Prüfung meldet Betriebsmittel an einer Wand, die keines von beidem sind, Schienen
oder Betriebsmittel über einer Tür oder einem Fenster, Rohrtrassen quer über einer Tür
oder einem Fenster, Elemente an der Decke außerhalb des Grundrisses, Innenwände, die
den Raum verlassen, Schränke und bodenstehende Artikel im Aufschlag einer Tür oder in
einer Innenwand, bodenstehende Artikel auf dem Platz eines Schranks oder solche, die
nicht für die Aufstellung auf dem Boden gemacht sind, sowie Schränke und bodenstehende
Artikel außerhalb des Raums (oder quer über einer Wand), Schränke, die höher sind als
seine Decke, Wanddurchbrüche, die in keinen Raum führen. Räume haben keine
Klimaberechnung und keine NC-Daten.

**Mehrere Räume** stehen an ihren Plätzen in der Anlage (*Hintere linke Ecke des Raums*
in seinen Eigenschaften, Befehle `add_room` / `set_room` mit `x`, `z` und `angle`). Der
Grundriss eines Raums zeigt die Räume um ihn herum blass mit ihren Namen und Schränken
(die neben ihm, nicht die auf den Ebenen darüber und darunter); ein Doppelklick öffnet
einen von ihnen. *Layoutbereich › Anzeigen › Alle Gehäuse* zeigt jeden Raum und Schrank
an seinem Platz in 3D, mit den Verlegewegen dazwischen.

Der Navigator (*Gehäuse und Räume*) hat das **Grundstück** an seiner Wurzel, über den
Gebäuden mit ihren Ebenen und Räumen und den Räumen außerhalb von Gebäuden. Ein Klick
auf einen Raum behält die 3D-Ansicht bei, wenn sie angezeigt wird, und zeigt sonst den
Grundriss des Raums. Ein Klick auf das Grundstück zeigt den Lageplan aller Gebäude im
Layoutbereich (das Mausrad zoomt, Ziehen verschiebt ihn, ein Klick auf einen Raum
öffnet dessen Grundriss; im Bearbeitungsmodus *Grundstück › Gebäude verschieben* wird
ein Gebäude an seinen Platz gezogen und R dreht es um 90° um seine Mitte, mit seinen
Räumen, Schränken, Treppen und Dächern und den Räumen, die in ihm stehen, ohne auf
einer seiner Ebenen zu sein) oder, in der 3D-Ansicht, die gesamte
Anlage mit ihren Gebäuden, Geschossdecken, Treppen und Dächern; ein Klick auf ein
Gebäude zeigt den Lageplan mit dem Gebäude hervorgehoben. Eine Ebene, die angeklickt
wird, während die 3D-Ansicht angezeigt wird, zeigt alle Räume dieser Ebene
nebeneinander an ihren Plätzen, mit den darin stehenden Schränken, den Geschossdecken
und den Treppen, die die Ebene erreichen (keine Dächer); ein zweiter Klick zeigt wieder
die gesamte Anlage. Das Kontextmenü des Grundstücks fügt ein Gebäude hinzu, zeichnet
die Grundstücksgrenze oder setzt den Bezugspunkt, das eines Gebäudes eine Ebene oben
oder ein Untergeschoss, das einer Ebene einen Raum darauf.

## Grundstücksgrenze und Bezugspunkt

Das **Grundstück** (*Layoutbereich › Grundstück*, das Kontextmenü des Grundstücks, der
Abschnitt *Grundstück* des Bereichs Gebäude; Plugin `buildings`) ist das Flurstück, auf
dem die Anlage steht.

- **Grundstücksgrenze** (*Grundstück › Grundstücksgrenze*, Befehl `set_plot`): im
  Lageplan Ecke für Ecke gezeichnet (ein Klick auf die erste Ecke, ein Doppelklick oder
  die Eingabetaste schließt sie, die Rücktaste nimmt die letzte Ecke zurück, Esc
  beendet); die Ecken rasten an den Ecken der Räume (innen und außen an ihren Wänden),
  der Schächte und am Bezugspunkt ein. Ihre Ecken werden danach im Lageplan gezogen.
  Die Grenze darf sich nicht selbst kreuzen; der Lageplan schreibt ihren Namen und ihre
  Fläche, der Bereich ihre Ecken, Fläche und Länge. **Alle Gebäude müssen innerhalb
  bleiben**: Die Prüfung *Gebäude, Ebenen und Schächte* meldet Räume, deren Außenwände
  über die Grenze hinausreichen, als Fehler und warnt vor Dächern, die mit ihrem
  Überstand darüber hinausreichen, und Schränken außerhalb; der Lageplan zeichnet sie
  in der Fehlerfarbe und `plot` listet sie auf.
- **Bezugspunkt** (*Grundstück › Bezugspunkt*, Befehl `set_reference_point`): ein
  georeferenzierter Punkt der Anlage (wo er in der Anlage liegt, sein Rechtswert,
  Hochwert und seine Höhe über dem Meeresspiegel in einem Koordinatenreferenzsystem wie
  `EPSG:25832`, die Richtung von Gitternord in der Anlage und, falls bekannt, seine
  geografische Breite und Länge). Jeder Punkt der Anlage wird von ihm aus gemessen: Der
  Lageplan zeigt die Koordinaten unter dem Mauszeiger (Rechtswert und Hochwert, oder
  wie weit östlich und nördlich des Punkts), einen Nordpfeil, und `list_buildings` den
  Rechtswert, Hochwert und die Höhe des Ursprungs jedes Gebäudes; `locate` rechnet
  zwischen der Anlage und dem Koordinatensystem um. IFC-Exporte schreiben den Namen des
  Grundstücks sowie Breite, Länge und Höhe in ihr Grundstück.
- **Messen** (*Grundstück › Messen*, Befehl `measure`): Zwei im Lageplan angeklickte
  Punkte ergeben ihren Abstand, wie weit östlich und nördlich der zweite liegt, und
  seine Richtung von Gitternord aus.

## Gebäude, Ebenen und Schächte

*Layoutbereich › Gebäude* (Plugin `buildings`) verwaltet die Gebäude der Anlage in
Registerkarten: *Grundstück* (das Grundstück, Freiflächenanlagen, die Verbindungen
zwischen Gebäuden), *Gebäude* (das gewählte Gebäude mit seinen Ebenen, Schächten und
Treppen), *Dächer*, *Erdung*, *Grundriss auf Papier* und *Elektroinstallation* (beide
für eine Ebene). Jede Registerkarte zeichnet einen Lageplan aller Räume (eine Ebene
hervorgehoben), Schächte, der Treppen, die die angezeigte Ebene erreichen (Stufen und
Lauflinie), der Dächer (Traufen, Firste und Grate gestrichelt) mit ihren Dachfenstern
und der Verbindungen.

- Ein **Gebäude** (Befehl `add_building`) hat einen Platz in der Anlage (x, z und eine
  Drehung) und **Ebenen** übereinander (`add_level`, `set_level`: Name, Höhe des
  Fußbodens über dem Geländeniveau des Gebäudes, Höhe von Fußboden zu Fußboden; eine
  negative Höhenlage ist ein Untergeschoss). Ein neues Gebäude erhält eine Ebene `L0`
  auf Geländeniveau und kommt neben das, was in der Anlage steht; neue Ebenen kommen
  oben darauf (`L1`, `L2`, …) oder darunter (`B1`, `B2`, …).
- **Räume stehen auf Ebenen** (Raumeigenschaften › *Gebäude*, im Bereich *Neuer Raum
  auf dieser Ebene* / *Raum hierher stellen*, Befehle `place_on_level`, `add_room` mit
  `building` und `level`): Der Platz des Raums wird dann im Grundriss des Gebäudes
  gemessen, er steht auf dem Fußboden der Ebene und bewegt sich mit dem Gebäude und
  seiner Ebene. Ein neuer Raum auf einer Ebene ist so hoch wie die Ebene abzüglich
  ihrer Geschossdecke (300 mm) und kommt neben die anderen Räume der Ebene. Der
  Navigator gruppiert die Räume nach Gebäude und Ebene: Ein Raum, der auf eine Ebene
  (oder auf einen Raum darauf) gezogen wird, kommt auf diese Ebene; eine darin
  angeklickte Ebene wird ausgewählt, und *Neuer Raum* stellt den neuen Raum dann auf
  sie (ein zweiter Klick hebt die Auswahl auf); das Kontextmenü einer Ebene und ihrer
  Räume enthält *Neuer Raum auf dieser Ebene*, das eines Raums in einem Gebäude *Aus
  dem Gebäude nehmen*. Das Entfernen einer Ebene oder eines Gebäudes lässt seine Räume
  dort, wo sie in der Anlage stehen.
- **Schächte** (`add_shaft`, `set_shaft`) laufen durch ein Gebäude von einer Ebene
  hinauf zu einer anderen, an einem Platz seines Grundrisses: Kabel, Schläuche und
  Rohre werden durch sie von einem Raum in die Räume darüber und darunter verlegt,
  deren Grundrisse der Schacht durchläuft, von unter der Decke des einen bis über den
  Fußboden des nächsten. Der Grundriss eines Raums zeigt die Schächte durch seine
  Ebene.
- **Treppen** (*Neue Treppe* im Bereich, `add_stair`, `set_stair`) führen vom Fußboden
  einer Ebene hinauf zum Fußboden einer anderen: *gerade*, *einmal gewinkelt* an einem
  Viertelpodest, *zweimal gewinkelt* an zwei Viertelpodesten, mit einem *Halbpodest*
  (zwei Läufe nebeneinander) oder als *Wendeltreppe* um eine Spindel, links- oder
  rechtsdrehend. Ihr Antritt ist die Mitte der Vorderkante der untersten Stufe im
  Grundriss des Gebäudes, mit der Richtung des ersten Laufs, der Breite der Läufe und
  dem Auftritt einer Stufe; die Stufen ergeben sich aus dem Höhenunterschied
  (Steigungen nahe 175 mm). Die Grundrisse der Räume zeigen die Treppen auf ihrer Ebene
  (gestrichelt, wo sie von unten heraufkommen), die 3D-Ansicht ihre Stufen und Podeste
  und die Öffnung in der Geschossdecke darüber.
- **Dächer** (*Neues Dach* im Bereich, `add_roof`, `set_roof`): *Flachdach*,
  *Pultdach*, *Satteldach*, *Walmdach* oder *Zeltdach* über einem Rechteck des
  Gebäudegrundrisses (ein neues überdeckt die Räume des Gebäudes), mit der Höhe seiner
  Traufe, Neigung, Überstand und Dicke; ein Sattel- oder Pultdach läuft entlang x oder
  z. Die Unterseite des Dachs beginnt an der Traufe über den Wänden und steigt mit der
  Neigung. Wo sie niedriger ist als die Decke eines Raums darunter, auch auf mehreren
  Ebenen zugleich (Traufe einen Kniestock über dem Fußboden des Dachgeschosses oder
  unter der Decke der Ebene darunter), wird sie zur Decke dieses Raums: Der Raum ist an
  verschiedenen Stellen verschieden hoch. Seine Wände enden in der 3D-Ansicht unter der
  Dachschräge, die Ansicht einer Wand zeichnet die Linie, an der sie endet, der
  Grundriss schattiert die Teile unter der Schräge mit ihrer geringsten Höhe, und der
  Bereich und `list_buildings` nennen die niedrigste und die höchste Decke jedes
  solchen Raums. **Dachfenster** (`add_roof_window`, `set_roof_window`; 780 × 1180 mm,
  sofern nicht anders eingestellt) liegen in einer Dachfläche an ihrer Mitte im
  Grundriss, mit ihrer Breite entlang der Traufe und ihrer Länge die Schräge hinauf;
  sie sind von außen und aus dem Raum darunter zu sehen.
- **PV-Anlagen** (*PV-Anlage* unter einem Dach im Bereich, `add_pv_array`,
  `set_pv_array`): Reihen von Solarmodulen auf jeder Dachform. Auf einer Dachschräge
  liegen sie **parallel** zur Eindeckung auf Schienen, die mit Dachhaken befestigt
  sind: zwei Schienen unter jeder Modulreihe im Hochformat, unter jeder Spalte im
  Querformat, die Haken höchstens im Hakenabstand voneinander (1200 mm, sofern nicht
  anders eingestellt). Auf einem Flachdach stehen sie **aufgeständert** zu einer Seite
  geneigt (15°, sofern nicht anders eingestellt, in die angegebene Richtung
  ausgerichtet) oder **Ost-West** paarweise auf ballastierten Montagegestellen, die
  Reihen so weit auseinander, dass sie sich bei einer Wintersonne von 17° nicht
  gegenseitig verschatten (oder in einem angegebenen Abstand). Eine Anlage liegt auf
  der Dachfläche unter ihrer Mitte; ihr Modulartikel (die generischen `GEN.PV-430W` und
  `GEN.PV-400W-FB` oder Module der mitgelieferten Kataloge `longi` und `jinko`) gibt
  die Größe der Module und ihre Spitzenleistung vor. Das Montagematerial kommt aus
  ihrem **Montagesystem**: eine Schiene oder ein Montagegestell der Artikeldatenbank,
  deren Zubehör ihre Dachhaken, Schienenverbinder, End- und Mittelklemmen (die für die
  Rahmenhöhe des Moduls), Ballast und Windbleche sind; das generische System
  `GEN.PV-RAIL-40` / `GEN.PV-BASE-S15` / `GEN.PV-BASE-EW10` ergänzt, was einem System
  fehlt. Die Module (sofern nicht ein Betriebsmittel, ein PV-Generator, sie zählt) und
  das Montagematerial kommen in die Stückliste; `list_buildings` nennt die Module, die
  Spitzenleistung und das Material jeder Anlage. Der Lageplan zeichnet die Module, die
  3D-Ansicht die Module mit ihren Rahmen und Anschlussdosen, die Schienen auf ihren
  Dachhaken, die Montagegestelle und die Windbleche.
  *Anzeigen › PV-Module ausblenden* lässt die Module weg, um die Montage zu zeigen,
  *Anzeigen › PV-Module durchsichtig* zeigt sie durchsichtig mit der Verkabelung
  darunter. Die Montage hat einen **Erdungspunkt** mit einer Erdungsklemme
  (`GEN.PV-EARTH-CLAMP`, in der Stückliste) an jeder Schiene oder Reihe von
  Montagegestellen; ihr **Erdungsleiter** (`earthing`: das Betriebsmittel einer
  Erdungs- oder Potenzialausgleichsschiene) verbindet sie und läuft zu dieser Schiene.
  Die 3D-Ansicht zeichnet die **Verkabelung** jedes Strings: die Leitungen jedes
  Moduls: seine Plusleitung (rot) und seine Minusleitung (schwarz) aus seiner
  Anschlussdose, wobei die Plusleitung eines Moduls die Minusleitung des nächsten an
  einem Steckverbinder unter den Modulen trifft, und die Minusleitung des ersten Moduls
  und die Plusleitung des letzten am Dach entlang, neben der Anlage durch das Dach und
  darunter hinab auf den **Generatoranschlusskasten** der Anlage (`combiner`, ein
  Betriebsmittel wie `GEN.GAK-2S`)
  und von dort ein DC-Hauptkabel (Plus und Minus) zu jedem Wechselrichter (ohne Generatoranschlusskasten
  hinab auf den Wechselrichter), und den Erdungsleiter (grün-gelb) hinab zu seiner
  Schiene, soweit die Betriebsmittel im Layout platziert sind. Ein Klick auf eine Anlage in der 3D-Ansicht öffnet ihre
  **Eigenschaften**: wo sie liegt, ihre Module mit ihren elektrischen Daten,
  Spitzenleistung und Fläche, ihre Montage mit dem Material und der Erdung, ihre
  Strings mit ihren Spannungen, Strömen und der Länge ihrer Leitungen, ihre Probleme
  und die Einstellungen des Bereichs Gebäude. Ein Raum unter dem Dach, der allein
  angezeigt wird, zeigt auch die Anlagen über ihm und die Dachfenster von außen. Ein
  **Klick auf ein Dach** in der 3D-Ansicht zeigt das Dach in den
  *Layout-Eigenschaften*: seine Form, Traufe, Neigung, Dachfenster und PV-Anlagen; ein
  dort hinzugefügtes Dachfenster oder eine dort hinzugefügte PV-Anlage kommt dorthin,
  wo das Dach angeklickt wurde, und die neue Anlage wird mit ihren Eigenschaften
  ausgewählt.
- **Freiflächenanlagen** (*Freiflächenanlagen* im Bereich: *PV-Anlage* vor dem, was in
  der Anlage steht, *Im Lageplan setzen* dort, wo geklickt wird; `add_pv_array` und
  `set_pv_array` ohne `building` und `roof`): bodenmontierte Anlagen auf dem
  Grundstück, ihre Mitte in der Anlage. Sie stehen in Reihen von **Tischen**, die in
  die angegebene Richtung ausgerichtet sind (25°, sofern nicht anders eingestellt):
  `stack` Module übereinander auf einem Tisch (2), die Unterkante `clearance` über dem
  Boden (800 mm), zwei Schienen unter jeder Modulreihe auf Tischträgern, jeder Träger
  auf einem vorderen und einem hinteren Rammpfosten, höchstens im Pfostenabstand
  entlang des Tischs voneinander (`span`, 2,5 m), die Tische so weit auseinander, dass
  sie sich bei einer Wintersonne von 17° nicht gegenseitig verschatten (oder im
  angegebenen Reihenabstand). Das generische System ist der Pfosten `GEN.PV-POST-RAM`
  mit dem Tischträger `GEN.PV-TABLE-BEAM`, den Schienen, Schienenverbindern,
  Modulklemmen und der Erdungsklemme (am ersten vorderen Pfosten jedes Tischs) als
  Zubehör; ein Montagesystem der Artikeldatenbank (Untergruppe *PV-Rammpfosten*) bringt
  sein eigenes mit. Die Module und die Montage kommen in die Stückliste, benannt nach
  dem Grundstück; der Lageplan und die 3D-Ansicht des gesamten Grundstücks zeichnen
  sie, ein Klick auf eine in einem von beiden öffnet ihre Eigenschaften. Ihre Strings
  sind die der Anlagen auf den Dächern; ihre Leitungen (und der Erdungsleiter) laufen
  auf der Rückseite der Module, hinab in einen 0,7 m tiefen Kabelgraben und hinauf auf
  den Generatoranschlusskasten, den Wechselrichter oder die Schiene. `plot` und
  `list_buildings` listen sie mit dem Grundstück auf; die Prüfungen melden
  Freiflächenanlagen aufeinander, auf einem Raum oder Schrank und außerhalb der
  Grundstücksgrenze.
- **PV-Strings** (*String* unter einer PV-Anlage im Bereich, `add_pv_string`,
  `set_pv_string`, `pv_strings`): die Module einer Anlage in Reihe als Strings, mehrere
  gleichartige Strings parallel, jeder an einem MPP-Tracker eines Wechselrichters oder
  Ladereglers (ein Betriebsmittel, dessen Artikel PV-Eingangsdaten hat: `Max. PV input
  voltage`, `MPPT voltage range`, `MPP trackers`, `Max. input current per MPPT`, `Max.
  short-circuit current per MPPT`; die generischen Wechselrichter `GEN.INV-PV-10KW` und
  `GEN.INV-PV-40KW`, der Victron Inverter RS Smart Solar und SmartSolar MPPT haben
  sie). Aus den Datenblattwerten des Moduls (`Open-circuit voltage`, `Short-circuit
  current`, `Voltage` / `Current at maximum power`, die Temperaturkoeffizienten von
  Voc, Isc und Pmax) und den Zelltemperaturen der Auslegung (-10 / 25 / 70 °C, sofern
  nicht anders eingestellt) folgen die **höchste Spannung** (Leerlauf bei der
  niedrigsten Temperatur), die **mittlere** und die **niedrigste Spannung** (bei
  maximaler Leistung bei der mittleren und der höchsten Temperatur, mit dem
  Koeffizienten der Leistung), der Strom bei maximaler Leistung, der Kurzschlussstrom
  bei der höchsten Temperatur und die Leistung. Die Auswertung *PV-Strings* listet
  jeden String mit diesen Werten, der Gesamtleistung und den Temperaturen auf. Die
  Prüfung meldet Strings, deren höchste Spannung die größte Eingangsspannung ihres
  Wechselrichters oder die Systemspannung ihrer Module überschreitet, Strings unter
  oder über dem MPP-Bereich ihres Trackers, Tracker, die der Wechselrichter nicht hat,
  Tracker, deren Strings zusammen ihren Kurzschluss- oder Eingangsstrom überschreiten
  oder sich in der Spannung unterscheiden, und Anlagen mit mehr Modulen in Strings, als
  sie haben, oder Modulen in keinem String.
- **Erder** (*Erdung und Blitzschutz* im Bereich, `add_earth_electrode`,
  `set_earth_electrode`): ein **Fundamenterder** in der Bodenplatte der untersten
  Ebene, ein geschlossener Ring 100 mm innerhalb der Außenwände, durch Querbänder in
  Maschen von höchstens 20 × 20 m geteilt (`GEN.ER-STRIP-FE30`, seine Abstandhalter
  alle 2 m und Kreuzverbinder), ein **Ringerder** im Erdreich einen Meter außerhalb der
  Wände und einen halben Meter tief (`GEN.ER-STRIP-V4A30`) und **Tiefenerder** an einem
  Platz des Grundrisses, ab einem halben Meter Tiefe eingetrieben (Teilstücke
  `GEN.ER-ROD-1500`, Spitze und Klemme). Jeder Erder hat einen **Werkstoff**
  (feuerverzinkter Stahl, Edelstahl V4A, Kupfer, verkupferter Stahl) und einen
  **Durchmesser** (eines Stabs oder eines Runddrahts statt eines Flachbands); sein
  Artikel ist der der Artikeldatenbank oder des Katalogs mit diesem Werkstoff und
  Durchmesser (nach den technischen Daten `Material` und `Diameter`), sofern keiner
  gewählt ist. Die Außenwände sind das Rechteck, das die Dächer überdecken,
  sonst das der Räume. Ein Erder, der an eine Haupterdungsschiene (ein Betriebsmittel)
  angeschlossen ist, läuft mit einer Anschlussfahne zu ihr hinauf. All das kommt in die
  Stückliste.
- **Blitzschutz** (`set_lightning_protection`, `add_air_rod`, `set_air_rod`): der
  Klasse I bis IV, eine **Fangeinrichtung** aus Leitern auf den Firsten und Graten
  geneigter Dächer und um den Rand sowie als Masche (5, 10, 15 oder 20 m je nach
  Klasse) auf Flach- und Pultdächern, und damit verbundene **Fangstangen**;
  **Ableitungen** rund um die Außenwände im Abstand von höchstens 10, 10, 15 oder 20 m
  (mindestens zwei oder so viele wie angegeben), jede von unter der Traufe zu ihrer
  **Trennstelle** einen Meter über dem Boden und weiter ins Erdreich zum Ringerder
  (sonst zum Fundamenterder oder zu den Tiefenerdern), der Dachleiter von der Traufe
  über das Dach zur Fangeinrichtung. Der **Werkstoff** und der **Durchmesser** der
  Leiter und jeder Stange bestimmen ihre Artikel, das **Montagematerial** sind die
  Leitungshalter auf dem Dach und an den Wänden (Dach-, First- oder
  Flachdach-Leitungshalter, Wandhalter mit oder ohne Abstandhalter) in einem eigenen
  Abstand, eine Stange steht in einem Stangenhalter oder auf einem Betonsockel; auch
  die Höhe der Trennstellen wird eingestellt. Der
  Runddraht, seine Halter, die Stangen mit ihren Haltern oder Sockeln, Trennstellen,
  Erdeinführungen und Klemmen kommen in die Stückliste. Ein Klick auf einen Erder,
  einen Leiter oder eine Stange in der 3D-Ansicht oder im Lageplan öffnet deren
  **Eigenschaften** mit all diesen Einstellungen. Die PV-Anlagen auf den Dächern müssen
  im Schutzbereich nach dem **Schutzwinkel** der Klasse liegen (70 bis 79° bis 2 m über
  der Fläche, hinab bis 23° beim Radius der Blitzkugel). Die 3D-Ansicht und der
  Lageplan zeichnen die Erder und die Anlage. *Grundstück › Erdung und Blitzschutz*
  fügt sie einem Gebäude hinzu: Fundament- und Ringerder, Tiefenerder und Fangstangen,
  in den Lageplan geklickt, Blitzschutz mit seiner Klasse.
- **Verbindungen zwischen Gebäuden** (`add_site_link`, `set_site_link`): ein
  Kabelgraben, ein Leerrohr oder eine Kabelbrücke von einer Hauseinführung eines Raums
  zu einer Hauseinführung eines Raums in einem anderen Gebäude, über Ecken in der
  Anlage. Verlegewege passieren sie wie einen Wanddurchbruch; ihre Länge ist angegeben
  oder folgt dem Weg (hinab bis 800 mm unter den Boden bei einem Graben oder Leerrohr,
  in der Höhe der höheren Einführung bei einer Brücke).

Die Prüfung *Gebäude, Ebenen und Schächte* meldet Räume auf Ebenen, die es nicht mehr
gibt, Räume, die höher sind als ihre Ebene oder sich auf einer Ebene überlappen,
Ebenen, die einander überlappen, Schächte, die keine zwei Räume verbinden, Schränke und
bodenstehende Artikel, die in einem Schacht stehen, Verbindungen, deren
Hauseinführungen es nicht mehr gibt, Treppen ohne gültige Ebenen, mit einer Steigung
außerhalb von 140 bis 200 mm oder mit verletzter Schrittmaßregel (zweimal die Steigung
plus der Auftritt 590 bis 650 mm), Dachfenster neben ihrer Dachfläche oder über einem
First oder Grat, Räume, bei denen das Dach bis auf den Fußboden herunterkommt,
Schränke, die höher sind als die schräge Decke an ihrem Platz, und Elemente an den
Wänden oder der Decke oberhalb der Schräge, PV-Anlagen neben ihrem Dach oder über einem
First, einem Grat oder seinem Rand, aufgeständert auf einem geneigten Dach, über einem
Dachfenster oder übereinander, Freiflächenanlagen aufeinander, auf einem Raum oder
außerhalb der Grundstücksgrenze, und PV-Anlagen, deren Betriebsmittel fehlt oder eine
andere Anzahl von Modulen zählt oder deren Erdungsschiene fehlt, Erder, deren Schiene
fehlt, Tiefenerder innerhalb der Wände, Blitzschutz ohne Erder oder mit weniger
Ableitungen, als seine Klasse erfordert, Fangstangen neben den Dächern und PV-Anlagen
außerhalb des Schutzbereichs der Fangeinrichtung. Beispiel 15 (`plant_buildings`) zeigt
ein Hauptgebäude mit zwei Ebenen und einem Schacht, ein Pumpenhaus, verbunden durch
einen Kabelgraben, durch den ein Motorkabel verlegt ist, Fassadenleuchten und eine
Wetterstation auf dem Dach; Beispiel 16 (`office_building`) ergänzt ein Bürogebäude
mit einem Dachgeschoss unter einem Satteldach, einer an einem Viertelpodest gewinkelten
Treppe, Dachfenstern, einer PV-Anlage aus zehn LONGi-Modulen auf Schienen und Dachhaken
in der hinteren Dachfläche mit ihrem Wechselrichter und der Potenzialausgleichsschiene
ihrer Montage an der Giebelwand des Dachgeschosses, einem Fundamenterder an der
Haupterdungsschiene, einem Ringerder und einer Blitzschutzanlage der Klasse III mit
einer Fangstange, Dosen und Schaltern von Gira und MDT und einem
COSMO-Heizkreisverteiler.

**In 3D** zeigt *Layoutbereich › Anzeigen › Ganzes Gebäude* das Gebäude des angezeigten
Raums mit all seinen Ebenen und Räumen, ihren Geschossdecken, den Treppen und den
Dächern; *Alle Gehäuse* zeigt jedes Gebäude der Anlage auf diese Weise. Dächer und
Giebelwände sind von außen wie die Wände zu sehen (durchscheinend oder mit *Wände davor
ausblenden* weggelassen); *Decke ausblenden* lässt die Decken und die Dächer weg, um in
das Gebäude zu blicken. Die Anlage in 3D zeichnet die Grundstücksgrenze auf dem Boden
und den Bezugspunkt als Pfosten.

![Grundriss des Beispiels electrical_room: L-förmiger Raum, Innenwand, Tür, Fenster und die zwei Schränke](../../images/room.png)

## Grundrisse auf Papier

Elektrische Installationspläne werden auf der Zeichnung des Architekten vom Gebäude
gezeichnet. Ein **Grundriss auf Papier** kommt unter eine Ebene (Bereich *Gebäude*,
Registerkarte *Grundriss auf Papier*, die Ebene des Lageplans wählen, *Grundriss…*;
Kommandozeile `import_floor_plan`): ein Bild (PNG, JPEG,
TIFF, GIF, WebP, BMP) oder eine Seite einer PDF-Datei, ein Scan (auch die von
Büroscannern, die eine Seite in ein Hintergrundbild und schwarz-weiße Faxmasken
zerlegen) oder eine Zeichnung (ihre Linien und gefüllten Flächen; Texte werden nicht
gezeichnet). Bei einer PDF-Datei mit mehreren Seiten wird die Seite im Bereich gewählt.
Der Lageplan zeigt ihn unter den Räumen der Ebene, und der Grundriss jedes Raums der
Ebene zeigt ihn unter dem Raum an seinem Platz (*Anzeigen › Grundriss auf Papier* im
Menüband des Layoutbereichs, standardmäßig an; der
Fußboden des Raums wird dann durchsichtig gezeichnet); das Bild wird im Projekt als graues PNG gehalten (das Crate `openecraft-scan` liest die
Dateien, `openecraft_core::panel::floorplan` enthält den Rest).

- **Maßstab und Platz** (*Maßstab*, *Auflösung*, *Linke obere Ecke*, *Drehung*,
  *Angezeigt*; Befehl `set_floor_plan`): Der Maßstab der Zeichnung (1:100 bei einer
  neuen) und die Auflösung des Scans (aus der PDF-Seite, 150 dpi bei einem Bild)
  ergeben die Größe eines Pixels im Gebäude. *Messen* (Befehl `calibrate_floor_plan`)
  setzt den Maßstab aus zwei im Lageplan angeklickten Punkten und ihrem Abstand, einem
  Maß der Zeichnung. Die linke obere Ecke des Bilds liegt an einem Punkt des
  Gebäudegrundrisses, das Bild wird um sie gedreht wie ein Raum.
- **Bereich** (*Wählen*, `area` von `set_floor_plan`): der Teil des Bilds, in dem die
  Räume gesucht werden, ohne Schriftfeld, Schnitte oder die anderen Zeichnungen des
  Blatts; gestrichelt dargestellt.
- **Räume erkennen** (Befehl `detect_floor_plan`) findet die Wände, Türen, Fenster und
  Räume im Grundriss und zeigt sie im Lageplan: die Räume umrandet, Türen in der
  Warnfarbe, Fenster blau, einen Türaufschlag als Punkt an seinem Scharnier. Noch
  ändert sich nichts.
- **Räume anlegen** (Befehl `trace_floor_plan`) legt für jeden gefundenen Raum einen
  Raum auf der Ebene an (`+ROOM1`, …): seine Kontur die Innenflächen seiner Wände, so
  hoch wie die Ebene abzüglich ihrer Decke, an seinem Platz im Gebäude; eine Tür kommt
  in den Raum, in den sie öffnet (ihr Aufschlag gezeichnet), oder in den kleineren der
  beiden Räume, angeschlagen an der gezeichneten Seite; ein Fenster in den Raum
  dahinter. Räume, deren Kontur keine Kontur aus rechtwinkligen Wänden ist, werden
  gemeldet und ausgelassen. Der Grundriss bleibt unter der Ebene, um die Räume damit
  zu vergleichen und darauf zu zeichnen; *Entfernen* nimmt ihn weg, die Räume bleiben.

**Wie die Räume gefunden werden** (`openecraft_core::wallscan`, reine Bildverarbeitung,
kein Modell): Wände, die gefüllt, schattiert oder schraffiert gezeichnet sind (so, wie
geschnittene Wände gezeichnet werden), rechtwinklig zu den Blattkanten.

1. *Tinte*: ein Pixel, das deutlich dunkler ist (10 Stufen) als das Papier um es herum
   (die hellste Stufe innerhalb der dicksten Wand, gemessen auf dem verkleinerten
   Blatt; Scans verblassen und vergilben ungleichmäßig), nach einem 3 × 3-Median, der
   Flecken entfernt.
2. *Wände*: die Tinte, morphologisch geöffnet mit der dünnsten Wand (90 mm): Text,
   Maßlinien, Türaufschläge und die Linien von Fenstern verschwinden, die gefüllten
   Wände bleiben. Von dem, was übrig ist, sind nur große Teile abseits der Blattkanten
   Wände.
3. *Gerade Stücke*: Läufe quer zu einer Wand (eine Spalte einer Wand entlang x), so
   dünn wie eine Wand, die ihre Lage über mindestens das Doppelte ihrer Dicke halten;
   Läufe, die über ein Stück hinausreichen (eine Wand, die darauf trifft, ein
   Bleistiftfleck an seiner Seite), setzen es fort.
4. *Öffnungen*: vom freien Ende eines Stücks (nicht dort, wo ein anderes Stück darauf
   trifft) ein Strahl in seiner Richtung in den Zeilen seiner Mitte bis zur nächsten
   Wand: in den meisten Zeilen so lang wie eine Tür oder ein Fenster (0,5 … 5 m). Endet
   er an einer querstehenden Wand (eine Tür neben einer Ecke), ist er höchstens 1,5 m
   lang; breiter nur bis zum nächsten Stück derselben Wand (ein Fenster zwischen zwei
   Stücken). Eine Öffnung, deren zwei Seiten ohnehin verbunden sind (ein Fleck in einem
   Raum), ist keine.
5. *Räume*: die freien Flächen, die von den Wänden mit geschlossenen Öffnungen
   umschlossen sind (die, die den Blattrand erreichen, sind außen), mindestens 1 m²
   (*Kleinster Raum*, Befehlsparameter `min_room`, mit dem Grundriss gespeichert: ein WC oder eine Abstellkammer ist kleiner). Ihre Kontur: geschnitten entlang
   der Linien, auf denen der Rand über mindestens 150 mm verläuft, die Zellen dieser
   Schnitte, die überwiegend im Raum liegen, bilden den Grundriss (rechtwinklige Wände,
   kein Versatz kürzer als 75 mm).
6. *Türen und Fenster*: Eine Öffnung zwischen zwei Räumen ist eine Tür; eine nach außen
   ein Fenster, wenn Linien entlang der Wand durch sie laufen (Glas, Rahmen,
   Fensterbank), sonst eine Tür. Ein Viertelkreis aus Tinte von einem Ende einer Tür in
   einen Raum ist ihr Aufschlag.

Auf den gescannten Plänen eines Hauses von 1972 (1:50, 150 dpi, Bleistift) findet das
Verfahren die Räume des Erdgeschosses mit ihren Türen und Fenstern in einer halben
Sekunde; Wände, die als zwei Linien ohne Füllung gezeichnet sind, schräge oder
gekrümmte Wände und schief gescannte Blätter werden noch nicht gefunden. Die gefundenen
Räume sind ein Ausgangspunkt zum Prüfen und Korrigieren: Räume, die durch eine Öffnung
ohne gezeichnete Tür verbunden sind, sind ein Raum, und Treppen oder Schornsteine sind
Wände.

## Installation: Dosen, Schalter, Leuchten und ihre Stromkreise

Der Installationsplan einer Ebene wird im Bereich *Gebäude* (Registerkarte
*Elektroinstallation*, die Ebene des Lageplans wählen) auf ihrem Grundriss gezeichnet;
Plugin `installation`, `openecraft_core::installation`. Alles davon steht im
Schaltplan: die Betriebsmittel, ihre Verbindungen und Kabel; die Stromkreise werden
von dort gelesen, und die Längen ihrer Kabel sind die der 3D-Verlegung.

- **Betriebsmittel einsetzen** (*Einsetzen* mit einer gewählten Schutzeinrichtung und
  einem gewählten Artikel; Befehl `place_installation`): Ein Klick in einen Raum des
  Lageplans setzt dort eine Steckdose, einen Schalter, eine Leuchte oder ein Heizgerät
  für die Schutzeinrichtung ein (ein Leitungsschutzschalter, FI/LS-Schalter oder eine
  Sicherung des Netzes bis 32 A im Schaltplan; die Liste kennzeichnet die freien,
  Befehl `list_breakers`): eine Deckenleuchte (`GEN.LAMP-CEILING-LED`,
  `GEN.LAMP-DOWNLIGHT-LED`, Artikel mit *Ceiling mounting: yes*) an der Decke an der
  angeklickten Stelle, alles andere an der Wand, die dem Klick am nächsten liegt
  (innerhalb von 0,8 m), in seiner Montagehöhe (Steckdosen 300 mm, Schalter 1050 mm,
  Wandleuchten 2000 mm). Es wird zu einem Betriebsmittel des Schaltplans in seinem Raum
  (`+ROOM1-XD1` Steckdosen, `-SF1` Schalter, `-EA1` Leuchten, `-EB1` Heizgeräte) mit
  dem Artikel, gezeichnet auf der **Installationsseite** der Schutzeinrichtung: einer
  Schaltplanseite nach der Seite der Schutzeinrichtung, angelegt mit ihrem ersten
  Betriebsmittel, deren Abgangs-Anschlusspunkt die Abbruchstelle `L-F5` dorthin erhält
  (bei einer dreipoligen `L1-F5` … `L3-F5` an ihren drei Abgangs-Anschlusspunkten). Die
  Seite zeichnet die Außenleiter von diesen Abbruchstellen und die Neutral- und
  Schutzleiter von den Potenzialen `N-F5` und `PE-F5` als Linien entlang ihres oberen
  Rands, jedes Betriebsmittel in einer Spalte darunter.
  Jedes Betriebsmittel erhält ein Kabel `-W<n>` vom Betriebsmittel davor (das erste von
  der Schutzeinrichtung, eine Leuchte vom ersten Schalter ihres Raums oder von der
  letzten Leuchte dahinter) oder vom gewählten Betriebsmittel oder der gewählten
  Abzweigdose (*Kabel von*; Parameter `from`): Dort verzweigt ein Stromkreis. Seine
  Drähte sind die Adern (BN, BU, GNYE; dreiphasig BN, BK, GY, BU, GNYE) von
  `NYM-J 3x1.5` (`5x`), oder dicker, wo die Schutzeinrichtung oder ein Betriebsmittel
  des Stromkreises mehr verlangt (Strombelastbarkeit unter den Verlegebedingungen). Das
  Kabel behält seine Quelle und sein Ziel (`cable.source`, `cable.target`): Die
  Verbindungen des Stromkreises, die Abschnitte der Liste und die Enden der 3D-Wege
  folgen ihnen, die Spalten der Seite dem Baum der Kabel. In einem Netz, dessen Drähte
  zu mehreren Kabeln gehören, nimmt jede Verbindung das Kabel der Drähte an ihren
  eigenen Anschlusspunkten.
- **Drehstromkreise**: Eine dreipolige Schutzeinrichtung speist CEE-Steckdosen
  (`GEN.SOCKET-CEE-16-5P`, `-32-5P`), Herdanschlussdosen (`GEN.CONN-BOX-COOKER-UP`)
  und dreiphasige Heizgeräte (`GEN.HEATER-SAUNA-9000`); ein Betriebsmittel mit L2 oder
  L3 wird an einem einphasigen Stromkreis abgelehnt.
- **Abzweigdosen und Geräteanschlussdosen** (`GEN.JBOX-UP`, `GEN.JBOX-5-UP`,
  `GEN.CONN-BOX-UP`; Registerkarte *Dosen*): Eine Abzweigdose ist ein Betriebsmittel
  des Stromkreises, von dem die Kabel anderer ausgehen; eine Geräteanschlussdose speist
  fest angeschlossene Geräte. Eine Abzweigdose ist in der Kabeldimensionierung keine
  Last: Ein an ihr endendes Kabel führt die größte Last der dort beginnenden Kabel
  (nach ihrer Quelle und ihrem Ziel, die `place_installation` speichert; ein von Hand
  gezeichnetes Kabel gibt keine weiter), einphasig berechnet für eine einphasige Last
  hinter einer fünfadrigen Zuleitung.
- **Wechselschaltungen**: Ein Wechselschalter (`GEN.SW-2WAY-UP`), der `from` einem
  anderen platziert wird, wird aufrecht in der Spalte nach ihm gezeichnet, die zwei
  Korrespondierenden ihres Kabels von dem einen zum anderen; für mehr Schaltstellen
  kommen Kreuzschalter (`GEN.SW-INTER-UP`) dazwischen, jeder `from` dem Schalter davor,
  gezeichnet auf dessen Seite (abgelehnt, wo diese Seite voll ist). Eine Leuchte ohne
  `from` hängt am letzten Schalter, der auch die Leuchten übernimmt, die an den
  Schaltern davor platziert wurden; andere von ihnen gespeiste Betriebsmittel bleiben.
  Ein Wechselschalter nach einem geschlossenen Paar (zwei Wechselschalter) beginnt
  einen eigenen Stromkreis, auch mit `from` einem des Paars. Die Kabel zwischen den
  Schaltern haben vier Adern: die zwei Korrespondierenden sowie den Neutral- und den
  Schutzleiter, die zu den Leuchten weitergeführt werden. Ein allein arbeitender
  Wechselschalter hat seine freie Korrespondierende in *Nicht belegte Anschlusspunkte*
  (`function.unused_pins`): Ein dort genannter Anschlusspunkt wird nicht als offen
  gemeldet. Die Korrespondierenden eines Kreuzschalters ohne letzten Schalter bleiben
  offen. Ein Jalousieschalter (`GEN.SW-BLIND-UP`) hat zwei echte Ausgänge und
  ist kein Wechselschalter.
- **Ohne Stromkreis**: Daten- und Antennendosen (`GEN.DATA-RJ45-2X-UP`,
  `GEN.TV-SAT-UP`) und Rauchwarnmelder (`GEN.SMOKE-ALARM`, an der Decke) werden nur in
  den Räumen platziert (`place_installation` ohne `breaker`; mit `tag` als dieses
  Betriebsmittel), im Lageplan angezeigt und mit den Elementen der Räume aufgelistet.
- **Außerhalb des Gebäudes**: Ein Punkt außerhalb des Raums setzt den Artikel auf die
  Außenseite seiner nächsten Wand in seiner Montagehöhe (eine Außensteckdose, eine
  Fassadenleuchte); Betriebsmittel
  im Garten kommen in
  einen Außenbereich, einen Raum, der außerhalb des Gebäudes platziert ist
  (`place_on_level` ohne Gebäude).
- **Was `place_installation` außerdem annimmt**: das Betriebsmittelkennzeichen (`tag`),
  die Montagehöhe (`height`, mm), Kennzeichen, Typ, Artikel, Aderzahl und Querschnitt
  des Kabels sowie eine bekannte Länge (`cable`, `cable_type`, `cable_part`, `cores`,
  `cross_section`, `length`); eine bekannte Länge wird beibehalten (*Länge bekannt*,
  `cable.fixed_length`) und statt des 3D-Wegs genommen. `place_cabinet_device` setzt
  die Schutzeinrichtungen in den Verteiler, den die Stromkreise für ihre Längen
  brauchen.
- **In der 3D-Ansicht und den Wandansichten** (*Einfügen › Dosen, Schalter, Heizung*):
  Das Menü wählt oben die Schutzeinrichtung (oder *Keins: nicht in den Schaltplan*);
  der gewählte Artikel wird mit einem Klick auf eine Wand oder die Decke des Raums in
  der 3D-Ansicht platziert (an einer Wand in seiner Montagehöhe; Esc beendet) oder auf
  der Wand in einer Wandansicht und kommt wie mit *Einsetzen* in den Schaltplan. Die
  Liste des Menüs ist so hoch wie ihre Artikel, mindestens wie der daneben gezeigte
  Artikel, höchstens etwas mehr als das halbe Fenster.
- **Der Lageplan** zeichnet die Betriebsmittel mit den Symbolen von Installationsplänen
  (IEC 60617-11) in der Farbe ihres Stromkreises und die Verbindungen der Stromkreise;
  in 3D sind es die Modelle der Artikel (erzeugte für die generischen Artikel und
  Artikel ohne eigenes Modell: Dosen und Schalter in ihren Rahmen, runde
  Deckenleuchten mit gewölbtem Diffusor, Einbaustrahler mit Flansch und Diffusor,
  rechteckige Wand-, Panel- und Lichtleisten-Leuchten, Notleuchten mit ihrem Zeichen,
  Wandkonvektoren mit Gitter und Thermostat, Handtuchheizkörper mit ihren Sprossen,
  Infrarot-Heizpaneele). Das 3D-Menü *Einfügen › Dosen, Schalter, Heizung* hat auch
  eine Registerkarte *Leuchten*.
- **Heizgeräte** werden auf dieselbe Weise eingesetzt: Wandkonvektoren
  (`GEN.HEATER-PANEL-1000`, `GEN.HEATER-PANEL-2000`, 400 mm) und Handtuchheizkörper
  (`GEN.HEATER-TOWEL-500`, 900 mm) an der Wand, Infrarot-Heizpaneele
  (`GEN.HEATER-IR-600`) an der Decke; Artikel der Untergruppen *Raumheizgerät*,
  *Handtuchheizkörper*, *Infrarot-Heizpaneel* oder mit einer Heizgerätefunktion für
  Wände oder Decken (Schaltschrankheizungen sind keine) (*Einfügen › Dosen, Schalter,
  Heizung*, Registerkarte *Heizung*).
- **Stromkreise des Schaltplans** (nur lesend; Befehl `list_circuits`, Auswertung
  *Installationsstromkreise*): jede Schutzeinrichtung des Netzes bis 32 A mit den
  Steckdosen, Schaltern, Leuchten und Heizgeräten dahinter: an den Netzen hinter der
  Schutzeinrichtung, über Schalter, Kontakte, Klemmen und die Hauptkontakte von
  Schützen, nicht über weitere Schutzeinrichtungen oder Lasten. Ein Betriebsmittel
  zählt nach seinem Artikel (einem für Wände oder Decken) oder, ohne Artikel, nach
  seinem Symbol (Steckdosen, Heizgeräte, Lampen, Leuchten, Installationssymbole;
  Kontakte eines Betriebsmittels, dessen Kennbuchstabe mit `S` beginnt); in einem
  Schrank platzierte Betriebsmittel werden ausgelassen. Die Liste zeigt für jeden die
  platzierten Betriebsmittel, seine Kabel, die Länge und die Spannungsfälle; ein Klick
  auf einen Stromkreis wählt seine Schutzeinrichtung.
- **Betriebsmittel des Schaltplans platziert** (*Alle platzieren* oder ein Klick auf
  ein Betriebsmittel des gewählten Stromkreises; Befehl `place_circuit_device`): Von
  Hand im Schaltplan gezeichnete Betriebsmittel werden mit einem Klick in einen Raum
  wie ein Artikel platziert (mit ihrem eigenen Artikel, dem gewählten Artikel, wenn er
  von ihrer Art ist, oder dem generischen ihrer Art, der zu ihrem Artikel wird); ein
  Klick auf eine Steckdose, einen Schalter, eine Leuchte oder ein Heizgerät, das zuvor
  ohne Betriebsmittelkennzeichen platziert wurde, macht dieses zum Betriebsmittel. Ein
  erneut platziertes Betriebsmittel wird verschoben; das nächste noch nicht platzierte
  Betriebsmittel folgt.
- **Kabellänge**: Jede Verbindung der Außenleiter eines Stromkreises zwischen zwei
  Betriebsmitteln, ein Kabel (einmal) oder ein einzelner Draht, hat die Länge ihres
  3D-Wegs (Verlegung in den Räumen, durch Wände und zwischen Ebenen,
  `crate::routing3d`). Bis sie verlegt ist, wird sie geschätzt (angezeigt mit `~`): von
  der Oberkante des Verteilers zum Betriebsmittel in der waagerechten
  Installationszone 300 mm unter der Decke (DIN 18015-3), entlang x und z der Anlage,
  hinauf und hinab in den senkrechten Zonen der Betriebsmittel, hinauf oder hinab
  zwischen Ebenen, 0,3 m für jede Dose. *Kabellängen schreiben*
  (`write_circuit_cables`) schreibt die Längen (auf den halben Meter aufgerundet) in
  die Kabelliste.
- **Spannungsfall ab der Einspeisung** (*Ab Einspeisung*; `total_drop_percent` von
  `list_circuits`): je Verbindung ΔU = k · I · L · (ρ / A · cos φ + X' · sin φ) (k = 2
  einphasig, √3 dreiphasig, ρ von Kupfer bei seiner Betriebstemperatur aus den
  elektrischen Einstellungen, X' = 0,08 mΩ/m, cos φ 0,95), summiert entlang der
  Verbindungen von der Schutzeinrichtung zum entferntesten Betriebsmittel. Die Kabel
  des Stromkreises führen den Betriebsstrom seiner Schutzeinrichtung (Lastberechnung)
  oder ihren Bemessungsstrom; die Zuleitungskabel von der Einspeisung zum Verteiler
  (die, denen die Kurzschlussberechnung folgt, mit ihren Längen in der Kabelliste) den
  Betriebsstrom der vorgelagerten Schutzeinrichtung oder ihren Bemessungsstrom. Die
  Summe wird gegen den Grenzwert der Regelwerke geprüft (*Spannungsfall*); der Tooltip
  nennt den Anteil der Zuleitung.
- **Prüfung** *Installationsstromkreise*: Stromkreise, deren Schutzeinrichtung in
  keinem Verteiler des Layouts sitzt, Kabel, die dünner sind, als ihre
  Schutzeinrichtung verlangt (Strombelastbarkeit unter den Verlegebedingungen),
  Spannungsfälle über dem Grenzwert, Betriebsmittel eines Stromkreises, die in keinem
  Raum platziert sind (Information); Steckdosen, Schalter, Leuchten und Heizgeräte in
  den Räumen, die nicht im Schaltplan stehen (Information).

## Hausanschlusskästen

Das Gehäusesystem *Hausanschlusskasten* (`house_connection`, Befehl `add_enclosure`
mit `system` `house_connection`) ist der Kasten des Netzbetreibers an der
Hauseinführung: glasfaserverstärktes Polyester (RAL 7035, IP54) mit einem
aufklappbaren, plombierbaren Deckel und einer Montageplatte, in zwei Standardgrößen
(`GEN.HAK-255X415X144` für NH00, `GEN.HAK-410X720X185` für NH2). Sein Deckel öffnet
sich in der 3D-Ansicht wie die Tür jedes Gehäuses. Hinein kommen das
NH-Sicherungsunterteil (`GEN.NH2-BASE-3P-400A`, von unten gespeist: Das
Zuleitungskabel kommt durch eine Verschraubung im Boden gerade hinauf zu seiner
Netzseite, das Hauptkabel geht gerade nach oben durch die Oberseite hinaus) und der
PEN-Klemmenblock (`GEN.TB-PEN-240-V`). Das Beispiel `building_services` hängt einen an
die Wand über der Kabeleinführung, mit der Haupterdungsschiene (`GEN.MET-BAR-10`) und
der Anschlussfahne des Fundamenterders (`GEN.FE-LUG-V4A-10`, Symbol *Erder*) daneben.

## Verteilerschränke mit Funktionsbausteinen

Ein Gehäuseartikel eines Herstellers, der mit seinem Sockel geliefert wird, sagt das in
seinen Daten (`Plinth: 200`): Ein neues Gehäuse davon steht auf dem Sockel, der
gezeichnet und exportiert wird, aber kein eigener Artikel in der Stückliste ist (der
Hager-univers-Schrank `HAG.FR25S2`, 1850 mm plus 200 mm Sockel, fünf Felder,
Schutzklasse II).
Seine Montageplatte wird durch `Mounting origin: 25/12.5/20` platziert (die linke obere
Ecke der Plattenfront von links, von oben und von der Rückseite des Gehäuses, mm; die
Platte hält rechts und unten denselben Abstand), wie die Gehäusepakete des
Rittal-Konfigurators es angeben ([Artikel](parts.md)). Eine nicht zentrierte Platte
gibt auch ihre Größe an (`Mounting size: 500 x 575`; `mounting_size` von
`add_enclosure` / `set_enclosure`). Ein Gehäuseartikel
eines Herstellers, der sein System nennt (`Enclosure system: compact`), erhält die
Wände und Daten dieses Systems, bleibt aber der Artikel des Gehäuses; nur Gehäuse des
eingebauten Katalogs folgen in ihrer Größe dem Korpus des Systems.
Verteilersysteme bauen den Schrank aus Funktionsbausteinen auf: Ein Baustein, auf dem
das Betriebsmittel steht, ist ein Zubehör, das *unter dem Artikel* montiert ist
(`HAG.UE31E1B` unter dem NH1-Sicherungslasttrennschalter `HAG.LT153`, `HAG.UE31SS400`
unter den Stromwandlern), so dass das Betriebsmittel die Grundfläche und Tiefe des
Bausteins übernimmt und der Artikel davor sitzt.
Das Beispiel `pv_transformer_metering` baut einen solchen Schrank Feld für Feld nach
einem realen auf: Netzanschlussraum und Überspannungsableiter, Bezugsmessung mit
NH1-Trennung an der Sammelschiene, Wandlerbaustein darüber, Spannungspfade und
Lasttrennschalter (die Leistung fließt im Feld nach oben: Die Leiter laufen von einem
Betriebsmittel gerade in das nächste), der NA-Schutz, die Erzeugungsmessung unter dem
Kuppelschalter und die Sammelschienen der Abgänge; Kranösen im Dach, die Kabel der
Verbraucher durch den Boden, die Einspeisung von unten; die zwei Zählerschränke des
Verteilnetzbetreibers mit ihren Prüfklemmenleisten daneben. Kabeleinführungsflansche
(Artikel-Untergruppe `Cable entry`, eine Membran, die mehrere große Kabel aufnimmt, wie
der univers FZ402) sind für die 3D-Verlegung Zugänge wie Kabelverschraubungen. Das
Beispiel `industrial_main_distribution` reiht zwei univers-Schränke, zueinander offen,
zur Hauptverteilung einer Industrieanlage an: die Aluminium-Zuleitungskabel auf
Sammelschienenklemmen und durch einen Einführungsflansch im Boden, die Wandler der
Messung in der Einspeise-Sammelschiene, ein 185-mm-Sammelschienensystem durch beide
Schränke mit senkrechten NH-Sicherungslasttrennleisten, die Stromwandler hinter den
gemessenen eingerastet, die Kuppelschalter des NA-Schutzes von Sammelschienenklemmen
gespeist. Keiner von beiden hat Kabelkanäle: Jeder Leiter darin wird von der
3D-Verlegung gelegt, so dass die hohen Ströme über die Schienen laufen und nur das ein
Draht ist, was sich mit seinem Radius biegen lässt.

## 19-Zoll-Racks

Das Gehäusesystem *19-Zoll-Rack* (`rack19`, *Layoutbereich › Neues Gehäuse*, Befehl
`add_rack` oder `add_enclosure` mit `system` `rack19`) ist ein Rahmen mit einer
perforierten Tür und zwei Paaren 19-Zoll-Profilschienen (IEC 60297) statt einer
Montageplatte; seine Standardgrößen reichen von einem Wandrack mit 12 HE bis 47 HE, und
die Schienen sind ein Artikel der Stückliste (`GEN.RK-RAIL-42U`). Seine Höheneinheiten
(1 HE = 44,45 mm) sind in der Frontansicht und auf der Montageplan-Seite neben den
Schienen von unten nummeriert.

- **19-Zoll-Teile** sind so breit wie eine 19-Zoll-Frontplatte (482,6 mm) und eine
  ganze Zahl von Einheiten hoch (technische Daten `Rack units: 2`): Der Katalog hat
  einen Netzwerk-Switch, ein Patchpanel, eine PDU, einen Server, eine USV, Blindplatten,
  einen Fachboden und ein Kabelführungspanel. *Einfügen › 19-Zoll-Teil* montiert eines
  in den höchsten freien Einheiten des angezeigten Racks (Befehl `mount_in_rack` mit
  `unit`, von unten gezählt); platziert oder gezogen rasten Betriebsmittel und Teile
  auf ganzen Einheiten ein, zentriert, ihre Frontplatte auf den vorderen Schienen, und
  werden von den Schienen gehalten (keine Bohrungen in einer Platte).
- Doppelt belegte Einheiten sind Kollisionen; die Prüfung meldet 19-Zoll-Teile
  außerhalb der Einheiten und, als Hinweis, andere Teile auf der Montageebene (sie
  brauchen einen Fachboden oder eine Montageplatte). Die Auswertung
  *19-Zoll-Rackbelegung* (und `list_racks`) listet die Teile jedes Racks von oben mit
  ihren Einheiten und Tiefen sowie die freien Einheiten auf.
- Racks lassen sich zu Anreihungen anreihen und stehen in Räumen wie andere Schränke.

## 3D-Verlegung

Kabel, Schläuche und (optional) Drähte werden durch die Anreihung verlegt: in
Kabelkanälen, entlang von Verlegewegen, die über die Montageplatte gezeichnet sind,
oder frei um die Betriebsmittel herum; Trennung der Klassen Leistung, Steuerung, Signal
und Fluid, Verlegereservierungen, Biegeradien, Kabelbündel mit ihren Abzweigen,
Schirmanschlüsse (Schirmklemmschiene, Pigtail) und ihre Prüfungen; Verlegungen können
für die Fertigung eingefroren werden und bleiben, wie sie sind, bis sie ungültig
werden.

Die Drähte eines Schranks mit Kabelkanälen auf seiner Montageplatte laufen durch die
Kanäle (die Layout-Einstellung *Drähte in 3D verlegen* verlegt sie stattdessen wie
Kabel). In einem Schrank ohne Kabelkanäle und in einem Raum gibt es nichts, worin sie
gelegt werden könnten: Jeder Draht wird in 3D von Anschlusspunkt zu Anschlusspunkt
verlegt. Er verlässt jeden Anschlusspunkt geradeaus in der Richtung, die der Punkt
vorgibt, so weit wie sein Biegeradius, und kommt in den anderen gegen die Richtung
hinein, die dieser vorgibt, niemals an ihm vorbei und zurück
(durch
den Quader seines eigenen Betriebsmittels, wo der Punkt darin liegt, wie die Klemmen
eines Schalters auf seinem Baustein, niemals durch ein anderes Betriebsmittel, das
direkt vor dem Punkt steht), hält die Hälfte seiner Dicke Abstand zu allem, biegt nur
mit seinem
Radius und hält Abstand zu jedem anderen Weg und zu sich selbst (wo seine Enden für
seinen Biegeradius zu nahe beieinander liegen, läuft er in einer Schleife herum, die
niemals über oder neben seinen eigenen Weg, die Bögen an seinen Ecken und den Stutzen,
aus dem er kommt, zurückkehrt; wo es keine solche Schleife gibt, wird er nicht
verlegt); Wege kommen nur dort zusammen, wo sie auf dasselbe Betriebsmittel zulaufen,
und dort liegen sie nebeneinander: Ein Weg läuft niemals quer über einen anderen, mit
dem er zusammenkommt (er darf es nur auf seinem eigenen Weg aus einem Anschlusspunkt
heraus oder dort, wo er auf das Betriebsmittel zuläuft, an dem der andere endet, und
geht auch dort um ihn herum, wo das wenig kostet). Der gerade Weg aus jedem
Anschlusspunkt und jeder Kabeleinführung heraus bleibt frei, bis sein eigener Weg
gelegt ist, damit kein anderer Weg quer über eine Klemme läuft, und nur ein Weg aus
einem Anschlusspunkt direkt daneben teilt ihn. Von den gleich dicken Drähten eines
Schranks wird der kürzeste zuerst verlegt: Zwischen zwei Betriebsmitteln liegen die
inneren Leiter ihnen am nächsten, die äußeren gehen um sie herum. Ein Draht, der
keinen solchen Weg findet, wird nicht gezeichnet: Er wird als Fehler gemeldet
(*… kann nicht verlegt werden: kein Weg, der mit der halben Dicke von allem frei bleibt
und mit dem Biegeradius von … mm biegt*). Die üblichen Ursachen sind ein Leiter, der
für den Raum zwischen seinen Enden zu dick ist (ein Sammelschienenquerschnitt, als
Draht gezeichnet: die Betriebsmittel stattdessen auf einem Sammelschienensystem
montieren), Anschlusspunkte, die voneinander weg zeigen, eine Klemmenleiste direkt
unter einem Schalter oder zwei Enden, die näher beieinander liegen, als die Bögen
dazwischen brauchen (eine Kabeleinführung direkt unter dem Kasten, in den sie führt,
zwei Armaturen übereinander mit ihren Anschlüssen an den Seiten). Der Bereich Meldungen springt zu einem Anschlusspunkt
des Drahts oder Kabels, und die 3D-Ansicht zeigt an jedem seiner Enden anstelle des
Wegs einen roten Hinweis mit dem Grund.

Die Verlegung arbeitet auf einem Raster (25 mm in einem Schrank, 50 mm in einem Raum);
ein Anschlusspunkt liegt, wo er liegt. Die 3D-Ansicht zeichnet einen Weg gerade aus
seinem Anschlusspunkt heraus: Seine erste Ecke liegt auf dieser Linie, das Stück danach
neigt sich hinüber oder kommt auf einem schrägen Stück zurück auf das Raster, kurze
Stücke dazwischen werden mitverschoben. Wo der Weg gerade aus einem Punkt heraus
versperrt ist und der Weg von der Zelle daneben ausgeht, braucht er dort Platz für
einen S-Bogen mit seinem Radius auf diese Linie (oder um sich zu ihr hinüberzuneigen),
sonst wird er nicht verlegt. Wege, die zusammen laufen, liegen nebeneinander auf
Spuren; ein Weg wechselt die Spur auf einem schrägen Teil eines Stücks, niemals in
einer Stufe an einer Ecke, und ein Versatz, der für die Bögen seiner zwei Ecken zu kurz
ist, wird als S-Kurve mit dem Biegeradius gezeichnet. Ein Kabel endet auf dem Bogen der
Ecke, bevor seine Adern sich trennen, wo es dort abgesetzt ist; die Adern trennen sich
innerhalb des Quaders ihrer Enden, und eine Ader zu einer Klemme hinter anderen ihrer
Leiste (eine über der anderen auf den Schienen) geht vor dem Betriebsmittel an ihnen
vorbei. Wo sich zwei Wege direkt an einem Betriebsmittel, auf das beide zulaufen, doch
noch kreuzen, hebt die Ansicht einen schärfer über den anderen, als ihr Biegeradius
erlaubt.

Ein Kabel zwischen zwei Schränken, die in keinem gemeinsamen Raum stehen, wird in jedem
von ihnen zu einer seiner Kabeleinführungen verlegt (ein Hinweis sagt, dass der Weg
dazwischen nicht Teil des Layouts ist); jedes Kabel nimmt eine eigene Einführung, wo es
genug passende gibt. Die Kabel durch einen Kabeleinführungsflansch gehen nebeneinander
durch dessen Membran (Artikeldaten `Cutout`), jedes an einem eigenen Platz, um die
Dicke des Kabels und einen Zentimeter auseinander: der freie Platz, der der Richtung
des Kabels am nächsten liegt, die Mitte, wo keiner frei ist.

Zwischen Räumen passiert ein Weg die Wanddurchbrüche, Schächte und Verbindungen
zwischen Gebäuden, die sie verbinden, durch so wenige wie möglich: durch den ersten
Raum zu einem Durchbruch, durch die Räume dazwischen, durch den letzten Raum zu seinem
Ende (seine Abschnitte je Raum und Schrank in der Verlegeliste); seine Länge enthält
die Wege durch die Wände, Schächte und Verbindungen. Ohne einen solchen Durchbruch
bleibt der Weg unverlegt (*kein Wanddurchbruch führt von Raum zu Raum*). Ein Weg von
einem Betriebsmittel eines Raums nach außerhalb des Gebäudes endet an seiner nächsten
Hauseinführung, auch von einem Betriebsmittel in einem Schrank, der im Raum steht
(durch die Verschraubung des Schranks und den Raum); von mehreren
Wanddurchbrüchen, die dieselben zwei Räume verbinden, nimmt der Weg den mit dem
kürzesten Weg.

In einem Raum laufen Wege durch seinen freien Raum: um seine Wände, Innenwände, die
darin stehenden Schränke und den Aufschlag seiner Türen herum, bevorzugt in seinen
Kabelkanälen und Rohrtrassen. Waagerecht durch den offenen Raum zu laufen, mehr als
300 mm von seinen Wänden, seiner Decke und dem, was darin steht, kostet das Zehnfache:
Rohre und Kabel halten sich an die Wände und die Decke und fallen gerade zu einem im
Raum stehenden Betriebsmittel ab. Kabel und Rohre biegen erst nach der Hälfte ihres
Biegeradius geradeaus ab (wo es einen solchen Weg gibt); von den Wegen, die etwa
gleich viel kosten, nimmt ein Weg einen, dessen Bögen Abstand halten (ein kurzes Stück
zuerst oder zuletzt). Ein Weg zwischen einem Schrank und einem Betriebsmittel in seinem
Raum (oder zwei Schränken eines Raums) läuft in Abschnitten: durch den Schrank zu einer
Kabelverschraubung an seinem Dach, seinen Seiten oder seiner Rückseite (zuerst eine
seines eigenen Schranks, sonst der Anreihung; ohne eine durch die Mitte des Dachs, was
gemeldet wird), dann durch den Raum. Eine Kabelverschraubung nimmt ein Kabel auf: die
freie, deren Klemmbereich zum Kabel passt, am nächsten zu seiner Richtung; ein
Kabeleinführungsflansch nimmt so viele auf, wie Plätze entlang seiner Membran sind.
Ein Kabel, das jede Verschraubung seines Schranks belegt findet, wird nicht verlegt und
als Fehler gemeldet (*… jede Kabelverschraubung von … ist von einem anderen Kabel
belegt. Fügen Sie eine Kabelverschraubung dafür hinzu*).
Eine als Gehäuse gekennzeichnete Abzweigdose eines Herstellers (Artikeldaten `Junction
box enclosure: yes`, die Spelsberg Abox) wird als Gehäuse des Raums an einer Wand
platziert, ihre Rückseite an der Wand, ihre Mitte in der Montagehöhe, mit einer
WAGO-221-415-Verbindungsklemme darin, die zum Betriebsmittel des Schaltplans wird (die
Kabel ihres Stromkreises laufen in die Dose). In der 3D-Ansicht platziert ein Klick auf
die Innenseite einer Wand oder die Decke eines Raums dort einen Artikel: innerhalb von
200 mm um seine Montagehöhe in dieser Höhe (seine Installationszone), weiter entfernt
in der angeklickten Höhe; der Artikel wird durchsichtig dort gezeigt, wo er hinkäme,
während sich der Mauszeiger bewegt.
Mit *Kabelverschraubungen für neue Kabel hinzufügen* (Gehäuseeigenschaften,
`set_auto_glands`; standardmäßig an) erhält nach jeder Änderung der Kabel oder des
Schaltplans ein Kabel, das einen Schrank zu einem Betriebsmittel außerhalb von ihm und
seiner Anreihung verlässt und keine freie Verschraubung findet, durch die es passt,
eine: aus Polyamid, die kleinste, deren Klemmbereich das Kabel aufnimmt, am ersten
freien Platz von der Mitte der Fläche aus. Ein Kabel zu einem Betriebsmittel in dem
Raum, in dem der Schrank steht, erhält sie im Dach (es kann nur durch das Dach, die
Seiten und die Rückseite hinaus, siehe unten; bei einem an einer Wand hängenden Schrank
im Boden), ein Kabel ins Feld im Boden. Wo diese Fläche voll ist, kommt die
Verschraubung in die nächste Fläche, durch die das Kabel hinaus kann: den Boden oder
das Dach, dann die linke und rechte Seite und die Rückseite (nicht eine Seite am
nächsten Gehäuse der Anreihung, auch nicht die Seiten eines Gehäuses mit weniger als
150 mm Tiefe, dessen Drähte keinen Weg zu ihnen fänden; nicht die Rückseite eines
Gehäuses, das an der Wand hängt oder an einer Wand seines Raums steht, und sein Boden
für ein Kabel in den Raum nur zu einem Platz unterhalb des Schrankbodens mit seinem
Sockel, wie die Verlegung ihn verlässt: das Betriebsmittel im Raum oder in einem dort
stehenden Schrank, sonst der erste Wanddurchbruch auf dem Weg zu seinem Raum; ein
Betriebsmittel auf dem Dach liegt darüber). Ein
Kabel, das auf keiner von ihnen einen freien Platz findet, erhält keine und wird gemeldet (von *Fehlende hinzufügen* in der Statuszeile,
als `glands_missing` eines Befehls und `missing` von `add_cable_glands`, jeweils mit
dem Schrank und dem Grund): Platz schaffen oder einen Kabeleinführungsflansch ergänzen.
Eine von Hand gelöschte Verschraubung bleibt gelöscht, bis sich die Kabel ändern;
*Fehlende hinzufügen* (`add_cable_glands`) fügt für jedes Kabel, das keine hat, eine
Verschraubung hinzu, auch bei ausgeschalteter Option. Ein Schrank mit einem
Kabeleinführungsflansch erhält keine. Ein
an einer Wand hängender Schrank (sein Boden über dem Fußboden) wird auch durch eine
Verschraubung in seinem Boden verlassen: Unter jeder solchen Verschraubung bleibt ein
Schacht hinab zum Fußboden und hinaus zur Vorderseite des Schranks für die Wege frei.
Eingefroren behält ein solcher Weg seine Abschnitte.

**Stützpunkte.** Eine Leitung, ein Kabel oder ein Schlauch wird durch Stützpunkte
anders geführt: Rechtsklick auf die Verlegung über der Montageplatte, auf einer Wand
eines Raums (die Vorderansicht eines Raums zeigt die Wege von der Fläche aus gesehen,
auch seine Leitungen) oder in der 3D-Ansicht und *Stützpunkt hinzufügen* wählen; der
Punkt kommt dort auf die Verlegung, wo geklickt wurde, nummeriert entlang ihr. Ein
Rechtsklick behält die gewählte Leitung, das Kabel oder den Schlauch, wo sie dort
verlaufen: In einem Kabelkanal, wo Leitungen übereinander liegen, zuerst die Leitung
anklicken (nochmals für die nächste), dann rechts klicken. Eine
Leitung mit Stützpunkten wird wie ein Kabel in 3D von Anschlusspunkt zu Anschlusspunkt
verlegt (eine in den Kabelkanälen verlässt sie), ohne sie kehrt sie in die Kanäle
zurück; eine Leitung zur Tür nimmt keine (die Verlegung meldet dafür einen Fehler).
Einen Stützpunkt ziehen, um ihn zu verschieben: in der Vorderansicht auf der gezeigten
Fläche (er behält seinen Abstand von ihr), in der 3D-Ansicht parallel zum Bildschirm.
Wo er losgelassen wird, wird der Weg neu gesucht, vom Anfang durch jeden Stützpunkt der
Reihe nach bis zum Ende: Jeder Punkt wird in der Mitte seiner Verlegezelle passiert, der
Weg läuft so aus ihm heraus, wie er hineinkam, hält die halbe Dicke Abstand von allem
und von den anderen Wegen, biegt nur mit seinem Radius und kommt nicht zu sich selbst
zurück. Wo das nicht geht (der Punkt liegt in einem Gerät oder außerhalb des Gehäuses,
kein Weg zu ihm bleibt frei und biegt mit dem Radius, die Punkte lassen keinen Platz für
die Bögen zwischen ihnen, der Weg kommt zu sich selbst zurück, oder der Punkt liegt in
keinem Schrank und keinem Raum, durch den verlegt wird), meldet die Verlegung einen
Fehler mit dem Stützpunkt, zeichnet ihn rot und verlegt ohne seine Stützpunkte, bis er
verschoben wird; die Statuszeile sagt, ob der Weg durch
seine Stützpunkte geführt wird oder warum nicht. Nur der Weg, dessen Stützpunkte sich
geändert haben, wird sofort neu gesucht, zwischen den anderen Wegen, wo sie liegen
(währenddessen läuft ein Bogen um seinen Stützpunkt); die ganze Verlegung folgt im
Hintergrund. Ein Punkt auf einer Ecke des Wegs rückt um den Biegeradius an ihm zurück,
und der Weg zurück von einem Punkt hält Abstand vom Weg dorthin (in einem Kabelkanal
darf er ihn kreuzen). Rechtsklick auf einen Stützpunkt entfernt ihn oder alle (auch in den
Eigenschaften der Verlegung oder Leitung). Ein Weg mit Stützpunkten ist nicht
eingefroren. Befehle `add_control_point`, `move_control_point` und
`remove_control_point` (Punkte in den Koordinaten eines Gehäuses; eine Leitung als
`from|to`, ihre Anschlusspunkte); `list_routing` listet sie auf.

**Unterputz in Räumen.** Jedes Kabel, jeder Draht und jeder Schlauch wird in Räumen
*Aufputz* (wie oben), *Unterputz in der Wand* oder *Unterputz im Fußboden* verlegt: wie
es die Verlegeregeln sagen (*Verlegeart in Räumen*, Befehl `routing_rules` mit
`laying`) oder wie dafür eingestellt (Eigenschaften des Wegs › *Verlegeart in Räumen*,
Befehl `set_laying` mit seinem Schlüssel). Unterputz läuft es durch den Raum in
Schlitzen in seinen Wänden, die *Einschlitztiefe* (30 mm, sofern nicht anders
eingestellt, `slot_depth`) hinter ihrer Oberfläche abzüglich seiner halben Dicke,
entlang der Mittellinien der **Installationszonen** (Verlegeregeln ›
*Installationszonen*, Befehl `installation_zones`): waagerechte Zonen über dem Fußboden
und unter der Decke, unterbrochen, wo Türen und Fenster sind, und um die Ecken des
Raums verbunden, senkrechte neben den Ecken, Türen und Fenstern. Ein Betriebsmittel an
einer Wand wird gerade von oben oder unten aus der nächsten waagerechten Zone gespeist
(oder seitlich aus einer senkrechten Zone, in der es liegt); ein freies Ende (eine
Kabelverschraubung oben auf einem Schrank) geht zuerst zur nächsten Wand. Im Fußboden
geht die Leitung gerade die Wand hinab (oder, niedrig über dem Fußboden, gerade nach
unten) und die *Tiefe im Fußboden* (60 mm, `floor_depth`) unter seiner Oberfläche
entlang der Zone im Fußboden. Die Zonen sind die der DIN 18015-3:2016, sofern nicht
geändert (*Deutsche Zonen (DIN 18015-3)* setzt sie zurück): ZW-o 150 bis 450 mm unter
der Decke, ZW-u 150 bis 450 mm über dem Fußboden, ZW-m 1000 bis 1300 mm über dem
Fußboden für Räume mit Arbeitsflächen (aus), ZS-t / ZS-f / ZS-e 100 bis 300 mm neben
Türen, Fenstern und Ecken, ZD-r 200 bis 500 mm von den Wänden in der Decke (angezeigt,
nicht durchverlegt) und ZF, dasselbe im Fußboden (die Norm hat keine Zonen im
Fußboden). Ein Weg, den die Zonen nicht verbinden, bleibt unverlegt (*kein Weg durch
die Installationszonen*). *Anzeigen › Installationszonen* zeigt die Zonen durchsichtig
an den Wänden in 3D und in den Ansichten der Wände, die im Fußboden im Grundriss;
*Anzeigen › Wände und Böden öffnen* blickt in 3D durch die Wände und Geschossdecken,
wie *Kabelkanäle öffnen*, um die Unterputzleitungen darin zu zeigen.
Schläuche, Rohre und Rohrleitungen enden an den Stutzen der erzeugten Modelle ihrer
Betriebsmittel (der Saug- und der Druckstutzen einer Pumpe, die Stutzen eines
Behälters, beide Enden eines Ventils oder einer Armatur, die Abgänge eines Verteilers):
Der Weg kommt zu einem Punkt, der doppelt so weit vor dem Stutzen liegt, wie das Rohr
dick ist (mindestens 30 mm), und geht gerade in ihn hinein. Ein Ventil, Zähler oder
Rückschlagventil in einem Rohr gilt als so herum montiert, dass seine beiden Rohre
näher an ihre Herkunft kommen.
Das Beispiel `pump_room` zeigt bodenstehende Pumpen, eine Rohrtrasse und die Wege.

In einem Schaltplan zeigt das Kontextmenü eines platzierten Betriebsmittels es im
Schaltschrankaufbau (die Montageplatte oder Tür seines Schranks, die Wand oder der
Grundriss seines Raums) oder in der 3D-Ansicht mit der Kamera nahe daran (*Im
Schaltschrankaufbau anzeigen*, *In der 3D-Ansicht anzeigen*).

Die Verlegewege sind Teil der Analyse des Projekts. Wenn ein Projekt geöffnet wird,
wird seine Analyse im Hintergrund aufgebaut: Der Editor zeigt ein Fenster mit dem
Schritt, bei dem er ist (Netzlisten, Verbindungen, Schaltschrankverdrahtung,
3D-Verlegung mit dem Weg, bei dem sie ist, Kanalfüllung, Prüfungen), und zeigt das
Projekt, sobald es fertig ist.

Der verlegte Schaltschrank ist die Eingabe der
[Verdrahtungs- und Montageassistenten](workshop.md) und der 3D-Exporte (glTF, STEP,
IGES, IFC, JT, Parasolid; siehe [Ausgabe und Datenaustausch](data-exchange.md)).

