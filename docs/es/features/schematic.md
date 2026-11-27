# Editor de esquemas

Páginas, dispositivos y su estructura, referencias cruzadas en vivo, herramientas de edición, navegadores y almacenamiento del proyecto. La [interfaz de usuario](../user-interface.md) lista los comandos y las teclas.

![El editor de esquemas con un circuito de potencia estrella-triángulo, el potencial L1 seleccionado resaltado y su color de hilo y sección automáticos](../../images/schematic.png)

## Proyectos con páginas

Páginas de esquema multifilar (A3, 10 columnas / 6 filas, cajetín),
portada y páginas de informe generadas. Navegador de páginas con reordenación.

## Ingeniería orientada a dispositivos

Los identificadores de dispositivo (`-K1`, `-S3`, …) se asignan automáticamente.
La función principal (bobina) y las funciones auxiliares (contactos) forman un solo
dispositivo. Seleccione una bobina y coloque contactos: se le asignan y se numeran
(13/14, 21/22, …).

## Cajas de dispositivo y puntos de conexión del dispositivo

Fuentes de alimentación, instrumentos de medida y módulos de mando sin símbolo de
biblioteca: *Insertar › Caja de dispositivo* dibuja una caja discontinua de cualquier
tamaño como dispositivo nuevo (`-A1`, con su texto de función y sus artículos como
cualquier otro dispositivo; seleccionada, se redimensiona con los tiradores de sus
esquinas y bordes, los puntos de un borde movido la acompañan), *Insertar › Punto de
conexión del dispositivo* coloca sus
conexiones un clic tras otro. Un punto colocado sobre el contorno de una caja pertenece
al dispositivo de la caja, sale del borde en el que está y sigue la numeración
(1, 2, 3 …; a `13` le sigue `14`); su designación (`L`, `N`, `PE`, `+24V`, …) se edita
en sus propiedades. Cada punto es una conexión propia, distinta de los bornes: la caja
nunca conecta dos de sus puntos, aparecen como `-A1:L`, `-A1:N` en los informes de
conexiones y de conexión de equipos, y la comprobación notifica una designación usada
dos veces en un dispositivo. Las propiedades de un punto nombran su caja de dispositivo
(otra caja lo asigna a ese dispositivo); un punto alejado de su caja muestra su
identificador de dispositivo, sobre su caja lo muestra la caja. Su designación se puede
arrastrar a cualquier lugar y alinear a la izquierda, centrada o a la derecha. Las
propiedades de una caja fijan su anchura y su altura: los puntos de un borde que se
mueve van con él y sus hilos los siguen. Mover, girar, copiar, cortar, eliminar y
guardar como macro toman una caja con sus puntos; las copias se convierten en un
dispositivo nuevo con sus propios puntos. Comandos: `add_device_box`,
`update_device_box`, `add_device_connections` (varios a una distancia, con designaciones
dadas o numeradas en serie), `update_device_connection`, `list_device_boxes`.

Una caja también puede continuar un dispositivo colocado en otro lugar: dele el
identificador del dispositivo y elija su *Rol* en las propiedades (`role` de
`add_device_box` y `update_device_box`). Una *Extensión* añade puntos de conexión al
dispositivo, para un dispositivo demasiado grande para una caja o un símbolo de
biblioteca al que le faltan algunas conexiones. Sus puntos son conexiones nuevas, y una
designación que el dispositivo ya tiene se notifica como usada dos veces. Una *Vista de
conexiones* muestra algunos puntos de conexión que el dispositivo ya tiene, por ejemplo
solo la alimentación de una tarjeta de PLC en la página de potencia; márquelos en las
propiedades (`show_device_pins`) o coloque puntos con sus designaciones. Un punto de una
vista es el punto de conexión que muestra: los hilos dibujados allí pertenecen al
potencial de ese punto en todas las páginas, los informes listan la conexión una sola
vez, desde el punto propio del dispositivo, y la comprobación notifica una vista de un
punto que el dispositivo no tiene. Ninguno de los dos roles es una segunda función
principal. La caja muestra una referencia cruzada al dispositivo, y la función principal
del dispositivo a sus cajas; copiadas sin el dispositivo, estas cajas siguen en él. Una
caja con un rol pero sin dispositivo colocado se notifica.

