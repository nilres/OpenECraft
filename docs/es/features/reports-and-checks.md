# Informes, formularios y la comprobación del proyecto

Informes como ventanas, CSV / Excel o páginas de informe vivas, cajetines y formularios de informe (también gráficos y formularios dibujados propios), y la comprobación del proyecto con su gestión de mensajes.

## Informes

Índice de contenido, lista de dispositivos, lista de materiales (con artículos y
precios), diagrama de bornes, lista de conexiones, lista de asignación PLC, resumen de
potenciales, resumen de cables, diagrama de cables – en una ventana,
como exportación CSV, o insertados como páginas de informe vivas.

El filtro del panel *Informes* encuentra un informe por su título, su nombre o las
normas que sigue, sin distinguir mayúsculas, espacios ni guiones: `VDE 0100 600`,
`0100-600` o `IEC 60364-6` encuentran los tres informes de inspección y ensayo,
`ISO 13849` las funciones de seguridad, `60909` el informe de cortocircuito
(`list_reports` con `search` en la línea de comandos).

El cable vendido por metros necesita una longitud positiva para su cantidad de material.
Una longitud que falta se muestra como `?`; si otros cables de ese artículo tienen
longitudes, la fila muestra la cantidad conocida más `?`. Los precios y la suma del
informe indican también la parte desconocida. La comprobación de cables nombra cada
cable por metros que necesita una longitud. El cable contado por piezas conserva su
cantidad de una pieza cuando no se indica longitud.

Los artículos de cable pueden indicar `Outer diameter: 6.3 mm` en sus datos técnicos.
Seleccionar el artículo aplica esta dimensión al tendido; un artículo sin una dimensión
válida vuelve a la estimación a partir de sus conductores y su sección.
`apply_cable_part` expone la misma operación de catálogo a través del registro de
comandos compartido.
El selector de cables incluye los artículos locales del proyecto además del catálogo
configurado; los metadatos y las clasificaciones locales tienen prioridad para los
números de artículo repetidos. `list_cable_parts` expone las opciones del proyecto y las
integradas a los plugins y a CLI/MCP.

## Formularios

