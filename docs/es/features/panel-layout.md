# Diseño del armario, salas y tendido 3D

### Tarjetas importadas en carriles DIN horizontales

OpenECraft alinea la pinza del dispositivo con el eje del carril. En las carcasas de E/S EL de Beckhoff de 12 × 100 × 68 mm con 8 o 16 bornes, y en el acoplador EK1100 correspondiente, el centro del carril está a 63,8 mm del borde superior. Por ejemplo, un carril centrado en Y = 100 mm sitúa el borde superior en Y = 36,2 mm. La cota de 35,4 mm es la abertura de la carcasa para el carril nominal de 35 mm. El importador conserva un desplazamiento del centro del artículo y una altura de enganche distintos de cero. Aplica el desplazamiento del centro basado en el plano solo a las carcasas correspondientes cuando el desplazamiento del centro del catálogo es cero. [Borne de 8 polos](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_8_pin.pdf) · [Borne de 16 polos](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_16_pin.pdf) · [Carcasa EK1100](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Coupler/pdf/ethercat-coupler_44mm_2xrj45_8pin.pdf)

Una tarjeta colocada o movida a menos de 4 mm de un borde vecino libre se ajusta a ras de él en la misma placa de montaje. La vista previa de la colocación muestra el resultado. El dibujo del armario usa primero la vista frontal asignada y después una representación de montaje importada. Si faltan, una vista general del fabricante aporta una ilustración ajustada dentro de la huella física.

Envolventes con sus dispositivos a tamaño real en 2D y 3D (en la línea de EPLAN Pro
Panel), cableado virtual por las canaletas, datos de taladrado y NC, climatización de
armarios, alineaciones, salas del edificio con sus equipos de pie sobre el suelo y sus
recorridos de tuberías, racks de 19 pulgadas y tendido 3D de cables, tubos y tuberías.
Los proyectos de ejemplo `conveyor`, `cabinet_lineup`, `electrical_room` y `pump_room`
los muestran.

![La alineación del ejemplo cabinet_lineup en 3D, con la puerta abierta, carriles, canaletas, dispositivos e hilos tendidos](../../images/panel-3d.png)

## Envolventes y dispositivos

- Envolventes (a partir de un artículo como `GEN.ENC-600X800` o de tamaño personalizado)
  con placa de montaje.
- *Nueva envolvente* (el botón del navegador *Envolventes y salas* o su menú
  contextual, sobre una sala *Nueva envolvente en esta sala…*) busca las envolventes de
  la base de datos de artículos, del proyecto y de los catálogos incluidos con
  OpenECraft: por texto, sistema de envolvente, fabricante, *Montaje* (pared o suelo),
  ancho, alto y fondo de … a, y las columnas y filas de los armarios de contadores y de
  distribución (*Columnas / filas*, datos de artículo `Fields: 3`, `Rows: 7`), ordenadas
  por número de artículo o tamaño. Un doble clic en una envolvente la crea; la búsqueda
  se mantiene para la siguiente. Una envolvente arrastrada desde la gestión de artículos
  (o el portal de datos) al navegador se crea al instante, en la sala sobre la que se
  soltó (también sobre un armario que esté en una).
- **Accesorios arrastrados a la envolvente:** un accesorio arrastrado desde la gestión
  de artículos sobre la vista 2D o 3D de una envolvente muestra los lugares donde
  encaja, resaltados: un panel de distribución de un armario de contadores (como el
  Striebel & John FV17, datos de artículo `Fields: 1`, `Rows: 7`) las columnas libres de
  la cuadrícula del armario, otro accesorio del artículo de la envolvente las caras en
  las que puede montarse, un accesorio del artículo de un dispositivo (un contacto
  auxiliar, un contacto de señalización) los dispositivos colocados a los que se ajusta.
  Soltado sobre un lugar va allí; sobre un dispositivo pasa a ser accesorio del
  dispositivo. El comando `fitting_places` enumera los lugares para la línea de
  comandos y los agentes de IA. Un campo de distribución trae sus carriles de montaje,
  uno por fila de sus módulos (datos de artículo `Module spaces`, `Row spacing`;
  cualquier otro campo con `Mounting rails: n`): los dispositivos se colocan en ellos y
  se mueven con el campo.
- Carriles de montaje y canaletas dibujados por longitud; seleccione uno y arrastre
  cualquiera de sus extremos para cambiar su tamaño, o muévalo (los dispositivos viajan
  con su carril). Los dispositivos se colocan desde una lista de "dispositivos por
  colocar" con sus dimensiones reales tomadas de los datos de artículos (contactor +
  bloque auxiliar se suman en profundidad, los regleteros son tan largos como sus bornes
  y reservas) y se ajustan a los carriles; girar, eliminar.
- Cortar, copiar y pegar en la vista frontal: haga clic con el botón derecho en un
  elemento para *Cortar*, *Copiar*, *Pegar*, *Propiedades* y *Quitar* (los dispositivos
  vuelven a los dispositivos por colocar), o `Ctrl+X` / `Ctrl+C` / `Ctrl+V`. Los
  carriles, canaletas, barras, recortes y accesorios se copian con los elementos montados
  sobre ellos; los dispositivos se omiten (un dispositivo se coloca una sola vez); los
  elementos cortados salen del diseño, los dispositivos cortados vuelven a los
  dispositivos por colocar. Los elementos pegados siguen al puntero hasta que un clic los
  coloca, en cualquier cara de cualquier envolvente; `Esc` o un clic con el botón derecho
  cancela. El comando `copy_panel_items` hace lo mismo desde la línea de comandos y para
  los agentes de IA.
- Vista frontal 2D y una vista 3D: arrastre para orbitar, botón derecho / central o
  Mayús+arrastrar para desplazar, la rueda amplía en el puntero, cada vez más cerca de
  la cara sólida que hay allí pero sin atravesarla (en el vacío sigue avanzando hacia el
  interior de una sala), un doble clic gira hacia lo que se ha pulsado y se acerca,
  `W` `A` `S` `D` / las teclas de flecha y `Q` `E` mueven por la vista (Mayús: más
  rápido), la barra de la vista tiene las vistas estándar y *Ajustar* (`Home`); haga clic
  para seleccionar. Las paredes, los techos y
  las puertas vistos desde fuera son transparentes (*Opacidad de las paredes* en los
  ajustes, en Apariencia); *Ocultar las paredes delante* omite las paredes situadas entre
  el ojo y el interior, *Ocultar el techo* los techos de las salas. Los hilos se dibujan
  en su color (los hilos bicolores, como el verde-amarillo, con una franja) y, como los
  cables y los tubos, con el grosor de su diámetro exterior. Un cable termina un poco
  antes del dispositivo al que está conectado y sus conductores se separan desde allí en
  curvas suaves, cada uno hacia su punto de conexión por el lado por el que sus hilos lo
  abandonan. En su punto de conexión, un hilo o un conductor termina en su terminación:
  una puntera (casquillo metálico y collar en el color de DIN 46228-4) o una puntera
  doble, un terminal de anillo (con su anillo alrededor del punto de conexión), un
  terminal de horquilla, de punta, faston macho o faston hembra (con su manguito aislado
  rojo, azul o amarillo; terminales tubulares sin aislar a partir de 10 mm²), o el
  conductor de cobre desnudo en la longitud pelada. A cada extremo de hilo se le puede
  dar su propia terminación en las propiedades del hilo (*Extremos del cable*), a todos
  los conductores de un cable en uno de sus dispositivos en las propiedades del cable
  (*Extremos del cable*), y en la línea de comandos a todos los extremos en un punto de
  conexión o en un dispositivo (`set_termination`); los extremos que no tienen ninguna
  reciben una puntera donde está activado *Extremos de hilo con punteras*, y el conductor
  desnudo en caso contrario. Una puntera doble la comparten los dos hilos que terminan en
  un mismo punto de conexión: solo se ofrece allí, se elige para ambos a la vez, se
  dimensiona para la sección mayor y se dibuja una vez, en el primero de ellos; con
  *Punteras dobles donde dos cables comparten un punto de conexión* (o `set_ferrules`)
  cada uno de esos pares sin terminación propia recibe una. La comprobación *Punteras
  dobles* advierte de una puntera doble con un solo hilo, en un punto con más de dos
  hilos o para dos secciones. Una puntera o un terminal engastado no se dobla: el hilo o
  el conductor sale recto de su punto de conexión hasta donde llega (con su collar o su
  manguito aislado) y solo se curva detrás. El tendido 3D planifica esa salida recta
  donde el espacio delante del punto de conexión está libre; donde no lo está (un hilo
  vecino, una canaleta cercana), el hilo se curva más cerrado justo detrás de la puntera
  o del terminal.
  La lista de hilos para producción, el asistente de
  cableado, las exportaciones para máquinas y la lista de materiales toman las mismas
  terminaciones; a través de prensaestopas, pasamuros y los pasos de cables
  entre armarios, los cables y los hilos se dibujan de una pieza, curvados allí, y varios
  a través de una misma brida de entrada de cables o de un mismo pasamuros, uno junto a
  otro. Los prensaestopas se dibujan con los modelos de sus artículos: por fuera el
  cuerpo hexagonal, el cuello y la cúpula sobre la superficie exterior de su pared, por
  dentro la contratuerca y el final de la rosca sobre su superficie interior, en latón
  niquelado o poliamida; los recorridos se mantienen separados de la contratuerca. En un armario sin canaletas en su placa
  de montaje, los hilos los tiende el tendido 3D (véase más abajo), de punto de conexión
  a punto de conexión.
- **Sistemas de barras** de cobre (barras dibujadas, metros y soportes en la lista de
  materiales) y **recortes** rectangulares o redondos en la placa de montaje. Un
  dispositivo colocado sobre un sistema de barras se monta sobre sus barras; los
  dispositivos del mismo sistema de barras se conectan mediante las barras, no mediante
  hilos. Los sistemas de barras de envolventes acopladas que se encuentran en su lado
  común (el mismo artículo a la misma altura, con sus extremos a no más de 100 mm) son un
  solo sistema, unido a través del lateral. Las barras se sitúan a la `Centre distance`
  del artículo (repartidas uniformemente si no hay ninguna) a lo largo del sistema, por
  corto que sea un tramo de él. Un dispositivo hecho para sistemas de barras de una
  distancia entre centros (montaje `Busbar 60 mm`) se asienta sobre barras a esa
  distancia y las alcanza todas; si no, la comprobación *Sistemas de embarrado*
  avisa. La vista 3D dibuja cada barra por separado, sobre
  soportes aislantes que llegan justo de un lado a otro de ellas, en las posiciones de
  taladrado de la placa pero junto a los dispositivos asentados en las barras (los
  transformadores que las barras atraviesan), nunca a través de ellos
  (`busbars::holder_places`). Un regletero de bornes de barras colocado transversalmente a las barras (`busbars::place_on_bars`)
  pone cada borne vertical sobre su propia barra, un regletero por cable de alimentación;
  en el tendido 3D cada uno de esos bornes es un sólido propio (un hilo hacia uno rodea a
  los demás), y a los bornes separados menos de 30 mm a lo largo de las barras se entra
  por delante.
  Cada barra
  es un conductor: la comprobación `busbars` notifica como cortocircuito un sistema cuyos
  dispositivos unen más conductores que barras tiene (dato de artículo `Bars`), como un
  borne PEN sobre las barras de las fases o las salidas situadas detrás de un interruptor
  de acoplamiento sobre la barra anterior a él. Toma el lado de alimentación de los
  interruptores y fusibles (el lado de salida cuando ese es el que comparten los demás
  dispositivos: un regletero de entrada), cada borne como un conductor, y sigue los
  conductores a través de los primarios de los transformadores de corriente.
