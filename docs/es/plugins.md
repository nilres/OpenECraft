# Plugins

OpenECraft se amplía mediante el crate `openecraft-api`. Un plugin añade

| Extensión | Qué es | Editor | Línea de comandos / MCP |
| --- | --- | --- | --- |
| Comando | función sobre el proyecto con parámetros tipados, devuelve JSON | panel *Comandos de plugin* (pestaña *Plugins*: cada comando por plugin con una búsqueda; favoritos, comandos usados recientemente y un menú por plugin externo en la pestaña; un formulario para los parámetros) y la búsqueda de comandos | un comando / una herramienta propios |
| Comprobación | añade incidencias a la comprobación del proyecto | panel *Mensajes* | `check` |
| Informe | tabla generada a partir del proyecto | panel *Informes*, páginas de informe y bloques de informes | `report --kind <name>` |
| Exportador | archivo generado a partir del proyecto | *Archivo › Exportación* | `export --format <name>` |

Los nombres van en snake_case y son únicos entre todos los plugins; los comandos no deben
tomar el nombre de una herramienta integrada. Los cambios que un comando hace en el
proyecto son un solo paso de deshacer.
`openecraft-cli plugins` lista los plugins cargados, *Herramientas › Gestionar plugins* en
el editor los activa y desactiva (el editor lo recuerda).

En *Plugins › Gestionar plugins*, introduzca una carpeta de plugin que contenga
`plugin.json`, o una carpeta superior cuyas subcarpetas contengan plugins, y haga clic en
*Cargar plugins*. El editor los carga para la sesión actual y recuerda las carpetas
utilizables para los inicios posteriores. Un plugin vecino que falla no impide recordar los
plugins que se cargaron.
`OPENECRAFT_PLUGINS` sigue disponible para la configuración basada en el entorno.

## Plugins de Rust

Los plugins integrados están en `crates/openecraft-api/src/builtin.rs` y sus submódulos y
se compilan en todas las interfaces (también en la versión web): las comprobaciones,
informes y exportaciones de proyecto de `openecraft-core`, estadísticas del proyecto,
comprobaciones del diseño del armario, fabricación del armario, ciclo de vida de los
artículos, y un plugin por área de comandos de proyecto (páginas, hojas, dibujo, cajas de
dispositivo, dispositivos, conectores, PLC, potenciales, ingeniería de procesos, esquemas
unifilares, tablas, ingeniería, electrotecnia, diseño del armario, salas, edificios, bastidores de 19 pulgadas, tendido, cableado
de campo, cableado, mazos, integración, idiomas, estructura, propiedades, configurador,
bloques de informes, gestión del proyecto). Un área lista sus comandos en una tabla
`COMMANDS` de `Spec` (`builtin/<area>.rs`); la línea de comandos y el servidor MCP los
ofrecen como herramientas junto a las herramientas de sesión de `cli/`. Las propias
comprobaciones del proyecto son entradas del registro de `openecraft_core::checks` (un
`Check` por área, con un nombre en snake_case), y las clases del registro, los parámetros y
las tablas se definen en
`openecraft_core::{ext, params, table}`. Un plugin implementa `Plugin`:

```rust
use openecraft_api::{param, Info, Kind, Plugin, Registry, Table, Text};
use openecraft_core::issue::{Issue, Severity};
use openecraft_i18n::tr_const;

pub struct Spares;

impl Plugin for Spares {
    fn info(&self) -> Info {
        Info { id: "spares".into(), name: Text::Borrowed(tr_const!("Spare parts")), version: "1.0".into(), description: Text::Borrowed("") }
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        reg.command("set_spares", tr_const!("Sets the spare quantity of a part"), vec![param("number", Kind::Str, tr_const!("Part number"))], |prj, args| {
            // change `prj`, return JSON
            Ok(serde_json::json!({ "part": args.req("number") }))
        })
        .check("spares_missing", tr_const!("Spare parts"), |prj, analysis| Ok(vec![Issue::plain(Severity::Info, "nothing missing", None)]))
        .report("spares", tr_const!("Spare parts"), |prj, analysis| Ok(Table::new([tr_const!("Part")])))
        .keywords(["IEC 60300-3-4", "Spares"])
        .exporter("spares_csv", tr_const!("Spare parts (CSV)"), "csv", |prj| Ok(String::new()));
        Ok(())
    }
}
```

`keywords` son términos de búsqueda adicionales del informe añadido en último lugar, sin
traducir: las normas que sigue y otros nombres. El panel *Informes*, `list_reports`
(`search`) y el sitio web encuentran un informe por cada palabra de la búsqueda en su
título tal como se muestra, en su nombre o en una palabra clave, sin distinguir mayúsculas,
espacios ni puntuación (`Report::matches`), de modo que `VDE 0100 600` encuentra el informe
de inspección y ensayo por `DIN VDE 0100-600`.

Los textos dados como `&'static str` son entradas del catálogo en inglés (`tr_const!`,
véase AGENTS.md) y se muestran traducidos. Las incidencias de una comprobación llevan o
bien un texto terminado (`Issue::plain`) o bien, como las comprobaciones integradas, una
entrada del catálogo con sus argumentos
(`openecraft_core::issue!(Severity::Warning, target, "Cable {} defined {} times", tag, n)`),
que `Issue::message()` representa en el idioma actual; la herramienta `check` informa de la
plantilla como `key` junto al mensaje.

## Plugins externos

Un plugin externo es un programa en cualquier lenguaje en una carpeta propia:

```
my_plugin/
  plugin.json
  main.py
```

```json
{
  "id": "my_plugin",
  "name": "My plugin",
  "version": "1.0",
  "description": "What it does",
  "api": 1,
  "run": ["python3", "main.py"]
}
```

Las carpetas que contienen carpetas de plugins se indican en la variable de entorno
`OPENECRAFT_PLUGINS` (separadas como `PATH`) para el editor y la línea de comandos, o con
`openecraft-cli --plugins <folder>`. Los plugins externos necesitan procesos y no están
disponibles en el navegador.

### Protocolo

Para cada petición OpenECraft inicia el programa (`run`, en la carpeta del plugin), escribe
un objeto JSON en su stdin y lee un objeto JSON de su stdout, ambos en UTF-8
(Python se inicia con `PYTHONUTF8=1` y `PYTHONIOENCODING=utf-8`; donde `python3` no está
instalado, como en Windows, se inicia `python` en su lugar). Cada petición
tiene `"api": 1` y un `method`. Una respuesta `{"error": "message"}` (o un código de salida
distinto de cero, con el mensaje en stderr) es un error. `project` es el proyecto como en
un archivo `.oec`. Un programa que no ha respondido al cabo de 5 minutos se detiene y la
petición falla. Los plugins se ejecutan con los derechos del usuario: cargue solo carpetas
de plugins en las que confíe.

Las peticiones incluyen además `project_features`, una matriz que anuncia lo que admite el
modelo del anfitrión en ejecución (`PROJECT_FEATURES` en `openecraft-api`); cada
característica se describe con los datos a los que afecta, por ejemplo
`directional_pin_contacts`, `explicit_wire_junctions`,
`directed_junction_paths`, `fluid_port_roles`, `wire_hide_labels` y
`scoped_connections`. Los importadores pueden exigir características antes de devolver un
proyecto modificado; usar una CLI actualizada aparte no da estas características a un
editor más antiguo.

