# Macros, configurador y edición en tablas

Reutilización de circuitos como macros con variantes y marcadores de posición, páginas generadas por el configurador y cambios en bloque en tablas y hojas de cálculo.

## Macros

Guarde una selección (macro de ventana) o una página entera (macro de página) en una
biblioteca de macros (guardada con la aplicación, importación / exportación JSON). Las
macros de ventana se colocan con el ratón (vista previa fantasma, los dispositivos
reciben identificadores nuevos), las macros de página insertan una página.
Las macros copiadas llevan consigo las definiciones de símbolos personalizados, las
asignaciones de dispositivos y los artículos de catálogo referenciados. Las vistas
previas resuelven esas definiciones incluidas antes de la inserción. Insertar en otro
proyecto conserva las entradas de catálogo existentes y las ediciones locales de
símbolos.
Los marcadores de posición como `{MOTOR}` en identificadores, textos, textos de función
y designaciones se rellenan a partir de **conjuntos de valores** al insertar.
**Variantes**: una macro puede contener la misma función en varias versiones (A, B, … –
p. ej. arranque directo y estrella-triángulo); guarde una selección con el nombre de una
macro existente y otra variante, y elija la variante al insertar. **Cajas de macro**:
las macros de ventana colocadas recuerdan su macro, su variante, los valores de los
marcadores de posición y sus elementos (marco discontinuo en pantalla, *Ver › Cajas de
macro*); *Macros › Cajas de macro › Actualizar macros modificadas* sustituye el
contenido de las cajas cuya macro cambió en la biblioteca, y una caja editada en la
página se guarda de vuelta en la biblioteca (comando `list_macro_boxes`, herramienta
`update_macro_boxes`).

## Configurador

**Configurador** (*Datos de proyecto › Configurador…*): opciones con sus elecciones
(p. ej. `STARTER` = DOL / SD) y reglas que insertan una macro de página en una variante
con valores de marcadores de posición (`NAME=Pump; POWER={POWER}`) cuando se cumple su
condición (`STARTER=DOL & LINE!=2`, `LINE=1|2`). *Generar páginas* sustituye las páginas
generadas antes (marcadas en las propiedades de la página; desactívelo para conservar
una); los valores de las opciones rellenan los marcadores de posición `{OPTION}`; los
problemas (macro, variante u opción desconocidas) lo detienen.

## Edición en tablas

**Edición en tablas** (*Datos de proyecto › Editar en tabla…*): dispositivos
(identificador, texto de función, descripción, artículo), conexiones (número de hilo,
color, sección, cable, conductor), bornes (regleta, número, texto de función) y el
informe de inspección y ensayo (los resultados, una fila por circuito; sus datos y la
inspección visual, una fila por valor) como
filas; filtrar, editar las celdas en su sitio, rellenar una
columna o buscar y reemplazar en las filas filtradas; exportación / importación CSV para
cambios en una hoja de cálculo (filas por clave, solo se aplican las celdas cambiadas);
las filas de páginas liberadas quedan sin cambios. Renombrar un dispositivo se lleva
consigo sus datos de regleta, de conector y de tarjeta de PLC.
El viaje de ida y vuelta funciona también con archivos **Excel** (`.xlsx`: exportar,
cambiar, importar); las tablas de informe se exportan como libros de Excel desde la
ventana Informes y con `report --format xlsx --path …` (negrita, fila de encabezado
inmovilizada con filtro, los números como números, los códigos como `007` conservados
como texto).
