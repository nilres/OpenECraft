# Gestión de artículos

La base de datos de artículos compartida por todos los proyectos (estilo EPLAN): datos
comerciales, técnicos y de montaje, plantillas de función, puntos de conexión, gráficos
2D / 3D y la importación de datos de fabricantes. Los artículos se comparten entre
instalaciones a través del [servidor de artículos](../parts-server.md).

## Base de datos de artículos

Una **base de datos de artículos** global compartida por todos los proyectos (árbol de
grupo principal de productos / grupo / subgrupo, filtro), con datos generales,
comerciales (precio, unidad de precio, embalaje, unidad de cantidad), técnicos y de
montaje (dimensiones, peso), accesorios y una **plantilla de función** por artículo
(p. ej. bobina A1/A2, contactos principales, 13/14 NA). La base de datos se guarda en el
almacenamiento de la aplicación (escritorio) / almacenamiento local (navegador).

- **Accesorios** que solo funcionan con otros artículos (contactos auxiliares y de
  señalización, bloques de contactos, pestaña *Accesorios*): un artículo lista sus
  accesorios, y un accesorio puede nombrar los artículos a los que se adapta
  (*Compatible con estos artículos*) – cualquiera de los dos lados forma la pareja, de
  modo que un accesorio sirve para muchos artículos y los datos de un fabricante pueden
  añadir uno sin cambiar sus artículos principales (el contacto de señalización Hager
  CZ001 nombra el interruptor diferencial CCA216D, el CDA440D nombra el CZ001). Un
  accesorio se marca como tal y dice dónde se monta: en el frente (apilado en
  profundidad), en el lado izquierdo o derecho (junto al artículo en el carril), bajo el
  artículo como la base sobre la que se apoya (el módulo de componentes univers N bajo
  el interruptor-seccionador con fusibles NH1 Hager LT153: el dispositivo toma la huella
  del módulo y el artículo queda centrado delante de él), integrado en el artículo (la
  bobina de mínima tensión y el interruptor auxiliar en las ranuras del interruptor
  automático h3+ HNT161GR: sin espacio propio, sus bornes están en el interruptor, y sus
  bobinas dibujadas junto al interruptor son funciones del accesorio, no una segunda
  función principal) o en ningún sitio.
  Varios de un mismo artículo base quedan uno junto a otro a lo largo del lado más largo
  del artículo (los transformadores de corriente Hager LVZW150 encajados detrás de los
  tres polos de una regleta seccionadora con fusibles NH LVSG), y artículos base
  distintos, unos sobre otros. En un proyecto, la pestaña *Artículos* del dispositivo
  ofrece los accesorios que se adaptan a sus artículos (*Añadir accesorio*) y muestra el
  artículo sobre el que se asienta cada uno; sus funciones se suman a los contactos del
  dispositivo; la comprobación `accessories` notifica un accesorio en un dispositivo sin
  un artículo al que se adapte; el diseño del armario y la vista 3D lo dibujan en su
  lugar con sus propios bornes, y el dispositivo se hace más ancho o más profundo. Línea
  de comandos / MCP: `list_accessories`, `add_accessory`; servidor de artículos:
  `GET /parts/{number}/accessories`; columnas CSV `Accessory`, `Accessory mounting`,
  `Fits`; referencias BMEcat de los tipos `accessories` y `base_product`; XML de EPLAN
  `P_ARTICLE_ISACCESSORY`.
- **Conjuntos**: un artículo puede constar de artículos componentes con cantidades
  (pestaña *Accesorios*, columna CSV `Components`: `NUM:2|NUM2`); la lista de materiales
  los lista bajo el conjunto (los conjuntos anidados se resuelven, con el precio como
  incluido).
- **Datos de montaje tipificados** por artículo (pestaña *Montaje*): distancias (arriba,
  abajo, izquierda, derecha, frente, CEM, espacio de servicio), taladros de montaje con
  rosca, recortes de puerta y de cara (varios por artículo, cada uno redondo o
  rectangular con su centro desde el medio de la vista frontal, un giro y un radio de
  esquina: tabla y vista previa en la pestaña, forma de texto
  `197 x 141 @ 0/-6.5 R3 15°; Ø16 @ 80/60`), potencia disipada, cuánto sobresale un
  accesorio, fijación, caras, clase de colisión y tipo de borne. Gobiernan las
  comprobaciones del armario, el taladrado, los recortes y el cálculo de climatización;
  el CSV tiene una columna por valor (`Power loss`, `Mounting holes`, …). Los artículos
  más antiguos con estos valores como líneas `Key: value` en los datos técnicos siguen
  funcionando; las importaciones y el catálogo integrado pasan tales líneas a los campos
  tipificados, y el editor de artículos lo ofrece para los artículos existentes.
