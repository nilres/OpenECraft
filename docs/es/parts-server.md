# Servidor de artículos y gestor de artículos

El portal de datos compartido por todas las instalaciones de OpenECraft y la aplicación que lo mantiene.

`openecraft-parts-server` es el portal de datos de todas las instalaciones de OpenECraft
(escritorio y web), en el espíritu del EPLAN Data Portal: catálogos de fabricantes con una
búsqueda por facetas, historial y ciclo de vida de los artículos, archivos de los artículos
(hojas de datos, imágenes, modelos, macros), cuentas de usuario con roles y una página web
para explorar. Los detalles de un artículo muestran su vista frontal 2D y su modelo 3D (se
gira arrastrando) cuando los tiene; los artículos del catálogo de ejemplo tienen ambos
cuando se conocen sus dimensiones, generados a partir de las dimensiones (dispositivos,
bornes, módulos de PLC, carriles, canaletas, sistemas de barras, envolventes y sus
componentes, accesorios). Es un ejecutable independiente y funciona con independencia del
editor.

```bash
cargo run --release -p openecraft-parts-server -- --bind 0.0.0.0:8090 --token <secret>
cargo run --release -p openecraft-parts-server -- --storage json --data parts-db.json
cargo run --release -p openecraft-parts-server -- --data parts-db.sqlite --migrate-from parts-db.json
cargo run --release -p openecraft-parts-server -- --install-catalogs all
cargo run --release -p openecraft-parts-server -- --config parts-server.toml
```

Como contenedor: `docker run -d -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data
ghcr.io/nilres/openecraft-parts-server` (véase [instalación](installation.md)).

- Archivo de configuración (`--config <file>` u `OPENECRAFT_PARTS_SERVER_CONFIG`): un
  archivo TOML con los ajustes de las opciones de la línea de comandos bajo sus nombres
  (`bind`, `storage`, `data`, `token`, `admin_user`, `install_catalogs`, `access`,
  `registration`, `dashboard`; véase
  [`parts-server/parts-server.example.toml`](../../parts-server/parts-server.example.toml)).
  Las opciones de la línea de comandos o del entorno tienen prioridad; las rutas relativas
  son relativas al archivo; las claves desconocidas detienen el servidor con un error.
  `--migrate-from` es solo una opción de la línea de comandos.
- Almacenamiento (`--storage sqlite|json`): SQLite por defecto (`--data parts-db.sqlite`,
  un archivo con artículos indexados, historial, cuentas y archivos); JSON guarda todo en
  un archivo escrito de forma atómica en cada cambio, con los archivos en `<name>.files/`
  junto a él. Un archivo de datos `.json` selecciona el almacenamiento JSON. Las bases de
  datos nuevas se llenan con el catálogo de ejemplo y los catálogos de fabricantes
  incluidos; `--migrate-from` copia una base de datos existente en una nueva.