## Identificadores de estructura (IEC 81346)

Las páginas llevan una función (`=A1`) y un lugar
(`+CAB1`). Los dispositivos colocados en una página reciben su estructura
(`=A1+CAB1-K1`); en la página, los identificadores y las referencias cruzadas muestran
solo lo que difiere de la estructura de la página (estilo EPLAN), los identificadores se
numeran por estructura, copiar / pegar mueve los dispositivos a la estructura de
destino, y cambiar la estructura de una página puede llevarse consigo sus dispositivos.
El navegador de dispositivos agrupa por estructura; el cajetín muestra el nombre
completo de la página (`=A1+CAB1/3`).

Los **recuadros de estructura** (*Insertar › Recuadro de estructura*) dan a una zona de
una página su propia función y su propio lugar (o lugar de instalación, producto):
dibuje un rectángulo (se redimensiona con los tiradores de sus esquinas y bordes cuando
está seleccionado) y fije en las propiedades los aspectos que sustituye; los aspectos
vacíos se heredan de la página o de un recuadro que lo contiene, los recuadros anidados
se resuelven hacia el más interior. Un dispositivo pertenece al recuadro en el que está
el punto de inserción de su función principal: recibe la estructura del recuadro
(`=SYS+FIELD-S1` en una página `=SYS+CAB1`), su identificador en la página omite lo que
el recuadro muestra, y su identificador completo es el que usan el navegador, la
numeración, las referencias cruzadas y los informes. Un aspecto de un identificador
igual a la estructura que rodea al símbolo es heredado, cualquier otro es propio del
dispositivo: mover la función principal dentro o fuera de un recuadro, o mover,
redimensionar, cambiar o eliminar el recuadro, renombra el dispositivo (todas sus
funciones) en sus aspectos heredados en el mismo paso de deshacer y conserva los
propios; un identificador que ya existe se rechaza y se notifica en lugar de fusionar
dos dispositivos. Las propiedades del símbolo muestran de dónde viene cada aspecto. Los
recuadros se copian, se pegan y se guardan en macros con sus dispositivos; la
comprobación *Identificadores de estructura* notifica los recuadros que se solapan sin
que uno esté dentro del otro, y los dispositivos dentro de un recuadro cuyo
identificador tiene otro valor para un aspecto que el recuadro fija.
Comandos `add_structure_box`, `update_structure_box`, `list_structure_boxes`.

## Referencias cruzadas en vivo

Los contactos muestran dónde está su bobina (`/3.4`), las bobinas muestran una
imagen de contactos con todos los contactos y sus posiciones, y los puntos de
interrupción remiten a sus homólogos en otras páginas. *Conexiones › Referencias
cruzadas…* fija el formato (prefijo, separador, fila: `/3.4`, `3-4`, `/3.4.B`), cuándo
se muestra la estructura de la página de destino (cuando difiere, siempre, nunca), a qué
remiten los puntos de interrupción (todos los demás puntos, estrella desde el primer
punto, o pares en el orden del esquema; la comprobación notifica un punto sin pareja) y
si las bobinas dibujan una imagen de contactos. Comando:
`set_cross_references`.

El nombre y la referencia cruzada de un punto de interrupción siguen a la flecha: en los
puntos horizontales, el nombre encima y la referencia cruzada debajo del conductor,
terminando en el punto de conexión, de modo que queden dentro de la página en su borde;
en los puntos verticales, junto a la flecha (la referencia cruzada bajo el nombre).
*Presentación*, en las propiedades, los pone en cambio encima, debajo, a la izquierda o
a la derecha de la flecha, para todos los puntos seleccionados a la vez (a la derecha de
una flecha hacia la derecha: en una línea tras su punta, para conductores separados
4 mm); con un solo punto seleccionado, cada texto se puede arrastrar por separado sin
mover el símbolo, y el botón de restablecer vuelve a la colocación automática. Las
posiciones se guardan con el proyecto y las usan las exportaciones PDF, SVG y DXF; la
referencia cruzada sigue siendo un enlace a su página.
Comando: `place_interruption_texts`.

