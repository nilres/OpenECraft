# Regletas de bornes, conectores, PLC y marcado

Regletas de bornes con bornes de varios niveles y accesorios, conectores, tarjetas de PLC con el intercambio con las herramientas de programación de PLC, y los marcadores para el taller.

## Editor de regletas de bornes

Orden de los bornes (subir / bajar, ordenar), renumeración 1 … n,
**puentes** (los bornes puenteados comparten su potencial), bornes de reserva (en el
diagrama de bornes y en la lista de materiales);
cada borne tiene su artículo: los bornes PE toman el artículo de la regleta con el dato
técnico `Terminal type: PE`, todos los demás su artículo de paso (lista de materiales,
longitud de la regleta en el armario, puntos de conexión por borne); **accesorios
automáticos** a partir de los datos técnicos de los artículos de borne
(`End bracket: …`, `End plate: …`, `Jumper: …` con artículos de puente `Poles: n`; un artículo de soporte final en la
lista de artículos de la regleta, grupo de productos `End bracket`, sustituye al de los
datos técnicos): un
soporte final en ambos extremos, una tapa final después del último borne de cada
artículo, puentes enchufables de la longitud más corta que encaje (cortados si son más
largos, con los dientes partidos en los bornes intermedios) – en la lista de
materiales, el diagrama de bornes, el diseño del armario (2D / 3D) y la comprobación;
la ventana de regletas de bornes muestra la regleta sobre el carril con sus puentes y
tiene un interruptor para los accesorios automáticos.
**Bornes de varios niveles**: un borne en el nivel 2 o superior se asienta en el bloque
del borne anterior (un artículo por bloque, dato técnico `Levels: 2`); el diagrama de
bornes lista cada nivel, la disposición del carril y la lista de materiales cuentan
bloques, y la comprobación notifica más niveles de los que tiene el artículo. **Tipos
de puente**: puentes enchufables (dato técnico `Jumper`), puentes de silleta
(`Saddle jumper`) y puentes de cable (cualquier nivel, sin artículo); los puentes
enchufables y los de silleta deben quedarse en un mismo nivel. Comandos:
`list_strips`, `get_strip`, `set_terminal`, `set_jumper_kind`, `set_strip`,
`renumber_strip`.

## Conectores

**Conectores** (*Dispositivos › Conectores…*, grupo de símbolos *Conectores*): macho y
hembra (pareja acoplada, dos puntos de conexión), pin macho y hembra (uno cada uno);
identificador = conector (`-X10`), designación = pin. Los pines macho y hembra de la
misma designación están acoplados: el potencial pasa a través, los dos lados son
conexiones distintas (números de hilo, cables). Los pines se numeran por conector al
colocarlos; la ventana de conectores y el informe *esquema de conectores* listan los
pines en el orden del conector con el lado hembra, el lado macho y el potencial; los
artículos con funciones `Plug pin` dan el número de pines (los pines no usados se listan
como reserva). La comprobación notifica los pines colocados dos veces y más pines de los
que tienen los artículos del conector; la exportación KBL nombra el artículo de carcasa
de los conectores con todas las cavidades del artículo.

## Asignación de pines de conectores y cables planos

**Asignación de pines** (ventana de conectores, exportación *Asignación de pines de
conectores (SVG)* `pinouts`, comando `get_pinout`): las caras de acoplamiento del lado macho y del lado
hembra (la imagen especular) con cada pin del artículo del conector en su lugar,
rellenado con el color del hilo o del conductor de cable del pin (las reservas vacías),
la codificación y una tabla de pines con potencial, destinos y conductores de cable;
pasar el puntero sobre un pin muestra sus detalles, un clic muestra su símbolo. La cara
procede de los datos técnicos del artículo del conector:
`Pin layout: circle 4` (pines sobre un círculo, el pin 1 a la derecha de la codificación
arriba, contando en sentido antihorario en la cara macho como en IEC 61076-2-101),
`circle 7+1` (los pines centrales al final), `circle 9+3` / `circle 11+5+1` (un anillo
interior y después el centro), `row 8`, `zigzag 10` (dos filas, los pines impares sobre
los pares: IDC), `grid 2x5` (numerados fila a fila), `cols 20x2` (numerados columna a
columna), `dsub 9`, cada uno opcionalmente con `+PE`; para cualquier otra cara, los
centros de los pines en mm: `points -3.5,2 3.5,2 0,-2` (cara rectangular) o
`round 0,9 -8,-3 8,-3` (cara circular con codificación). `Pitch`, `Coding` y `Gender`
(`male`, `female`, `hermaphroditic`) la completan. Los pines numerados de un artículo
más pequeño están en su número (M8 de 3 pines: 1, 3, 4); los conectores sin disposición
se muestran en una fila.