| Petición | Respuesta |
| --- | --- |
| `{"method": "describe"}` | `{"commands": [{"name", "help", "params": [{"name", "kind", "required", "help"}]}], "checks": [{"name", "title"}], "reports": [{"name", "title", "keywords": ["…"]}], "exporters": [{"name", "title", "extension", "params"}]}` (todas las listas son opcionales, también `params` de los exportadores y `keywords` de los informes) |
| `{"method": "command", "name", "params": {…}, "project": {…}}` | `{"result": <any JSON>, "project": {…}}` – `project` solo si el comando lo modificó |
| `{"method": "check", "name", "project"}` | `{"issues": [{"severity": "error" \| "warning" \| "info", "message", "page": <page index>, "item": <item id>}]}` – `page` / `item` opcionales; `page` sola señala la página, una página que el proyecto no tiene no señala nada |
| `{"method": "report", "name", "project"}` | `{"headers": ["…"], "rows": [["…"]]}` |
| `{"method": "export", "name", "params": {…}, "project"}` | `{"content": "file content"}` (texto; `params` contiene lo que pasó la interfaz: por convención `page`, el nombre de una página, y `file`, el nombre del archivo) |

Clases de parámetros: `str`, `int`, `num`, `bool`, `ids` (lista de identificadores de
elemento), `points` (lista de `[x, y]` en mm), `object` (objeto JSON). `required` es `true`
por defecto.

Un comando que responde con un error deja el proyecto sin cambios, también en el editor y
en el servidor MCP.

`describe` se pide una vez cuando se carga el plugin; las comprobaciones se ejecutan tras
cada cambio del proyecto en el editor, por lo que deben responder con rapidez. Los ajustes
de comprobación del proyecto se aplican también a las incidencias de los plugins: una
comprobación se puede desactivar por su nombre, y sus mensajes sin plantilla comparten el
nombre de la comprobación como código (los cambios de nivel se aplican entonces a todos
ellos) y se pueden aceptar uno a uno.

Los artículos pueden fijar `symbol` a la clave de una definición de símbolo del proyecto.
La lógica compartida de colocación de artículos usa esa definición en el editor, la CLI y
MCP. Una clave vacía mantiene la selección existente basada en plantillas. `source_data` en
los artículos y en las definiciones de símbolo puede conservar identificadores del
importador y propiedades del proveedor como JSON.
`Project.import_sources` asigna identificadores de origen estables a conjuntos de datos
JSON, incluidos los registros que no tienen una superficie de edición nativa. Estos campos
son opcionales para los archivos más antiguos.

Los clips de macro nativos portables pueden llevar `symbol_defs`, una matriz de
definiciones de símbolo del proyecto. Insertar un clip instala las definiciones que faltan
y renombra las claves entrantes en conflicto, conservando las ediciones locales y
actualizando las instancias entrantes. El mapa opcional `devices` y la matriz `parts`
llevan las asignaciones de dispositivos y los registros de catálogo.
La inserción renombra las asignaciones de dispositivos junto con sus instancias e instala
los artículos que faltan sin sobrescribir los registros de catálogo locales. `Part.symbol`
sigue a las definiciones renombradas.
Las vistas previas de macro resuelven las definiciones incluidas antes de consultar el
proyecto de destino.
El
comando `insert_macro` acepta una macro nativa como cadena JSON más `page`, `x`
e `y`. El editor, la CLI y MCP exponen el mismo comando. Los objetos de parámetros grandes
de la CLI se pueden suministrar como `--json @parameters.json` para evitar los límites de
argumentos del intérprete de comandos.
Los pines de los símbolos de usuario pueden fijar `internal: true` para los nodos de
fijación de puentes. Siguen participando en los puentes locales pero no aparecen como
extremos físicos de dispositivo. Un hilo que pasa por un pin interno se conecta a él; los
demás pines necesitan allí un extremo de hilo.
`list_nets` expone la pertenencia nativa por identificador de símbolo e índice de pin
basado en cero; su matriz `pins` contiene los extremos físicos de dispositivo. La matriz
adicional `connection_points` incluye los puntos de fijación de nodos de conexión y de
puentes internos que pertenecen a una red cableada o física, sin añadir extremos de
dispositivo ni redes vacías. Los importadores pueden verificar los caminos que terminan en
estos puntos mediante el mismo comando en el editor, la CLI y MCP.
`bridge_terminals` es verdadero por defecto y se puede desactivar para tramos de hilo
individuales.
`list_connections` devuelve los destinos físicos de hilo de pin a pin como objetos `from` y
`to` que contienen un identificador de símbolo y un índice de pin basado en cero, junto con
el potencial, el número de conductor, el color, la sección, el cable y el núcleo. Los
puentes internos de dispositivo conservan la continuidad eléctrica sin crear hilos externos
de fabricación.
Los símbolos de nodo de conexión de usuario pueden fijar `directed_junction: true`: sus
`bridges` describen entonces también los caminos físicos de destino entre puertos
separados. `set_symbol_junction_paths`
acepta una clave de símbolo y `paths: {"links": [[0, 1], [0, 2]], "directed": true}`;
el editor de símbolos expone los mismos caminos. La propiedad de página
`page.use_drawn_wire_targets` conserva las derivaciones explícitas en los puntos de
conexión de los dispositivos y sigue los hilos a través de los puentes internos de los
dispositivos. En esas páginas y en las páginas con nodos de conexión explícitos, los pines
de un símbolo en la misma posición cuentan como una sola fijación.
Los símbolos de usuario pueden suministrar `pin_directions`, un mapa de índices de pin
basados en cero a máscaras de dirección locales (arriba 1, derecha 2, abajo 4,
izquierda 8). Las entradas omitidas permiten todos los lados; cero desactiva los contactos laterales
automáticos. Las direcciones siguen el giro y la simetría de la instancia. Los extremos de
hilo guardados siempre conectan. `set_pin_connection_directions`
acepta `symbol`, `pin` y `directions` (`"up,right"`, `"all"` o `"none"`); el
editor de símbolos expone las mismas direcciones. Los hilos automáticos nuevos usan estas
direcciones.
La propiedad de página `page.explicit_wire_junctions` hace que los cruces de un extremo con
el interior de un hilo no conduzcan salvo que un nodo de conexión los una, y los puertos de
dispositivo conectan solo con extremos de hilo dibujados. Las fijaciones internas de
carril y los puertos de nodo de conexión conservan los contactos laterales. Los puntos de
conexión dibujados y las comprobaciones de extremos abiertos usan las mismas reglas.
`device_box_roles` añade `SymbolInst.box_role` a las cajas de dispositivo: `extension` añade puntos de conexión al
dispositivo del identificador de la caja, `view` muestra puntos de conexión que ya tiene y los conecta entre
páginas por dispositivo y designación; el valor por defecto `device` se omite en el archivo.
Una instancia de interrupción puede almacenar `function.connection_scope` en sus `values`.
Los puntos se conectan y se referencian de forma cruzada solo con otros del mismo ámbito y
la misma etiqueta; un ámbito vacío mantiene el comportamiento existente en todo el
proyecto. La etiqueta sigue siendo el nombre mostrado. Esta propiedad se puede editar
mediante la base de datos de propiedades en el editor y los comandos de propiedades
compartidos de CLI/MCP.
Los símbolos de usuario pueden suministrar `fluid_ports`, un mapa de índices de pin basados
en cero a
`supply`, `pressure`, `working`, `exhaust`, `return`, `pilot`, `suction`, `leakage`,
`measuring` o `passage`. Una entrada omitida mantiene la inferencia basada en la
designación.
Estos roles sobreviven a las copias de biblioteca, a las macros portables y a la
serialización del proyecto;
`list_symbols` los expone y `set_fluid_port_role` edita o borra una entrada
(`role: "automatic"`) mediante el panel Plugins del editor, la CLI y MCP. El editor de
símbolos ofrece también el rol en el punto de conexión de fluido seleccionado.
`FieldKind::TagName` muestra el nombre del dispositivo sin la estructura heredada mientras
sigue vinculado al identificador de la instancia. Los marcos pueden fijar `visible: false`
cuando el dibujo importado incluye su propio borde y cajetín; la geometría del papel sigue
disponible. `ZeroBasedNumbers` se admite para las etiquetas de columna de los planos de
origen.
Las imágenes de dibujo PNG, incluidas las imágenes con paleta y la transparencia, se
exportan a PDF con una máscara suave de imagen.
`Func::Graphic` mantiene los dibujos decorativos fuera de la identificación de dispositivos
y de la evaluación de la lista de redes, conservando primitivas de símbolo editables.