En el editor, Ctrl + clic sigue una referencia cruzada (la tecla que abre los enlaces,
*Archivo › Ajustes › Teclas y gestos*): se abre la página indicada y, en un punto de
interrupción, su contraparte queda seleccionada y centrada. Mientras la tecla está
pulsada, el puntero es una mano sobre una referencia cruzada.

## Conectividad / potenciales

Los hilos, las piezas en T (con puntos de conexión), los bornes,
los puntos de conexión de potencial (L1, N, PE, +24V, …) y los puntos de interrupción se
evalúan en redes que abarcan varias páginas. Seleccionar un hilo resalta todo su
potencial.

## Edición

Seleccionar / seleccionar por ventana / mover con hilos que siguen y se mantienen
ortogonales, girar, reflejar, copiar / cortar / pegar (a través del portapapeles, los
dispositivos reciben identificadores nuevos), deshacer / rehacer, textos, ajuste a la
rejilla, desplazamiento y zoom. Un hilo unido a algo que se mueve conserva su dirección:
una esquina o una derivación en su extremo lejano se desliza allí a lo largo del hilo
(una derivación sobre una barra se queda en la barra); en otro caso el hilo recibe un
escalón o una ruta alrededor de lo que estorba, y un hilo que no se puede hacer
ortogonal sin tocar otra conexión queda en diagonal. Los puntos de definición de
conexión y las líneas de definición de cable se quedan sobre sus hilos salvo que se
muevan ellos mismos; una línea que no puede seguir cruzando exactamente sus hilos se
queda donde está. Con *Volver a trazar las conexiones de los elementos movidos*
(*Conexiones › Cruces de cables…*, desactivado de forma predeterminada) un hilo así se
traza de nuevo junto con las esquinas que le siguen, hasta el siguiente punto de
conexión, derivación o ramal, por el camino de menor peso: su longitud en pasos de
rejilla, 1,2 por codo y 1,5 por hilo cruzado; una conexión sin tal camino se hace
ortogonal como antes.

Un hilo movido también sigue conectado: su extremo en un punto de conexión de un símbolo
que no se mueve recibe un tramo nuevo desde ese punto (un escalón en su lugar donde el
tramo tocaría otro punto de conexión u otro hilo), su extremo sobre un hilo pasante se
desliza a lo largo de ese hilo, y los demás hilos del punto se quedan allí; un hilo
recto conserva su dirección y cae donde se arrastró, y un hilo sujeto por ambos extremos
no se mueve a lo largo de sí mismo. Las bajadas de una barra movida la siguen solo en
sentido transversal a la barra, de modo que conservan su lugar cuando la barra se mueve
a lo largo de sí misma. *Edición › Mantener conexiones* (activado de forma
predeterminada) desactiva esto junto con el estiramiento de los hilos unidos, de modo
que la selección se mueve sola y deja sus conexiones; mantener pulsada Alt al empezar un
arrastre invierte el ajuste para ese arrastre. Para desconectar un extremo, seleccione
el hilo y arrastre el tirador cuadrado de ese extremo: solo se mueve ese extremo, se
conecta a aquello sobre lo que se suelta y el hilo sigue ortogonal. Comandos:
`move_items` (`keep_connections`), `move_wire_end`.

Ayudas de edición: tamaños de rejilla de 1, 2, 4, 8 y 16 mm (Ver);
los puntos de conexión y los extremos de hilo fuera de esa rejilla (dibujos importados,
cajas de dispositivo de cualquier tamaño) añaden líneas de rejilla invisibles que pasan
por ellos, de modo que el puntero se ajusta a ellos donde están más cerca que la
rejilla, los hilos pueden empezar en tales puntos y se pueden conectar cosas a tales
hilos; un símbolo que se coloca o se mueve se ajusta desde sus propios puntos de
conexión (y un hilo movido desde sus extremos), de modo que caen sobre tales líneas y la
autoconexión los une; un
campo de coordenadas en la barra de herramientas (`x y` absoluto o `@dx dy` relativo al
último punto, Enter actúa como un clic de la herramienta actual); *Inicio › Alinear*
(bordes o centros), *Distribuir* (uniformemente, en horizontal o en vertical), *Mover la
selección…* y *Copia múltiple…* (n copias a una distancia fija, identificadores y
números de borne nuevos); los extremos de hilo unidos siguen. Comandos: `align_items`,
`distribute_items`, `copy_items`.