- **Puntos de conexión** por artículo (designación, posición en la vista frontal, la
  dirección por la que sale el hilo); los artículos que no tienen propios los reciben de
  las plantillas de función según la convención IEC (designaciones impares arriba, pares
  abajo, los dispositivos de puerta por detrás, los módulos de PLC abajo, los
  descargadores de sobretensiones con la tierra y el contacto de señalización remota
  abajo). Los dispositivos de los catálogos incluidos que no se cablean así (Victron,
  Pylontech, Beckhoff, fuentes de alimentación Phoenix Contact) y los dispositivos de
  19 pulgadas (entrada de alimentación por detrás) tienen los suyos; los modelos
  generados dibujan sus bornes en los puntos de conexión. Se editan en el editor de
  artículos (pestaña *Puntos de conexión*: tabla, clic en la vista frontal para colocar
  uno); el diseño del armario cablea hasta ellos; columna CSV `Connection points`:
  `A1:22.5/0/55/up|A2:…`.

## Gráficos y modelos 3D

Vista frontal 2D (SVG) y modelo 3D por artículo:

- **STL**, **OBJ**;
- **STEP**: sólidos y modelos de superficies de archivos AP203 / AP214 / AP242,
  colocados según su estructura de ensamblaje, teselados y convertidos a mm;
- **IGES 5.3**: sólidos y superficies recortadas;
- **Parasolid** (`.x_t` / `.x_b`): cuerpos de archivos de transmisión V13 y posteriores,
  planos, cilindros, conos, esferas, toros y superficies B teselados con exactitud,
  redondeos y otras caras procedurales rellenados desde su contorno;
- **IFC**: cuerpos teselados, facetados y extruidos de todos los productos, colocados y
  convertidos a mm;
- **JT**: las mallas de tiras de triángulos de los archivos JT 8.

Los artículos con dimensiones pero sin modelo reciben uno generado según lo que son:
dispositivos de carril DIN con sus bornes, palanca o pantalla, bornes en sus colores (PE
verde, N azul), módulos de PLC, interruptores-seccionadores con fusibles, electrónica de
montaje mural con pantalla, prensaestopas (cuerpo hexagonal, cuello y cúpula, en latón
niquelado o poliamida según su material y color), dispositivos de 19 pulgadas y baterías
de rack, módulos fotovoltaicos, bombas, motores, depósitos, bombas de calor,
compresores, secadores frigoríficos, válvulas con palanca o actuador, accesorios de
tubería, colectores y unidades de tratamiento de aire. Los dispositivos de los catálogos
incluidos se dibujan según sus fotos de producto: los inversores Victron con sus
cantoneras negras y su banda naranja, el Cerbo GX y el SmartShunt, el distribuidor Lynx,
las baterías de rack Pylontech con asas y bornes, los interruptores-seccionadores con
fusibles, contadores y aparatos modulares Hager, los descargadores DEHN con sus módulos
enchufables, los terminales EtherCAT de Beckhoff, las fuentes de alimentación Phoenix
Contact, los guardamotores y contactores Siemens. Los modelos generados están pintados
(un color por triángulo, `colors` de la malla); los importados toman el color que la
vista 3D da al dispositivo. Los artículos de pie se apoyan en vertical sobre el suelo de
una sala.

## Artículos en el proyecto

- **Artículos del proyecto**: los artículos se copian al proyecto cuando se asignan;
  sincronizar proyecto ← base de datos, transferir proyecto → base de datos, quitar los
  artículos del proyecto no usados.
- Diálogo de **Selección de artículo** con *selección de dispositivo* (solo artículos
  que proporcionan la función principal del dispositivo); varios artículos por
  dispositivo con cantidad (contactor + bloque auxiliar).
- Las plantillas de función gobiernan los **contactos de reserva** en la imagen de
  contactos, «Aplicar designaciones» (números de los puntos de conexión tomados del
  artículo) y la comprobación del proyecto (más funciones colocadas de las que
  proporcionan los artículos, artículos que faltan en el proyecto).

## Importación y exportación

- **CSV** (separado por `;`, nombres de encabezado) y **JSON**, importación y
  exportación.
- Catálogos **BMEcat** (1.2 / 2005, tal como los publican fabricantes y mayoristas,
  p. ej. del EPLAN Data Portal o de los fondos de datos ETIM / eCl@ss): identificación,
  descripción, precio, hoja de datos, la clase ETIM / eCl@ss y cada característica como
  dato técnico; las dimensiones, el peso, la corriente / tensión asignada y la potencia
  disipada van a sus campos (por el nombre de la característica, o por los códigos ETIM
  de anchura, altura y profundidad).