Las definiciones de símbolo del proyecto pueden contener una primitiva de dibujo `Image`.
Lleva un `name` de archivo, `data` codificados en base64 y cuatro `corners` en
coordenadas locales del símbolo, ordenadas arriba a la izquierda, arriba a la derecha,
abajo a la derecha, abajo a la izquierda.
El editor y la salida SVG/PDF aplican a estas esquinas el giro y la simetría del símbolo.
Las representaciones existentes de primitivas de dibujo se admiten junto con las imágenes.

Los símbolos de usuario pueden especificar un `domain` de `Electrical`, `Fluid` o `Process`.
Los archivos de símbolos más antiguos toman `Electrical` por defecto. Los puntos de conexión
de fluido y de proceso participan en las redes de fluido, y conectarlos a un pin eléctrico
se notifica como una red mixta.

Las primitivas `Field` aceptan un `angle` opcional en grados con la misma convención que el
texto de los símbolos. Omitirlo mantiene el comportamiento histórico de etiqueta
horizontal. Su `kind` puede ser también `{"Pin": 0}` (índice de pin basado en cero), que
sigue la designación de conexión de la instancia colocada. La designación mostrada de un
borne usa `Label`.

La función `Auxiliary` representa una función de dispositivo adicional con pines eléctricos
y una referencia cruzada a su función principal, sin inventar un tipo de contacto.
`Junction` no tiene identificador de dispositivo y conecta solo los caminos listados en sus
puentes. Sus puntos de conexión son nodos locales de tendido y se excluyen de los extremos
de dispositivo de la lista de redes.

Los hilos pueden fijar `hide_labels: true` cuando sus números de hilo y sus etiquetas de
cable/núcleo ya están dibujados como anotaciones separadas. Las propiedades eléctricas y
los informes siguen disponibles. El panel de propiedades del hilo y el comando
`update_wire` pueden restaurar las etiquetas automáticas; los hilos nuevos muestran las
etiquetas por defecto.

Hay un ejemplo completo en [`examples/plugins/page_notes`](../../examples/plugins/page_notes).

### Coordenadas de los dibujos CAD

Las herramientas `import_dxf` e `import_dwg` de CLI/MCP devuelven `source_bounds` (los
puntos `min` y `max` en milímetros del dibujo decodificado, Y hacia arriba) y `page_origin`
(la posición en la página del límite superior izquierdo del origen), junto con `scale`.
Para un punto de página importado `p`, sus coordenadas de origen son `source_bounds.min.x +
(p.x - page_origin.x) / scale` y `source_bounds.max.y -
(p.y - page_origin.y) / scale`. Esto incluye la conversión de unidades del dibujo y permite
a los importadores externos aplicar una ventana gráfica definida en el origen. La
colocación existente en la página y los valores por defecto de los parámetros no cambian.

### Macros de ventana de EPLAN

El plugin de dibujo acepta XML de macros de ventana PXF con `import_ema` (parámetro `xml`). Su resultado lista las definiciones de símbolo nativas, los tipos de función, los identificadores de variante, las macros de varios dispositivos y las referencias sin resolver. Una reimportación conserva las definiciones editadas localmente. El comando `place_part` acepta una clave de símbolo `variant` opcional. La herramienta de archivos `import_eplan_symbols` de CLI y MCP lee un EMA, un EDZ, un XML de artículo acompañante o una carpeta de proveedor con un número de artículo seleccionado.

`product_designation_fields` anuncia la clase `{"Product":N}` de `Prim::Field`. Muestra los N últimos niveles de la designación de producto del identificador de la instancia actual, excluidos los aspectos de función y de ubicación; cero se comporta como uno. El campo se actualiza tras un renombrado y sobrevive a la serialización de la biblioteca de símbolos y del proyecto. El editor de símbolos expone el número de niveles; los plugins usan la misma primitiva serializada mediante las definiciones de símbolo. Los anfitriones sin esta característica no pueden cargar estas variantes de campo.

`function_text_fields` anuncia la clase `"FunctionText"` de `Prim::Field` y el `default_function_text` opcional de los símbolos de usuario. La colocación compartida inicializa el texto de función nativo de la instancia a partir del valor por defecto de la biblioteca. Los campos representan el valor actual de la instancia, incluido el texto de varias líneas transformado; el bloque automático de identificador omite un texto de función duplicado cuando existe un campo explícito. `list_symbols` devuelve el valor por defecto de la biblioteca.

`function_part_selection` anuncia las referencias a artículos asignados específicas de cada función. La propiedad `function.part_index` selecciona una posición basada en uno entre los artículos del dispositivo y registra su número de artículo en `SymbolInst.values["function.part_number"]`. Quitar artículos anteriores no cambia esta referencia. Un artículo seleccionado que se quita da una selección vacía, la posición cero y una incidencia del proyecto; fijar una posición existente la repara, mientras que borrar la posición restablece la selección automática del primer artículo. Las propiedades `part.*` de una función leen solo su artículo seleccionado; las propiedades de dispositivo conservan su comportamiento agregado. `function.manufacturer` admite un valor explícito editable que tiene prioridad, con el artículo seleccionado como alternativa, y `function.selection_text` conserva el texto de selección introducido con independencia de los números de tipo del catálogo. El comando registrado `list_function_parts` devuelve el artículo seleccionado de cada función y los valores de atributo editables para una inspección por lotes.

