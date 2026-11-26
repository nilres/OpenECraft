# Primeros pasos: de la primera página a la lista de materiales

Este tutorial recorre un pequeño circuito de mando en OpenECraft: páginas, símbolos,
hilos, dispositivos con artículos, bornes, un PLC, la comprobación del proyecto, los
informes, el diseño del armario y la salida. Los comandos se escriben como *Pestaña ›
Comando*: las pestañas de la cinta de opciones, en la parte superior, reúnen los
comandos por temas, y *Archivo* abre la vista Backstage con nuevo, abrir, guardar,
importar, exportar e imprimir. Cada comando se encuentra también por su nombre con
*Buscar un comando* (Ctrl+Shift+P), Alt muestra las teclas de acceso de la cinta y F1
abre la ayuda del panel en uso. La interfaz de usuario está disponible en inglés,
alemán, francés, español y otros 16 idiomas (*Ver › Idioma*); los nombres que siguen
son los españoles.

La forma más rápida de verlo todo de una vez es *Archivo › Nuevo › Abrir proyecto de
demostración*: el mando de una cinta transportadora con circuitos de potencia y de
mando, bornes, un cable, un PLC, un mazo y un diseño del armario. La mayoría de los
pasos siguientes se pueden probar en él.

## 1. Un proyecto nuevo

- *Archivo › Nuevo › Nuevo proyecto* inicia un proyecto vacío con una página de esquema;
  las plantillas que hay debajo parten de una plantilla integrada (marcos de estilo IEC
  o NFPA, estructura, potenciales) y *Desde un archivo de plantilla…* de cualquier
  proyecto guardado. La página de inicio muestra las mismas opciones y los proyectos
  abiertos por última vez.
- *Datos de proyecto › Propiedades del proyecto…* contiene los datos del proyecto que se
  muestran en los cajetines (nombre, cliente, autor) y el marco y el cajetín
  predeterminados.
- El proyecto se guarda como un único archivo `.oec` (*Archivo › Guardar*, Ctrl+S). En el
  navegador, el proyecto se conserva en el almacenamiento del navegador y se descarga con
  *Archivo › Descargar proyecto*. Para el control de versiones (git), la versión de
  escritorio lo guarda también como proyecto dividido (*Archivo › Guardar como proyecto
  dividido…*): una carpeta con el índice `<name>.oecp` y un archivo pequeño por página,
  dispositivo y artículo, de modo que un cambio solo toca los archivos a los que afecta.
- *Archivo › Cerrar el proyecto* (Ctrl+Shift+F4) cierra el proyecto, pregunta si
  guardar los cambios sin guardar y cierra sus páginas de esquema, disposiciones de
  armario y herramientas: solo queda la página de inicio hasta que se abre o se empieza
  un proyecto.

## 2. Páginas y estructura

- El navegador *Páginas*, a la izquierda, lista las páginas; *Inicio › Nueva página de
  esquema* añade una después de la página actual, *Inicio › Nueva página* las de los
  demás tipos: portada, vista general, gráficos, documento (un PDF o una imagen
  incrustados), páginas de informe y de distribución de montaje. Las páginas abiertas
  últimamente son pestañas situadas sobre la página (Ctrl+Tab cambia de una a otra).
- *Inicio › Propiedades de la página…* (o un clic derecho sobre una página) da a una
  página su descripción y su estructura: la función (`=A1`) y el lugar (`+CAB1`). Los
  dispositivos colocados en la página toman esta estructura (`=A1+CAB1-K1`); en la
  página solo se muestra la parte que difiere.
- El botón de árbol del navegador *Páginas* agrupa las páginas por estructura; *Inicio ›
  Renumerar páginas…* las renumera. *Inicio › Nueva subpágina* inserta `3.1`, `3.2`, …
  después de la página actual; al renumerar siguen siendo subpáginas de su página.