## Textos

Textos libres con giro (0 … 359°, p. ej. para leer hacia arriba a lo largo de una
regleta de bornes), alineación a la izquierda / centrada / a la derecha, un anclaje
vertical, ajuste de línea a una anchura fijada, un factor de anchura de las letras
(0,8: rotulación estrecha; se dibuja así en pantalla y en las exportaciones SVG, PDF y
DXF / DWG), negrita / cursiva / subrayado, un borde y un fondo opaco, y un hipervínculo
(Ctrl + clic lo abre; se puede pulsar en la exportación PDF y SVG). Los **textos
especiales** muestran propiedades del proyecto y de la página en cualquier lugar de una
página: los marcadores de posición como `{page}`, `{project}` o `{prop:Name}` (los
mismos que en los cajetines, Insertar › Texto especial) se rellenan cuando se dibuja la
página. Un **texto de función de camino** (Insertar › Texto de función de camino) es el
texto de función de todas las funciones de su columna que no tienen uno propio; lo usan
la lista de dispositivos y los diagramas de bornes, de conectores y de PLC. Los textos
giran con una selección girada; la importación DXF / DWG conserva el giro y la
alineación de los textos. Línea de comandos /
MCP: `add_text`, `update_text`.

## Gráficos del esquema

Líneas, rectángulos, círculos, elipses, arcos, polilíneas y
polígonos (grosor, color, discontinuo, relleno), cotas (alineadas / horizontales /
verticales, escala, unidad), imágenes PNG / JPEG (también en la exportación PDF y SVG).
**Capas** (navegador › Capas): visible, bloqueada, impresa, color; exportación DXF / DWG
por capa. *Exportar…* e *Importar…* del panel Capas llevan las capas con sus ajustes a
otros proyectos como esquema de ajustes (el esquema completo también las incluye).

Un elemento seleccionado muestra **tiradores** que cambian su tamaño o su forma al
arrastrarlos, sobre la rejilla: las esquinas y los centros de los bordes de rectángulos,
elipses, recuadros de estructura y cajas de dispositivo, las esquinas de las imágenes (se
mantiene la relación de aspecto), los extremos y vértices de líneas, polilíneas, polígonos
y cotas, cuatro puntos de un círculo para su radio y ambos extremos de un arco. Un
**tirador de giro** redondo sobre una forma, imagen o texto seleccionado lo gira
alrededor de su centro, en pasos de 15° (1° con Alt pulsada), con el ángulo junto al
puntero; los rectángulos, elipses e imágenes guardan el ángulo (*Rotación* en sus
propiedades) y sus tiradores lo siguen, las demás formas giran sus puntos. Las
propiedades de una forma muestran su tamaño: el ancho y el alto de rectángulos y elipses,
la longitud de las líneas y el radio de los círculos se escriben, el tamaño y la longitud
del contorno de las demás se muestran. La **escala** de una página (propiedades de la
página, `set_page scale`, 1 : 50 es 50) es la longitud real por milímetro en la página:
los tamaños de las formas se muestran en ella y las cotas nuevas miden en ella, para
planos de implantación y planos de planta sencillos dibujados a escala.

## Cruces de hilos y conexiones en T

Los hilos que se cruzan sin conexión se dibujan como un cruce simple, o el hilo vertical
(u horizontal) salta sobre el otro con un **arco**, un **puente** cuadrado o una
**separación**; los saltos cercanos en un hilo se convierten en uno. Las conexiones en T
muestran un **punto**, un **punto grande** o **ningún punto** (una T simple); una
conexión de cuatro o más hilos conserva siempre su punto, para que nunca parezca un
cruce. *Conexiones › Cruces de cables…* fija ambos con una vista previa; el ajuste se
guarda con el proyecto y en los esquemas de ajustes y se aplica por igual al lienzo y a
la exportación PDF, SVG y DXF. Línea de comandos / MCP:
`wire_settings` (`jump`: none, arc, bridge, gap; `jump_wire`: vertical, horizontal;
`tee`: dot, large_dot, none; `reroute`).

## Trazado