`property_blocks` anuncia `Prim::PropertyBlock` con filas ordenadas `Property(key)`, `Field(FieldKind)` y `Text(value)`, los vectores locales `pos` y `step`, el `size` de la fuente, el `anchor` horizontal, el `angle` del texto, `align` (`Start`, `Middle`, `End`) y un `color` RGB opcional. El representador compartido del lienzo y de las exportaciones resuelve las propiedades sobre la función original con su identificador completo, omite los valores vacíos y distribuye todas las líneas antes de aplicar las transformaciones del símbolo. Los `default_values` de los símbolos de usuario inicializan las propiedades editables de la instancia durante la colocación compartida; `list_symbols` expone los valores por defecto y los bloques. El editor de símbolos admite la creación y la edición de bloques y de propiedades por defecto.

La característica de proyecto `wire_type` conserva la construcción del conductor en `Wire.wire_type`.
`update_wire` acepta `wire_type`; `list_connections` lo devuelve para cada conexión
física. La tabla de conexiones editable añade Tipo de conductor después de sus nueve
columnas originales; los informes de conexiones incluyen el mismo valor. Vacío significa
sin especificar.

La característica `physical_wire_routes` anuncia el seguimiento de conductores con
conocimiento de los contactos.
`list_connections` incluye `route_traced` y `wires` (identificador nativo de página e
identificador de hilo).
Cuando `route_traced` es falso, la lista de hilos es el ámbito de propiedades existente de
toda la red, no un camino demostrado de punto a punto. Las páginas con nodos de conexión
explícitos mantienen separados los vértices locales de cada hilo hasta que un extremo, un
nodo de conexión o un contacto direccional de pin válidos los unen. Estas reglas se aplican
también cuando un cruce pertenece a una misma red por otro camino.

`trace_wire_routes` acepta `routes: {"pairs": [{"from": {"symbol": 12, "pin": 0},
"to": {"symbol": 34, "pin": 0}}]}` y devuelve un resultado ordenado para cada par.
Sigue los extremos dibujados locales exactos, incluidos los puntos de interrupción
virtuales, sin saltar entre nombres iguales de potencial o de interrupción. Cada resultado
tiene `route_traced` y `wires` con los identificadores nativos de página/segmento y sus
propiedades de conductor.
Esto verifica una derivación concreta aunque la lista de cableado físico elija otro camino
válido a través de una red con nombre. Un extremo no válido o desconectado no tiene camino
seguido.

### Canales completos de los módulos de PLC

La característica de proyecto `multi_channel_plc` anuncia `UserSymbol.plc_channels`, expuesto también por `list_symbols`. Cada definición de canal contiene una `key` estable, un `index` direccional basado en cero, una dirección `input` opcional, `analog`, `data_type`, `designation`, los `pins` físicos basados en cero y un `signal_pin` opcional. Los metadatos existentes `plc_channel` de un solo canal siguen admitiéndose.

Los valores de instancia de los canales usan las claves de propiedad basadas en uno `function.plc_address[1]`, `function.plc_symbol[1]`, `function.plc_comment[1]` y `function.plc_data_type[1]`. Las definiciones de biblioteca deben dejar vacíos las direcciones configuradas y los nombres de señal. Los bloques de propiedades pueden referirse a estas claves para mostrar la configuración actual. `get_plc_points` devuelve una fila por canal con sus índices de pin físicos; `set_plc_channel` edita un canal mediante un identificador de símbolo y un índice de canal basado en cero. Las claves de fila de la tabla de PLC usan `symbol_id:channel_index` para los módulos completos y conservan los identificadores numéricos de símbolo para los puntos individuales heredados. Los canales en blanco y sin dirección se excluyen de las exportaciones de variables de PLC.

### Representaciones vinculadas de canales de PLC

La característica `linked_plc_channels` añade `SymbolInst.plc_bindings`, un mapa de índices de canal basados en cero a `{group, primary}`. Un grupo no vacío identifica el mismo canal lógico en las representaciones con el mismo identificador completo de dispositivo. Fije `primary` en la representación eléctrica cuyos pines físicos y cuya ubicación deben usarse para la navegación de E/S. Los identificadores completos de dispositivo distintos mantienen independientes los dispositivos copiados.

El resumen PLC, los recuentos de hardware, las tablas y las exportaciones de variables usan canales lógicos. Las ediciones de canal, las ediciones de propiedades de canal indexadas, el redireccionamiento de tarjetas y la importación de tablas de símbolos actualizan todas las vistas vinculadas; una vista liberada rechaza toda la edición del canal. Los valores introducidos existentes siguen almacenados en cada vista. Los campos mostrados vacíos pueden recurrir a otra vista vinculada. `get_plc_points` devuelve por defecto filas de E/S lógicas; `representations:true` devuelve todas las representaciones del esquema con sus propios valores almacenados, índices de pin físicos y metadatos de vinculación para verificar una importación sin pérdidas.

La característica `plc_channel_indices` añade el `ChannelBinding.index` opcional, el número de canal de hardware relativo a la tarjeta. Almacene el número de hardware del resumen del módulo en cada vista vinculada, incluso cuando una vista eléctrica contiene un solo canal local. `get_plc_points.hardware_index` expone este número por separado del índice `channel` local del símbolo que usa `set_plc_channel`. El redireccionamiento y la designación lógica del canal usan el número de hardware. Las vinculaciones más antiguas sin un número explícito recurren a la definición de módulo vinculada más completa, lo que mantiene utilizables los proyectos continuados tras reordenar las hojas.

Los artículos que representan módulos de PLC completos usan una `FunctionTemplate` por canal, con `PlcInput`, `PlcOutput`, `PlcAnalogInput` o `PlcAnalogOutput` y las designaciones de los pines físicos de ese canal. Deje vacías las descripciones de señal reutilizables. El uso de funciones nativo y los cálculos de reserva cuentan una vez los canales de esquema vinculados y excluyen los gráficos de armario. `list_device_descriptions` incluye las capacidades de función analógicas; `list_unplaced` reconoce los canales de módulo ocupados por la identidad de hardware, con independencia del texto de dirección configurado.

`set_plc_card` conserva las direcciones de canal introducidas para los cambios que solo afectan a los metadatos. Suministrar `start_byte`, `output_byte`, `readdress:true` o un `slot` de Rockwell reconstruye las direcciones a partir de la configuración. Una descripción de cabecera importada con puertos vacíos y sin bus configurado mantiene una lista de puertos vacía; no adquiere una interfaz PROFINET inferida.

La conversión del esquema de estación incluye los canales digitales de los módulos completos y sus representaciones vinculadas. Conserva cada desplazamiento de dirección configurado y cada nombre de señal. El texto de dirección personalizado que no es una dirección digital admitida no cambia.

