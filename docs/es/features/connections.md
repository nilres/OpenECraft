# Hilos, cables, potenciales y numeración

Cómo reciben las conexiones su cable, su color, su sección y su número, y cómo se numeran los dispositivos, las páginas y los hilos.

## Definiciones de potencial

**Definiciones de potencial** (*Conexiones › Potenciales…*): tipo (L, N, PE, CC +/−,
señal), tensión, color de hilo (códigos IEC 60757) y sección por potencial; los ajustes
predefinidos IEC 60204-1 (maquinaria, proyectos nuevos) e IEC 60445 (colores de fase).
Los hilos sin color / sección propios los toman del potencial de su conexión (también
detrás de bornes), y si no del valor predeterminado del proyecto para conexiones sin
potencial definido; los conductores de cable conservan los valores del cable. Las
propiedades del hilo muestran el valor automático y de dónde viene; lo usan las listas
de hilos, el cableado del armario, el llenado de canaletas, las punteras, los mazos y
el marcado. La comprobación notifica GNYE fuera de los conductores de protección y PE
en otros colores, valores manuales que difieren de la definición y potenciales sin
definición. *Ver › Colores de los hilos* (se guarda con el proyecto) dibuja los hilos en
su color en el esquema y en la exportación PDF / SVG; los hilos bicolores como GNYE se
dibujan discontinuos en su segundo color.

## Cables

Definiciones de cable (identificador, tipo, conductores, sección, longitud, artículo del
grupo de productos *Cables*). Los hilos se asignan a conductores (varios hilos a la vez
reciben los conductores libres en orden, los hilos de una misma conexión comparten un
conductor); los conductores y el identificador del cable se muestran en el esquema, con
un trazo sobre el hilo donde lo cruza una línea de definición de cable del cable (si no,
al 30 % del hilo). La designación del conductor y el identificador del cable de un hilo
seleccionado se pueden arrastrar a su sitio y conservan su desplazamiento desde allí;
*Restablecer* en las propiedades del hilo (o `update_wire` con `core_offset` /
`cable_offset` vacíos) los vuelve a colocar automáticamente. Informes de resumen de
cables y
diagrama de cables, cables en la lista de materiales por longitud, comprobaciones de
cables desconocidos y de conductores usados dos veces o que no están en el cable.

## Definiciones de conexión y de cable

Los textos de los conductores de cable se mantienen automáticamente separados de la
línea de definición y de su hilo. La disposición comprueba las áreas de texto estimadas,
los demás textos de cable, los hilos, los gráficos de símbolos cercanos, los textos de
pines, propiedades e identificadores y los textos libres. Prueba un conjunto acotado de
posiciones cercanas en un orden determinista; no garantiza que todo dibujo abarrotado
tenga un lugar libre. Los conflictos restantes tienen un contorno rojo en el editor.
Los desplazamientos manuales existentes conservan su significado original y nunca se
restablecen en silencio.

Este cambio de representación se aplica también a los textos automáticos de los
proyectos existentes, incluidas las páginas liberadas y las exportaciones PDF/SVG.
Liberar una página protege sus datos frente a ediciones; no congela el motor de dibujo.
Los desplazamientos manuales conservan sus posiciones. Para tener un registro inalterado
de una versión anterior, conserve el PDF exportado en la liberación.
El editor calcula una disposición de página por fotograma y usa las áreas de texto
realmente dibujadas tanto para la selección como para el arrastre. Las exportaciones de
varias páginas reutilizan su índice del proyecto.

Pase el puntero sobre el texto de un conductor o sobre el identificador del cable junto
a un hilo para ver su zona de arrastre. Arrástrelo directamente para mover solo el
texto, sin seleccionar antes el hilo; esto funciona también con una selección múltiple
previa. Los extremos de hilo siguen siendo tiradores de geometría aparte.
Las páginas liberadas y los proyectos abiertos en solo lectura no se pueden cambiar así.