La autoconexión dibuja hilos entre puntos de conexión libres enfrentados cuando se
colocan o se mueven símbolos; mientras un símbolo se coloca o se mueve con el ratón,
unas tenues líneas de puntos muestran los hilos que dibujaría; la herramienta de hilos
puede trazar rodeando símbolos y otros hilos
(A); los tramos de hilo colineales se pueden unir. También como comandos `route` /
`autoconnect`.

## Colocación de propiedades

Arrastre el bloque de identificador de un símbolo seleccionado; coloque junto a él otras
propiedades del dispositivo (descripción, número de artículo / de tipo, fabricante,
designación, puntos de conexión, estructura).

Las designaciones de los puntos de conexión se colocan junto al trazo de su punto de
conexión, desplazadas a lo largo de él, a su otro lado o fuera, junto al hilo, donde
estorban los gráficos del símbolo, los hilos, los símbolos cercanos o los demás textos
del símbolo. Arrastre una de un símbolo seleccionado para colocarla a mano, o elija
izquierda, derecha, arriba o abajo para cada una en *Colocación de propiedades*;
*Restablecer* la devuelve a la colocación automática. La colocación gira y se refleja
con el símbolo y nunca mueve el punto de conexión. Comando:
`place_pin_label`. La designación de un símbolo con un único punto de conexión (como un
punto de conexión del dispositivo) se arrastra libremente y conserva su alineación.

Colocado automáticamente, el bloque de identificador queda a la izquierda de un símbolo
vertical y encima de uno horizontal. Donde allí chocaría con un símbolo vecino, pasa al
otro lado, encima o debajo, lo primero que quede libre de símbolos y de hilos con todas
sus líneas, incluidas las referencias cruzadas. El identificador de
una función situada justo al lado de otra del mismo dispositivo en la misma fila (como
los descargadores de sobretensiones dibujados polo a polo) se muestra una vez, en la
primera de ellas, como el identificador de regleta de una fila de bornes. Un bloque de
identificador colocado a mano se queda donde está.

## Buscar

**Buscar** (*Inicio › Buscar…*, `Ctrl+F`): busca en todo el proyecto (nombres y
descripciones de páginas, identificadores de dispositivo, textos de función, textos
libres, números de hilo, bornes y pines, cables, números de artículo, descripciones)
con opciones de mayúsculas y de palabra completa, salta a los resultados y reemplaza
texto en los textos libres, los textos de función y las descripciones (las páginas
liberadas se quedan como están). Comandos: `find`, `replace_text`.

## Navegadores

Páginas (opcionalmente agrupadas por estructura con las descripciones de la
gestión de identificadores de estructura), dispositivos (seleccionar en la página,
colocar más funciones, renombrar), conexiones (filtrar, mostrar, seleccionar, añadir una
definición de conexión), cables (conductores en uso y libres, definiciones de cable,
dibujar una definición de cable), capas y la comprobación.
La pestaña *No colocados* lista lo que los datos del proyecto tienen y el esquema no:
los canales de las tarjetas de PLC con descripción de dispositivo que no tienen símbolo
de entrada / salida (con su dirección), las funciones de los artículos asignados que
ningún símbolo usa (bobinas, contactos con sus designaciones) y los potenciales
definidos en el proyecto que ningún punto de conexión de potencial lleva todavía;
arrastre uno a la página (o haga clic en él y después en la página) para colocarlo con
identificador, dirección y designaciones. Comando:
`list_unplaced`.

## Idiomas

Inglés, alemán, francés, español, portugués (Brasil), italiano, neerlandés,
polaco, checo, ruso, ucraniano, turco, sueco, danés, finés, húngaro, rumano,
chino (simplificado), japonés y coreano (*Ver › Idioma*; el editor y el gestor de
artículos recuerdan la elección y se inician en el idioma del sistema / navegador si es
compatible). El chino, el japonés y el coreano vienen con fuentes incluidas (subconjuntos
de Noto Sans SC / JP / KR); los textos del proyecto en estas escrituras usan además las
fuentes CJK del sistema en el escritorio, en pantalla y en las exportaciones PDF
(incrustadas como subconjunto).

## Textos del proyecto en varios idiomas