- *Inicio › Nueva sección* estructura las páginas como directorios: una sección contiene
  páginas y subsecciones, y los nombres de las páginas empiezan por su número (`2.3.1`
  es la página 1 de la sección 2.3). En el navegador *Páginas*, arrastre una página
  entre dos páginas o sobre una sección (o elija su *Sección* en las propiedades de la
  página) para moverla allí, y arrastre una sección entre secciones o sobre otra; la
  línea o el marco muestra dónde cae, la etiqueta junto al puntero el nombre que recibe,
  una sección contraída se abre cuando el arrastre se mantiene sobre ella, y el espacio
  vacío bajo la lista es el nivel superior. Un clic derecho sobre una sección añade
  páginas y subsecciones, la mueve, la renumera o la elimina, y las páginas la siguen.
  *Renumerar páginas…* vuelve a contar en cada sección y puede numerar antes las
  secciones 1, 2, 3 … En cuanto un proyecto tiene secciones, las subpáginas llevan
  letras (`2.3.1.a`).

## 3. Colocar símbolos

- El *Centro de inserción*, a la derecha (*Insertar › Centro de inserción*, Ins),
  encuentra símbolos de todas las bibliotecas (IEC 60617, NFPA, técnica de fluidos,
  P&ID, plano de instalación, …), macros, dispositivos a partir de artículos y funciones
  no colocadas; `*` y `?` son comodines. Haga clic en una entrada y después en la página
  para colocarla. La colocación continúa hasta Esc o un clic derecho. La estrella guarda
  una entrada entre los favoritos; las usadas por última vez aparecen primero.
- R gira y M refleja el símbolo que se está colocando (o la selección). Las teclas se
  pueden cambiar en *Archivo › Ajustes › Teclas y gestos*, igual que los botones del
  ratón y los modificadores.
- Los identificadores de dispositivo se asignan automáticamente a partir de la letra
  identificativa (`-K1`, `-S2`). Para colocar los contactos de un contactor, seleccione
  primero su bobina: los contactos colocados después le pertenecen y reciben los
  siguientes números libres (13/14, 21/22, …). Los contactos y las bobinas muestran sus
  referencias cruzadas (`/3.4`) en vivo.
- Seleccione un símbolo para editarlo en el panel *Propiedades*, a la derecha: sus
  pestañas contienen el identificador, el texto de función y las designaciones de los
  puntos de conexión (*Dispositivo*), la colocación de sus textos (*Presentación*), los
  artículos del dispositivo (*Artículos*), otras propiedades y las conexiones con las
  referencias cruzadas. F2 o un doble clic muestra lo mismo en una ventana; un clic
  derecho en la página ofrece los comandos para la selección.

## 4. Hilos y potenciales

- W (*Insertar › Hilo*) inicia la herramienta de hilos: haga clic en el inicio, en las
  esquinas intermedias y en el final; Space cambia la dirección de la esquina, Enter o
  un doble clic termina el hilo, Esc o un clic derecho termina la herramienta. Un hilo
  que termina en el vacío lo notifica la comprobación *Extremos de conexión abiertos*.
  Con *Edición › Autoconexión* activada, los símbolos colocados en línea con otros se
  conectan automáticamente.
- Los potenciales (`L1`, `+24V`, `0V`, …) son símbolos de potencial (*Insertar ›
  Potencial* o la biblioteca *Potenciales*); *Conexiones › Potenciales* define su tipo,
  tensión, color de hilo y sección. El navegador *No colocados* lista los potenciales
  definidos que aún no están en ninguna página.
- Los puntos de interrupción (*Insertar › Punto de interrupción*) llevan una red de una
  página a otra; sus referencias cruzadas se ajustan en *Conexiones › Referencias
  cruzadas…* (estrella o cadena).
- *Conexiones › Numeración de hilos…* numera las conexiones (por potencial, por página,
  formatos como `{page}.{n}`); un número escrito en las propiedades de un hilo se
  conserva.