Seleccione una definición de cable y elija **Alinear textos de conductores…** en su
menú contextual, en la cinta Conexiones o en sus propiedades. Elija arriba/izquierda o
abajo/derecha y una distancia mínima. La página muestra una vista previa; nada cambia
hasta **Aplicar**. Los textos de los conductores cruzados de ese cable en esta página se
alinean juntos, con una distancia común que se aumenta cuando hace falta para evitar
obstáculos. Aplicar sustituye deliberadamente sus posiciones manuales en un solo paso de
deshacer. Las posiciones del identificador del cable, la geometría de los hilos, las
asignaciones de cable/conductor y las demás páginas no cambian. Si no se encuentra una
colocación libre, cambie las opciones o coloque los textos uno a uno. Las posiciones
guardadas sobreviven a guardar/reabrir y usan la misma ruta de dibujo para la
exportación SVG/PDF.

Para renombrar un cable, haga clic derecho en su línea de definición o en su texto y
elija **Renombrar cable…**. La misma acción está en el navegador de cables y en las
propiedades que se abren con F2 o un doble clic. El diálogo selecciona el identificador
completo: escriba el identificador nuevo y pulse Enter, o pulse Escape para cancelar.
Las referencias en todas las páginas siguen al cambio de nombre, que se puede deshacer
en un solo paso. Los identificadores duplicados y los cambios en páginas liberadas se
rechazan. Los proyectos compartidos adquieren los bloqueos del proyecto y de las páginas
afectadas antes de aplicar el cambio. Cancelar libera solo los bloqueos recién
solicitados por el diálogo, una vez terminada su solicitud; los bloqueos que protegen
otras ediciones locales permanecen.
La acción está también en *Conexiones › Cables*. La automatización puede usar
`rename_cable` con `cable` (identificador completo actual) y `tag` (identificador
completo nuevo).
Se admiten servidores de proyectos más antiguos: si falta el punto de acceso de
instantánea de sesión, el editor combina la instantánea antigua con el estado de
bloqueos de la sesión en la misma revisión. Una revisión más reciente durante la carga
se reintenta en lugar de mezclar estados.

Al dibujar una definición de cable se previsualizan los hilos cruzados y los conductores
libres. Sus propiedades muestran los recuentos de asignados/sin asignar y el orden de
asignación, que empieza en el primer punto de la línea. Invertir la línea invierte el
orden para los conductores libres; las asignaciones existentes conservan su conductor.
Los conductores ya usados en otro lugar del proyecto no están disponibles. El texto del
cable (por ejemplo `4x1.5`) describe el cable introducido, no el número de cruces.

Si hay demasiado pocos conductores libres o un hilo cruzado ya tiene una asignación en
conflicto, **no se asigna ninguno de los hilos pendientes**. La línea permanece en el
plano con una advertencia de la comprobación y una indicación de corrección en sus
propiedades. Edite la lista de conductores o seleccione otro cable, y después elija
**Asignar los cables cruzados**. Las asignaciones existentes deben cambiarse
explícitamente en las propiedades del hilo; nunca se sobrescribe otro cable en silencio.
La lista física de cable/conductores nunca se amplía automáticamente para hacer
desaparecer una discrepancia.

Cuando la creación de un cable nuevo queda bloqueada por una asignación en conflicto,
solo se conserva la línea de definición: no se crea ningún cable sin usar. Seleccione un
cable en sus propiedades después de corregir el conflicto. Una designación de conductor
importada sin cable es también un conflicto; bórrela o cámbiela explícitamente en las
propiedades del hilo. El botón **Asignar los cables cruzados** desactivado explica las
carencias y los conflictos al pasar el puntero. Las vistas previas interactivas
reutilizan el índice de asignaciones de la sesión; al aplicar se comprueba siempre de
nuevo el proyecto actual.

La automatización usa `preview_cable_definition` con `page`, dos `points` y un `cable`
opcional para una vista previa de solo lectura. `apply_cable_definition` toma `page` y
el `id` de la definición, y devuelve `changed` además de su `assignment` resultante.
`add_cable_definition` conserva sus campos de respuesta existentes y añade `assignment`,
con los recuentos, los conductores libres, los conflictos y los candidatos ordenados de
hilo/conductor.

Un punto de definición de conexión sobre un hilo fija
el número, el color y la sección de toda la conexión y el conductor de su hilo; una
línea de definición de cable trazada sobre hilos los convierte en conductores de un
cable (un cable nuevo con conductores numerados, o uno elegido). La comprobación
notifica los puntos situados junto a los hilos, los puntos contradictorios o no
aplicados, y las líneas de cables desconocidos.