La característica `explicit_panel_mounts` admite `EnclosureKind::Assembly`, un lienzo de montaje abierto sin geometría implícita de carcasa ni de placa, e `ItemKind::MountingPlate`, un elemento explícito con posición, tamaño y grosor definidos. Los elementos de placa explícitos suprimen además la placa por defecto de un armario. Las coordenadas siguen en el sistema de referencia común de la envolvente. `PanelItem.mounted_on` hace referencia a otro elemento de la misma envolvente; `mounting_explicit:true` distingue un elemento explícitamente libre de la colocación en carril inferida heredada. `rail_mount_override:true` conserva la referencia de la instancia durante la carga nativa y la actualización de artículos. `rail_vertical:true` mide la referencia transversal al carril a lo largo de x en lugar de y.

Use `add_enclosure` / `set_enclosure` con `assembly:true`, `place_mounting_plate` y `set_panel_item` con `mounted_on`, las dimensiones, `rail_offset`, `clip_on`, `rail_vertical` y `rail_mount_override`. Los desplazamientos y los cambios de profundidad del elemento padre arrastran a los descendientes una vez, incluidos los soportes anidados. Los elementos separados explícitamente mantienen su posición. Las listas de elementos exponen los campos de montaje. Las copias portables y las copias en patrón reasignan las referencias de montaje internas; eliminar un soporte separa a los hijos que quedan. Tanto los carriles horizontales como los verticales admiten el ajuste y la disposición. Los datos de proyecto originales sin estos campos opcionales conservan el comportamiento heredado.

La característica `individual_panel_terminals` añade `PanelItem.terminal`: una designación no vacía liga una huella y su propio artículo a un borne de una regleta. Vacío conserva la disposición de la regleta completa. Los puntos físicos, las mallas, el cableado virtual y los extremos de cable 3D resuelven cada borne por separado; las colocaciones repetidas ambiguas no seleccionan un elemento arbitrario. Los artículos seleccionados específicos de cada función dirigen también la disposición de las regletas de bornes. `place_panel_device` crea un dispositivo o un borne con dimensiones explícitas; `set_panel_item` puede editar su designación de borne. `get_panel_points` expone los puntos físicos generados o exactos mediante la API compartida.

La característica `explicit_panel_geometry` conserva `PanelItem.single_part`: la huella representa solo su propio artículo de catálogo incluso sin designación de borne. Las dimensiones de catálogo que faltan pueden generar puntos de conexión no exactos usando la huella de la instancia, sin reescribir el catálogo. Varias vistas físicas se resuelven por el artículo seleccionado solo cuando es único. `Enclosure.no_implicit_plate` suprime la placa por defecto, mientras que `mounting_origin: [x,y,z]` da una referencia explícita de la cara de montaje en coordenadas de la envolvente. Los montajes abiertos toman por defecto la referencia cero y toda la extensión del lienzo; sus espacios de tendido no tienen paredes de armario. Los comandos de añadir y fijar envolventes exponen `implicit_plate`, `mounting_origin` (objeto con x/y/z; vacío lo restablece), `mounting_size` (objeto con width/height para una placa que no está centrada; vacío lo restablece) y `door_hinge` (`left` o `right`, el borde en el que está abisagrada la puerta vista desde el frente), y los comandos de colocación y edición de dispositivos exponen `single_part`. Las placas explícitas aportan su propio artículo a la lista de materiales.


La característica `cached_impossible_routes` añade `layout.routing.reuse_failed_searches` (falso por defecto), expuesto también por el comando `routing_rules`. Un proyecto puede conservar explícitamente los resultados imposibles de la búsqueda anticipada tras los cambios de llenado de las canaletas en lugar de repetir búsquedas exhaustivas. Los proyectos ordinarios mantienen el comportamiento de reintento existente. Los caminos correctos en caché siguen requiriendo las comprobaciones de capacidad y de distancia.

La característica `literal_part_quantities` añade `PartRef.total`. Un valor verdadero hace
de `count` la cantidad literal para todo el dispositivo o la regleta (incluido un cero
explícito), en lugar de multiplicar los artículos de borne por las posiciones de la
regleta. El valor falso por defecto conserva el recuento heredado, incluido el valor
heredado de cero como uno. Las referencias automáticas primarias y las literales
adicionales pueden usar el mismo número de artículo; sus cantidades se suman, de modo que
los bornes recién dibujados siguen aumentando el material primario sin multiplicar las
reservas. Los dispositivos literales solo de material aparecen también en `list_devices` y
en la lista de materiales.
Los comandos compartidos `add_device_part` y `set_device_part_quantity` exponen tanto la
cantidad como el modo; el segundo usa un índice de asignación basado en 1. El editor de
artículos del dispositivo expone también el indicador de cantidad total. Las referencias
literales de cero no aportan capacidad de función, piezas, peso, pérdidas de calor ni
cantidad de material.

`Enclosure.device_tag` liga un cuerpo físico a su asignación de dispositivo primaria, como
el `tag` de un elemento estructural del armario. Su artículo automático primario lo cuenta
el diseño físico, y los artículos literales adicionales se cuentan por separado. Cambiar el
artículo físico mediante `set_enclosure`, `set_panel_item` o la sustitución de perfiles
actualiza su asignación primaria y las vinculaciones de artículos de función conservando
las reservas literales. Los comandos de añadir y fijar envolventes exponen `device_tag`;
las ediciones de elementos exponen `part`.

La característica `material_ownership` añade `owner_device` y `bom_excluded` a los
elementos del armario y a las envolventes, expuestos por los comandos de consulta y de
edición del armario. La propiedad registra el dispositivo que da cuenta de un artículo y
nunca cambia `mounted_on`. Una exclusión de colocación suprime solo su entrada de material
físico; las propiedades de dispositivo editables
`device.material_owner` y `device.exclude_from_bom` conservan los dispositivos solo de
material y pueden suprimir todo el material asignado a un dispositivo. Las referencias de
propiedad siguen el renombrado de los dispositivos. Las macros de armario reutilizables
borran los identificadores de propietarios externos y conservan la exclusión de colocación
explícita. Las consultas del armario exponen `mounting_height` sobre el soporte explícito
(null si se desconoce); los separadores generados de los carriles sobre un soporte
explícito usan esa altura y requieren
`layout.hardware`, en lugar de tratar el grosor del soporte como un separador.

Las vinculaciones de dispositivos del armario participan también en `list_devices` y en la
lista de materiales cuando no hay ninguna función de esquema dibujada, con independencia de
cualquier artículo adicional.
Un borne colocado individualmente que aún no está representado en el esquema aporta su
artículo primario una vez por designación de borne. Las vistas físicas repetidas y dibujar
de nuevo el mismo borne no añaden otra pieza. La colocación existente de la regleta
completa mantiene su expansión de bornes del esquema.

La característica `portable_macro_materials` garantiza que la inserción de macros portables
conserva los valores de las propiedades de dispositivo y las referencias automáticas
y literales distintas al mismo artículo, incluido el cero. Al ampliar un dispositivo
existente, se completan los modos de referencia y las propiedades que faltan, mientras que
las cantidades y los valores de propiedad locales no cambian. Los artículos que el
dispositivo no usa se
añaden solo cuando aún no tiene ninguno o cuando una función del clip los selecciona, salvo que se haya aceptado explícitamente como se indica más abajo. Los propietarios de material dentro de un clip
siguen la estructura de la página de destino y el renombrado por colisión; los propietarios
externos se borran.
Los identificadores de dispositivo entrantes se reservan antes de asignar sustitutos,
incluidos los identificadores existentes solo de material. Las vistas auxiliares
conductoras siguen el identificador de su principal.
Los marcadores de posición de las propiedades de dispositivo se resuelven junto con los
identificadores y los números de artículo de la macro.