- Catálogos incluidos: `crates/openecraft-parts/catalogs/<id>.json` (un `catalog` y sus
  `parts`, listados en `openecraft_parts::bundled`, que también usan los proyectos de
  ejemplo) son conjuntos iniciales seleccionados de artículos reales de
  fabricantes: `abb` (interruptores automáticos, interruptores diferenciales, contactores,
  guardamotores manuales), `beckhoff` (acoplador y terminales EtherCAT), `cosmo`
  (colectores de circuitos de calefacción de acero inoxidable para 2 - 14 circuitos y cajas
  de regulación por estancia RTL / TH para suelo radiante), `dehn` (dispositivos de protección contra sobretensiones de tipo 1 - 3, para instalaciones fotovoltaicas y para líneas de datos), `gira` (tomas de corriente
  System 55, mecanismos de interruptor y de pulsador, toma de red Cat.6A con teclas, tapas
  y marcos), `hager` (interruptores automáticos,
  interruptores diferenciales, contactores de instalación, interruptores seccionadores,
  seccionadores fusibles NH para barras de 60 mm y regletas seccionadoras fusibles para
  185 mm con transformadores de corriente enchufables, interruptores automáticos h3+ con
  mandos motorizados y disparadores, el relé de protección de red e instalación EU400,
  armarios univers con sus módulos componentes, contadores de
  energía), `janitza` (analizador de redes UMG 604-PRO), `ledvance` (luminarias LED de techo SURFACE
  CIRCULAR, downlight DOWNLIGHT SLIM, PANEL 600, regleta DAMP PROOF, apliques estancos y
  apliques estancos de emergencia), `jinko` (módulos fotovoltaicos Tiger Neo
  54HL4-B all black 405 - 415 Wp), `longi` (módulos fotovoltaicos Hi-MO X6 Explorer LR5-54HTH
  425 - 435 Wp), `loxone` (contador de energía para
  transformadores de corriente de 1 / 5 A en el bus Tree), `mdt` (actuadores KNX, interfaz
  / router IP, fuente de alimentación, pulsadores KNX, pulsador de cristal y regulador de
  temperatura ambiente para montaje empotrado), `mennekes` (tomas murales CEE de 16 A y 32 A,
  5 polos), `merten` (caja de conexión para cocina de montaje empotrado), `phoenix_contact` (bornes, módulos
  de relé, fuentes de alimentación), `pylontech` (módulos de batería LiFePO4 de 19 pulgadas), `siemens` (contactores SIRIUS, relés de sobrecarga,
  guardamotores, interruptores automáticos SENTRON), `signify` (Philips CoreLine downlight
  DN140B, panel RC132V, regleta estanca WT120C, aplique de pared WL120V), `spelsberg` (cajas
  de derivación Abox 025-L a 350-L, colocadas como envolventes de la estancia), `striebel_john` (armarios murales ComfortLine A de 5 a 9 filas
  y de 1 a 5 anchos de campo con los paneles de contadores, de distribución y multimedia de su altura,
  puertas, cierres, bridas y demás accesorios), `victron` (inversor /
  cargador MultiPlus-II, Inverter RS Smart Solar, SmartSolar MPPT, Orion-Tr, Cerbo GX, SmartShunt, BMV-712, Lynx
  Distributor; en su mayoría artículos para montaje directo en pared), `wago` (bornes de empalme 221 y
  2273 para cajas de derivación) y `weidmueller` (bornes,
  módulo de relé, fuentes de alimentación, canaletas de cableado CD). `--install-catalogs all` (o identificadores como
  `siemens`) los instala o actualiza en una base de datos existente: los artículos nuevos
  se añaden, los artículos que nadie ha modificado se actualizan, los artículos modificados
  o eliminados por los usuarios se conservan. Los artículos sin modelos reciben unos
  generados a partir de sus dimensiones de montaje. Añada solo datos que se puedan
  redistribuir (hechos de hojas de datos públicas, sin textos, imágenes ni modelos
  copiados). Cada catálogo incluido tiene un logotipo, `catalogs/logos/<id>.svg`, que
  recibe al instalarse mientras no tenga logotipo: el logotipo de la empresa de Wikimedia
  Commons (dominio público o CC BY-SA 4.0, fuentes y licencias en
  `catalogs/logos/README.md`; los logotipos son marcas de sus propietarios), o un
  distintivo neutro con el nombre cuando Commons no tiene uno con licencia libre (COSMO,
  Janitza, MDT, Pylontech).
- Logotipos: cada catálogo puede tener el logotipo de su empresa (PNG, JPEG, GIF, WebP o
  SVG, como máximo 512 kB), que se muestra en las tarjetas de catálogo, en la lista de
  catálogos de la administración y en los artículos del catálogo. Quien puede modificar un
  catálogo cambia su logotipo en *Administración › Catálogos › Edición* (`PUT` / `DELETE
  /api/v1/catalogs/{id}/logo?name=`, `GET` lo sirve con una política de seguridad de
  contenido de aislamiento, de modo que un SVG no ejecuta scripts). Guardar un catálogo
  conserva su logotipo; el archivo del logotipo se almacena con los archivos de los
  artículos y forma parte de las copias de seguridad.
- Bibliotecas: documentos JSON compartidos por todos los proyectos (`GET` / `PUT /api/v1/libraries/{name}`),
  hasta ahora `forms`, los cajetines, esquemas de informe y marcos de la empresa que el
  editor (*Formularios › Cajetines › Biblioteca de la empresa*) y la línea de comandos
  publican y cargan. Todo el que puede leer artículos las lee; modificarlas necesita el
  derecho a editar catálogos en todo el servidor (editores, administradores).
