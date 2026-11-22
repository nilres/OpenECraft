# Verdrahtungs- und Montageassistenten

Der Schaltschrankaufbau als Checkliste für die Werkstatt, am Desktop oder auf dem Tablet.

![Der Verdrahtungsassistent mit der Checkliste, dem aktuellen Draht mit seinen Daten und dem in 3D hervorgehobenen Draht](../../images/wiring-assistant.png)

## Verdrahtungsassistent

**Verdrahtungsassistent** (im Sinne von EPLAN Smart Wiring): Der Arbeitsbereich
*Verdrahtung* macht aus den verlegten Drähten des Schaltschrankaufbaus eine
touchfreundliche Checkliste für die Werkstatt – sortiert nach Betriebsmittel, Position,
Querschnitt oder Drahtnummer – mit dem aktuellen Draht in 3D hervorgehoben, seiner
Farbe, seinem Querschnitt, seiner Länge, der Abisolierlänge und der Aderendhülse oder
dem Kabelschuh jedes Endes (zwei Drähte, die sich eine Zwillingsaderendhülse teilen,
nacheinander, *zusammen mit der Leitung nach … crimpen*) sowie dem Fortschritt in Drähten und Metern. Drähte werden als verdrahtet oder als
Problem (mit einer Notiz) markiert; die Markierungen werden im Projekt gespeichert, so
dass Drähte, die im Schaltplan geändert wurden, nachdem sie verdrahtet waren, von der
Prüfung gemeldet werden. Die Web-Version öffnet den Assistenten direkt mit `…/#wiring`
(Desktop: `--wiring`), was ein Tablet in der Werkstatt zu einem günstigen
Verdrahtungsplatz macht; Checkliste mit Status als CSV.

## Montageassistent

**Montageassistent** (im Sinne von EPLAN Smart Mounting): Der Schalter *Montage* des
Arbeitsbereichs *Verdrahtung* listet die Elemente des Schaltschrankaufbaus in
Montagereihenfolge auf – Ausschnitte, Schienen, Kanäle und Sammelschienen,
Betriebsmittel, Zubehör, dann Tür und Wände – mit dem aktuellen Element in 3D
hervorgehoben, seinem Platz, seiner Größe und seinem Artikel sowie dem Fortschritt.
Elemente werden als montiert oder als Problem (mit einer Notiz) markiert; Elemente, die
verschoben wurden oder einen anderen Artikel erhalten haben, nachdem sie montiert
waren, werden von der Prüfung gemeldet. Befehle `mounting_list` und `mark_mounted`;
Checkliste mit Status als CSV.