- **XML de artículos de EPLAN**: la exportación de la gestión de artículos de EPLAN
  (`<part>` con atributos `P_ARTICLE_*` y variantes, textos en varios idiomas, grupos de
  productos por sus números de intercambio) se importa como los demás formatos, y los
  artículos se exportan como XML de artículos de EPLAN para el camino de vuelta (gestión
  de artículos, `parts_export --format eplan_xml`). Las plantillas de función se
  convierten en plantillas de función (las entradas y salidas de PLC, en funciones de
  PLC), las definiciones de puntos de conexión en puntos de conexión (los puntos de
  conexión en conectores como `X10.1`) con los hilos que admiten sus bornas (sección
  mínima y máxima en mm², convertida desde AWG donde solo se da esta, y el mayor número
  de hilos), los patrones de taladrado en taladros de montaje: taladros pasantes
  (`Ø3.3`) o roscados (`M4`), mientras que una posición rectangular o un taladro mayor
  que un tornillo es un recorte del artículo, con su lugar y su giro (el `197 x 141` de
  un panel HMI, 6,5 mm por encima del medio de su frente); los registros de taladrado se
  conservan con el artículo tal como son (su subtipo y su tercera dimensión aún no se
  interpretan). Los datos de carril de EPLAN (altura de encaje, desplazamiento del
  centro) o un texto que nombre el carril (*DIN rail*, *Hutschiene*, *TS 35*, *TS 15*,
  *35 x 15*, *G 32*) hacen del artículo uno para un carril de montaje de ese tamaño
  (*Carril de montaje 35 mm*).
- **EDZ** (archivos del EPLAN Data Portal, 7-Zip o ZIP como los publica ABB): el XML de artículos que contiene se
  lee como arriba, junto con las distancias, la potencia disipada y el enlace a la hoja
  de datos. Las macros de ventana 2D incrustadas se convierten en símbolos del proyecto.
  La imagen del producto se conserva con el artículo como un JPEG pequeño (192 px como
  máximo) y se muestra en los detalles del artículo con sus datos de montaje (montaje,
  taladros, recorte, distancias) y los hilos que admiten sus puntos de conexión.
  Una macro 3D del artículo (EPLAN Pro Panel, `…_3D.ema`, `P_ARTICLE_MACRO`) le da su
  modelo 3D cuando no tiene ninguno, también para el XML de artículos de un archivo EDZ
  desempaquetado o de una carpeta de proveedor; una macro cuyos cuerpos están en una
  forma que esta versión no lee se notifica en los diagnósticos de EPLAN del artículo.
  Parts Manager y `parts_import` guardan también los archivos originales como adjuntos.
  Los registros de artículo del proyecto y del servidor llevan las definiciones
  convertidas y los diagnósticos.
  El XML de artículo de un archivo EDZ desempaquetado (`items/partxml/…part.xml`) se lee
  del mismo modo: con los archivos de puntos de conexión y de patrones de taladrado
  junto a él y las macros y las imágenes bajo `items/`.