- Páginas de distribución de montaje (vista de modelo 2D a escala con cotas) para
  impresión / SVG; las canaletas, los carriles de montaje y los sistemas de barras llevan
  una etiqueta con su sección y su longitud de corte (*60 × 100 mm, L = 940 mm*) en el
  tramo más largo que ningún dispositivo cubre, reducida para que quepa, o solo la
  longitud (*L = 430 mm*) donde no cabe la etiqueta entera. Uno totalmente ocupado la
  recibe junto a su extremo final (o, si no, junto al inicial), alineada con él, hasta el
  siguiente elemento o el borde de la placa; se omite donde ni siquiera allí sería
  legible. Los carriles, las canaletas y las barras
  aparecen en la lista de materiales por longitud total (o por piezas de almacén), con
  las piezas que hay que cortar por envolvente y longitud debajo de cada artículo (sin
  número de artículo propio, para que las exportaciones ERP cuenten el material una sola
  vez); las envolventes, por unidad. La leyenda del armario enumera, después de los
  dispositivos, cada carril, canaleta y sistema de barras con su número de artículo, su
  sección (anchura × altura) y su longitud de corte.
- Salto entre un dispositivo en el esquema y su posición en el diseño del armario.

## Comprobaciones de colisión y de límites

Solapamientos, fuera de la placa, demasiado profundo para la envolvente incluida una
holgura de puerta, montado sobre un recorte, dentro de la distancia que un dispositivo
necesita por encima y por debajo según sus datos de artículo (`Clearance: 25 mm`); cuando
un dispositivo tiene un modelo 3D, se comprueba la penetración de los **modelos**, no
solo de sus cajas.

## Puerta del armario

Los dispositivos de mando y señalización (pulsadores, lámparas …) se colocan en la puerta
(vista frontal *Puerta*); cada uno recibe sus **recortes de puerta** (Ø22,5 de forma
predeterminada o según los datos de artículo `Door cutout: Ø30.5` / `92 x 68`, o, si no,
sus recortes), colocados donde el artículo los tiene y girados con el dispositivo,
comprobados frente al borde libre de la puerta (marco, bisagras) y el frente del
dispositivo (se notifica un recorte que lo sobrepasa) y, en 3D, frente al equipamiento de
la placa de montaje; sus hilos se tienden por encima de la bisagra; la página de
distribución de montaje muestra la placa y cada cara con elementos (puerta, paredes
laterales, pared trasera, techo, fondo) una junto a otra.

La puerta lleva las bisagras a la izquierda o a la derecha (*Bisagras de la puerta* en
las propiedades de la envolvente, `door_hinge` de `add_enclosure` / `set_enclosure`, dato
de artículo `Door hinge: right` del artículo de envolvente de un fabricante): la vista de
la puerta muestra las bisagras y la maneta en sus lados, la vista 3D abre la puerta
alrededor de ese borde, y los hilos de la puerta discurren a lo largo de la puerta hasta
ese borde, por el bucle de la bisagra y hasta el borde de la placa en ese lado.

## Zonas restringidas

Un artículo de envolvente puede mantener libres zonas de sus caras (*Zonas restringidas*
en los datos de montaje del artículo, importadas con los paquetes de envolventes del
configurador de Rittal): las zonas de *colocación* no admiten dispositivos, carriles,
canaletas ni barras, las zonas de *recorte* no admiten recortes ni taladros, p. ej. las
bisagras, la cerradura y el refuerzo de la puerta, o los puntos de fijación de la placa
de montaje. Las caras las muestran rayadas (las zonas de colocación con rayado
ascendente, las de recorte descendente), en el editor y en las páginas de distribución de
montaje, y la vista 3D como zonas translúcidas (colocación en azul, recorte en naranja)
que acompañan a su pared o a la puerta. La comprobación notifica un elemento situado en
una zona de colocación (*Fuera*) y los recortes y taladros situados en una zona de
recorte (*Fabricación*); tocar el borde de una zona está permitido. El editor de
artículos las enumera en *Datos de montaje › Zonas restringidas* (tipo, cara, posición y
tamaño), donde se añaden, se modifican y se quitan.

## Modelos de envolventes de fabricante