## 5. Artículos

- *Dispositivos › Gestión de artículos* abre la base de datos de artículos (grupos de
  productos, filtro, editor de artículos con datos comerciales, técnicos y de montaje
  tipificados, plantillas de función, puntos de conexión, modelos 2D / 3D, conjuntos).
  Los artículos se importan desde CSV, BMEcat, XML de artículos de EPLAN, archivos EDZ
  del EPLAN Data Portal (*Archivo › Importación*) o el portal de datos (*Datos maestros ›
  Portal de datos*).
- Asigne artículos en la pestaña *Artículos* de las propiedades de un dispositivo
  (*Añadir artículo…*); la selección de dispositivo ofrece solo artículos que
  proporcionan la función principal del dispositivo. *Aplicar designaciones* toma del
  artículo las designaciones de los puntos de conexión. La sección *Dispositivos* del
  Centro de inserción coloca un dispositivo nuevo a partir de un artículo con funciones.
- Un artículo también se puede arrastrar desde el árbol de *Gestión de artículos* o
  desde los resultados del *Portal de datos* a una página de esquema: el símbolo de su
  función principal se coloca allí como dispositivo nuevo con el artículo asignado. Un
  artículo que aún no está en la base de datos de artículos se importa primero (del
  catálogo de ejemplo, o descargado del servidor de artículos). La línea de comandos
  hace lo mismo con `place_part`.
- El navegador *No colocados* lista entonces las funciones de los artículos asignados
  que aún no están en el esquema (por ejemplo, los contactos de reserva de un
  contactor); arrastre una a la página para colocarla con su identificador y sus
  designaciones.

## 6. Bornes y conectores

- Los bornes se colocan como los demás símbolos (biblioteca *Bornes*); los bornes con el
  mismo identificador forman una regleta. *Dispositivos › Regletas de bornes* los
  ordena, añade reservas, puentes (enchufables, de silleta, de cable), soportes y tapas
  finales y bloques de varios niveles.
- El navegador *Regletas / conectores* muestra cada regleta en el orden del carril y
  cada conector con sus pines; un clic muestra un borne o un pin colocado, y las
  reservas y los pines no usados se arrastran a la página.
- *Conexiones › Cables* define los cables; seleccione hilos y use *Asignar al cable*, o
  trace sobre ellos una línea de definición de cable (*Insertar › Definición de
  manguera*).

## 7. PLC

- Las entradas y salidas del PLC se colocan desde la biblioteca *PLC* con el
  identificador de su tarjeta y su dirección. *Dispositivos › Hardware PLC y topología
  de bus* añade estaciones, bastidores y módulos a partir de descripciones de
  dispositivo (GSDML, ESI); *Dispositivos › Resumen PLC* lista todos los puntos con sus
  dispositivos conectados y redirecciona las tarjetas.
- El navegador *PLC* muestra las tarjetas con sus canales; los canales libres de las
  tarjetas con descripción de dispositivo se arrastran a la página.
- *Archivo › Exportación* escribe la tabla de símbolos para la herramienta de
  programación del PLC: TIA Portal, Studio 5000, STEP 7 classic, TwinCAT 3, Control
  Expert, GX Works o IEC 61131-3.

## 8. La comprobación del proyecto

- El panel *Mensajes*, bajo la página, ejecuta las comprobaciones de forma continua
  (dispositivos, conexiones, bornes, cables, PLC, diseño del armario, …) y lista los
  mensajes en una tabla; un clic en una columna ordena, un clic en un mensaje salta a
  su lugar (F4 muestra el panel).
- Haga clic derecho en un mensaje para aceptarlo como revisado, para cambiar el nivel de
  su tipo o para desactivar su comprobación; *Datos de proyecto › Ajustes de
  comprobación* los lista e intercambia esquemas de comprobación con otros proyectos.

## 9. Informes