La característica `explicit_macro_part_additions` añade el `Clip.device_part_additions` opcional, un mapa de identificadores de dispositivo a matrices de números de artículo. Solicita explícitamente añadir las referencias de accesorios **literales** que faltan y que ya están presentes en los `devices[tag].parts` correspondientes del clip. No puede añadir un artículo no asignado ni una referencia automática, sustituir un artículo existente ni sobrescribir una cantidad local. Sin este mapa, el pegado y la inserción de macros mantienen las reglas para dispositivos existentes indicadas arriba. Los identificadores y los números admiten los parámetros de macro, la estructura de destino y el renombrado por colisión. Es una intención de inserción, no una instrucción de cambiar los valores por defecto ordinarios de copiar y pegar del proyecto.

La característica `per_wire_junctions` añade el `Wire.explicit_junctions` opcional: omitido
hereda `page.explicit_wire_junctions`, verdadero requiere contactos de nodo de conexión
dibujados, y falso permite los contactos en T implícitos. Las uniones implícitas de un hilo
con el interior de otro requieren que ambos conductores las permitan; los nodos de conexión
explícitos y los extremos guardados siguen conectando. Las fijaciones del lado del pin usan
la política del conductor individual. Un hilo explícito
activa también los destinos físicos dibujados en su red eléctrica.
Los comandos compartidos add_wire, connect, route y update_wire aceptan
`junction_policy: inherit|explicit|implicit`. update_wire devuelve esa política;
las consultas de página y los clips portables conservan el campo nativo opcional. El editor
ofrece las mismas opciones en las propiedades del hilo. Insertar una macro explícita nunca
cambia el ajuste de la página ni los demás hilos. La caché de exploración sigue las
ediciones de política, y la limpieza de hilos y la fusión de tramos colineales conservan
las políticas y propiedades distintas.

`apply_cable_part` (`cable`, `part`) aplica un artículo del proyecto o del catálogo
integrado a un cable existente. Usa la misma operación que el editor, incluidos el tipo,
los nombres de núcleo suministrados y la sección, y `Outer diameter` de los datos técnicos
en milímetros. Los diámetros que faltan o no son válidos restablecen a cero el valor
forzado, de modo que el tendido usa su estimación. Devuelve los campos de cable
configurados y `diameter_mm` (cero significa estimado), y rechaza los nombres de cable o de
artículo desconocidos sin cambiar el estado.

`list_cable_parts` no tiene parámetros y devuelve los artículos de cable disponibles
de los artículos del proyecto y del catálogo integrado, ordenados por número y sin
duplicados. Las definiciones del proyecto tienen prioridad, incluido un cambio local del
grupo de productos. Las filas incluyen el número, el tipo, la descripción, el fabricante,
los núcleos suministrados y la sección, y un `diameter_mm` declarado positivo o null. El
editor usa la misma prioridad con su base de datos de artículos configurada.

La característica `portable_cables` lleva los inventarios de cables y sus artículos locales
referenciados en `Clip.cables`. Copiar y pegar y la inserción de macros asignan
identificadores de cable nuevos e identificadores sin colisiones en la estructura de
destino, reasignando de forma coherente los símbolos marcadores, los hilos, las
definiciones de conexión y las líneas de definición de cable. Las asignaciones de núcleos
se conservan solo para los cables que pertenecen al clip. Los clips heredados sin
definiciones de cable siguen borrando las referencias de hilo/núcleo. Los campos de texto
de cable admiten también los marcadores de posición nativos de macro. Este comportamiento
lo comparten todas las interfaces.

Una referencia explícita de artículo de función puede resolver el artículo de cable
correspondiente cuando el mismo identificador tiene además un registro ordinario de
material de dispositivo. El artículo seleccionado debe coincidir con una asignación real;
las referencias desconocidas siguen sin ser válidas.
El material de cable permanece en el inventario de cables en lugar de duplicarse en
la lista de materiales ordinaria de dispositivos.

Los resultados de conexión física pueden contener el mismo par de extremos más de una vez
cuando núcleos de cable separados tienen caminos dibujados simples distintos. Cada conexión
conserva sus propios identificadores de hilo y sus propiedades de cable/núcleo para los
informes y el tendido. Un ramal suelto no se trata como una conexión completa adicional.
Los comandos de conexión existentes y el registro de informes exponen este comportamiento
en todas las interfaces.

El comando compartido `apply_cable_part` actualiza las referencias explícitas de funciones
colocadas que pertenecían solo al artículo de cable anterior. Las referencias a un artículo
ordinario de dispositivo bajo el mismo identificador, y las referencias explícitas no
válidas o no relacionadas, se conservan. El editor de cables usa la misma operación de
referencia de núcleos para la selección de catálogo y para las ediciones directas del
número de artículo. La longitud y las asignaciones de conductores se conservan al aplicar
los datos de cable del catálogo.

La característica de anfitrión `macro_definition_parameters` extiende los marcadores de
posición nativos de macro a las letras de las definiciones de símbolo portables, a las
etiquetas, textos de función y propiedades por defecto, a las etiquetas de pin, a las
primitivas de texto y a las filas literales de los bloques de atributos. Las claves de
símbolo, las claves de propiedad, las imágenes codificadas y los metadatos de origen sin
procesar conservan su identidad. La resolución opera sobre una copia del clip antes de la
instalación y la comparten el escritorio, la CLI y MCP.
Los importadores que convierten parámetros de plantilla de origen deben exigir esta
característica para que los anfitriones más antiguos no dejen gráficos estáticos
incoherentes con las instancias resueltas.

`insert_macro` acepta `values` opcionales, un objeto que asigna nombres de marcador de
posición a cadenas. Resuelve juntos los campos de instancia y los de las definiciones
portables antes de pegar; los marcadores de posición desconocidos quedan literales. Los
valores no válidos que no son cadenas fallan antes de cualquier edición.
Omitir el parámetro conserva el texto de plantilla sin resolver existente. Esta opción está
disponible mediante el registro compartido en todas las interfaces.

La característica de anfitrión `parallel_cable_cores` confirma que los consumidores de
conexiones físicas conservan los caminos distintos de núcleos paralelos dibujados. Los
importadores deben comprobar el anfitrión en ejecución además de cualquier CLI de
verificación aparte antes de aceptar circuitos que contienen núcleos de cable asignados; un
verificador más reciente no puede actualizar el motor de informes de un editor más antiguo.

La característica `page_property_blocks` resuelve las propiedades de proyecto y de página
en los bloques de atributos de símbolo usando su contexto de dibujo colocado. Los demás
campos conservan la selección de función, incluido el artículo seleccionado. La propiedad
de solo lectura `page.local_name` devuelve el número de página sin su prefijo de sección;
`page.section_pages` cuenta las páginas de la misma sección (las páginas de nivel superior
forman la sección cero). Ambas se actualizan tras las ediciones.