## Símbolos de conexión: piezas en T, esquinas, cruces y puentes

Tres dispositivos en un mismo potencial se pueden cablear como A–B y A–C o como A–B y
B–C; la lista de redes por sí sola no puede distinguirlos. Una **pieza en T**
(*Insertar › Conexiones › Pieza en T*, colocada donde se encuentran tres hilos) guarda
esta elección con el proyecto, en el espíritu de los símbolos de conexión de EPLAN: sus
tres conexiones son el tronco (la dirección de la pieza: izquierda, derecha, arriba o
abajo) y los dos brazos de la barra que lo cruza. Cuatro variantes por dirección
nombran la conexión de *origen* y el orden de los dos *destinos*:

| Variante | Origen | Primer destino | Segundo destino |
| --- | --- | --- | --- |
| 1 | brazo izquierdo / superior | el otro brazo (en línea recta) | el tronco |
| 2 | brazo derecho / inferior | el otro brazo (en línea recta) | el tronco |
| 3 | tronco | brazo izquierdo / superior | brazo derecho / inferior |
| 4 | tronco | brazo derecho / inferior | brazo izquierdo / superior |

Los dos hilos físicos de la conexión van del dispositivo del origen al dispositivo del
primer destino y al dispositivo del segundo destino. El dispositivo de una conexión de
la pieza es el primer punto de conexión a lo largo de su hilo, también a través de
puntos de interrupción; donde el hilo llega a otra pieza en T, esta transmite sus
destinos (cuando el hilo entra por su origen) o su origen. La lista de conexiones, los
diagramas de conexiones, el cableado del armario, los mazos y cualquier otro consumidor
de las conexiones usan este resultado. Las conexiones sin pieza en T conservan el orden
del esquema como antes, y una pieza que no encaja con sus hilos (no hay tres hilos, no
hay uno en su dirección, hay un punto de conexión en el sitio) se ignora y la notifica
la comprobación *Símbolos de conexión*, al igual que dos símbolos en un mismo punto y
una conexión cuyos símbolos la cablean en bucle (más hilos de los que necesitan sus
dispositivos, p. ej. los orígenes de dos piezas en T enfrentados a través de un hilo).

El símbolo es una diagonal corta del origen al segundo destino (el primer destino está
en línea recta); *Representación › Punto* dibuja en su lugar el punto de conexión
habitual y conserva el encaminamiento. El panel de propiedades muestra las cuatro
variantes como vistas previas con los destinos numerados, la dirección y los hilos de la
conexión. Girar o reflejar el dibujo gira la pieza con sus hilos. Los símbolos de
conexión siguen a sus hilos: cuando un dispositivo o una bajada se mueve a lo largo de
la barra, la pieza en T se desliza con la bajada; cuando se mueve la barra, sus bajadas
se estiran y la pieza se queda en ella; cuando se mueve todo el nodo, la pieza se mueve
con él.

Mientras está activa una herramienta de símbolo de conexión, el panel Propiedades (que
la herramienta vuelve a mostrar si estaba cerrado) contiene los encaminamientos del
siguiente símbolo para elegir, Tab los recorre y R (Girar) gira el símbolo. Un símbolo
recién colocado no queda seleccionado, de modo que estas elecciones solo afectan a los
símbolos aún por colocar; uno colocado se cambia seleccionándolo. Donde ya se encuentran
tres hilos, una pieza en T toma de ellos su dirección y el encaminamiento elegido.

Una **esquina** (*Insertar › Conexiones › Esquina*) es el segundo símbolo de conexión:
dos hilos que se encuentran en ángulo recto, con cuatro orientaciones (Tab o R elige la
siguiente mientras se coloca).
No cambia ningún encaminamiento; su utilidad es el cableado automático: con
*Autoconexión* activada, colocar una esquina (o una pieza en T) dibuja hilos desde los
puntos de conexión que miran en línea recta a sus brazos libres (también desde una pieza
en T sobre un hilo que pasa por ella), y un símbolo colocado o movido de modo que un
punto de conexión mire a un brazo libre se cablea a él. Así, una esquina en el punto de
encuentro de dos conexiones las cablea ambas. Una esquina que encaja con sus hilos se
dibuja como nada más que el codo; una sin sus dos hilos muestra sus brazos en el color
de error y la notifica la comprobación.