Los cajetines son listas de líneas, rectángulos, textos con marcadores de posición
(`{project}`, `{page}`, `{sheet}`, `{page.previous}` / `{page.next}`, `{rev.date}`,
tablas de revisiones con `{rev.1.name}`, `{rev.2.name}` …, propiedades de usuario
`{prop:Name}` …) e imágenes, editados en el editor de cajetines y compartidos como
archivos; uno por proyecto con una sustitución por página (integrados: Standard,
Compact, None). Los textos pueden estar en negrita, en cursiva y girados (p. ej. 90°
para textos que van hacia arriba). Un elemento de imagen conserva su propia imagen
(PNG, JPEG, p. ej. el logotipo de la empresa, que viaja con el archivo de cajetines) o
muestra el logotipo del proyecto de *Propiedades del proyecto › Logotipo* (p. ej. el del
cliente, distinto en cada proyecto), ajustado a su rectángulo con su relación de
aspecto; la portada muestra el logotipo del proyecto sobre el nombre del proyecto o en
un lugar propio (*Propiedades del proyecto › Logotipo en la portada*,
`set_cover_logo`). Los formularios de informe dibujados admiten los mismos elementos.
Cada elemento se puede limitar a páginas de algunos tipos (p. ej. un campo solo en las
páginas de informe) o a un marcador de posición relleno (p. ej. la segunda fila de una
tabla de revisiones solo con `{rev.2.name}`); las líneas, los rectángulos y los textos
reciben su propio color, las líneas y los rectángulos trazos discontinuos. Los **códigos
QR** codifican un texto con marcadores de posición (p. ej. un enlace al proyecto en la
gestión documental con `{project}` y `{page}`). El cajetín de una página es el suyo
propio, si no el de su tipo de página (*Cajetín por tipo de página*), el de su marco
(p. ej. uno más estrecho para A4 vertical) o el predeterminado del proyecto.
`{rev.status}`, `{rev.approved_by}` y `{rev.approved_date}` muestran el estado del flujo
de trabajo de revisiones y quién aprobó la última revisión.
**Sello**: un texto grande que cruza cada página y sigue el flujo de trabajo de
revisiones, p. ej. BORRADOR mientras se prepara una revisión, PARA APROBACIÓN en
revisión, nada (o SEGÚN CONSTRUCCIÓN) cuando está liberada (*Formularios › Cajetines ›
Sello*, `set_stamp`).
**Biblioteca de la empresa**: los cajetines, los esquemas de informe y los marcos
propios del proyecto se publican en el servidor de artículos y se cargan en otros
proyectos (*Cajetines › Biblioteca de la empresa*, `parts_company_forms` (`publish`);
publicar requiere el derecho a editar catálogos). Los formularios de informe eligen las
columnas, su orden, el encabezado y las filas por página de informe, la alineación de
cada columna, las columnas sumadas en una fila de totales bajo la última fila (con la
unidad de sus valores) y el sombreado de una de cada dos filas.
**Formularios gráficos**: un formulario de informe del diagrama de bornes, de cables o
de conectores o de la asignación PLC puede cambiar a su forma gráfica – la regleta
dibujada con una caja por borne (los niveles marcados), soportes finales y tapas
finales, puentes y los destinos externos e internos con potencial, texto de función y
lugar vinculado; o cada cable como una banda con sus conductores de extremo a extremo
(potencial, número de hilo, conductores de reserva discontinuos); el esquema de
conectores como una carcasa con una mitad hembra y una mitad macho por pin; la
asignación PLC como un módulo por tarjeta con las entradas que llegan por la izquierda y
las salidas que salen por la derecha.
**Formularios dibujados**: cualquier informe (también los de los plugins) se puede
dibujar con un formulario propio – gráficos dibujados una vez por página y un dibujo de
fila repetido para cada fila, hacia abajo para las listas o hacia un lado (p. ej. un
borne junto al otro); sus textos toman las columnas como marcadores de posición
(`{Device tag}`, `{Part number}` …), `{report}` y los marcadores de posición del
cajetín. El editor de formularios (*Formularios de informe › Formulario dibujado
propio*) parte de la tabla del informe y muestra la página con sus filas reales mientras
se añaden y se arrastran líneas, rectángulos y textos.

![La forma gráfica del diagrama de bornes con soportes finales, tapas finales y los destinos de cada borne](../../images/terminal-diagram.png)

## Comprobación del proyecto

Funciones auxiliares sin función principal, funciones principales duplicadas,
puntos de conexión abiertos, puntos de interrupción sin pareja, cortocircuitos entre
potenciales, números de borne / direcciones de PLC duplicados, dispositivos sin
artículo, más hilos en un punto de conexión de los que admite su borna o hilos demasiado
finos o demasiado gruesos para ella (comprobación *Cables en los puntos de conexión*,
según los datos de puntos de conexión de los artículos, p. ej. de archivos EDZ).
**Gestión de mensajes** (en el espíritu de los esquemas de comprobación de EPLAN): cada
mensaje tiene un código por tipo (`wires.3f9a1c`, el mismo en todos los idiomas) y una
huella por aparición. Haga clic derecho en un mensaje del panel Mensajes para aceptarlo
como revisado (oculto y sin contar hasta que cambie), para cambiar el nivel de su tipo
(error, advertencia, nota, desactivado) o para desactivar su comprobación; los ajustes
(⚙) listan las comprobaciones, los niveles cambiados y los mensajes aceptados (con nota,
usuario, fecha y si aún se notifican). La pestaña filtra por nivel, página y texto y
exporta los mensajes como CSV.
Los ajustes se guardan con el proyecto; las comprobaciones desactivadas y los niveles
cambiados se intercambian como archivo de **esquema de comprobación** (JSON) entre
proyectos. Comandos:
`list_checks`, `check_settings`, `set_check`, `set_message_level`, `accept_message`;
herramientas `export_check_scheme`, `import_check_scheme`; `check` muestra los códigos y
las huellas.