Una envolvente cuyo artículo es de un fabricante (no uno del catálogo integrado) con un
modelo 3D de la anchura y la altura de la envolvente, como lo aportan los paquetes de
envolventes del configurador de Rittal, muestra ese modelo en la vista 3D y en las
exportaciones 3D en lugar del cuerpo, las paredes, la puerta y las placas genéricos. La
puerta y lo que va montado en ella (hojas de bisagra, cerraduras, juntas), tal como los
marca la macro del fabricante, son translúcidos y se abren con la puerta; un modelo sin
esa marca abre su frente 20 mm. Los demás triángulos acompañan a la pared en cuyo plano
se encuentran (se ocultan con ella cuando se ocultan las paredes de delante) o se quedan
dentro (placa de montaje, placa de prensaestopas, bridas). Los zócalos, los kits de
acoplamiento y los perfiles de 19 pulgadas siguen siendo genéricos. La puerta se abre por
el lado de sus bisagras (el Rittal AX lleva las bisagras a la derecha, véase
[Puerta del armario](#puerta-del-armario)).

## Patrón de taladrado y datos NC

Los carriles, las canaletas y los soportes de barras reciben taladros roscados a lo largo
de su longitud; los dispositivos atornillados a la placa reciben taladros según sus datos
de artículo (`Mounting holes: M5: 9,12; 69,72`) o dos taladros predeterminados;
**exportación NC** por envolvente y cara como DXF o DWG (contorno, una capa por diámetro
de taladro, recortes, los girados o redondeados como su contorno) o CSV (con el giro y el
radio de esquina de cada recorte).

Un carril de montaje siempre se atornilla o se remacha (*Fijación* en sus propiedades:
agujero roscado, agujero pasante con tuerca, tuerca remachable, remache ciego o
espárrago soldado; `set_panel_item fastening=rivet`): sus puntos de fijación están en los
agujeros alargados de un carril perforado, la vista 3D muestra en ellos las cabezas de
los tornillos o remaches y la lista de materiales los tornillos, tuercas o remaches
ciegos. El informe *Patrón de taladrado* lista cada taladro y cada recorte de cada cara
con su letra, sus cotas desde la esquina inferior izquierda de la cara (como los datos
NC), su tamaño, su rosca y su operación. Una página *Patrón de taladrado* de una
envolvente (*Nueva página de patrón de taladrado*,
`add_page kind=drilling enclosure=+CAB1`) dibuja sus caras con sus taladros y recortes a
una misma escala, acotados desde esa esquina, y al lado los tipos de taladros y recortes
con su número.

## Cableado virtual y grados de llenado de las canaletas

Cada conexión del esquema entre dispositivos colocados se tiende desde su punto de
conexión (dispositivos girados, bloques auxiliares, borne a borne) hasta la canaleta
hacia la que mira el borne y a través de la red de canaletas (camino más corto) → lista
de hilos con longitudes, hilos mostrados en 2D / 3D.

**Grados de llenado de las canaletas**: secciones de los hilos tendidos (con
aislamiento), de los cables y de los tubos frente a la sección interior de la canaleta
(entre sus paredes, su base y su tapa) en su tramo más lleno, tanto en la placa de
montaje como en las paredes, el techo y el suelo de una sala;
canaletas coloreadas en verde / naranja / rojo en la
vista frontal, grado de llenado en las propiedades de la canaleta, límite en las
propiedades de la envolvente (70 % de forma predeterminada); informe *Grados de llenado
de las canaletas*, la comprobación advierte por encima del límite y da una indicación por
encima del 80 % de él; cuando una canaleta está demasiado llena, el cableado se vuelve a
tender, primero los hilos más gruesos, y los hilos dan un rodeo (hasta 10 × más largo)
para evitar los tramos que llenarían por encima del límite; los cables y los tubos
tendidos en 3D hacen lo mismo alrededor de los tramos que ya llenan los hilos y los
cables anteriores; el dispositivo seleccionado muestra sus puntos de conexión.

## Climatización

IEC/TR 60890, en las propiedades de la envolvente: potencia disipada de los dispositivos
colocados a partir de los datos técnicos de sus artículos (`Power loss: 7.5 W`) más una
pérdida adicional, superficie efectiva según el tipo de instalación (individual, en
fila, exenta, mural, con el techo cubierto), material (coeficiente de transmisión
térmica), temperatura ambiente y temperatura interior admisible
→ temperatura interior, caudal de aire del ventilador con filtro (corregido según la
altitud del emplazamiento), intercambiador de calor aire/aire (W/K, cuando hay que
mantener el grado de protección) o potencia del equipo de refrigeración, potencia de la
calefacción que además mantiene el interior por encima del punto de rocío del aire
ambiente (humedad relativa); informe *Climatización de armarios*, la comprobación
advierte cuando se necesita refrigeración.

El mapa térmico de la placa de montaje estratifica la temperatura media y añade un
penacho sobre cada dispositivo con datos de potencia disipada; las canaletas horizontales
retienen los penachos (el aire caliente se acumula debajo de ellas), los ventiladores y
los equipos de refrigeración los debilitan.

![Mapa térmico de una placa de montaje junto al cálculo de climatización de la envolvente](../../images/heat-map.png)

## Alineaciones

Envolventes acopladas una junto a otra con pasos de cables entre ellas (recortes en las
paredes laterales enfrentadas, o laterales abiertos entre sí); la vista 3D muestra toda
la alineación, los hilos pasan de una envolvente a la siguiente. La vista *Sala* muestra
todas las envolventes una junto a otra en sus posiciones, cada alineación en la posición
y con el giro fijados a mano o tomados de la distribución de la planta.

## Salas

*Espacio de montaje › Nueva sala* (plugin `rooms`): una sala del edificio con su plano,
su altura y su posición en la planta. El plano es un rectángulo o un contorno de paredes
en ángulo recto (salas en forma de L, U o T): *Plano* lo muestra desde arriba, *Dibujar
paredes* lo dibuja haciendo clic en las esquinas (en una rejilla de 50 mm, cerrado al
hacer clic en la primera esquina o con Intro), las paredes se arrastran para moverlas o
reciben una longitud en las propiedades de la sala (comandos `set_room_outline`,
`set_room_wall`).

- **Paredes interiores** (*Espacio de montaje › Dibujar › Paredes interiores*, comandos
  `add_inner_wall`, `set_inner_wall`): se dibujan en el plano haciendo clic en sus
  extremos (una cadena, en ángulo recto; un extremo cercano a una pared se ajusta a
  ella), se arrastran transversalmente para moverlas, reciben longitud y grosor en las
  propiedades de la sala y se quitan con Supr; ambos lados (`inner1a`, `inner1b`) son caras como
  las paredes.
- **Puertas y ventanas** (*Espacio de montaje › Dibujar › Puerta / Ventana*, comandos
  `add_wall_opening`, `set_wall_opening`) son aberturas en una pared, colocadas haciendo
  clic en la pared en el plano o en la vista de la pared: una puerta se apoya en el
  suelo, una ventana tiene su altura de antepecho; el tamaño, la posición, el antepecho y
  la forma de abrirse están en sus propiedades. *Apertura* (`opens`) elige las bisagras
  en el borde izquierdo o derecho (visto desde dentro de la sala) con giro hacia dentro o
  hacia fuera de la sala, o una hoja corredera que se desplaza a la izquierda o a la
  derecha a lo largo de la pared. El plano dibuja la hoja con su barrido (desde la
  superficie exterior de la pared cuando abre hacia fuera) o la hoja corredera con su
  recorrido; la vista de la pared, el símbolo de apertura (su vértice en las bisagras, a
  trazos cuando la hoja se abre alejándose del observador) o una flecha. La comprobación
  mantiene libres de armarios el barrido de una puerta que abre hacia dentro y el
  recorrido de una puerta corredera por delante de la pared.
- **Los armarios** (sus alineaciones) se sitúan en ella en una posición del plano, a la
  que se arrastran en el plano (R gira el seleccionado alrededor de su centro), y se
  mueven con la sala. Un armario fuera de la sala, atravesando una pared, en una pared
  interior o en el barrido de una puerta se dibuja en el color de advertencia, y el
  puntero sobre él indica el motivo.
- Cada pared (vista desde dentro de la sala) y el techo llevan carriles de montaje,
  canaletas y dispositivos: un dispositivo en una pared va sobre un carril, salvo que su
  artículo esté marcado con *Montaje mural* (editor de artículos › Datos de montaje, un
  distintivo `WALL` en los árboles de artículos, filtro *Solo artículos para montaje
  mural directo*); entonces va directamente en la pared (*Insertar › Artículo de montaje
  mural*, también sin designación de dispositivo).
- **Tomas, interruptores y calefacción** (*Insertar › Tomas, interruptores,
  calefacción*): tomas de corriente, tomas de datos y de antena, interruptores,
  pulsadores, reguladores de luz, termostatos de ambiente, colectores de circuitos de
  calefacción y cajas de regulación individual (RTL / TH), dibujados con sus modelos en
  la vista 3D. El menú tiene una pestaña para tomas, otra para interruptores y otra para
  calefacción, una búsqueda, filtros por fabricante y tipo, la lista y el artículo
  elegido con su modelo 3D (arrastre para girarlo), su tamaño y su altura de montaje
  (*Colocar en la pared* o un doble clic lo coloca). Enumera
  los artículos genéricos del catálogo (`GEN.SOCKET-F-UP`, `GEN.SW-2WAY-UP`, …,
  colectores `GEN.HCM-2` … `GEN.HCM-12`, con el número de circuitos elegido en el menú,
  `GEN.RTL-BOX`) y los de la base de datos de artículos y de los catálogos suministrados
  `gira` (System 55), `mdt` (pulsadores KNX, pulsador de cristal, regulador de
  temperatura ambiente) y `cosmo` (colectores Standard y Balance, cajas de regulación);
  un artículo de un catálogo suministrado entra en el proyecto cuando se elige. Un
  artículo con una *Altura de montaje* en sus datos técnicos va en la pared a esa altura
  (su centro sobre el suelo: tomas de corriente y de datos 300 mm, interruptores 1050 mm,
  termostatos 1500 mm, cajas de regulación 250 mm) donde se hace clic en la pared. Los
  colectores tienen una boca para la ida y otra para el retorno y un par por circuito,
  hacia abajo, para el tendido de tuberías.
- El **suelo** (cara `floor`) lleva los artículos marcados con *Montaje en suelo*
  (bombas, depósitos, compresores; un distintivo `FLOOR`, filtro *Solo artículos de pie
  sobre el suelo*): *Insertar › Artículo de pie sobre el suelo*, dispositivos por colocar
  en la vista del suelo, comando `mount_on_wall` con `face` `floor`. Se dibujan en el
  plano, se arrastran allí para moverlos y se quitan con Supr.
- **Pasamuros** (*Espacio de montaje › Dibujar › Pasamuros*, comando
  `add_wall_opening` con `kind` `penetration`; 300 x 200 mm, 300 mm por debajo del techo
  salvo que se indique otra cosa): van en las paredes exteriores y conducen a la sala
  situada detrás de la pared (hasta a 1 m de distancia, también a través de salas giradas
  una respecto a otra): los cables, los tubos y las tuberías se tienden de sala a sala a
  través de ellos. El plano los dibuja, y el paso a través de la pared a trazos;
  `list_rooms` indica adónde conduce cada uno.
- **Entradas al edificio** (*Espacio de montaje › Dibujar › Entrada al edificio*, comando
  `add_wall_opening` con `kind` `entry`; 200 x 200 mm, 600 mm por encima del suelo salvo
  que se indique otra cosa): son las acometidas de las paredes exteriores; el cable de
  alimentación, la acometida de agua y otras líneas procedentes del exterior del edificio
  entran por ellas. Un cable o una tubería desde un dispositivo de la sala hasta un
  dispositivo situado fuera del edificio (su lugar no es ninguna de las salas ni
  envolventes del diseño, como el armario de distribución de cables de la compañía
  suministradora) se tiende hasta la entrada al edificio más cercana.
- **Recorridos de tuberías** (*Espacio de montaje › Dibujar › Recorrido de tuberías*,
  comando `add_pipe_run`): se dibujan en una pared, en el techo o en el suelo haciendo
  clic en sus esquinas (en ángulo recto), a 100 mm por delante de una pared, 300 mm por
  debajo del techo o 200 mm por encima del suelo salvo que se indique otra cosa: las
  mangueras, los tubos y las tuberías del esquema se tienden a lo largo de ellos como los
  hilos a lo largo de las canaletas. El plano los muestra a trazos.
- **El exterior del edificio:** cada pared de una sala tiene también su exterior (caras
  `outside1`, `outside2`, …, vistas desde fuera, *Exterior del muro n* en el menú de
  paredes) y el techo su parte superior (cara `rooftop`): las luminarias de fachada, los
  sensores y las cámaras van en el exterior de una pared exterior como en una pared
  interior (sobre un carril o, si están marcados con *Montaje mural*, directamente); una
  estación meteorológica u otros artículos marcados con *Montaje en suelo* se sitúan
  sobre la cubierta. Sus cables y tuberías atraviesan la pared (o el techo) justo detrás
  de ellos hacia la sala. La comprobación notifica esos elementos cuando hay otra sala
  justo detrás de la pared o encima del techo (a menos de 1 m).

La comprobación notifica los dispositivos en una pared que no son ni lo uno ni lo otro,
los carriles o dispositivos sobre una puerta o ventana, los recorridos de tuberías que
cruzan una puerta o ventana, los elementos del techo fuera del plano, las paredes
interiores que salen de la sala, los armarios y artículos de pie sobre el suelo en el
barrido de una puerta o en una pared interior, los artículos de pie sobre el suelo en el
lugar de un armario o no previstos para apoyarse en el suelo, y los armarios y artículos
de pie sobre el suelo fuera de la sala (o atravesando una pared), los armarios más altos
que su techo y los pasamuros que no conducen a ninguna sala. Las salas no tienen cálculo
de climatización ni datos NC.

**Varias salas** se sitúan en sus posiciones en la planta (*Esquina trasera izquierda de
la sala* en sus propiedades, comandos `add_room` / `set_room` con `x`, `z` y `angle`). El
plano de una sala muestra tenuemente las salas que la rodean con sus nombres y armarios
(las contiguas, no las de los niveles superior e inferior); un doble clic abre una de
ellas. *Espacio de montaje › Mostrar › Todos los armarios* muestra cada sala y cada
armario en su posición en 3D, con los recorridos entre ellos.

El navegador (*Envolventes y salas*) tiene el **emplazamiento** en su raíz, por encima de
los edificios con sus niveles y salas y de las salas situadas fuera de edificios. Un clic
en una sala mantiene la vista 3D cuando está visible y, en caso contrario, muestra el
plano de la sala. Un clic en el emplazamiento muestra el plano de situación de todos los
edificios en el Espacio de montaje (la rueda amplía, arrastrar lo desplaza, un clic en
una sala abre su plano; en el modo de edición *Parcela › Mover edificios* un edificio se
arrastra a su posición y R lo gira 90° alrededor de su centro, con sus salas, armarios,
escaleras y cubiertas y las salas situadas en él sin estar en uno de sus niveles) o, en
la vista 3D, toda
la planta con sus edificios, forjados, escaleras y cubiertas; un clic en un edificio
muestra el plano de situación con él resaltado. Un nivel pulsado mientras se muestra la
vista 3D muestra todas las salas de ese nivel una junto a otra en sus posiciones, con los
armarios situados en ellas, los forjados y las escaleras que llegan al nivel (sin
cubiertas); un segundo clic vuelve a mostrar toda la planta. El menú contextual del
emplazamiento añade un edificio, dibuja el límite de la parcela o coloca el punto de
referencia; el de un edificio, un nivel encima o un sótano; el de un nivel, una sala en
él.

## Límite de la parcela y punto de referencia

La **parcela** (*Espacio de montaje › Parcela*, el menú contextual del emplazamiento, la
sección *Parcela* del panel Edificios; plugin `buildings`) es el terreno sobre el que se
levanta la planta.

- **Límite de la parcela** (*Parcela › Límite de la parcela*, comando `set_plot`): se
  dibuja en el plano de situación esquina a esquina (un clic en la primera esquina, un
  doble clic o Intro lo cierra, Retroceso deshace la última esquina, Esc termina); las
  esquinas se ajustan a las esquinas de las salas (por dentro y por fuera de sus
  paredes), de los patinillos y al punto de referencia. Sus esquinas se arrastran después
  en el plano de situación. El límite no debe cruzarse a sí mismo; el plano de situación
  escribe su nombre y su superficie, el panel sus esquinas, su superficie y su longitud.
  **Todos los edificios tienen que quedar dentro de él**: la comprobación *Edificios,
  niveles y patinillos* notifica como errores las salas cuyas paredes exteriores
  sobrepasan el límite, y advierte de las cubiertas que lo sobrepasan con su voladizo y
  de los armarios situados fuera de él; el plano de situación los dibuja en el color de
  los errores y `plot` los enumera.
- **Punto de referencia** (*Parcela › Punto de referencia*, comando
  `set_reference_point`): un punto georreferenciado de la planta (dónde está en la
  planta, su coordenada este, su coordenada norte y su altura sobre el nivel del mar en
  un sistema de referencia de coordenadas como `EPSG:25832`, la dirección del norte de la
  cuadrícula en la planta y, si se conocen, su latitud y su longitud). Cada punto de la
  planta se mide desde él: el plano de situación muestra las coordenadas bajo el puntero
  (este y norte, o cuánto al este y al norte del punto), una flecha de norte, y
  `list_buildings` las coordenadas este y norte y la altura del origen de cada edificio;
  `locate` convierte entre la planta y el sistema de coordenadas. Las exportaciones IFC
  escriben en su emplazamiento el nombre de la parcela y la latitud, la longitud y la
  altura.
- **Medir** (*Parcela › Medir*, comando `measure`): dos puntos pulsados en el plano de
  situación dan su distancia, cuánto al este y al norte se encuentra el segundo y su
  dirección respecto al norte de la cuadrícula.

## Edificios, niveles y patinillos

*Espacio de montaje › Edificios* (plugin `buildings`) gestiona los edificios de la planta
en pestañas: *Parcela* (la parcela, los campos fotovoltaicos sobre suelo, los enlaces
entre edificios), *Edificio* (el edificio elegido con sus niveles, patinillos y
escaleras), *Cubiertas*, *Puesta a tierra*, *Plano en papel* e *Instalación eléctrica*
(ambas para un nivel). Cada pestaña dibuja un plano de situación de todas las salas (un
nivel resaltado), los patinillos, las escaleras que llegan al nivel mostrado (peldaños y
línea de huella), las cubiertas (aleros, cumbreras y limatesas a trazos) con sus ventanas
de tejado, y los enlaces.

- Un **edificio** (comando `add_building`) tiene una posición en la planta (x, z y un
  giro) y **niveles** superpuestos (`add_level`, `set_level`: nombre, altura del suelo
  sobre la cota de terreno del edificio, altura de suelo a suelo; una cota negativa es un
  sótano). Un edificio nuevo recibe un nivel `L0` a la cota del terreno y se sitúa junto
  a lo que hay en la planta; los niveles nuevos van encima (`L1`, `L2`, …) o debajo
  (`B1`, `B2`, …).
- **Las salas se sitúan en niveles** (propiedades de la sala › *Edificio*, *Nueva sala en
  este nivel* / *Colocar una sala aquí* del panel, comandos `place_on_level`, `add_room`
  con `building` y `level`): la posición de la sala se mide entonces en el plano del
  edificio, la sala se apoya en el suelo del nivel y se mueve con el edificio y su nivel.
  Una sala nueva en un nivel es tan alta como el nivel menos su forjado de techo (300 mm)
  y se sitúa junto a las demás salas del nivel. El navegador agrupa las salas por
  edificio y nivel: una sala arrastrada a un nivel (o a una sala de él) se coloca en ese
  nivel; un nivel pulsado en él queda seleccionado, y *Nueva sala* coloca entonces la
  sala nueva en él (un segundo clic lo deselecciona); el menú contextual de un nivel y de
  sus salas tiene *Nueva sala en este nivel*, el de una sala de un edificio *Sacar del
  edificio*. Al quitar un nivel o un edificio, sus salas se quedan donde están en la
  planta.
- **Los patinillos** (`add_shaft`, `set_shaft`) atraviesan un edificio desde un nivel
  hasta otro superior, en una posición de su plano: los cables, los tubos y las tuberías
  se tienden a través de ellos desde una sala hacia las salas superiores e inferiores
  cuyos planos atraviesa el patinillo, desde debajo del techo de una hasta encima del
  suelo de la siguiente. El plano de una sala muestra los patinillos que atraviesan su
  nivel.
- **Las escaleras** (*Nueva escalera* en el panel, `add_stair`, `set_stair`) llevan desde
  el suelo de un nivel hasta el suelo de otro: *recta*, *girada una vez* en un
  descansillo de esquina, *girada dos veces* en dos descansillos de esquina, con un
  *descansillo intermedio* (dos tramos uno junto a otro) o *de caracol* alrededor de un
  eje, girando a la izquierda o a la derecha. Su arranque es el centro del borde
  delantero del peldaño más bajo en el plano del edificio, con la dirección del primer
  tramo, la anchura de los tramos y la huella de un peldaño; los peldaños resultan de la
  diferencia de altura (contrahuellas cercanas a 175 mm). Los planos de las salas
  muestran las escaleras de su nivel (a trazos donde suben desde abajo), la vista 3D sus
  peldaños y descansillos y el hueco en el forjado superior.
- **Las cubiertas** (*Nueva cubierta* en el panel, `add_roof`, `set_roof`): *plana*, *a
  un agua*, *a dos aguas*, *a cuatro aguas* o *piramidal* sobre un rectángulo del plano
  del edificio (una nueva cubre las salas del edificio), con la altura de su alero, la
  pendiente, el voladizo y el grosor; una cubierta a dos aguas o a un agua discurre a lo
  largo de x o de z. La cara inferior de la cubierta empieza en el alero sobre las
  paredes y sube con la pendiente. Donde queda más baja que el techo de una sala situada
  debajo, también en varios niveles a la vez (alero a la altura de un murete sobre el
  suelo del ático, o por debajo del techo del nivel inferior), se convierte en el techo
  de esa sala: la sala tiene alturas distintas en lugares distintos. Sus paredes terminan
  bajo el faldón en la vista 3D, la vista de una pared dibuja la línea donde termina, el
  plano sombrea las partes situadas bajo el faldón con su altura mínima, y el panel y
  `list_buildings` dan el techo más bajo y el más alto de cada una de esas salas. Las
  **ventanas de tejado** (`add_roof_window`, `set_roof_window`; 780 × 1180 mm salvo que
  se indique otra cosa) se sitúan en un faldón según su centro en el plano, con su
  anchura a lo largo del alero y su longitud faldón arriba; se ven desde fuera y desde la
  sala inferior.
- **Campos fotovoltaicos** (*Campo fotovoltaico* bajo una cubierta en el panel,
  `add_pv_array`, `set_pv_array`): filas de módulos solares sobre cualquier forma de
  cubierta. En un faldón se sitúan **paralelos** a la cobertura sobre carriles fijados
  con ganchos de tejado: dos carriles bajo cada fila de módulos en vertical, bajo cada
  columna en horizontal, con los ganchos separados como máximo la distancia entre ganchos
  (1200 mm salvo que se indique otra cosa). En una cubierta plana se colocan
  **inclinados** hacia un lado (15° salvo que se indique otra cosa, orientados en la
  dirección indicada) o **este-oeste** por parejas sobre bases de montaje lastradas, con
  las filas tan separadas que no se dan sombra unas a otras con un sol de invierno de 17°
  (o a una distancia dada). Un campo se sitúa en el faldón que hay bajo su centro; su
  artículo de módulo (los genéricos `GEN.PV-430W` y `GEN.PV-400W-FB`, o módulos de los
  catálogos suministrados `longi` y `jinko`) da el tamaño de los módulos y su potencia
  pico. El material de montaje procede de su **sistema de montaje**: un carril o una base
  de montaje de la base de datos de artículos cuyos accesorios son sus ganchos de tejado,
  uniones de carril, grapas finales e intermedias (las correspondientes a la altura del
  marco del módulo), lastre y deflectores de viento; el sistema genérico
  `GEN.PV-RAIL-40` / `GEN.PV-BASE-S15` / `GEN.PV-BASE-EW10` completa lo que le falte a un
  sistema. Los módulos (salvo que un dispositivo, un generador fotovoltaico, los cuente)
  y el material de montaje pasan a la lista de materiales; `list_buildings` da los
  módulos, la potencia pico y el material de cada campo. El plano de situación dibuja los
  módulos; la vista 3D, los módulos con sus marcos y cajas de conexión, los carriles
  sobre sus ganchos de tejado, las bases de montaje y los deflectores de viento.
  *Mostrar › Ocultar los módulos fotovoltaicos* omite los módulos para mostrar el
  montaje, *Mostrar › Módulos fotovoltaicos transparentes* los muestra transparentes con
  el cableado bajo ellos. El montaje
  tiene un **punto de puesta a tierra** con una grapa de puesta a tierra
  (`GEN.PV-EARTH-CLAMP`, en la lista de materiales) en cada carril o fila de bases de
  montaje; su **conductor de puesta a tierra** (`earthing`: el dispositivo de una barra
  de puesta a tierra o de conexión equipotencial) los une y llega hasta esa barra. La
  vista 3D dibuja el **cableado** de cada string: los cables
  de cada módulo: su cable positivo (rojo) y su cable negativo (negro) que salen de su
  caja de conexión, el cable positivo de un módulo que se une al cable negativo del
  siguiente en un conector bajo los módulos, y el cable negativo del primer módulo y el
  cable positivo del último a lo largo de la cubierta, a través de ella junto al campo y,
  por debajo, hacia abajo hasta la
  **caja de conexión del generador** del campo (`combiner`, un dispositivo como
  `GEN.GAK-2S`) y desde allí un cable principal de CC (positivo y negativo) hasta cada inversor (sin caja de conexión del generador,
  hacia abajo hasta el inversor), y el conductor de puesta a tierra (verde-amarillo)
  hacia abajo hasta su barra, en la medida en que los dispositivos estén colocados en el diseño. Un clic en un campo en la vista 3D abre sus
  **propiedades**: dónde está, sus módulos con sus datos eléctricos, potencia pico y
  superficie, su montaje con el material y la puesta a tierra, sus strings con sus
  tensiones, corrientes y la longitud de sus cables, sus problemas y los ajustes del
  panel Edificios. Una sala bajo la cubierta mostrada sola muestra también desde fuera
  los campos situados sobre ella y las ventanas de tejado. Un **clic en una cubierta** en
  la vista 3D muestra la cubierta en las *Propiedades del montaje*: su forma, su alero,
  su pendiente, sus ventanas de tejado y sus campos fotovoltaicos; una ventana de tejado
  o un campo fotovoltaico añadido allí se sitúa donde se hizo clic en la cubierta, y el
  campo nuevo queda seleccionado con sus propiedades.
- **Campos fotovoltaicos sobre suelo** (*Campos fotovoltaicos sobre suelo* en el panel:
  *Campo fotovoltaico* delante de lo que hay en la planta, *Colocar en el plano de
  situación* donde se haga clic; `add_pv_array` y `set_pv_array` sin `building` ni
  `roof`): campos montados sobre el terreno en la parcela, con su centro en la planta. Se
  colocan en filas de **mesas** orientadas en la dirección indicada (25° salvo que se
  indique otra cosa): `stack` módulos uno encima de otro en una mesa (2), el borde
  inferior a `clearance` sobre el terreno (800 mm), dos carriles bajo cada fila de
  módulos sobre vigas de mesa, cada viga sobre un poste hincado delantero y otro trasero
  separados como máximo la distancia entre postes a lo largo de la mesa (`span`, 2,5 m),
  con las mesas tan separadas que no se dan sombra unas a otras con un sol de invierno de
  17° (o a la distancia entre filas indicada). El sistema genérico es el poste
  `GEN.PV-POST-RAM` con la viga de mesa `GEN.PV-TABLE-BEAM`, los carriles, las uniones de
  carril, las grapas de módulo y la grapa de puesta a tierra (en el primer poste
  delantero de cada mesa) como accesorios; un sistema de montaje de la base de datos de
  artículos (subgrupo *Poste de suelo fotovoltaico*) aporta los suyos. Los módulos y el
  montaje pasan a la lista de materiales con el nombre de la parcela; el plano de
  situación y la vista 3D de todo el emplazamiento los dibujan, y un clic en uno de ellos
  en cualquiera de los dos abre sus propiedades. Sus strings son como los de los campos
  de las cubiertas; sus cables (y el conductor de puesta a tierra) discurren por la parte
  trasera de los módulos, bajan a una zanja de cables de 0,7 m de profundidad y suben
  hasta la caja de conexión del generador, el inversor o la barra. `plot` y
  `list_buildings` los enumeran con la parcela; las comprobaciones notifican los campos
  sobre suelo situados unos sobre otros, sobre una sala o un armario, y fuera del límite
  de la parcela.
- **Strings fotovoltaicos** (*String* bajo un campo fotovoltaico en el panel,
  `add_pv_string`, `set_pv_string`, `pv_strings`): los módulos de un campo en serie como
  strings, varios strings del mismo tipo en paralelo, cada uno en un seguidor MPP de un
  inversor o regulador de carga (un dispositivo cuyo artículo tiene datos de entrada
  fotovoltaica: `Max. PV input voltage`, `MPPT voltage range`, `MPP trackers`,
  `Max. input current per MPPT`, `Max. short-circuit current per MPPT`; los inversores
  genéricos `GEN.INV-PV-10KW` y `GEN.INV-PV-40KW`, el Victron Inverter RS Smart Solar y
  el SmartSolar MPPT los tienen). De los valores de la hoja de datos del módulo
  (`Open-circuit voltage`, `Short-circuit current`, `Voltage` / `Current at maximum
  power`, los coeficientes de temperatura de Voc, Isc y Pmax) y de las temperaturas de
  célula del diseño (-10 / 25 / 70 °C salvo que se indique otra cosa) resultan la
  **tensión máxima** (circuito abierto a la temperatura más baja), la **tensión media** y
  la **tensión mínima** (a máxima potencia a la temperatura media y a la más alta, con el
  coeficiente de la potencia), la corriente a máxima potencia, la corriente de
  cortocircuito a la temperatura más alta y la potencia. El informe *Strings
  fotovoltaicos* enumera cada string con estos valores, la potencia total y las
  temperaturas. La comprobación notifica los strings cuya tensión máxima supera la mayor
  tensión de entrada de su inversor o la tensión de sistema de sus módulos, los strings
  por debajo o por encima del rango MPP de su seguidor, los seguidores que el inversor no
  tiene, los seguidores cuyos strings superan en conjunto su corriente de cortocircuito o
  de entrada o difieren en tensión, y los campos con más módulos en strings de los que
  tienen o con módulos que no están en ningún string.
- **Electrodos de tierra** (*Puesta a tierra y protección contra el rayo* en el panel,
  `add_earth_electrode`, `set_earth_electrode`): un **electrodo de cimentación** en la
  losa de cimentación del nivel más bajo, un anillo cerrado a 100 mm por dentro de las
  paredes exteriores dividido por pletinas transversales en mallas de 20 × 20 m como
  máximo (`GEN.ER-STRIP-FE30`, sus separadores cada 2 m y conectores en cruz), un
  **anillo de tierra** en el terreno a un metro por fuera de las paredes y a medio metro
  de profundidad (`GEN.ER-STRIP-V4A30`), y **picas de tierra** en una posición del plano,
  hincadas desde medio metro de profundidad (tramos `GEN.ER-ROD-1500`, punta y grapa).
  Cada electrodo tiene un **material** (acero galvanizado en caliente, acero inoxidable
  V4A, cobre, acero cobreado) y un **diámetro** (de una pica, o de un conductor redondo
  en lugar de una pletina); su artículo es el de la base de datos de artículos o del
  catálogo con ese material y ese diámetro (según los datos técnicos `Material` y
  `Diameter`) salvo que se elija uno. Las paredes exteriores son el rectángulo que cubren las cubiertas o,
  si no, el de las salas. Un electrodo conectado a una barra principal de puesta a tierra
  (un dispositivo) sube hasta ella con una pletina de conexión. Todo ello pasa a la lista
  de materiales.
- **Protección contra el rayo** (`set_lightning_protection`, `add_air_rod`,
  `set_air_rod`): de clase I a IV, un **sistema de captación** de conductores en las
  cumbreras y limatesas de las cubiertas inclinadas y alrededor del borde y como malla
  (5, 10, 15 o 20 m según la clase) en las cubiertas planas y a un agua, y **puntas
  captadoras** unidas a ellos; **bajantes** alrededor de las paredes exteriores separados
  como máximo 10, 10, 15 o 20 m (al menos dos, o tantos como se indique), cada uno desde
  debajo del alero hasta su **punto de seccionamiento** a un metro sobre el terreno y,
  ya en el terreno, hasta el anillo de tierra (o, si no, el electrodo de cimentación o
  las picas de tierra), y el conductor de cubierta desde el alero por la cubierta hasta
  el sistema de captación. El **material** y el **diámetro** de los conductores y de cada
  punta eligen sus artículos; los **herrajes de montaje** son los soportes de conductor
  en la cubierta y en las paredes (soportes de tejado, de cumbrera o de cubierta plana,
  soportes de pared con o sin distanciador) a una distancia propia; una punta se sitúa en
  un soporte de punta o sobre una base de hormigón; también se fija la altura de los
  puntos de seccionamiento. El conductor redondo, sus soportes, las puntas con sus
  soportes o bases, los puntos de seccionamiento, las entradas a tierra y las grapas
  pasan a la lista de materiales. Un clic en un electrodo, un conductor o una punta en la
  vista 3D o en el plano de situación abre sus **propiedades** con todos estos ajustes.
  Los campos fotovoltaicos de las cubiertas tienen que quedar dentro de su protección
  según el **ángulo de protección** de la clase (70 - 79° hasta 2 m sobre la superficie,
  bajando hasta 23° en el radio de la esfera rodante). La vista 3D y el plano de
  situación dibujan los electrodos y el sistema. *Parcela › Puesta a tierra y protección
  contra el rayo* los añade a un edificio: electrodos de cimentación y anillos de tierra,
  picas de tierra y puntas captadoras colocadas con un clic en el plano de situación, y
  protección contra el rayo con su clase.
- **Enlaces entre edificios** (`add_site_link`, `set_site_link`): una zanja de cables, un
  conducto o un puente de cables desde una entrada al edificio de una sala hasta una
  entrada al edificio de una sala de otro edificio, pasando por esquinas en la planta.
  Los recorridos los atraviesan como un pasamuros; su longitud se indica o resulta del
  camino (bajando hasta 800 mm por debajo del terreno para una zanja o un conducto, a la
  altura de la entrada más alta para un puente).

La comprobación *Edificios, niveles y patinillos* notifica las salas en niveles que ya no
existen, las salas más altas que su nivel o solapadas en un nivel, los niveles que se
solapan entre sí, los patinillos que no unen dos salas, los armarios y artículos de pie
sobre el suelo situados en un patinillo, los enlaces cuyas entradas al edificio ya no
existen, las escaleras sin niveles válidos, con una contrahuella fuera de 140 - 200 mm o
que incumplen la regla del paso (dos veces la contrahuella más la huella, 590 - 650 mm),
las ventanas de tejado fuera de su faldón o sobre una cumbrera o limatesa, las salas en
las que la cubierta baja hasta el suelo, los armarios más altos que el techo inclinado en
su posición y los elementos de las paredes o del techo situados por encima del faldón,
los campos fotovoltaicos fuera de su cubierta o sobre una cumbrera, una limatesa o su
borde, inclinados en una cubierta inclinada, sobre una ventana de tejado o unos sobre
otros, los campos fotovoltaicos sobre suelo situados unos sobre otros, sobre una sala o
fuera del límite de la parcela, y los campos fotovoltaicos
cuyo dispositivo falta o cuenta otro número de módulos o cuya barra de puesta a tierra
falta, los electrodos de tierra cuya barra falta, las picas de tierra dentro de las
paredes, la protección contra el rayo sin electrodo de tierra o con menos bajantes de los
que necesita su clase, las puntas captadoras fuera de las cubiertas y los campos
fotovoltaicos fuera de la protección del sistema de captación. El ejemplo 15
(`plant_buildings`) muestra un edificio principal con dos niveles y un patinillo, una
caseta de bombas enlazada por una zanja de cables por la que se tiende un cable de motor,
luminarias de fachada y una estación meteorológica en la cubierta; el ejemplo 16
(`office_building`) añade un edificio de oficinas con un ático bajo una cubierta a dos
aguas, una escalera girada en un descansillo de esquina, ventanas de tejado, un campo
fotovoltaico de diez módulos LONGi sobre carriles y ganchos de tejado en el faldón
trasero con su inversor y la barra equipotencial de su montaje en la pared del hastial
del ático, un electrodo de cimentación en la barra principal de puesta a tierra, un
anillo de tierra y un sistema de protección contra el rayo de clase III con una punta
captadora, tomas e interruptores de Gira y MDT y un colector de circuitos de calefacción
COSMO.

**En 3D**, *Espacio de montaje › Mostrar › Edificio completo* muestra el edificio de la
sala mostrada con todos sus niveles y salas, sus forjados, las escaleras y las cubiertas;
*Todos los armarios* muestra así todos los edificios de la planta. Las cubiertas y los
hastiales se ven desde fuera como las paredes (translúcidos, u omitidos con *Ocultar las
paredes delante*); *Ocultar el techo* omite los techos y las cubiertas para mirar dentro
del edificio. La planta en 3D dibuja el límite de la parcela sobre el terreno y el punto
de referencia como un poste.

![Plano del ejemplo electrical_room: sala en forma de L, pared interior, puerta, ventana y los dos armarios](../../images/room.png)

## Planos en papel

Los planos de la instalación eléctrica se dibujan sobre el plano del arquitecto del
edificio. Un **plano en papel** se coloca bajo un nivel (panel *Edificios*, pestaña
*Plano en papel*, elija el nivel del plano de situación, *Plano…*; línea de comandos
`import_floor_plan`): una imagen (PNG, JPEG,
TIFF, GIF, WebP, BMP) o una página de un archivo PDF, un escaneado (también los de los
escáneres de oficina, que dividen una página en una imagen de fondo y máscaras de fax en
blanco y negro) o un dibujo (sus líneas y áreas rellenas; los textos no se dibujan). De
un archivo PDF con varias páginas, la página se elige en el panel. El plano de situación
lo muestra bajo las salas del nivel, y el plano de cada sala del nivel lo muestra bajo la
sala en su posición (*Mostrar › Plano en papel* en la cinta del espacio de montaje,
activado de forma predeterminada; el suelo de la sala se dibuja entonces transparente);
la imagen se conserva en el proyecto como PNG en grises (el crate `openecraft-scan` lee los
archivos, `openecraft_core::panel::floorplan` contiene el resto).

- **Escala y posición** (*Escala*, *Resolución*, *Esquina superior izquierda*, *Giro*,
  *Mostrado*; comando `set_floor_plan`): la escala del dibujo (1:100 para uno nuevo) y la
  resolución del escaneado (de la página PDF, 150 ppp para una imagen) dan el tamaño de
  un píxel en el edificio. *Medir* (comando `calibrate_floor_plan`) fija la escala a
  partir de dos puntos pulsados en el plano de situación y su distancia, una cota del
  dibujo. La esquina superior izquierda de la imagen se sitúa en un punto del plano del
  edificio, con la imagen girada alrededor de él como una sala.
- **Zona** (*Elegir*, `area` de `set_floor_plan`): la parte de la imagen en la que se
  buscan las salas, sin cajetín, secciones ni los demás dibujos de la hoja; se muestra a
  trazos.
- **Detectar salas** (comando `detect_floor_plan`) detecta las paredes, puertas, ventanas
  y salas en el plano y las muestra en el plano de situación: las salas con su contorno,
  las puertas en el color de las advertencias, las ventanas en azul, el barrido de una
  puerta como un punto en su bisagra. Todavía no cambia nada.
- **Crear salas** (comando `trace_floor_plan`) crea en el nivel una sala por cada sala
  detectada (`+ROOM1`, …): su contorno son las caras interiores de sus paredes, tan alta
  como el nivel menos su forjado, en su posición en el edificio; una puerta va en la sala
  hacia la que abre (con su barrido dibujado) o en la menor de las dos salas, con las
  bisagras en el lado dibujado; una ventana, en la sala situada detrás de ella. Las salas
  cuyo contorno no es un contorno de paredes en ángulo recto se notifican y se omiten. El
  plano permanece bajo el nivel para comprobar las salas frente a él y para dibujar
  encima; *Quitar* lo retira, las salas se quedan.

**Cómo se detectan las salas** (`openecraft_core::wallscan`, puro procesamiento de
imágenes, sin modelo): paredes dibujadas rellenas, sombreadas o rayadas (como se dibujan
las paredes seccionadas) en ángulo recto respecto a los bordes de la hoja.

1. *Tinta*: un píxel claramente más oscuro (10 niveles) que el papel que lo rodea (el
   nivel más claro dentro del grosor de la pared más gruesa, medido sobre la hoja
   reducida; los escaneados se decoloran y amarillean de forma desigual), tras una
   mediana de 3 × 3 que elimina las motas.
2. *Paredes*: la tinta abierta morfológicamente con la pared más delgada (90 mm): el
   texto, las líneas de cota, los barridos de puerta y las líneas de las ventanas
   desaparecen, las paredes rellenas se quedan. De lo que queda, solo las partes grandes
   alejadas de los bordes de la hoja son paredes.
3. *Tramos rectos*: series transversales a una pared (una columna de una pared a lo largo
   de x) tan delgadas como una pared, que mantienen su posición a lo largo de al menos el
   doble de su grosor; las series que sobrepasan un tramo (una pared que se encuentra con
   él, una mancha de lápiz en su lado) lo prolongan.
4. *Aberturas*: desde el extremo libre de un tramo (no donde otro tramo se encuentra con
   él) un rayo a lo largo de él, en las filas de su centro, hasta la siguiente pared: tan
   largo como una puerta o una ventana (0,5 … 5 m) en la mayoría de las filas. Si termina
   en una pared transversal (una puerta junto a una esquina) mide como máximo 1,5 m; más
   ancho solo hasta el siguiente tramo de la misma pared (una ventana entre dos tramos).
   Una abertura cuyos dos lados están unidos de todos modos (una mancha en una sala) no
   lo es.
5. *Salas*: las áreas libres encerradas por las paredes con las aberturas cerradas (las
   que llegan al borde de la hoja están fuera), de al menos 1 m² (*Sala más pequeña*,
   parámetro de comando `min_room`, que se conserva con el plano: un aseo o un armario empotrado son más pequeños). Su contorno: se
   corta a lo largo de las líneas en las que el borde discurre durante al menos 150 mm, y
   las celdas de esos cortes que quedan mayoritariamente dentro de la sala forman el
   plano (paredes en ángulo recto, ningún escalón de menos de 75 mm).
6. *Puertas y ventanas*: una abertura entre dos salas es una puerta; una hacia el
   exterior es una ventana cuando hay líneas que la recorren a lo largo de la pared
   (vidrio, marco, alféizar), y si no, una puerta. Un cuarto de círculo de tinta desde un
   extremo de una puerta hacia una sala es su barrido.

En los planos escaneados de una casa de 1972 (1:50, 150 ppp, lápiz) detecta las salas de
la planta baja con sus puertas y ventanas en medio segundo; las paredes dibujadas como
dos líneas sin relleno, las paredes inclinadas o curvas y las hojas escaneadas torcidas
todavía no se detectan. Las salas detectadas son un punto de partida que hay que
comprobar y corregir: las salas unidas por una abertura sin puerta dibujada son una sola
sala, y las escaleras o las chimeneas son paredes.

## Instalación: tomas, interruptores, luminarias y sus circuitos

El plano de instalación de un nivel se dibuja en el panel *Edificios* (pestaña
*Instalación eléctrica*, elija el nivel del plano de situación) sobre su plano; plugin
`installation`, `openecraft_core::installation`. Todo lo que contiene está en el esquema:
los dispositivos, sus conexiones y sus cables; los circuitos se leen de allí, y las
longitudes de sus cables son las del tendido 3D.

- **Insertar dispositivos** (*Insertar* con un dispositivo de protección y un artículo
  elegidos; comando `place_installation`): un clic en una sala del plano de situación
  coloca allí una toma de corriente, un interruptor, una luminaria o un elemento
  calefactor para el dispositivo de protección (un interruptor automático, un
  interruptor combinado o un fusible de la red de hasta 32 A en el esquema; la lista
  marca los de reserva, comando `list_breakers`): una luminaria de techo
  (`GEN.LAMP-CEILING-LED`, `GEN.LAMP-DOWNLIGHT-LED`, artículos con *Montaje en techo:
  sí*) en el techo donde se hace clic, y cualquier otra cosa en la pared más cercana al
  clic (a menos de 0,8 m) a su altura de montaje (tomas 300 mm, interruptores 1050 mm,
  luminarias de pared 2000 mm). Se convierte en un dispositivo del esquema en su sala
  (`+ROOM1-XD1` tomas de corriente, `-SF1` interruptores, `-EA1` luminarias, `-EB1`
  elementos calefactores) con el artículo, dibujado en la **página de instalación** del
  dispositivo de protección: una página de esquema situada después de la página del
  dispositivo de protección, creada con su primer dispositivo, cuyo punto de conexión de
  salida recibe el punto de interrupción `L-F5` hacia ella (uno tripolar, `L1-F5` …
  `L3-F5` en sus tres puntos de conexión de salida). La página dibuja los conductores de
  fase desde esos puntos de interrupción y los conductores neutro y de protección desde
  los potenciales `N-F5` y `PE-F5` como líneas a lo largo de su parte superior, y cada
  dispositivo en una columna debajo de ellas.
  Cada dispositivo recibe un cable `-W<n>` desde el dispositivo anterior (el primero
  desde el dispositivo de protección, una luminaria desde el primer interruptor de su
  sala o desde la última luminaria situada detrás de él), o desde el dispositivo o la
  caja de derivación elegidos (*Cable desde*; parámetro `from`): un circuito se ramifica
  allí. Sus hilos son los conductores (BN, BU, GNYE; en trifásico BN, BK, GY, BU, GNYE)
  de `NYM-J 3x1.5` (`5x`), o más gruesos donde el dispositivo de protección o un
  dispositivo del circuito lo exige (intensidad admisible en las condiciones de
  instalación). El cable conserva su origen y su destino (`cable.source`,
  `cable.target`): las conexiones del circuito, los tramos de la lista y los extremos de
  los recorridos 3D los siguen, y las columnas de la página siguen el árbol de los
  cables. En una red cuyos hilos pertenecen a varios cables, cada conexión toma el cable
  de los hilos situados en sus propios puntos de conexión.
- **Circuitos trifásicos**: un dispositivo de protección tripolar alimenta tomas CEE
  (`GEN.SOCKET-CEE-16-5P`, `-32-5P`), cajas de conexión para cocinas
  (`GEN.CONN-BOX-COOKER-UP`) y elementos calefactores trifásicos
  (`GEN.HEATER-SAUNA-9000`); un dispositivo con L2 o L3 se rechaza en un circuito
  monofásico.
- **Cajas de derivación y cajas de conexión de aparatos** (`GEN.JBOX-UP`,
  `GEN.JBOX-5-UP`, `GEN.CONN-BOX-UP`; pestaña *Cajas*): una caja de derivación es un
  dispositivo del circuito del que salen los cables de otros; una caja de conexión de
  aparatos alimenta equipos fijos. Una caja de derivación no es una carga en el
  dimensionado de cables: un cable que termina en ella lleva la mayor carga de los cables
  que salen de allí (según su origen y su destino, que `place_installation` guarda; un
  cable dibujado a mano no transmite ninguna), calculada en monofásico para una carga
  monofásica situada detrás de una línea de alimentación de cinco conductores.
- **Circuitos conmutados**: un conmutador (`GEN.SW-2WAY-UP`) colocado `from` otro se
  dibuja en vertical en la columna siguiente, con los dos viajeros de su cable del uno al
  otro; para más puntos de mando se intercalan conmutadores de cruce
  (`GEN.SW-INTER-UP`), cada uno `from` el interruptor anterior, dibujado en su página (se
  rechaza donde esa página está llena). Una luminaria sin `from` cuelga del último
  interruptor, que también asume las luminarias colocadas en los interruptores
  anteriores; los demás dispositivos alimentados desde ellos se quedan. Un conmutador
  situado después de una pareja cerrada (dos conmutadores) inicia un circuito propio,
  también con `from` uno de la pareja. Los cables entre los
  interruptores tienen cuatro conductores: los dos viajeros, y el conductor neutro y el
  de protección que se llevan hasta las luminarias. Un conmutador que trabaja solo tiene
  su viajero sobrante en *Puntos de conexión no utilizados* (`function.unused_pins`): un
  punto de conexión nombrado allí no se notifica como abierto. Los viajeros de un
  conmutador de cruce sin un último interruptor quedan abiertos. Un interruptor de
  persianas (`GEN.SW-BLIND-UP`) tiene dos salidas reales y no es un conmutador.
- **Sin circuito**: las tomas de datos y de antena (`GEN.DATA-RJ45-2X-UP`,
  `GEN.TV-SAT-UP`) y los detectores de humo (`GEN.SMOKE-ALARM`, en el techo) solo se
  colocan en las salas (`place_installation` sin `breaker`; con `tag` como ese
  dispositivo), se muestran en el plano de situación y se enumeran con los elementos de
  las salas.
- **Fuera del edificio**: un punto fuera de la sala coloca el artículo en el exterior de
  su pared más cercana a su altura de montaje (una toma de exterior, una luminaria de
  fachada); los dispositivos
  del jardín van en
  una zona exterior, una sala colocada fuera del edificio (`place_on_level` sin
  edificio).
- **Qué más admite `place_installation`**: la designación de dispositivo (`tag`), la
  altura de montaje (`height`, mm), la designación, el tipo, el artículo, el número de
  conductores y la sección del cable y una longitud conocida (`cable`, `cable_type`,
  `cable_part`, `cores`, `cross_section`, `length`); una longitud conocida se conserva
  (*Longitud conocida*, `cable.fixed_length`) y se toma en lugar del recorrido 3D.
  `place_cabinet_device` coloca los dispositivos de protección en el cuadro de
  distribución, que los circuitos necesitan para sus longitudes.
- **En la vista 3D y en las vistas de pared** (*Insertar › Tomas, interruptores,
  calefacción*): el menú elige el dispositivo de protección en su parte superior (o
  *Ninguno: no al esquema*); el artículo elegido se coloca con un clic en una pared o en
  el techo de la sala en la vista 3D (en una pared, a su altura de montaje; Esc termina),
  o en la pared en una vista de pared, y pasa al esquema como con *Insertar*. La lista
  del menú es tan alta como sus artículos, al menos como el artículo mostrado junto a
  ella y como máximo un poco más de la mitad de la ventana.
- **El plano de situación** dibuja los dispositivos con los símbolos de los planos de
  instalación (IEC 60617-11) en el color de su circuito y las conexiones de los
  circuitos; en 3D son los modelos de los artículos (generados para los artículos
  genéricos y los artículos sin modelo propio: tomas e interruptores en sus marcos,
  luminarias de techo redondas con su difusor abovedado, downlights empotrados con aro y
  difusor, luminarias rectangulares de pared, de panel y de regleta, luminarias de
  emergencia con su rótulo, calefactores de panel con rejilla y termostato, radiadores
  toalleros con sus barras, paneles calefactores por infrarrojos). El menú 3D
  *Insertar › Tomas, interruptores, calefacción* tiene también una pestaña *Luminarias*.
- **Los elementos calefactores** se insertan del mismo modo: calefactores de panel
  (`GEN.HEATER-PANEL-1000`, `GEN.HEATER-PANEL-2000`, 400 mm) y radiadores toalleros
  (`GEN.HEATER-TOWEL-500`, 900 mm) en la pared, paneles calefactores por infrarrojos
  (`GEN.HEATER-IR-600`) en el techo; artículos de los subgrupos *Calefactor de sala*,
  *Radiador toallero*, *Panel calefactor por infrarrojos* o con una función de elemento
  calefactor para paredes o techos (las resistencias calefactoras de los armarios no lo
  son) (*Insertar › Tomas, interruptores, calefacción*, pestaña *Calefacción*).
- **Circuitos del esquema** (solo lectura; comando `list_circuits`, informe
  *Circuitos de instalación*): cada dispositivo de protección de la red de hasta 32 A con
  las tomas de corriente, los interruptores, las luminarias y los elementos calefactores
  situados detrás de él: en las redes situadas detrás del dispositivo de protección, a
  través de interruptores, contactos, bornes y los contactos principales de los
  contactores, no a través de otros dispositivos de protección o cargas. Un dispositivo
  cuenta por su artículo (uno para paredes o techos) o, sin artículos, por su símbolo
  (tomas de corriente, elementos calefactores, lámparas, luminarias, símbolos de
  instalación; contactos de un dispositivo cuya letra identificativa empieza por `S`);
  los dispositivos colocados en un armario se omiten. La lista muestra cuántos
  dispositivos de cada uno están colocados, sus cables, la longitud y las caídas de
  tensión; un clic en un circuito elige su dispositivo de protección.
- **Dispositivos del esquema colocados** (*Colocar todos* o un clic en un dispositivo del
  circuito elegido; comando `place_circuit_device`): los dispositivos dibujados a mano en
  el esquema se colocan con un clic en una sala como un artículo (con su propio artículo,
  con el artículo elegido si es de su tipo, o con el genérico de su tipo, que pasa a ser
  su artículo); un clic en una toma de corriente, un interruptor, una luminaria o un
  elemento calefactor colocado antes sin designación de dispositivo convierte a ese en el
  dispositivo. Un dispositivo colocado de nuevo se mueve; a continuación sigue el
  siguiente dispositivo aún no colocado.
- **Longitud de cable**: cada conexión de los conductores de fase de un circuito entre
  dos dispositivos, un cable (una vez) o un hilo individual, tiene la longitud de su
  recorrido 3D (tendido en las salas, a través de las paredes y entre niveles,
  `crate::routing3d`). Hasta que está tendida se estima (se muestra con `~`): desde la
  parte superior del cuadro hasta el dispositivo por la zona de instalación horizontal
  situada 300 mm por debajo del techo (DIN 18015-3), a lo largo de x y z de la planta,
  hacia arriba y hacia abajo en las zonas verticales de los dispositivos, hacia arriba o
  hacia abajo entre niveles, 0,3 m por cada caja. *Escribir longitudes de cable*
  (`write_circuit_cables`) escribe las longitudes (redondeadas al medio metro superior)
  en la lista de cables.
- **Caída de tensión desde la alimentación** (*Desde la alimentación*;
  `total_drop_percent` de `list_circuits`): por conexión
  ΔU = k · I · L · (ρ / A · cos φ + X' · sin φ) (k = 2 en monofásico, √3 en trifásico, ρ del cobre a su temperatura de
  servicio de los ajustes eléctricos, X' = 0,08 mΩ/m, cos φ 0,95), sumada a lo largo de
  las conexiones desde el dispositivo de protección hasta el dispositivo más alejado. Los
  cables del circuito llevan la corriente de diseño de su dispositivo de protección
  (cálculo de cargas) o su corriente asignada; los cables de alimentación desde el
  suministro hasta el cuadro de distribución (los que sigue el cálculo de cortocircuito,
  con sus longitudes en la lista de cables), la corriente de diseño del dispositivo de
  protección aguas arriba o su corriente asignada. La suma se comprueba frente al límite
  de los conjuntos de reglas (*Caída de tensión*); la información emergente indica la
  parte de la línea de alimentación.
- **Comprobación** *Circuitos de instalación*: circuitos cuyo dispositivo de protección
  no está en ningún cuadro de distribución del diseño, cables más delgados de lo que
  exige su dispositivo de protección (intensidad admisible en las condiciones de
  instalación), caídas de tensión por encima del límite, dispositivos de un circuito no
  colocados en ninguna sala (información); tomas de corriente, interruptores, luminarias
  y elementos calefactores de las salas que no están en el esquema (información).

## Cajas generales de protección

El sistema de envolventes *Caja general de protección* (`house_connection`, comando
`add_enclosure` con `system` `house_connection`) es la caja de la compañía suministradora
en la entrada al edificio: poliéster reforzado con fibra de vidrio (RAL 7035, IP54) con
una tapa abisagrada y precintable y una placa de montaje, en dos tamaños estándar
(`GEN.HAK-255X415X144` para NH00, `GEN.HAK-410X720X185` para NH2). Su tapa se abre en la
vista 3D como la puerta de cualquier envolvente. Dentro van la base portafusibles NH
(`GEN.NH2-BASE-3P-400A`, alimentada desde abajo: el cable de alimentación sube recto
hasta su lado de línea a través de un prensaestopas en el fondo, el cable principal sale
recto hacia arriba por la parte superior) y el bloque de bornes PEN
(`GEN.TB-PEN-240-V`). El ejemplo `building_services` cuelga una en la pared encima de la
entrada de cables, con la barra principal de puesta a tierra (`GEN.MET-BAR-10`) y la
pletina de conexión del electrodo de cimentación (`GEN.FE-LUG-V4A-10`, símbolo *Electrodo
de tierra*) a su lado.

## Armarios de distribución con módulos de componentes

Un artículo de envolvente de un fabricante que se suministra con su zócalo lo indica en
sus datos (`Plinth: 200`): una envolvente nueva de ese artículo se apoya en el zócalo,
que se dibuja y se exporta pero no es un artículo propio en la lista de materiales (el
armario Hager univers `HAG.FR25S2`, 1850 mm + 200 mm de zócalo, cinco campos, clase de
protección II). Su placa de montaje se coloca mediante `Mounting origin: 25/12.5/20` (la
esquina superior izquierda del frente de la placa desde la izquierda, la parte superior y
la parte trasera de la envolvente, mm; la placa mantiene la misma distancia a la derecha
y abajo), como lo dan los paquetes de envolventes del configurador de Rittal
([artículos](parts.md)). Una placa que no está centrada
da además su tamaño (`Mounting size: 500 x 575`; `mounting_size` de `add_enclosure` /
`set_enclosure`). Un artículo de envolvente de un fabricante
que nombra su sistema (`Enclosure system: compact`) recibe las paredes y los datos de ese
sistema pero sigue siendo el artículo de la envolvente; solo las envolventes del catálogo
integrado siguen su tamaño con el cuerpo del sistema.
Los sistemas de distribución construyen el armario a partir de módulos de componentes: un
módulo sobre el que se apoya el dispositivo es un accesorio montado *debajo del artículo*
(`HAG.UE31E1B` bajo el interruptor-seccionador con fusibles NH1 `HAG.LT153`,
`HAG.UE31SS400` bajo los transformadores de corriente), de modo que el dispositivo toma
la huella y la profundidad del módulo y el artículo queda delante de él.
El ejemplo `pv_transformer_metering` construye un armario así campo a campo siguiendo uno
real: zona del punto de conexión y descargador de sobretensiones, medida del consumo con
seccionamiento NH1 en la barra, módulo de transformadores encima, circuitos de tensión e
interruptor-seccionador de carga (la potencia fluye hacia arriba por el campo: los
conductores van de un dispositivo directamente al siguiente), la protección NA, la medida
de la generación debajo del interruptor de acoplamiento y las barras de las salidas;
cáncamos de elevación en el techo, los cables de los consumidores por el fondo, la
alimentación desde abajo; los dos armarios de contadores del gestor de la red de
distribución con sus bloques de bornes de ensayo a su lado. Las bridas de entrada de
cables (subgrupo de artículos `Cable entry`, una membrana que admite varios cables
grandes, como la univers FZ402) son accesos para el tendido 3D como los prensaestopas. El
ejemplo `industrial_main_distribution` acopla dos armarios univers, abiertos entre sí,
para formar la distribución principal de una planta industrial: los cables de
alimentación de aluminio sobre bornes de barras y a través de una brida de entrada en el
fondo, los transformadores de la medida en la barra de alimentación, un sistema de barras
de 185 mm a través de ambos armarios con regletas verticales de
interruptores-seccionadores con fusibles NH, los transformadores de corriente enganchados
detrás de los medidos, y los interruptores de acoplamiento de la protección NA
alimentados desde bornes de barras.
Ninguno de los dos tiene canaletas: cada conductor lo tiende en ellos el tendido 3D, de
modo que las corrientes elevadas circulan por las barras y solo es un hilo lo que se
puede doblar con su radio.

## Racks de 19 pulgadas

El sistema de envolventes *Rack de 19 pulgadas* (`rack19`, *Espacio de montaje › Nueva
envolvente*, comando `add_rack` o `add_enclosure` con `system` `rack19`) es un bastidor
con una puerta perforada y dos pares de perfiles de montaje de 19 pulgadas (IEC 60297) en
lugar de una placa de montaje; sus tamaños estándar van desde un rack mural de 12 U hasta
47 U, y los perfiles son un artículo de la lista de materiales (`GEN.RK-RAIL-42U`). Sus
unidades de rack (1 U = 44,45 mm) se numeran desde abajo junto a los perfiles en la vista
frontal y en la página de distribución de montaje.

- **Las piezas de 19 pulgadas** son tan anchas como un panel de 19 pulgadas (482,6 mm) y
  tienen un número entero de unidades de altura (dato técnico `Rack units: 2`): el
  catálogo tiene un conmutador de red, un panel de parcheo, una PDU, un servidor, un SAI,
  paneles ciegos, una bandeja y un panel pasacables. *Insertar › Pieza de 19 pulgadas*
  monta una en las unidades libres más altas del rack mostrado (comando `mount_in_rack`
  con `unit`, contada desde abajo); al colocarlos o arrastrarlos, los dispositivos y las
  piezas se ajustan a unidades enteras, centrados, con su panel frontal sobre los
  perfiles delanteros, y los sujetan los perfiles (sin taladros en una placa).
- Las unidades ocupadas dos veces son colisiones; la comprobación notifica las piezas de
  19 pulgadas fuera de las unidades y, como indicación, otros artículos en el plano de
  montaje (necesitan una bandeja o una placa de montaje). El informe *Ocupación del rack
  de 19 pulgadas* (y `list_racks`) enumera las piezas de cada rack desde arriba con sus
  unidades y profundidades, y las unidades libres.
- Los racks se acoplan en alineaciones y se sitúan en salas como los demás armarios.

## Tendido 3D

Los cables, los tubos y (opcionalmente) los hilos se tienden a través de la alineación:
en canaletas, a lo largo de trayectos de tendido dibujados sobre la placa de montaje, o
libremente alrededor de los dispositivos; segregación de las clases potencia, mando,
señal y fluido, reservas de tendido, radios de curvatura, haces de cables con sus
derivaciones, terminaciones de pantalla (carril de abrazaderas de pantalla, rabillo) y
sus comprobaciones; los recorridos se pueden congelar para la producción y se quedan como
están hasta que dejan de ser válidos.

Los hilos de un armario con canaletas en su placa de montaje discurren por las canaletas
(el ajuste de diseño *Tender los cables en 3D* los tiende como cables en su lugar). En un
armario sin canaletas, y en una sala, no hay nada donde alojarlos: cada hilo se tiende en
3D de punto de conexión a punto de conexión. Sale de cada punto de conexión en línea
recta en la dirección que da el punto, hasta la distancia de su radio de curvatura, y
entra en el otro en sentido contrario a la dirección que da este, sin pasar nunca de
largo para volver
(a través de
la caja de su propio dispositivo cuando el punto se encuentra dentro de ella, como los
bornes de un interruptor sobre su módulo, nunca a través de otro dispositivo situado
justo delante del punto), mantiene la mitad de su grosor libre de todo, se curva
únicamente con su
radio y se mantiene separado de todos los demás recorridos y de sí mismo (donde sus
extremos están demasiado cerca para su radio de curvatura, da la vuelta en un bucle que
nunca vuelve a cruzar ni a acompañar su propio camino, las curvas de sus esquinas ni la
boca de la que sale; donde no existe tal bucle no se tiende); los recorridos solo se
juntan donde llegan al mismo dispositivo, y allí van uno junto a otro: un recorrido nunca
cruza otro con el que se junta (solo puede hacerlo en su propia salida de un punto de
conexión, o donde llega al dispositivo en el que termina el otro, y también allí lo rodea
cuando eso cuesta poco). La salida recta de cada punto de conexión y de cada entrada de
cables permanece libre hasta que se tiende su propio recorrido, de modo que ningún otro
recorrido cruza un borne, y solo la comparte un recorrido que sale de un punto de
conexión situado justo al lado. De los hilos de un armario con el mismo grosor se tiende
primero el más corto: entre dos dispositivos, los conductores interiores quedan más cerca
de ellos y los exteriores los rodean. Un hilo que no encuentra un camino así no se
dibuja: se notifica como error
(*… no se puede tender: ningún camino que mantenga la mitad de su grosor libre de todo y
se curve con su radio de … mm*). Las causas habituales son un conductor demasiado grueso
para el espacio entre sus extremos (una sección de barra dibujada como hilo: monte los
dispositivos sobre un sistema de barras en su lugar), puntos de conexión orientados en
sentidos opuestos, un
regletero justo debajo de un interruptor, o dos extremos más próximos entre sí de lo que
necesitan las curvas entre ellos (una entrada de cables justo debajo de la caja a la que
conduce, dos accesorios uno encima de otro con sus conexiones en los lados). El panel de mensajes salta a un punto de conexión del
hilo o del cable, y la vista 3D muestra una nota roja con el motivo en cada uno de sus
extremos en lugar del recorrido.

El tendido trabaja sobre una rejilla (25 mm en un armario, 50 mm en una sala); un punto
de conexión está donde está. La vista 3D dibuja un recorrido saliendo recto de su punto
de conexión: su primera esquina está sobre esa línea, el tramo siguiente se inclina o
vuelve a la rejilla en una pieza oblicua, y los tramos cortos intermedios se desplazan.
Donde el camino recto de salida de un punto está cerrado y el recorrido empieza desde la
celda contigua, necesita allí espacio para una curva en S de su radio hasta esa línea (o
para inclinarse hacia ella); en caso contrario no se tiende. Los caminos que discurren
juntos van uno junto a otro en carriles; un camino cambia de un carril a otro en una
pieza oblicua de un tramo, nunca con un escalón en una esquina, y un quiebro demasiado
corto para las curvas de sus dos esquinas se dibuja como una curva en S con el radio de
curvatura. Un cable termina en la curva de la esquina anterior a la separación de sus
conductores cuando se corta allí; los conductores se separan dentro de la caja de sus
extremos, y un conductor hacia un borne situado detrás de otros de su regletero (uno
encima de otro sobre las barras) pasa por delante de ellos, por el frente del
dispositivo. Donde dos recorridos se cruzan aun así justo en un dispositivo al que ambos
llegan, la vista eleva uno sobre el otro de forma más brusca de lo que permite su radio
de curvatura.

Un cable entre dos armarios que no están en ninguna sala común se tiende en cada uno de
ellos hasta una de sus entradas de cables (una nota indica que el camino entre ellos no
forma parte del diseño); cada cable toma una entrada propia cuando hay suficientes que le
sirvan. Los cables que pasan por una misma brida de entrada de cables atraviesan su
membrana (dato de artículo `Cutout`) uno junto a otro, cada uno en un lugar propio,
separados el grosor del cable más un centímetro: el lugar libre más cercano a donde se
dirige el cable, o el centro cuando no hay ninguno libre.

Entre salas, un recorrido atraviesa los pasamuros, los patinillos y los enlaces entre
edificios que las unen, por el menor número posible de ellos: por la primera sala hasta
un pasamuros, por las salas intermedias y por la última sala hasta su extremo (sus tramos
por sala y armario figuran en la lista de tendido); su longitud incluye los caminos a
través de las paredes, los patinillos y los enlaces. Sin un paso así, el recorrido
queda sin tender (*ningún pasamuros lleva de una sala a otra*). Un recorrido desde un
dispositivo de una sala hasta el exterior del edificio termina en su entrada al edificio
más cercana, también desde un
dispositivo de un armario situado en la sala (a través del prensaestopas del armario y de la sala); de varios
pasamuros que unen las mismas dos salas, el recorrido toma el del camino más corto.

En una sala, los recorridos discurren por su espacio libre: rodeando sus paredes, sus
paredes interiores, los armarios situados en ella y el barrido de sus puertas, con
preferencia por sus canaletas y recorridos de tuberías. Discurrir en horizontal por el
espacio abierto de una sala, a más de 300 mm de sus paredes, de su techo y de lo que hay
en ella, cuesta diez veces más: las tuberías y los cables se ciñen a las paredes y al
techo y bajan en vertical hasta un dispositivo situado en la sala.
Los cables y las tuberías solo giran después de avanzar en línea recta la mitad de su
radio de curvatura (cuando existe un camino así); de los caminos que cuestan
aproximadamente lo mismo, un recorrido toma uno cuyas curvas guardan su distancia (un
tramo corto al principio o al final). Un recorrido entre un armario y un dispositivo de su
sala (o entre dos armarios de una misma sala) discurre por tramos: por el armario hasta
un prensaestopas en su techo, sus laterales o su parte trasera (primero uno de su propio
armario, si no, de la alineación; a falta de uno, por el centro del techo, lo que se
notifica), y después por la sala. Un prensaestopas
admite un cable: el libre cuyo rango de apriete se ajusta al cable y más cercano a donde
este se dirige; una brida de entrada de cables admite tantos como lugares tiene a lo
largo de su membrana.
Un cable que encuentra ocupados todos los prensaestopas de su armario no se tiende y se
notifica como error (*… cada prensaestopas de … está ocupado por otro cable. Añada un
prensaestopas para él*).
Una caja de derivación de un fabricante marcada como envolvente (dato de artículo
`Junction box enclosure: yes`, la Spelsberg Abox) se coloca en una pared como una
envolvente de la sala, con su parte trasera en la pared y su centro a la altura de
montaje, con un conector de empalme WAGO 221-415 en su interior que pasa a ser el
dispositivo del esquema (los cables de su circuito entran en la caja). En la vista 3D, un
clic en el interior de una pared o en el techo de una sala coloca allí un artículo: a
menos de 200 mm de su altura de montaje, a esa altura (su zona de instalación); más
lejos, a la altura pulsada; el artículo se muestra transparente donde quedaría mientras
se mueve el puntero.
Con *Añadir prensaestopas para cables nuevos* (propiedades de la envolvente,
`set_auto_glands`; activado de forma predeterminada), después de cada cambio de los
cables o del esquema, un cable que sale de un armario hacia un dispositivo situado fuera
de él y de su alineación y no encuentra ningún prensaestopas libre por el que pueda pasar
recibe uno: de poliamida, el más pequeño cuyo rango de apriete admite el cable, en el
primer lugar libre desde el centro de la cara. Un cable hacia un dispositivo de la sala
en la que está el armario lo recibe en el techo (solo puede salir por el techo, los
laterales y la parte trasera, véase más abajo; en un armario colgado en una pared, en el
fondo), y un cable hacia el campo, en el fondo. Donde esa cara está llena, el
prensaestopas va en la siguiente cara por la que puede salir el cable: el fondo o el
techo, después los laterales izquierdo y derecho y la parte trasera
(no un lateral contiguo a la siguiente envolvente de la alineación, ni los laterales de
una envolvente de menos de 150 mm de profundidad, cuyos hilos no encontrarían camino
hasta ellos; no la parte trasera de una colgada en la pared o apoyada contra una pared de
su sala, y su fondo, para un cable hacia la sala, solo hacia un lugar situado por debajo
del fondo del armario con su zócalo, tal como lo deja el tendido: el dispositivo de la
sala o de un armario situado allí o, si no, el primer pasamuros en el camino hacia su
sala; un dispositivo sobre la cubierta está arriba). Un
cable que no encuentra ningún lugar libre en ninguna de ellas no recibe ninguno y se notifica (mediante *Añadir los que faltan* en la línea
de estado, como `glands_missing` de un comando y `missing` de `add_cable_glands`, cada
uno con el armario y el motivo): haga sitio o añada una brida de entrada de cables. Un
prensaestopas eliminado a mano sigue eliminado hasta que cambian los cables; *Añadir los
que faltan* (`add_cable_glands`) añade un prensaestopas para cada cable que no tiene
ninguno, también con la opción desactivada. Un armario con una
brida de entrada de cables no recibe ninguno. De un
armario colgado en una pared (con su fondo por encima del suelo) también se sale por un
prensaestopas en su fondo: debajo de cada uno de esos prensaestopas queda libre para los
recorridos un pozo hasta el suelo y hacia el frente del armario. Congelado, un recorrido
así conserva sus tramos.

**Puntos de paso.** Un hilo, cable o tubo se lleva por otro camino mediante puntos de
paso: clic derecho en su recorrido sobre la placa de montaje, en una pared de una sala
(la vista frontal de una sala muestra los recorridos vistos desde la cara, también sus
hilos) o en la vista 3D y elija *Añadir punto de paso*; el punto se pone en el recorrido
donde se hizo clic, numerado a lo largo de él. Un clic
derecho mantiene el hilo, cable o tubo seleccionado donde pasa por ahí: en una
canaleta, donde los hilos se superponen, haga clic primero en el hilo (otra vez para el
siguiente) y luego clic derecho. Un hilo con puntos de paso se tiende en
3D de punto de conexión a punto de conexión como un cable (uno en las canaletas sale de
ellas), sin ellos vuelve a las canaletas; un hilo a la puerta no admite ninguno (el
enrutado informa de un error para él). Arrastre un punto de paso para moverlo: en la
vista frontal sobre la cara mostrada (mantiene su distancia a la cara), en la vista 3D
paralelo a la pantalla. Donde se suelta, el recorrido se busca de nuevo desde su inicio
a través de cada punto de paso en orden hasta su final: cada punto se pasa por el centro
de su celda de enrutado, el recorrido sale de él como entró, mantiene la mitad de su
grosor libre de todo y apartado de los demás recorridos, solo se curva con su radio y no
vuelve sobre sí mismo. Donde no es posible (el punto está dentro de un aparato o fuera
de la envolvente, ningún camino hasta él queda libre curvándose con el radio, los puntos
no dejan sitio para los arcos de las curvas entre ellos, el camino vuelve sobre sí mismo,
o el punto no está en ningún armario ni sala por los que pasa el recorrido), el enrutado
informa de un error que nombra el punto de paso, lo dibuja en rojo y tiende el
recorrido sin sus puntos de paso hasta que se mueva; la línea de estado
indica si el recorrido pasa por sus puntos de paso o por qué no. Solo el recorrido
cuyos puntos de paso cambiaron se busca de nuevo enseguida, entre los demás recorridos
donde están (mientras tanto un arco gira alrededor de su punto de paso); todo el
enrutado sigue en segundo plano. Un punto dado en una esquina del recorrido retrocede a
lo largo de él el radio de curvatura, y el camino de vuelta desde un punto se mantiene
apartado del de ida (en una canaleta puede cruzarlo). Un clic derecho en un punto de paso
lo elimina, o todos (también en las propiedades del recorrido o del hilo). Un recorrido
con puntos de paso no está congelado. Comandos `add_control_point`,
`move_control_point` y `remove_control_point` (puntos en las coordenadas de una
envolvente; un hilo por `from|to`, sus puntos de conexión); `list_routing` los lista.

**Empotrado en las salas.** Cada cable, hilo y tubo se tiende en las salas *en
superficie* (como arriba), *empotrado en la pared* o *empotrado en el suelo*: como dicen
las reglas de tendido (*Tendido en las salas*, comando `routing_rules` con `laying`) o
como se haya fijado para él (propiedades del recorrido › *Tendido en las salas*, comando
`set_laying` con su clave). Empotrado, discurre por la sala en rozas en sus paredes, a la
*Profundidad de roza* (30 mm salvo que se indique otra cosa, `slot_depth`) por detrás de
su superficie menos la mitad de su grosor, a lo largo de las líneas medias de las **zonas
de instalación** (reglas de tendido › *Zonas de instalación*, comando
`installation_zones`): zonas horizontales por encima del suelo y por debajo del techo,
cortadas donde hay puertas y ventanas y unidas alrededor de las esquinas de la sala, y
zonas verticales junto a las esquinas, las puertas y las ventanas. Un dispositivo en una
pared se alimenta en vertical, hacia arriba o hacia abajo, desde la zona horizontal más
cercana (o lateralmente desde una zona vertical en la que se encuentre); un extremo libre
(un prensaestopas en la parte superior de un armario) va primero a la pared más cercana.
En el suelo, la línea baja en vertical por la pared (o, a poca altura sobre el suelo,
directamente hacia abajo) y discurre a la *Profundidad en el suelo* (60 mm,
`floor_depth`) por debajo de su superficie a lo largo de la zona del suelo. Las zonas
son las de DIN 18015-3:2016 salvo que se cambien (*Zonas alemanas (DIN 18015-3)* las
restablece): ZW-o 150 - 450 mm por debajo del techo, ZW-u 150 - 450 mm por encima del
suelo, ZW-m 1000 - 1300 mm por encima del suelo para salas con encimeras (desactivada),
ZS-t / ZS-f / ZS-e 100 - 300 mm junto a puertas, ventanas y esquinas, ZD-r 200 - 500 mm
desde las paredes en el techo (se muestra, no se tiende por ella) y ZF, la misma en el
suelo (la norma no tiene zonas en el suelo). Un recorrido que las zonas no unen queda sin
tender (*ningún camino por las zonas de instalación*). *Mostrar › Zonas de instalación*
muestra las zonas transparentes sobre las paredes en 3D y en las vistas de las paredes, y
la del suelo en el plano; *Mostrar › Abrir paredes y suelos* deja ver a través de las
paredes y los forjados en 3D, como *Abrir las canaletas*, para mostrar las líneas
empotradas en ellos.
Las mangueras, los tubos y las tuberías terminan en las bocas de los modelos generados de
sus dispositivos (la boca de aspiración y la de impulsión de una bomba, las bocas de un
depósito, ambos extremos de una válvula o de un accesorio, las salidas de un colector):
el recorrido llega a un punto situado fuera de la boca al doble del grosor de la tubería
(al menos 30 mm) y continúa recto hacia su interior. Una válvula, un contador o una
válvula de retención en una tubería se considera montada en el sentido que acerca sus dos
tuberías al lugar del que proceden.
El ejemplo `pump_room` muestra bombas de pie sobre el suelo, un recorrido de tuberías y
los recorridos.

En un esquema, el menú contextual de un dispositivo colocado lo muestra en el diseño del
armario (la placa de montaje o la puerta de su armario, la pared o el plano de su sala) o
en la vista 3D con la cámara cerca de él (*Mostrar en el diseño del armario*, *Mostrar en
la vista 3D*).

Los recorridos forman parte del análisis del proyecto. Cuando se abre un proyecto, su
análisis se elabora en segundo plano: el editor muestra una ventana con el paso en el que
se encuentra (listas de redes, conexiones, cableado del armario, tendido 3D con el
recorrido en curso, llenado de canaletas, comprobaciones) y muestra el proyecto una vez
listo.

El armario tendido es la entrada de los [asistentes de cableado y montaje](workshop.md) y
de las exportaciones 3D (glTF, STEP, IGES, IFC, JT, Parasolid; véase
[salida e intercambio de datos](data-exchange.md)).