- Usuarios: los lectores guardan favoritos, los fabricantes modifican los artículos y
  catálogos de su propio fabricante, los editores todos los artículos, los administradores
  además gestionan los usuarios. El `--token` (u `OPENECRAFT_TOKEN`) es el token de
  administrador; `--admin-user <name>` (u `OPENECRAFT_ADMIN_USER`) crea una cuenta de
  administrador con la contraseña de `OPENECRAFT_ADMIN_PASSWORD`.
  Los usuarios inician sesión con `POST /login` (`name` y `password`; `user` se acepta por
  `name`, como en el servidor de proyectos) y envían el token de sesión como
  `Authorization: Bearer <token>`. Un token que el servidor no conoce (una sesión caducada)
  se rechaza con `401`, también para leer, de modo que los clientes lo noten en lugar de
  leer de forma anónima. Las contraseñas de
  las cuentas locales (también la de `--admin-user`) tienen al menos 8 caracteres; tras 5 inicios de sesión fallidos de un nombre de usuario cada intento posterior espera
  (1 s, duplicándose hasta 5 minutos, respuesta `429`). Sin token, cuentas ni directorios el servidor
  es de solo lectura.
- Registro: cuando está abierto, cualquiera crea una cuenta local de lector en la página
  web (*Registrarse* junto al inicio de sesión, `POST /register` con `name` y `password`);
  los administradores dan más derechos. Por defecto la cuenta nueva espera la aprobación de
  un administrador (`202`, sin sesión): no puede iniciar sesión hasta que un administrador
  la apruebe en *Administración › Usuarios* (`POST /users/{name}/approval`) o la rechace
  (elimine); las cuentas en espera se listan primero y se cuentan en la vista general. Sin
  aprobación (desactivada en *Administración › Derechos de acceso*, almacenada como
  `"registration_approval": false`) el usuario nuevo inicia sesión de inmediato. Aprobar
  sigue siendo una decisión de un administrador aunque las opciones de inicio fijen el
  registro. Las cuentas que esperan cuando se desactiva la aprobación siguen esperando
  hasta ser aprobadas. Está cerrado salvo que se active: por los administradores en
  *Administración › Derechos de acceso* (almacenado como `"registration": true` en el
  archivo de ajustes de acceso), o fijado por las opciones de inicio
  `--registration true|false`, `OPENECRAFT_REGISTRATION` o `registration` en el archivo de
  configuración, que tienen prioridad sobre el interruptor. Los usuarios registrados leen
  los artículos incluso donde leer necesita un inicio de sesión. Un cliente puede registrar
  5 cuentas, después cada intento posterior espera (1 s, duplicándose hasta 5 minutos); los
  nombres a los que se refieren las asignaciones (también con comodines) y `admin` no están
  disponibles, para que nadie se registre en los derechos de una asignación. Un nombre
  registrado también se antepone a un usuario de directorio del mismo nombre que aún no ha
  iniciado sesión: en los servidores con directorios, asigne por grupos.
- Control de acceso (`<data>.access.json`, `--access`): inicio de sesión con directorios
  Active Directory o LDAP junto a las cuentas locales, grupos de usuarios, roles definidos
  por los administradores y asignaciones de roles a usuarios y grupos por fabricante o por
  catálogo, con asignaciones de denegación; la lectura anónima se puede desactivar. Véase
  [docs/access-control.md](access-control.md).
- Panel: el *Panel* de la página web muestra el número de artículos, catálogos, fabricantes
  y archivos y gráficos de los artículos por fabricante, grupo de productos, catálogo,
  grupo principal y ciclo de vida, los últimos cambios por mes, cuántos artículos tienen
  descripción, precio, dimensiones, modelos, archivos y hojas de datos, los usuarios por
  rol (administradores) y los artículos modificados en último lugar; sus barras abren los
  artículos que cuentan (`GET /api/v1/dashboard`). Los administradores lo ocultan en
  *Administración › Vista general* (`"dashboard": false` en los ajustes de acceso,
  `PUT /api/v1/dashboard/settings`); la opción de inicio `--dashboard true|false`,
  `OPENECRAFT_DASHBOARD` o `dashboard` en el archivo de configuración lo fijan.