La característica `page_template_values` conserva el mapa `values` de una plantilla de
página al guardarla, intercambiarla e insertarla. Los mapas heredados vacíos siguen siendo
compatibles. El comando compartido list_page_templates devuelve estos valores por defecto,
y todas las interfaces los aplican mediante la misma operación de inserción. Los
importadores deben comprobar ambas características antes de suministrar marcos de origen
reutilizables con campos de contexto editables.

La característica `page_frame_fields` añade las propiedades calculadas
`page.section_last_number`, `page.previous_local_name` y `page.next_local_name`. El último
número usa la ordenación natural; las páginas adyacentes siguen el orden editable de su
sección, con valores vacíos en ambos extremos. Para las hojas 11 a 23, el número de páginas
es 13 mientras que el último número es 23. Las inserciones, las eliminaciones y la
renumeración actualizan estos campos. `page.function_designation` y
`page.location_designation` incluyen los signos de aspecto `=` y `+`, incluidos los signos
solos para un aspecto sin configurar.
Estas propiedades de solo lectura están disponibles mediante el registro compartido de
propiedades, los bloques de atributos de símbolo, los marcadores de posición de texto libre
y los formularios de cajetín.


La característica de anfitrión `function_cable_references` proporciona la propiedad
editable de Función `function.cable`, que contiene el identificador completo del cable para
un marcador colocado.
Renombrar el cable mediante la tabla de cables o el editor actualiza estas referencias,
los identificadores de marcador coincidentes, las asignaciones de hilos, los puntos de
conexión y las líneas de definición, incluidas las opciones de modelo inactivas. Los
símbolos de dispositivo ordinarios que comparten un identificador siguen separados.
Eliminar la entrada del inventario borra su asociación explícita de marcador y las
asignaciones de núcleos, quita las definiciones de camino y de accesorios que le pertenecen
y conserva los lugares ordinarios de extremo de dispositivo. El contenido portable de macro
o del portapapeles lleva el cable referenciado
y asigna la asociación al nuevo identificador del cable pegado. Use `set_properties` sobre
`function:<symbol id>` para editar la asociación; las operaciones de registro existentes se
aplican por igual al escritorio, la CLI y MCP.


Las celdas de identificador de cable (`table_set`, tabla `cables`, columna `Cable`) y el
gestor de cables usan la misma operación atómica `ops::rename_project_cable`. Recorta el
identificador introducido, valida su designación, rechaza los duplicados y los cambios en
planos liberados, y actualiza las referencias solo después de la validación. El gestor de
cables mantiene el texto del borrador fuera del proyecto hasta Intro o la pérdida del foco;
Escape cancela el borrador.

Una función con una referencia explícita `function.cable` resuelve su artículo primario
seleccionado y asignado mediante esa entrada del inventario de cables. Un artículo de cable
cambiado, borrado o ausente se refleja de inmediato, también en las ediciones directas de
las propiedades del cable y de la tabla. Las selecciones de artículo de función en caché no
ocultan esta asociación.
La selección ordinaria de artículo de función mantiene sus reglas existentes de referencia
explícita.


La característica `cable_property_blocks` resuelve las filas de la clase Cable en los
bloques de atributos de símbolo con respecto a la referencia explícita `function.cable` del
símbolo. Un destino explícito ausente queda sin resolver; los símbolos sin esa referencia
conservan el contexto heredado de cable con el mismo identificador. Este contexto se aplica
también a las propiedades de cable colocadas y lo comparten el lienzo y las exportaciones.
El editor de símbolos ofrece las propiedades de Cable.
Las propiedades colocadas de la clase Artículo resuelven el artículo de catálogo
seleccionado de la función, incluido el artículo de cable vinculado actual y las
asignaciones de dispositivo no primarias.

Los anfitriones que anuncian `placed_wire_labels` conservan `Wire.labels` incluso cuando
`hide_labels` suprime las etiquetas automáticas. Cada disposición contiene `kind` (`Core`,
`Cable`, `Number` o `Color`), la fracción de fijación `along` (0..1 de a a b),
`offset` en mm de dibujo, `size`, `anchor`, `angle` y un `color` RGB opcional.
El texto procede del hilo propietario actual. `update_wire` acepta
`labels: {"items": [...]}` para sustituir las disposiciones, o una matriz items vacía para
quitarlas. Las disposiciones no válidas rechazan todo el comando. Las etiquetas siguen a su
hilo durante la edición y se redistribuyen una vez cuando el tendido crea tramos nuevos.

Los marcadores de núcleo no fijados pueden conservar texto editable independiente en las
propiedades indexadas `function.cable_core[1]` a `[64]`. Los importadores deben usar las
etiquetas de hilo colocadas para los marcadores que hacen referencia a un hilo físico real,
de modo que las ediciones del núcleo del hilo cambien automáticamente el texto mostrado.

`rename_symbol_device` acepta el `id` de un símbolo colocado y `new_tag`. Una referencia
explícita `function.cable` que coincide con su identificador mostrado selecciona el
inventario de cables; los dispositivos ordinarios con identificador compartido siguen
siendo independientes. Los destinos de cable ausentes, los identificadores de cable
duplicados y las referencias en planos liberados rechazan la operación. El renombrado de la
función seleccionada en el editor usa la misma operación. `rename_device` sigue al
inventario de cables cuando ninguna identidad de dispositivo ordinario comparte ese
identificador. Las referencias de propietario de material siguen las ediciones de cable
solo mientras el identificador antiguo no tiene identidad de dispositivo ordinario.
Renombrar un dispositivo o un cable puede conservar un prefijo importado sin cambios que
contiene espacios; el nuevo nombre final del dispositivo sigue las reglas normales de
validación de identificadores.

### Orientación explícita en el armario

Los anfitriones que anuncian `explicit_panel_rotations` conservan los valores numéricos
opcionales de `PanelItem.rotation` 0, 90, 180 y 270 (en sentido horario en el plano de la
cara).
Los múltiplos negativos de 90 se aceptan y se normalizan. Los valores ausentes conservan la
inferencia heredada a partir de las dimensiones; un cero explícito suprime esa inferencia.
Los `x`, `y`, `w`, `h` guardados siguen siendo la huella ya orientada. Los importadores no
deben girar esas dimensiones una segunda vez. Los gráficos, las mallas, las posiciones y
las direcciones de salida de los puntos de conexión, los accesorios de montaje y los
patrones de taladrado exactos siguen el ángulo.

`set_panel_item(rotation: ...)` cambia la orientación sin mover ni redimensionar la
huella. `infer_rotation: true` borra el ángulo explícito; suministrar ambos es un
error. Las consultas de elementos devuelven `rotation` y `effective_rotation`.
`arrange_panel_items(action: "rotate")` gira los elementos seleccionados 90 grados
alrededor de sus centros e intercambia la anchura y la altura, como la acción Girar del
editor.