Un **cruce** (*Insertar › Conexiones › Cruce*) conecta los dos hilos que se cruzan en su
punto, que sin él quedan sin conectar. Son dos piezas en T unidas: cada extremo de su
línea recta (vertical u horizontal) se cablea primero en línea recta al otro extremo y
después, en transversal, a un brazo; las dos variantes deciden qué extremo toma el brazo
izquierdo / superior. En el ejemplo de EPLAN, un cruce vertical de la variante 2 lleva
una conexión desde arriba primero hacia abajo y después a la izquierda, y una desde
abajo primero hacia arriba y después a la derecha. Un **puente** (*Insertar ›
Conexiones › Puente*) tiene también cuatro hilos y es una estrella desde el lado por el
que llega la conexión hacia los otros tres: primero el lado opuesto, después el
izquierdo o superior, después el derecho o inferior. Ambos se pueden dibujar como un
punto, conservan entonces su encaminamiento, y giran con el dibujo. Cada símbolo es una
lista de vínculos entre sus brazos; donde un hilo llega a otro símbolo, ese símbolo
transmite los dispositivos de los brazos vinculados con aquel por el que entra el hilo.

Comandos: `add_t_junction` (la dirección se toma de los hilos que hay allí cuando no se
indica), `add_corner` (los brazos, de los hilos que hay allí, p. ej. `down left`),
`add_crossing` (`line` vertical u horizontal, `variant` 1 o 2), `add_bridge`
(`direction` por la que llega la conexión), todos con `autoconnect` para dibujar los
hilos hacia los puntos de conexión enfrentados, `update_connection_symbol`,
`list_connection_symbols` (tipo, encaminamiento, vínculos y los hilos de cada uno).
`autoconnect` admite también los identificadores de símbolos de conexión.

## Números de hilo

Un número por conexión (todos sus segmentos de hilo; los dos lados de un
borne son conexiones distintas). Numeración automática: secuencial, página/columna
(`3.4.1`) o por nombre de potencial (`L1`, `+24V`), con prefijo y valor inicial; los
números fijados a mano se conservan. Se muestran en el esquema, en la lista de
conexiones y en la lista de hilos del armario; comprobaciones de números en conflicto y
duplicados.

## Numeración

**Numeración** (en el espíritu de la numeración offline de EPLAN): *Dispositivos ›
Numeración de dispositivos…* renumera los identificadores de dispositivo por estructura
y letra identificativa en el orden del esquema (página, columna, de arriba abajo) según
un formato: `{letter}{n}` (`-K1`),
`{letter}{page}.{n}` (`-K3.1`), `{letter}{page}{column}` (`-K34`) o el estilo NFPA
`{page}{column}{letter}` (`-34K`), con valor inicial, paso, letras y ámbito, y una
vista previa de los identificadores nuevos; todo lo que remite a un identificador lo
sigue (artículos, regletas, conectores, datos de PLC, diseño del armario, mazos); los
identificadores de los dispositivos que no están colocados (lista de dispositivos,
diseño del armario) no se dan a otros. *Inicio › Renumerar páginas…* numera las páginas
con inicio y paso, reiniciando por estructura y conservando las subpáginas (`3.a`;
*Inicio › Nueva página › Nueva subpágina* inserta `3.1`, `3.2`, … después de la página
actual). Las secciones de páginas contienen páginas y subsecciones como directorios
(*Inicio › Nueva sección*, arrastre páginas sobre ellas en el navegador *Páginas*); los
nombres de las páginas llevan los números de sección (`2.3.1`), siguen cuando una
sección se renumera o se mueve, y la renumeración cuenta por sección. La numeración de
hilos recibe un esquema de *Formato* con marcadores de posición (`{page}`, `{column}`,
`{row}`, `{potential}`, `{kind}`, `{color}`, `{cross_section}`, el contador `{n}`
contado por cada valor del resto), dígitos y paso del contador, y tipos de potencial que
no reciben número (p. ej. PE). Comandos: `renumber_devices`, `renumber_pages`,
`set_wire_numbering`.