El catálogo tiene, cada uno en macho y en hembra:

| Familia | Artículos |
| --- | --- |
| Sensor / actuador | M5, M8 (3 – 8 pines), M12 A, B (PROFIBUS), D, X (Ethernet de 10 Gbit/s) y las codificaciones de potencia S, K, T, L, 7/8" (3 – 6 pines) |
| Circulares | M16 según IEC 61076-2-106 (3 – 24 pines), M23 de señal (12, 17, 19 pines), conectores de potencia M23 y M40 para servomotores, conectores de aviación GX12 / GX16 / GX20 |
| Válvulas | DIN EN 175301-803 formas A, B y C, 2 + PE y 3 + PE |
| Rectangulares | insertos de conectores industriales pesados de las series A, D, E y Q con PE, D-sub 9 – 37, DIN 41612 tipo C (32, 64, 96), conectores frontales de PLC (20, 40) |
| Placa | conectores hembra IDC y bases con carcasa 10 – 64, una fila de 2,5 mm, dos filas de 3,0 y 4,2 mm, bloques de bornes enchufables de 3,5 / 5,08 / 7,62 mm |
| Datos | RJ11, RJ12, RJ45, BNC |
| Máquinas móviles | tipo DT estanco (2 – 12 pines) y sistema de 1,5 mm (1 – 6 pines) |
| Potencia | CEE IEC 60309 16 / 32 / 63 A con 3, 4 y 5 polos, conectores para aparatos IEC 60320 C5 – C20, Schuko, BS 1363, NEMA 5-15, L14-30, L21-30, conectores de instalación (3 – 5 polos), conectores fotovoltaicos, conectores de batería (hermafroditas, 50 – 175 A), conectores banana de seguridad, conectores de termopar |

**Cables planos** (`Cable form: Flat` en el artículo de cable: cables de cinta de
10 – 64 conductores con paso de 1,27 mm, cables planos para sistemas de festón):
`connect_flat_cable` convierte los hilos de un lado de un conector en conductores del
cable, tomando el pin n el conductor n. Los cables de cinta (cables planos con un
`Pitch`) muestran en las caras conductores grises con el conductor 1 en rojo. La
comprobación *Asignación de pines de conectores y cables planos* notifica los pines que
el artículo del conector no tiene, los conductores de un cable plano en un conector IDC
(`Termination: IDC`) que caen en otro pin y los cables planos con un número de
conductores distinto del número de pines del conector IDC.

## Navegador de regletas y conectores

**Navegador de regletas y conectores** (pestaña de navegador *Regletas / conectores*):
las regletas de bornes con sus bornes en el orden del carril (niveles de los bloques de
varios niveles) y las reservas, los conectores con sus pines y destinos y los pines no
usados de los artículos de conector; un clic muestra uno colocado, las reservas y los
pines no usados se arrastran a la página.

## Navegador de PLC

**Navegador de PLC** (pestaña de navegador *PLC*): las tarjetas con sus canales en el
orden de las direcciones, los colocados con el texto de función y los dispositivos
conectados (un clic muestra el símbolo), y los canales libres de las tarjetas con
descripción de dispositivo para arrastrarlos a la página.

## Resumen PLC

Seleccione una vez la tarjeta importada en **Centro de inserción › Dispositivos** y después haga clic en la hoja para cada canal sucesivo. En una EL1008, los clics del uno al ocho comparten un identificador de tarjeta. El clic nueve inicia el canal 1 en otra tarjeta. Escape termina la colocación. Despliegue **Representaciones** cuando se requiera una variante de cableado concreta.