- Las exportaciones llevan la versión del servidor, de modo que una versión posterior pueda
  convertirlas al importarlas: `GET /export.json` es un objeto con `export` (`format`,
  `format_version`, `server`, `server_version`, `revision`, `exported`) y `parts`
  (`/import` y los editores lo leen como antes), `GET /export.csv` empieza con una línea
  `# format=openecraft-parts;format_version=1;server_version=…` (la importación CSV omite
  esas líneas antes del encabezado), las copias de seguridad guardan los campos como
  `export.<name>` en su tabla `meta`, los catálogos y los ajustes de acceso exportados en
  la página web tienen un objeto `export` y el CSV de usuarios la misma primera línea. Las
  respuestas llevan además el encabezado `X-OpenECraft-Server-Version`.
- Los artículos se comprueban antes de almacenarse: los números de artículo tienen como
  máximo 128 caracteres, los demás campos como máximo 256 (descripción, nota, datos
  técnicos, certificados y hoja de datos 20000), el catálogo que nombra un artículo debe
  existir, y una importación rechaza un archivo que contiene dos veces un número de
  artículo.
- Cada cambio es una revisión nueva; cada artículo conserva su revisión, la hora del cambio
  y las últimas 20 versiones (historial). Los artículos tienen un ciclo de vida (activo, en
  retirada, descatalogado) y un sucesor. Los archivos se almacenan una vez por contenido;
  un archivo se elimina en cuanto ni un artículo ni una versión del historial lo usan (y al
  iniciar).
- Grupos de productos: los grupos de productos genéricos, grupos de productos y subgrupos
  de EPLAN con sus números de intercambio (`openecraft_parts::productgroups`) se ofrecen en
  el editor de artículos junto a los grupos propios de OpenECraft.
- Búsqueda (`POST /search`): texto (cada palabra debe aparecer), grupo principal, grupo,
  subgrupo, fabricante, catálogo, ciclo de vida y datos técnicos (tensión, corriente,
  potencia y las líneas `Name: value` de los datos técnicos), ordenada y paginada, con
  facetas que cuentan los artículos coincidentes por valor.
- Página web: `http://<server>:8090/`: panel, búsqueda con facetas, detalles de los
  artículos con archivos e historial, catálogos, inicio de sesión y favoritos, en inglés,
  alemán, francés y español. El logotipo y el título de arriba a la izquierda llevan de
  vuelta a la página de inicio (todos los artículos, sin filtros). La lista muestra 10, 25,
  50, 100 o 200 artículos por página (*Por página*, lo recuerda el navegador). La dirección
  conserva la búsqueda y el artículo abierto (marcadores, el botón Atrás del navegador);
  `?lang=de` elige el idioma.
- Artículos nuevos en la página web: los usuarios que pueden modificar artículos de algún
  fabricante o catálogo obtienen *Nuevo artículo* encima de la lista y *Edición* /
  *Eliminar* en los artículos que pueden modificar; el formulario contiene los datos
  generales, técnicos, comerciales y de montaje, ofrece los catálogos a los que el usuario
  puede añadir artículos y nunca sustituye un artículo existente (`PUT /parts/{number}`
  con `If-None-Match: *` responde `409`).
- Administración en la página web (*Administración*, se muestra a los administradores y a
  los usuarios que pueden modificar catálogos o artículos; inicie sesión con un usuario o,
  en un servidor nuevo, con el token de administrador): una vista general del servidor
  (almacenamiento, revisión, usuarios por rol, ajustes de acceso, los derechos del
  llamante); *Usuarios*: crear, modificar y eliminar usuarios locales (rol, fabricante,
  grupos, contraseña nueva), ver su último inicio de sesión y sus sesiones abiertas y
  cerrar su sesión en todas partes; *Derechos de acceso*: el rol de las peticiones sin
  inicio de sesión, los roles definidos con sus derechos y las asignaciones a usuarios y
  grupos por fabricante o catálogo; *Grupos*: grupos de usuarios con sus miembros;
  *Directorios*: las conexiones de Active Directory y LDAP con una prueba de la conexión y
  de un inicio de sesión; *Catálogos*: crear, modificar y eliminar catálogos, los permisos
  de usuarios y grupos sobre cada catálogo, e instalar o actualizar los catálogos
  incluidos; *Importar artículos* (CSV,
  JSON, BMEcat, EPLAN XML); *Exportación y copia de seguridad*: los artículos como CSV o
  JSON, los catálogos, los usuarios (CSV, sin contraseñas) y los ajustes de acceso (JSON,
  que se restauran de nuevo), y una copia de seguridad de toda la base de datos. La página
  solo ofrece lo que el usuario puede hacer; el servidor comprueba cada cambio.