- *Datos de proyecto › Informes* muestra cada informe (lista de dispositivos, lista de
  materiales, diagrama de bornes, diagrama de cables, esquema de conectores, asignación
  PLC, lista de conexiones, …) como tabla con exportación a CSV y Excel.
- *Insertar como página de informe* (o *Inicio › Nueva página › Nueva página de
  informe*) inserta un informe como páginas del proyecto; se actualizan cuando el
  proyecto cambia. *Datos maestros › Formularios* elige las columnas de un formulario de
  informe o cambia los diagramas de bornes, de cables y de conectores y la asignación
  PLC a su forma gráfica.
- La lista de materiales suma todos los artículos de los dispositivos, los bornes, los
  cables, los mazos, el material del diseño del armario, las punteras y el material de
  marcado; los conjuntos listan sus componentes.
- *Datos de proyecto › Editar en tabla* cambia en bloque dispositivos, conexiones,
  bornes, cables, pines de conector, puntos de PLC, páginas y artículos (filtrar,
  rellenar, buscar y reemplazar, CSV); los navegadores lo abren para sus filas.

## 10. Diseño del armario

- El espacio de trabajo *Diseño del armario* (*Ver › Espacios de trabajo* o el selector
  de la barra de acceso rápido) muestra la pestaña *Espacio de montaje* de la cinta:
  armarios, placas de montaje, carriles, canaletas, pletinas, recortes y los
  dispositivos del esquema; la vista 3D muestra el armario con la puerta, los hilos
  tendidos y el llenado de las canaletas.
- El espacio de trabajo *Cableado* es el asistente de cableado para el taller (los hilos
  en orden con color, sección, longitud y tratamiento de los extremos); el espacio de
  trabajo *Montaje* es el asistente de montaje (los elementos en orden de montaje con
  lugar, tamaño y artículo). *Cableado › Modo tableta* amplía los textos y los botones
  para una pantalla táctil.
- Los patrones de taladrado (*Espacio de montaje › Datos CN*) y el diseño del armario en
  3D (STEP, glTF, JT, IFC, *Archivo › Exportación*) se exportan para la producción.

## 11. Salida

- *Archivo › Exportación* lista todas las exportaciones: PDF (todas las páginas o una
  selección de páginas como `1-3, 5`, con marcadores y referencias cruzadas en las que
  se puede hacer clic), HTML para imprimir, DXF / DWG, SVG, la lista de hilos para
  producción, el marcado (con esquemas de marcado para programas de marcado), KBL de
  mazos y planos de clavos, tablas de PLC, AutomationML y formatos 3D. Las exportaciones
  con opciones las piden primero.
- *Archivo › Imprimir* abre todas las páginas o la página actual en el navegador con su
  diálogo de impresión.
- *Datos de proyecto › Exportar como PDF en idioma* escribe el PDF en uno de los idiomas
  del proyecto (*Datos de proyecto › Idiomas del proyecto*).

## 12. Cambios y comentarios

- *Datos de proyecto › Revisiones* crea revisiones; los cambios desde la última revisión
  liberada se marcan en las páginas y se listan con los valores antiguos y nuevos en el
  resumen de revisiones. Las páginas liberadas quedan bloqueadas.
- El espacio de trabajo *Comentarios* e *Insertar › Comentario* / *Marcado* contienen
  los comentarios y el marcado redline / greenline; *Importar comentarios PDF…* toma los
  comentarios que un revisor hizo en una exportación PDF.

## 13. Automatización

Todo lo anterior está disponible también sin la interfaz de usuario: la línea de
comandos (`cargo run -p openecraft-cli -- help`) y el servidor MCP para agentes de IA
(`openecraft-cli mcp`) ofrecen los mismos comandos, comprobaciones, informes y
exportaciones, y los plugins añaden los suyos (véase [plugins.md](plugins.md)); en el
editor están en *Plugins › Comandos de plugin*, donde la estrella fija un comando a la
pestaña *Plugins*.
