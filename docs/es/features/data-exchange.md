# Salida e intercambio de datos

Exportaciones PDF, DXF / DWG, SVG y 3D, importación DXF / DWG, la capa de integración empresarial (ERP, PDM, PLM, ECAD, MCAD, compras, máquinas de cableado) y Autodesk Vault.

## Salida / intercambio de datos

**PDF** de todas las páginas o de una selección de páginas (`1-3, 5`;
también para el documento HTML de impresión; *Archivo › Imprimir* abre todas las páginas
o la actual en el navegador con su diálogo de impresión) con marcadores por página,
referencias cruzadas y filas de informes en las que se puede hacer clic (los enlaces a
páginas omitidas se descartan); los textos más allá de Latin-1, como el polaco, el checo,
el turco, el griego o el cirílico, se escriben con un subconjunto incrustado de la fuente
Ubuntu, de modo que se pueden buscar y copiar, **DXF** (R12, capas por tipo) y **DWG**
(AutoCAD 2004, el mismo dibujo) por página o todas las páginas una junto a otra, SVG por
página, HTML imprimible, lista de hilos para producción (CSV con números de hilo,
extremos, color, sección, longitud tendida, longitud de pelado y terminación de
ambos extremos (puntera según DIN 46228-4, puntera doble, terminal de anillo, de
horquilla, de punta, faston macho o faston hembra) con sus artículos, una puntera doble
nombrada con el otro hilo y enumerada en el primero de ellos, conductores de cable; los
hilos del armario ordenados por sección, color y longitud para las máquinas de procesado
de hilos; las punteras y los terminales también en la lista de materiales; con una marca
de orden de bytes UTF-8, como los CSV de informes del editor, para que las hojas de
cálculo muestren `mm²` y las diéresis), CSV de informes, tablas de símbolos de PLC,
datos NC de las placas de montaje, el **diseño del armario en 3D** como **glTF 2.0**
(visores, web, PLM; metros, un nodo por envolvente y elemento, colores), **STEP AP214**
(MCAD; sólidos facetados con colores, milímetros, z hacia arriba), **IGES 5.3** (MCAD;
sólidos B-rep con caras planas, cáscaras abiertas para mallas no cerradas, colores y
nombres), **IFC4** (BIM; un cuadro de distribución por envolvente con sus componentes
como elementos triangulados), **JT** (visores PLM; mallas de tiras de triángulos por
elemento, estructura JT 8.1) y **Parasolid XT** (MCAD como SolidWorks, NX, Solid Edge,
Onshape; texto de transmisión del esquema V13 en metros, cada pieza cerrada un cuerpo
sólido facetado cuyos triángulos coplanarios se fusionan en caras planas, las mallas
abiertas un cuerpo de superficie) –
paredes, puerta, placa de montaje, carriles, canaletas, barras y dispositivos con los
modelos 3D de sus artículos.
**Importación DXF / DWG** (*Archivo › Importación › Importar DXF / DWG como página…*): dibujos antiguos como gráficos
en una nueva página de esquema – líneas, polilíneas, círculos, arcos, elipses, splines,
textos, MTEXT, referencias de bloque y cotas, en sus unidades, reducidos a escala hasta
el área de dibujo cuando son más grandes. Los archivos DWG de AutoCAD R12 … 2018 se leen
(y se escriben) con el crate [acadrust](https://crates.io/crates/acadrust), escrito en
Rust puro, también en el navegador. Los archivos EDZ del EPLAN Data Portal se leen como artículos (véase [gestión de artículos](parts.md)); sus macros de ventana 2D se convierten en símbolos y sus macros 3D en los modelos 3D
de los artículos.

## Integración empresarial

**Integración empresarial** (*Herramientas › Integración empresarial…*): una capa de
integración con conectores para **ERP** (maestro de artículos y lista de materiales,
CSV), **PDM** (metadatos de proyecto y de páginas, JSON), **PLM** (lista de materiales
de ingeniería, JSON), **ECAD** (componentes y lista de redes para el diseño de circuitos
impresos, JSON), **MCAD** (colocación de dispositivos, CSV), **compras** (lista de
pedido por proveedor, teniendo en cuenta existencias y unidades de embalaje, CSV) y
**máquinas de cableado** (lista de trabajos con longitud de corte, pelado y punteras,
CSV). Todos los conectores funcionan en ambos sentidos y comparten un vocabulario de
campos neutro: las exportaciones usan los códigos de campo como columnas / claves, las
importaciones reconocen los códigos, los nombres de columna habituales en inglés y en
alemán (`Material`, `Bestand`, `refdes` …) y la asignación de columnas de un *Perfil*
(p. ej. `MATNR` → `erp_number` para SAP). Las importaciones se muestran como una lista de
cambios que se revisan y se adoptan: números ERP, precios, existencias y plazos de
entrega, ID de artículo PLM, revisiones y estados del ciclo de vida (la comprobación
señala los artículos obsoletos en uso), números y estados de documento PDM, asignaciones
de artículos procedentes del sistema ECAD, posiciones de dispositivos procedentes del
sistema MCAD, confirmaciones de pedido y la respuesta del procesado de hilos como marcas
de cableado. Cada exportación guarda los valores exportados, de modo que los valores
modificados en ambos lados desde entonces se notifican como conflictos y los valores que
el otro sistema dejó sin cambios nunca sobrescriben las ediciones locales. Un archivo de
intercambio neutro (`openecraft.exchange/1`, JSON) lo contiene todo para los sistemas sin
conector; el registro de intercambio se conserva en el proyecto.

## Autodesk Vault

OpenECraft se conecta a Autodesk Vault Professional 2025.2 o posterior a través de la
Vault Data API (REST, `/AutodeskDM/Services/api/vault/v2`), de modo que los proyectos,
los documentos y los modelos 3D se intercambian directamente con el sistema PDM en lugar
de mediante archivos.

- En el editor: *Herramientas › Autodesk Vault…* – introduzca la dirección del servidor,
  el almacén (knowledge vault), el nombre de usuario y la contraseña (o el token de
  acceso de una sesión de Vault o de una cuenta de Autodesk) e *Iniciar sesión*. Examine
  las carpetas y busque en las propiedades de los archivos por debajo de la carpeta
  mostrada.
- **Importación**: un archivo seleccionado se abre como proyecto (`.oec`, `.oecz`), se importa en
  la base de datos de artículos (CSV, JSON, BMEcat, EPLAN XML, EDZ, paquetes de envolventes como ZIP), se asigna como modelo 2D / 3D de un
  artículo (SVG, STEP, IGES, Parasolid, JT, IFC, STL, OBJ – p. ej. modelos del diseño mecánico guardados
  en el almacén), se inserta como página (DXF, DWG) o como página de documento (PDF, archivos de Office,
  imágenes), se lee con un conector de la capa de integración empresarial o se guarda
  como archivo local. El nombre del archivo sugiere el uso.
- **Exportación**: el archivo de proyecto, todas las exportaciones de *Archivo › Exportación* (PDF, DXF / DWG, el
  diseño del armario en 3D como STEP, IGES, Parasolid, JT, glTF o IFC, informes, tablas de PLC, exportaciones de
  plugins) y las exportaciones de los conectores van a la carpeta mostrada. Un archivo
  con el mismo nombre recibe allí una nueva versión (se extrae, se carga por partes y se
  protege con el comentario; la extracción se deshace cuando algo falla); en caso
  contrario, el archivo se añade.
  Un proyecto abierto desde el almacén se vuelve a proteger con *Archivo › Guardar (Proteger … en Vault)*.
- **Elementos**: *Actualizar los vínculos de artículos desde elementos de Vault* busca el
  elemento de Vault de cada artículo del proyecto (por número de elemento o por su
  propiedad *Part Number*) y muestra los números de elemento, las revisiones y los
  estados del ciclo de vida como cambios del conector PLM para su revisión, con la misma
  detección de conflictos que las importaciones de archivos.
- Línea de comandos / MCP: `vault_connect`, `vault_browse`, `vault_items`, `vault_download`,
  `vault_open`, `vault_upload`, `vault_sync_items`. Una sesión MCP inicia sesión una vez
  con `vault_connect`; la línea de comandos inicia una sesión por llamada y toma el
  servidor, el token y el id del almacén de `OPENECRAFT_VAULT_URL`,
  `OPENECRAFT_VAULT_TOKEN` y `OPENECRAFT_VAULT_ID` (`vault_connect` muestra el token y el
  id del almacén de un inicio de sesión).

```bash
openecraft-cli vault_connect --url https://vault.example.com --user anna --password …
export OPENECRAFT_VAULT_URL=https://vault.example.com OPENECRAFT_VAULT_TOKEN=V:… OPENECRAFT_VAULT_ID=101
openecraft-cli vault_browse --query "Plant 1"
openecraft-cli -p plant.oec vault_upload --folder 42 --format step --comment "Panel layout"
```

- Las contraseñas no se guardan y el token solo se mantiene en memoria. La versión para
  navegador necesita un servidor de Vault (o un proxy inverso delante de él) que permita
  peticiones de origen cruzado desde la dirección del editor.