- Copia de seguridad (`GET /backup`, administradores): un archivo SQLite con los artículos,
  su historial y archivos, los catálogos, los usuarios con sus hashes de contraseña y los
  favoritos, sea cual sea el almacenamiento del servidor. Restáurela iniciando un servidor
  con `--data <backup>.sqlite`, o cópiela en una base de datos nueva con `--migrate-from`.
  El archivo de ajustes de acceso (directorios, roles, asignaciones) no forma parte de
  ella: guarde también una copia de `<data>.access.json`.
- API REST (`/api/v1`, véase `openecraft_parts::api`): `GET /info`, `GET /parts`,
  `POST /search`, `GET|PUT|DELETE /parts/{number}`, `GET /parts/{number}/accessories` (sus accesorios
  y, para un accesorio, los artículos a los que se adapta), `GET /parts/{number}/history[/{revision}]`,
  `POST|DELETE /parts/{number}/attachments`, `GET /files/{id}` (los PDF, las imágenes de
  trama y el texto se muestran en el navegador, otros archivos como SVG y XML se descargan,
  de modo que ningún archivo ejecuta un script en la dirección del servidor), `POST /import` (CSV,
  JSON o BMEcat), `GET /export.csv` (todos los artículos, o los de la consulta de la dirección:
  `q`, grupos, fabricante, catálogo, ciclo de vida, orden), `GET /export.json` (lo mismo como
  lista de artículos JSON que lee `/import`), `GET /backup`, `GET|PUT|DELETE /catalogs/{id}`,
  `GET /bundled`, `POST /bundled/{id}` (instalar un catálogo incluido), `POST /login`,
  `POST /register`, `GET|PUT /registration`, `POST /users/{name}/approval`,
  `POST /logout`, `GET|PUT /favorites`, `GET /users` (con sus sesiones abiertas y su último
  inicio de sesión), `PUT|DELETE /users/{name}`, `DELETE /users/{name}/sessions` (cerrar
  sesión en todas partes), `GET|PUT /access`, `GET|PUT|DELETE /catalogs/{id}/logo`, `GET|PUT /catalogs/{id}/grants`, `GET /groups`,
  `PUT|DELETE /groups/{name}`, `GET /directories`, `PUT|DELETE /directories/{id}`,
  `POST /directories/{id}/test`, `GET /dashboard`, `GET|PUT /dashboard/settings`.
- En el editor: *Datos maestros › Portal de datos*: buscar en el servidor, ver los detalles
  con la vista frontal y el modelo 3D, añadir artículos a la base de datos local, guardar sus archivos, añadir sus macros a la biblioteca de macros y comprobar si
  en la base de datos hay versiones más recientes, artículos en retirada y descatalogados.
  Los detalles muestran el catálogo del artículo con su logotipo (se oculta con *Ajustes ›
  Apariencia › Portal de datos*). Los logotipos se dibujan sobre una placa blanca; los
  logotipos SVG se representan con `resvg` y la fuente propia del editor, de modo que se
  ven igual en el escritorio y en el navegador. *Dispositivos › Gestión de artículos ›
  🌐 Servidor de artículos* descarga todos los artículos o sube artículos. La comprobación
  del proyecto avisa de los dispositivos con artículos descatalogados; `use_successor_parts`
  los cambia por sus sucesores.