- **Paquetes de armario** (archivos ZIP de configuradores de armarios como el
  configurador de Rittal): la lista de materiales (`bill-of-material.csv`) da los
  artículos; el primero es el armario configurado (prefijo de fabricante y nombre
  tomados de la macro de EPLAN, p. ej. `RIT.1360000`, tamaño de
  *WHD: 600x600x350 mm*, GTIN, identificador de configuración y sistema de armario
  compacto en sus datos técnicos) con las demás líneas como sus componentes. La vista
  previa de la macro de EPLAN Pro Panel se convierte en su imagen. Los cuerpos de la
  macro (carcasa, puerta, placa de montaje, placa de entrada de cables, bisagras y
  cierre) se convierten en su modelo 3D, puesto en vertical con el frente hacia +z; una
  macro sin cuerpos lo deja al modelo STEP, sin sus zonas restringidas de recorte y de
  colocación (Parts Manager y el editor; `parts_import` solo adjunta el archivo STEP).
  El frente de la placa de montaje (o, si no, la mayor superficie orientada al frente en
  la mitad trasera de la carcasa) da los datos de artículo
  `Mounting origin: 25/12.5/20` y `Mounting size: 550 x 575`, el lado de los bloques de
  bisagra junto a la puerta
  `Door hinge: right`, de modo
  que un armario del artículo recibe esa placa (véase
  [diseño del armario](panel-layout.md)).
  Las zonas restringidas de la macro se convierten en las *Zonas restringidas* del
  artículo (datos de montaje, `placing plate 0/0 550 x 20; cutout left 10/20 30 x 40`:
  tipo, cara, lugar y tamaño en las coordenadas de la cara), cada una en la cara en cuyo
  plano se encuentra; las del marco que rodea la puerta se omiten. El diseño del armario
  las dibuja y las comprueba (véase
  [diseño del armario](panel-layout.md#zonas-restringidas)). La macro, el proyecto de
  EPLAN (`.zw1`), el archivo STEP y los documentos PDF se guardan como adjuntos.
- **Un artículo ya conocido:** un archivo EDZ, un XML de artículos de EPLAN o un
  paquete de envolvente cuyo artículo ya está en la base de artículos, en el servidor
  de artículos o en un catálogo incluido en OpenECraft se combina con ese artículo en
  lugar de reemplazarlo (el editor, Parts Manager y `parts_import`). El artículo
  conocido es el del mismo número, si no el de la misma referencia de pedido del mismo
  fabricante (o el único artículo con esa referencia); conserva su número, y los
  archivos y referencias de la importación lo siguen. Los valores de la importación
  ganan; lo que le falta se mantiene, como los datos de artículo `Fields`, `Rows` y
  `Module spaces` de los armarios y campos de Striebel & John. El grupo y el subgrupo
  se mantienen, los datos técnicos se combinan línea por línea (el `Clave: valor` de la
  importación para la misma clave), los accesorios se unen. Los archivos CSV, JSON y
  BMEcat siguen reemplazando los artículos de sus números. Parts Manager y
  `parts_import` suben un archivo grande en varias solicitudes; un archivo que contiene
  más grande de lo que acepta el servidor de artículos (32 MB, como algunas macros 3D
  de ABB) no se guarda y se nombra, el artículo conserva el modelo leído de él.

### Colocar un símbolo de EPLAN

Abra **Importar artículos** y seleccione el EDZ o el XML de artículo del proveedor; un cuadro abajo a la derecha indica el resultado (el número y la descripción del artículo, o cuántos artículos) y se cierra al cabo de unos segundos o con un clic. En el escritorio, el XML de artículo puede cargar un EMA asociado del mismo directorio. En **Centro de inserción › Dispositivos**, busque el artículo, selecciónelo y después seleccione la variante antes de colocarlo. Cada fila tiene una vista previa y lista las propiedades o las referencias de biblioteca sin resolver. Un EMA suelto también se puede abrir con **Importar biblioteca de símbolos**. Las ventanas de varios dispositivos etiquetadas explícitamente se pueden colocar como macros nativas.

El decodificador lee macros de ventana PXF, bloques de dibujo incrustados, transformaciones afines, gráficos vectoriales básicos, texto y campos de propiedades seleccionados. Sus asignaciones SPECIAL integradas cubren los contactos de PLC y de dispositivo 13, 17 y 20, las formas de contacto de conector y de bus 34, 35 y 37, la forma 22 de dos lados con una designación común y la forma de vista general 40. Estas formas usan los dibujos de contactos de la vista general de símbolos IEC de EPLAN de 2007, págs. 1–2. Los puntos de conexión físicos XYZ siguen perteneciendo al modelo del armario.

El importador lee las referencias locales de ranura de pin y las referencias entrecomilladas de transferencia de potencial de conector/designación. Las referencias recíprocas e inequívocas entre contactos estáticos asignados del mismo dispositivo crean puentes nativos. Las reglas unidireccionales, los destinos que faltan, las identidades de dispositivo inciertas y el comportamiento de dispositivo no compatible quedan como diagnósticos. El importador mantiene las interrupciones de autoconexión, los metadatos de cable/pantalla y los puntos de definición de conexión separados de los segmentos de hilo. El XML de origen no contiene necesariamente una lista de hilos tendidos de origen a destino. [Lógica de puntos de conexión de EPLAN](https://www.eplan.help/en-us/Infoportal/Content/Plattform/2022/Content/htm/xfctdefbrowsergui_d_anschlusslogik.htm)

Despliegue **Colocaciones sin resolver** en una variante de artículo para ver el maestro, el objeto de origen y las designaciones de conector y de borne. Para un archivo o una carpeta de proveedor entera, `cargo run -p openecraft-edz --example inspect -- PATH --references` emite líneas JSON con los pines nativos, las referencias pendientes, los controles de conexión y el estado de transferencia por representación. Las coordenadas de las colocaciones sin resolver son puntos de inserción. Una definición nativa verificada con la clave `user:eplan-lib:LIBRARY:NUMBER:VARIANT` proporciona anclajes de pin explícitos. El material gráfico PDF no registrado se puede conservar bajo `user:pdf-review:LIBRARY:NUMBER` para comparar y no participa en la asignación.

Las variantes parciales se pueden inspeccionar y colocar. Se notifican los símbolos maestros que faltan, las propiedades no compatibles, los estados de marcador de posición y otros objetos no tratados. Los ajustes de capa incrustados, los grosores de pluma explícitos y los colores predefinidos se trasladan. Las paletas personalizadas, los patrones de trazos exactos y los rellenos pueden diferir de EPLAN. La reconstrucción automática del cableado de un proyecto EPLAN completo y la conversión de modelos 3D quedan fuera del decodificador 2D. La reimportación actualiza los gráficos no editados con extremos compatibles. Las ediciones locales y las geometrías de pin existentes tienen prioridad sobre una revisión en conflicto.