Los textos de función, las descripciones de páginas, los textos
libres y las descripciones se escriben en un idioma de origen; el diccionario de
traducción del proyecto (*Datos de proyecto › Idiomas del proyecto…*) los traduce línea
a línea a cualquier número de idiomas de destino, con exportación / importación CSV para
traductores y una comprobación de textos sin traducir. *Datos de proyecto › Exportar
como PDF en idioma* (línea de comandos: `export` con `language`) emite la documentación
por idioma – los textos del proyecto a partir del diccionario, los informes y los
cajetines en ese idioma (inglés, alemán, francés, español; los demás idiomas reciben
los encabezados de informe en inglés).

## Plantillas de proyecto

**Plantillas de proyecto** (en el espíritu de los proyectos básicos de EPLAN):
*Archivo › Nuevo › Nuevo proyecto desde plantilla* parte de una plantilla integrada
(proyecto IEC 60204-1 vacío; máquina con portada, índice de contenido, páginas de
esquema estructuradas y páginas de informe; cuadro de distribución con vista general
unifilar e informes de climatización y de cortocircuito) o de cualquier archivo de
proyecto; *Guardar como plantilla de proyecto…* escribe el proyecto actual sin
revisiones, líneas base, comentarios, marcas de cableado ni mensajes de comprobación
aceptados. Línea de comandos: `new_project --template Machine` (o un archivo);
`new_project --example <name>` parte de un
[proyecto de ejemplo](../../../README.md#example-projects).

Las **plantillas paramétricas** piden valores al iniciar un proyecto a partir de ellas y
calculan otros con fórmulas: las incluidas *Arrancador directo de motor*, *Arrancador
estrella-triángulo de motor* y *Arrancador inversor de motor* piden la máquina
accionada, la potencia asignada, la tensión de red, cos phi, el rendimiento y si se
añaden las páginas de informe, y dimensionan el arrancador a partir de ellos – corriente
asignada (7,5 kW a 400 V: 14,6 A), rango de ajuste del guardamotor (10 - 16 A),
contactores (AC-3), sección del circuito del motor y los artículos del catálogo
integrado. Los valores rellenan los marcadores `{NAME}` en los textos del proyecto
(identificadores, textos de función, textos, datos de equipo, números de artículo,
descripciones de página); una página con una condición (`REPORTS=yes`, escrita como en
el configurador) se omite cuando no se cumple. Una fórmula conoce números, `"textos"`,
los nombres de los parámetros y de las fórmulas anteriores, `+ - * / ^`, comparaciones,
`&` (une textos) y `sqrt`, `abs`, `round`, `ceil`, `floor`, `min`, `max`, `if` y
`step(x, límite, valor, …, si no)`, el valor del primer límite que `x` no supera.

El panel *Datos maestros › Parámetros de plantilla* muestra el proyecto abierto como
plantilla: nombre, descripción, parámetros (etiqueta, unidad, valor por defecto,
opciones), fórmulas con sus valores y la condición de cada página. *Guardar como
plantilla de proyecto…* guarda el proyecto en la **carpeta de plantillas**
(`OpenECraft/data/templates` en la carpeta de datos del usuario, o la carpeta de
`OPENECRAFT_TEMPLATES`; en el navegador, el almacenamiento del editor), donde también la
encuentra la línea de comandos, y *Exportar como archivo…* escribe un archivo de
plantilla para pasar. Las plantillas de la carpeta aparecen junto a las incluidas en
*Nuevo proyecto desde plantilla*; una con el nombre de una plantilla incluida ocupa su
lugar, así se amplía una plantilla incluida cambiándola (*Editar plantilla* en el menú
de su mosaico abre la propia plantilla) y guardándola con su nombre. Línea de comandos:
`list_project_templates`, `template_values`, `new_project --template "Motor starter
direct on line" --values '{"POWER_KW": 7.5}'`, `new_project --edit`,
`save_project_template`, `delete_project_template` y los comandos `get_template`,
`set_template`, `set_template_parameter`, `set_template_formula`,
`remove_template_parameter` y `set_page --condition`.

## Almacenamiento

Archivos de proyecto JSON (`.oec`); el proyecto actual y la base de datos de
artículos se guardan además automáticamente (comprimidos) en la carpeta de datos de la
aplicación en el escritorio y en IndexedDB en el navegador, cuyo almacenamiento local
solo admite unos 5 MB (hasta que la primera escritura en IndexedDB tiene éxito se
conservan también en el almacenamiento local).