- El servidor habla HTTP sin cifrar, de modo que los tokens y las contraseñas cruzan la red
  sin cifrar. Manténgalo en `127.0.0.1` detrás de un proxy inverso con TLS cuando sea
  accesible desde otras máquinas, p. ej. con [Caddy](https://caddyserver.com/), que obtiene
  el certificado: `caddy reverse-proxy --from parts.example.com --to 127.0.0.1:8090`.

## Servidor de artículos público

El proyecto mantiene un servidor de artículos público en
[parts-server.openecraft.org](https://parts-server.openecraft.org) con el catálogo de
ejemplo y los catálogos de fabricantes incluidos. Todo el mundo lo lee sin cuenta: en su
página web (búsqueda, catálogos, panel, los detalles de un artículo con su vista frontal y
su modelo 3D), en el editor (*Datos maestros › Portal de datos*), en el gestor de artículos
y en la línea de comandos o mediante agentes de IA. Es el servidor preestablecido de todos
ellos (`openecraft_parts::api::PUBLIC_URL`): el editor y el gestor de artículos se inician
con su dirección, la línea de comandos lo usa sin `--server` / `OPENECRAFT_SERVER`. Un
servidor propio se introduce en su lugar como URL del servidor (`--server`, o
`parts_connect` con su `url`); se conserva la dirección introducida en último lugar. Es un
lugar para probar el portal de datos y para llevar artículos a su propia base de datos; una
empresa guarda sus propios artículos en un servidor propio (véase arriba), donde decide
quién puede modificarlos. Los datos de los catálogos se han recopilado de hojas de datos
públicas: compruebe los artículos con los datos del fabricante antes de hacer un pedido.

## Aviso legal, exención de responsabilidad y aviso de privacidad

Al pie de la página web están los enlaces *Privacidad* y, cuando el operador tiene nombre,
*Aviso legal* (en alemán *Datenschutz* e *Impressum*), cada uno de los cuales abre un
diálogo, y las exenciones de responsabilidad de OpenECraft (sin garantía, los nombres de
producto que menciona pertenecen a sus propietarios). Están ahí sin inicio de sesión, como
en el [sitio web](../website.md#legal-notice), cuyos campos y variables son los mismos:

| Clave de `[legal]` | Variable | Muestra |
| --- | --- | --- |
| `name` | `OPENECRAFT_LEGAL_NAME` | La persona o empresa que opera el servidor; vacío oculta el aviso legal |
| `address` | `OPENECRAFT_LEGAL_ADDRESS` | Su dirección postal |
| `email` | `OPENECRAFT_LEGAL_EMAIL` | Dirección de correo electrónico, como enlace |
| `phone` | `OPENECRAFT_LEGAL_PHONE` | Número de teléfono |
| `representative` | `OPENECRAFT_LEGAL_REPRESENTATIVE` | Quién representa a una empresa |
| `register` | `OPENECRAFT_LEGAL_REGISTER` | Registro mercantil y número |
| `vat` | `OPENECRAFT_LEGAL_VAT_ID` | Número de identificación a efectos del IVA |
| `responsible` | `OPENECRAFT_LEGAL_RESPONSIBLE` | Quién es responsable del contenido, con dirección |
| `text` | `OPENECRAFT_LEGAL_TEXT` | Texto adicional: exenciones de responsabilidad, resolución de litigios |
| `privacy` | `OPENECRAFT_LEGAL_PRIVACY` | Lo que el operador añade al aviso de privacidad: el alojamiento, los registros, a quién preguntar |

Defínalos en la tabla `[legal]` del archivo de configuración o con las variables, que
tienen prioridad; las líneas de la dirección y de los textos se separan con `|` (o con
saltos de línea en el archivo). `GET /api/v1/legal` los devuelve como JSON. Las etiquetas
siguen el idioma de la página; los valores se muestran tal como están definidos.

El aviso de privacidad dice lo que hacen la página y el servidor, en el idioma de la
página: sin cookies, sin seguimiento, nada cargado desde otros servidores; el idioma, los
ajustes de la página y la clave de la sesión en el almacenamiento del navegador; las
cuentas (nombre, rol, grupos, la contraseña como hash); los inicios de sesión fallidos
recordados durante poco tiempo; quién modificó un artículo y cuándo (su historial) y los favoritos de un usuario. Lo que solo sabe el operador (quién aloja el
servidor, qué registra un proxy inverso y durante cuánto tiempo, a quién preguntar, como
pide el artículo 13 del RGPD) se define en `privacy` y se muestra debajo. Mantenga los
textos veraces: un cambio que almacena o envía otra cosa los cambia también (`renderLegal`
de `common.js`, `privacy()` del script de la página). Si un servidor necesita un aviso
legal, y qué debe decir, depende del operador y del país: OpenECraft solo muestra lo que
está definido.

## EPLAN Data Portal

La gestión de artículos también se conecta al EPLAN Data Portal: elija *EPLAN Data Portal*
como tipo de servidor en la ventana del portal de datos o en *Gestión de artículos ›
🌐 Servidor de artículos*, mantenga la dirección `https://dataportal.eplan.com/api` e
introduzca un token de acceso personal (creado para el Data Portal en el perfil de su
cuenta de EPLAN, ePulse). El portal funciona entonces como un servidor de artículos:
búsqueda por texto, fabricante y grupo de productos, detalles de los artículos con los
datos comerciales (CSV), dibujos DXF e imágenes, adición de artículos a la base de datos
local y la comprobación de actualizaciones de la base de datos.

El PAT se envía como `Authorization: Bearer PAT:<token>` (también se aceptan los prefijos
`PAT:` y `Bearer ` pegados). La prueba de conexión lee el catálogo público y después
descarga los datos comerciales del primer artículo devuelto para comprobar el acceso de
descarga autenticado. Una búsqueda pública correcta no demuestra por sí sola la
autenticación; `/authorization` no se usa porque rechaza PAT que la pasarela de descarga
acepta. Las descargas de DXF y
de datos comerciales envían por POST el identificador del artículo a
`/download/{type}/part`, como el cliente web del portal; los identificadores de adjuntos
existentes en los proyectos guardados siguen siendo compatibles. El uso
del portal y de los datos de artículos tomados de él se rige por las condiciones de uso de
EPLAN y la licencia de la cuenta: compruébelas antes de poner artículos del portal en un
servidor de artículos compartido. OpenECraft no está afiliado a EPLAN.

- `openecraft_client::eplan` envuelve la API REST del portal (JSON:API): construye las
  peticiones del portal a partir de las consultas del servidor de artículos y convierte las
  respuestas en `ServerInfo`, `SearchResult` y `Part`. `openecraft_client::Connection`
  ofrece las funciones de lectura de ambos tipos de servidor; el editor y la línea de
  comandos lo usan.
- La conexión es de solo lectura: sin subidas, catálogos, usuarios, inicio de sesión,
  favoritos, historial ni descarga de toda la base de datos. Las facetas cuentan los
  artículos de la página mostrada, ya que el portal no los cuenta; los catálogos, el ciclo
  de vida y los datos técnicos no se pueden filtrar.
- El portal encuentra los artículos por sus propios identificadores, no por número: un
  artículo cargado de él conserva el identificador (`portal_id`), de modo que la adición de
  artículos marcados y la comprobación de actualizaciones los cargan en una sola petición.
  La comprobación de actualizaciones compara solo los artículos que proceden del portal; un
  número de artículo aislado se encuentra con una búsqueda exacta.
- Los fabricantes se filtran por su nombre corto, los grupos de productos por el
  identificador de la entrada del catálogo de EPLAN: (grupo de productos genérico + 1) ·
  10⁸ + (grupo de productos + 1) · 10⁴ + (subgrupo + 1), p. ej. `200050000` para
  electrotecnia / conectores.
- Las envolventes (grupo de productos *Enclosure*) llegan del portal sin dimensiones: su
  tamaño, su sistema de envolvente y la falta de placa de montaje se leen de la
  descripción (`WHD: 800x2000x600 mm`, *baying*, *compact*, *terminal box*, *without
  mounting plate*), de modo que se colocan en el diseño del armario como las
  envolventes de los catálogos (probado con Rittal VX25, TS 8, AX y KX).
- Un artículo cargado del portal lista sus accesorios por número; los detalles del
  artículo los muestran con su descripción, fabricante y grupo de productos, de modo
  que se abren y se añaden como otros artículos y encajan con su artículo en el diseño
  del armario.
- Los artículos en formato EPLAN (macros, puntos de conexión) llegan como archivos EDZ
  desde la página web del portal con una licencia de EPLAN (una cuenta gratuita solo
  descarga datos comerciales y dibujos DXF); impórtelos como otros archivos de artículos (véase
  [gestión de artículos](features/parts.md)).
- Solo la versión de escritorio y la línea de comandos llegan al portal: no permite
  peticiones de origen cruzado desde los navegadores, de modo que la versión web
  necesitaría un proxy.

### PAT compartido del Data Portal en Windows

El editor y CLI/MCP pueden usar el mismo PAT a través del Administrador de credenciales de
Windows. Introduzca el PAT en el Data Portal del editor, seleccione **Recordar el token en
este equipo** y use **Probar conexión**. Recordar está desactivado por defecto: sin esta
aceptación el PAT permanece en la sesión. Solo una comprobación correcta del servicio
protegido guarda un sustituto.
Un error de almacenamiento se muestra por separado y no invalida la conexión. El PAT se
almacena para el usuario actual de Windows en este equipo, no en un proyecto, en ajustes
JSON, en un archivo de entorno ni en una respuesta MCP.

Deje vacío el campo del PAT para usar automáticamente el valor guardado. En la primera
petición, la interfaz gráfica lo carga en segundo plano antes de construir esa petición;
las peticiones simultáneas comparten la búsqueda. El hilo de la interfaz nunca espera la
lectura de la credencial. El resultado, incluido un token ausente o un error de lectura, se
guarda en caché durante la sesión.
**Probar conexión** fuerza una búsqueda nueva tras una sustitución o eliminación externa o
un error de lectura. **Eliminar token guardado** vacía también la caché. Un punto de acceso
distinto o un token explícito usa una caché separada, de modo que una búsqueda anterior no
puede sustituirlo.

MCP usa el valor guardado cuando
`parts_connect` selecciona `Eplan` sin un token explícito. Cada petición resuelve de nuevo
la credencial, de modo que las sustituciones y eliminaciones surten efecto sin reiniciar
MCP. Un token explícito suministrado para la conexión EPLAN actual tiene prioridad;
MCP no guarda los tokens de sesión explícitos en el almacén compartido.

Las credenciales están ligadas al origen HTTPS, al puerto y a la ruta de la API. Cambiar de
servidor no reutiliza el PAT guardado de otro punto de acceso. **Eliminar token guardado**
elimina solo la entrada del portal actual y vacía su campo en la interfaz gráfica. Las
demás plataformas mantienen credenciales solo de sesión; no se usa ninguna persistencia en
texto sin cifrar como alternativa.

Al actualizar desde una versión que mantenía los PAT solo en memoria, introduzca el PAT una
vez en el editor actualizado, seleccione **Recordar el token en este equipo** y pruebe la
conexión. No es necesario introducirlo en MCP. Las pruebas de conexión consultan el
servicio protegido de tokens del proveedor de archivos y los metadatos públicos del
catálogo; no descargan un archivo de datos comerciales ni dependen de que un artículo
concreto del catálogo tenga uno.

## Gestor de artículos

`openecraft-parts-manager` es una aplicación independiente que mantiene los artículos de un
servidor de artículos: crear artículos nuevos, editar los existentes (incluidos los modelos
2D / 3D), renombrar y eliminar artículos, subir archivos de artículos CSV / JSON y exportar
todos los artículos. Usa el mismo editor de artículos que la gestión de artículos del
editor y funciona en el escritorio y en el navegador.

```bash
cargo run --release -p openecraft-parts-manager        # desktop
cd parts-manager && trunk serve                        # web, http://127.0.0.1:8082
cd parts-manager && trunk build --release              # static files in parts-manager/dist/
```

*Aviso legal y privacidad* de la barra superior muestra la exención de responsabilidad y el
aviso de privacidad del gestor de artículos y, en una compilación web cuyo `config.js`
nombra a un operador, su aviso legal
([installation.md](installation.md#aviso-legal-de-un-editor-web)).

Introduzca la URL del servidor y el token de acceso (se mantiene solo en memoria) y haga
clic en *Conectar*, o inicie sesión con un nombre de usuario y una contraseña. Los cambios
se guardan en el servidor con *Guardar* (`Ctrl+S`); el editor pregunta antes de descartar
los cambios sin guardar y antes de eliminar un artículo. *Archivos e historial* encima del
editor de artículos sube y quita los archivos de un artículo y muestra sus cambios;
*Catálogos* mantiene los catálogos de fabricantes (con sus logotipos: *Elegir logotipo…*,
*Quitar logotipo*, para un catálogo sin cambios sin guardar) y *Usuarios*
(administradores) las cuentas y los roles. *Logotipos* junto al menú de idioma muestra u
oculta los logotipos en la lista de catálogos (se recuerda).