Los parámetros booleanos opcionales ofrecen en el editor Usar el valor predeterminado del
plugin, Activado y Desactivado. El primero omite el argumento, de modo que los plugins
externos conservan sus valores por defecto documentados; un falso explícito sigue siendo
distinto de la omisión en todas las interfaces.

### Revisiones de las definiciones de símbolo

`merge_symbol_definitions` acepta `definitions: {"items": [<UserSymbol>, ...]}`.
El lote completo se deserializa y se valida antes de cualquier cambio. Devuelve
`keys`, una clave resuelta por entrada en el orden de origen, y `added`, el número de
definiciones nuevas. Las definiciones tipadas idénticas reutilizan su clave existente. Los
conflictos prueban `<original>_import_1`, `_import_2`, y así sucesivamente, reutilizando
una revisión existente igual.
Las definiciones existentes y las instancias de página no cambian. Dos entradas que
comparten una clave original pueden devolver claves de revisión distintas, por lo que los
importadores deben asignar las referencias dentro de cada clip entrante en lugar de
construir un único mapa global de claves originales.

El editor, la CLI, MCP y los plugins externos usan todos el mismo comando registrado.
El tipado nativo normaliza los campos por defecto y la precisión numérica antes de la
comparación; los importadores deben usar esta operación en lugar de comparar objetos JSON
guardados sin procesar.
Es la misma operación del núcleo que se usa al pegar definiciones de macros portables.

### Marcos definidos por el autor

Los anfitriones que anuncian `plot_frame_artwork` aceptan `forms.frames[].artwork`, una
matriz de primitivas de dibujo de símbolo en milímetros de papel. Un dibujo no vacío
sustituye el borde y el cajetín generados; `visible` controla el dibujo completo del marco.
Los bloques de propiedades resuelven las propiedades vivas de proyecto y de página para
cada página que lo usa.
El dibujo pertenece al marco reutilizable y no es contenido de circuito seleccionable. Las primitivas con un `color` (trazados, círculos, textos, bloques de propiedades: un logotipo en color) lo conservan; las demás se dibujan en el color del marco.
El comando compartido `set_frame` acepta el JSON opcional `artwork: {"items": [...]}`; una matriz items vacía
restaura el marco generado. `list_frames` incluye el dibujo. El lienzo, la vista previa del
marco y las exportaciones usan el mismo representador. Los archivos de proyecto más
antiguos mantienen los marcos generados.

### Imágenes y estilos de texto de los cajetines

Los anfitriones que anuncian `title_block_images` aceptan elementos `Image` en los
cajetines (`forms.title_blocks[].items`) y en los formularios de informe dibujados
(`layout.header`, `layout.row`):
`{"type": "Image", "a": {…}, "b": {…}, "picture": {"file": {"name", "data"}, "px": [w, h]}}`
con un PNG o JPEG en base64 ajustado al rectángulo `a`..`b` manteniendo su relación de
aspecto.
Sin `picture` el elemento muestra `info.logo` (la misma forma que `picture`), el logotipo
del proyecto que también muestra la portada: encima del nombre del proyecto, o ajustado a
`info.cover_logo` (`x`, `y`, `width`, `height` en mm desde la esquina superior izquierda
del área de dibujo; comando `set_cover_logo`). Los elementos `Text` admiten `bold`,
`italic` y `angle` opcionales (grados, en sentido antihorario). Los marcadores de posición
`{page.previous}`, `{page.next}` y
`{rev.N.name|date|author|description}` (la N-ésima última revisión liberada) se rellenan
como los demás. Los comandos compartidos `list_title_blocks`, `set_title_block` (elementos
como `items: {"items": [...]}`), `delete_title_block`, `set_default_title_block` y
`set_page` (`title_block`) los editan; la línea de comandos añade `add_title_block_image`
y `set_project_info` (`logo`) para los archivos de imagen.

### Condiciones, estilos, códigos QR y sellos de los formularios

Los anfitriones que anuncian `form_customization` aceptan estos campos adicionales de
`forms` (todos opcionales, los archivos y plugins más antiguos los omiten):

- Cada elemento de un cajetín o de un formulario dibujado admite `show: {"filled": "…",
  "page_types": [...]}`: se dibuja solo cuando `filled` (con marcadores de posición) no
  está vacío y solo en las páginas de los tipos indicados (`schematic`, `cover`, `report`,
  `mounting`, `drilling`, `overview`, `graphic`, `document`). `Line`, `Rect` y `Text` admiten
  `color: [r, g, b]`, `Line` y `Rect` `dashed`. `{"type": "Qr", "a": {…}, "size": mm, "text": "…"}`
  dibuja un código QR (modo byte, nivel M) del texto rellenado.
- `forms.page_type_title_blocks` asigna tipos de página a nombres de cajetín y
  `forms.frames[].title_block` nombra el cajetín de un marco. Una página usa su propio
  cajetín; si no, el de su tipo, el de su marco, el predeterminado del proyecto, Standard.
- `forms.stamp` (`draft`, `review`, `approved`, `released`, `size`, `angle`, `color`)
  es un texto dibujado sobre cada página según el estado del flujo de revisión; los
  marcadores de posición `{rev.status}`, `{rev.approved_by}`, `{rev.approved_date}` y
  `{rev.N.approved_by|approved_date}` dan el estado y quién aprobó una revisión.
- Los formularios de informe admiten `totals` (columnas sumadas debajo de la última fila),
  `shade_rows` y `align` (`{column: "Left" | "Center" | "Right"}`); las páginas de informe
  congeladas los mantienen en `totals`, `shade_rows` y `align` de su tabla.

Comandos: `set_stamp`, `set_default_title_block` (`page_type`), `set_frame`
(`title_block`); `set_title_block` admite los nuevos campos de elemento. La biblioteca de
formularios (el archivo y la biblioteca `forms` del servidor de artículos, `GET` /
`PUT /api/v1/libraries/forms`) lleva también `frames`; la línea de comandos la carga y la
publica con `parts_company_forms` (con `publish`).

### Campos vivos de referencia de función

Los anfitriones que anuncian `function_cross_references` resuelven la propiedad de solo
lectura `function.cross_references` a partir del índice nativo actual de dispositivos e
interrupciones y de los ajustes de referencia. Las filas de los bloques de propiedades
conservan la colocación y la distribución definidas mientras su texto sigue las páginas y
secciones renombradas y los destinos de dispositivo cambiados.
Sus enlaces a la página de destino por línea sobreviven a la exportación a PDF; el texto
del lienzo y del SVG usa el mismo resolutor. Un campo explícito sustituye
el texto de referencia automático; los símbolos ordinarios conservan las anotaciones
automáticas.
Las vistas previas de biblioteca y las fantasma sin contexto de proyecto dejan vacía la
fila de referencia.

El comando compartido `list_cross_references` devuelve de cada función su `id`, `page`,
`tag`, el `text` de varias líneas y las `references` (texto y página de destino, null para
los destinos ausentes). Los filtros opcionales `id` y `page` usan identificadores nativos
de símbolo y nombres de página.
La propiedad está disponible también mediante `get_properties` y los selectores de
propiedades; `set_properties` rechaza los intentos de congelar su valor calculado.