Un EMA suelto con definiciones de canal reconocidas tiene una entrada en **Símbolos** y usa la misma secuencia de colocación. Importe su XML de artículo para obtener un artículo de catálogo, la asignación a la lista de materiales y las dimensiones para el armario.

Las macros analógicas de Beckhoff con una señal identificable y sus bornes asociados producen un punto de PLC. En la EL3174, la entrada de tensión `U1` usa el borne 1 para la señal y el borne 9 para su referencia. Ambos bornes se pueden cablear por separado. Las representaciones en modo de corriente son alternativas para los mismos cuatro canales. Use el Centro de inserción para seleccionar una representación concreta. El resumen PLC indica el borne de señal, y redireccionar la tarjeta deja intactas las etiquetas de los canales analógicos. Las declaraciones de variables de TwinCAT conservan el tipo de datos suministrado. Los canales sin tipo de datos declarado siguen disponibles en la hoja y en el resumen.

Para las Beckhoff EL1008 y EL2008, importe el XML de artículo con el EMA correspondiente al lado. Coloque un artículo en el esquema para usar su siguiente canal disponible, o elija un canal multifilar concreto en el Centro de inserción. Los canales de la misma familia de macros comparten un identificador de tarjeta. Una vez colocados los ocho, la siguiente colocación predeterminada inicia otra tarjeta. El navegador de PLC lista los canales restantes después de la primera colocación. El resumen PLC edita las direcciones y los textos de función. El redireccionamiento usa el orden de canales importado aunque los puntos se coloquen desordenados. Por ejemplo, el canal 2 usa el borne físico 5 y el bit 1. Fije un byte inicial distinto para otra tarjeta en el mismo espacio de direcciones. El acoplador EK1100 conserva sus conexiones de alimentación y de bus.

Todas las entradas / salidas por tarjeta con los dispositivos conectados, direcciones
y textos de función editables, byte inicial / bastidor / ranura de la tarjeta,
redireccionamiento (de los puntos digitales; los analógicos conservan sus direcciones de
palabra), **exportación / importación de la tabla de símbolos** (CSV: nombre, tipo de
datos, dirección, comentario) para herramientas de programación de PLC, la tabla de
variables de **Siemens TIA Portal** (hoja de Excel *PLC Tags*), la importación de tags
de **Rockwell Studio 5000** (alias de los canales de los módulos), la tabla de símbolos
de **Siemens STEP 7 classic** (SDF), una lista de variables globales para **Beckhoff
TwinCAT 3** (TcGVL, `AT %I*` / `AT %Q*`), variables para **Schneider Control Expert**
(XSY, `EBOOL` con direcciones topológicas), comentarios de dispositivo para **Mitsubishi
GX Works** (CSV, `X` / `Y` en octal) y variables globales IEC 61131-3 (CODESYS); se
notifican las direcciones usadas en varias tarjetas.

## Marcado

**Marcado** (*Conexiones › Marcado…*: texto del marcador de hilo, material por tipo con
número de piezas y exportación; *Archivo › Exportación › Exportar marcado (CSV)…*,
informe *Marcado*): marcadores de equipo (placas de leyenda para los equipos de puerta;
el lugar del armario se omite en el identificador), marcadores de borne en el orden del
carril con el marcador de la regleta, marcadores de hilo (el número de hilo en ambos
extremos, o marcado de destino: cada extremo muestra el otro extremo), marcadores de
cable; agrupados por material de marcado en el orden de aplicación, como CSV para
software de marcado (UTF-8, `;`, encabezado
`Material;Category;Text;Count;Enclosure;Group`); el material está en la lista de
materiales. Los textos y los materiales son ajustes del proyecto (`marking`).
Los **esquemas de marcado** eligen los marcadores, las columnas y su orden, el
separador, el encabezado y una fila por etiqueta impresa, para la exportación CSV /
texto y Excel; tablas de importación integradas para Phoenix Contact CLIP PROJECT,
Weidmüller M-Print PRO, Brady, Murrplastik y Partex (puntos de partida – cópielos y
ajústelos a la versión del programa), los esquemas propios se guardan en el proyecto
(comandos `marking_schemes`, `set_marking_scheme`, exportadores `marking` /
`marking_xlsx` con `scheme`).
