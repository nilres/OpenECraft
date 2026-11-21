# Servidor de proyectos

Proyectos compartidos para un equipo: varios usuarios trabajan en el mismo proyecto y cada página la bloquea uno de ellos.

`openecraft-project-server` almacena proyectos para un equipo. Varios usuarios abren el
mismo proyecto al mismo tiempo y trabajan en páginas distintas; una página la edita un solo
usuario a la vez.

```bash
cargo run --release -p openecraft-project-server -- --bind 0.0.0.0:8091 --data projects --token <secret>
cargo run --release -p openecraft-project-server -- --config project-server.toml
```

Como contenedor: `docker run -d -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret>
-v projects:/data ghcr.io/nilres/openecraft-project-server` (véase [instalación](installation.md)).

- Archivo de configuración (`--config <file>` u `OPENECRAFT_PROJECT_SERVER_CONFIG`): los
  ajustes de las opciones de la línea de comandos en TOML (`bind`, `data`, `token`,
  `admin_user`, `access`, `web`; véase
  [`project-server/project-server.example.toml`](../../project-server/project-server.example.toml)),
  como para el servidor de artículos.
- **Página web** (opcional: `--web`, `--web true`, `OPENECRAFT_PROJECT_WEB=true` o
  `web = true` en el archivo de configuración; desactivada por defecto):
  `http://<server>:8091/` lista los proyectos que el usuario puede ver (con las páginas, la
  revisión, quién los creó y los modificó por última vez, quién trabaja en ellos y los
  derechos del usuario) y muestra cada uno en solo lectura: las páginas por sección o
  agrupadas por estructura, los dispositivos con los lugares en los que se muestran, y las
  páginas dibujadas como en papel (zoom, arrastrar para mover, teclas de flecha o Re Pág /
  Av Pág para la página siguiente). Las páginas bloqueadas por otros usuarios se marcan con
  🔒 y el usuario; la página consulta el proyecto cada 20 s y sigue las revisiones nuevas.
  Los usuarios inician sesión con su cuenta o con el token del servidor; sin inicio de
  sesión la página muestra lo que pueden ver las peticiones anónimas (los ajustes de
  acceso). Los jefes de proyecto editan las asignaciones de un proyecto (*Permisos*). Los
  administradores disponen de un área de *Administración*: una vista general (proyectos en
  uso, sesiones abiertas, usuarios por rol, ajustes de acceso), las sesiones (las abiertas
  con su usuario, proyecto, inicio, última actividad, guardados y páginas bloqueadas, y el
  historial de las terminadas con su duración y cómo terminaron: cerrada, caducada, sesión
  cerrada o usuario eliminado), los usuarios (rol, proyectos, grupos, contraseña), los
  derechos de acceso (acceso sin inicio de sesión, roles definidos, asignaciones) y los
  directorios (Active Directory, LDAP, con una prueba de la conexión y de un inicio de
  sesión). *Informe de inspección y ensayo* de un proyecto muestra su informe para la
  inspección en obra, hecho para una tableta o un teléfono: los revisores y los editores
  introducen allí los datos del informe, la inspección visual y los resultados medidos de
  cada circuito, sin abrir el proyecto y sin bloqueo, firman el informe y lo descargan como
  libro de Excel
  ([informe de inspección y ensayo](features/calculations.md#informe-de-inspección-y-ensayo)).
  La página está en todos los idiomas de la aplicación; usa la API indicada más abajo, de
  modo que el servidor comprueba cada petición como para el editor. Sin `--web` el servidor
  responde a `/` con un texto breve y no tiene rutas de página.
- **Creado por, última modificación por, última modificación**: el servidor guarda para
  cada proyecto quién lo creó y quién guardó su último cambio, y cuándo; la lista de
  proyectos del editor (panel *Servidor de proyectos*), la página web y
  `GET /api/v1/projects` (`created_by`, `created_at`, `changed_by`, `changed_at`; segundos
  desde 1970, UTC) los muestran a todo el que pueda ver el proyecto. El nombre es el del
  usuario con sesión iniciada; en un servidor sin cuentas (o con el token del servidor) es
  el nombre de usuario con el que se abrió la sesión. Es independiente de *Creado por* en
  las propiedades del proyecto, que editan los usuarios. Solo cuenta un guardado que cambia
  el proyecto: abrir o leer un proyecto, las peticiones de estado que mantienen viva una
  sesión, los bloqueos y guardar un contenido sin cambios lo dejan como está. Los proyectos
  de versiones anteriores del servidor muestran *Desconocido* hasta su siguiente cambio (la
  creación sigue siendo desconocida), y un proyecto subido como copia de otro es un
  proyecto nuevo del usuario que lo subió.
- Los nombres de proyecto que solo difieren en mayúsculas y minúsculas son un mismo nombre
  (Windows y macOS no distinguen sus archivos): las peticiones encuentran un proyecto con cualquier grafía de su nombre; los nombres de dispositivo de Windows (`CON`, `NUL`, `COM1` …) y los nombres que terminan
  en punto se rechazan.
- Cada proyecto se almacena como proyecto dividido en una carpeta propia
  (`<name>/<name>.oecp` con un archivo pequeño por página, dispositivo y artículo; se abre
  en el editor de escritorio y se puede mantener bajo control de versiones) junto a
  `<name>.state.json` (revisión, contador de identificadores, quién creó y modificó por
  última vez el proyecto) en el directorio de datos. Un guardado escribe solo los archivos que han cambiado. Los archivos de proyecto de versiones anteriores (`<name>.oec`) se convierten en el
  primer inicio y se conservan como `<name>.oec.bak`.
- Con `--token` (u `OPENECRAFT_PROJECT_TOKEN`) cada petición necesita
  `Authorization: Bearer <token>`; sin él, sin cuentas de usuario y sin directorios, todo
  el que llega al servidor puede modificar sus proyectos, pero solo los clientes de la
  propia máquina del servidor pueden crear usuarios y cambiar los ajustes de acceso (para
  que nadie más se apodere del servidor creando el primer administrador).
- El servidor habla HTTP sin cifrar, de modo que los tokens y las contraseñas cruzan la red
  sin cifrar. Manténgalo en `127.0.0.1` detrás de un proxy inverso con TLS cuando sea
  accesible desde otras máquinas, p. ej. con [Caddy](https://caddyserver.com/), que obtiene
  el certificado: `caddy reverse-proxy --from projects.example.com --to 127.0.0.1:8091`.
  El editor usa entonces `https://projects.example.com`.
- **Cuentas de usuario** (`users.json` en el directorio de datos, hashes de contraseña
  Argon2): los lectores abren los proyectos en solo lectura, los revisores además guardan
  comentarios, marcado y aprobaciones, los editores bloquean, modifican, guardan y crean
  proyectos, los jefes de proyecto además eliminan proyectos y gestionan quién puede
  usarlos, los administradores además gestionan los usuarios; el rol de un usuario se puede
  limitar a algunos proyectos (nombres con `*` / `?`). `--admin-user <name>`
  (u `OPENECRAFT_PROJECT_ADMIN_USER`) crea el primer administrador con la contraseña de
  `OPENECRAFT_PROJECT_ADMIN_PASSWORD` (al menos 8 caracteres, como toda contraseña). Los
  usuarios inician sesión desde *Herramientas › Servidor de proyectos…* (nombre de usuario,
  contraseña, *Iniciar sesión*); el servidor usa el nombre de la cuenta en los bloqueos,
  diga lo que diga la petición, y para los comentarios, respuestas, marcado y aprobaciones
  que añade un usuario con sesión iniciada (las aprobaciones de otros usuarios quedan como
  están). Los llamantes anónimos no pueden usar el nombre de una cuenta. Los gestores de un
  proyecto pueden añadirle denegaciones, pero solo los administradores las levantan. Los
  nombres de usuario tienen como máximo 64 caracteres y ningún carácter de control; un
  usuario nuevo necesita un rol, grupos de directorio o una asignación en los ajustes de
  acceso. Cerrar sesión cierra los proyectos abiertos con ese inicio de sesión y libera sus
  bloqueos; eliminar un usuario cierra todos los suyos. Un inicio de sesión fallido
  responde `wrong user name or password`, sea cual sea de los dos el incorrecto.
- **Control de acceso** (`access.json` en el directorio de datos, `--access`): los usuarios
  inician sesión con una cuenta local o con directorios Active Directory / LDAP; los roles
  (integrados o definidos por los administradores) se asignan a usuarios y grupos de
  directorio por proyecto, las asignaciones de denegación quitan derechos. Quien crea un
  proyecto lo gestiona; los jefes de proyecto editan sus asignaciones en la sección *Acceso
  al proyecto* de la ventana del servidor de proyectos. Véase
  [docs/access-control.md](access-control.md).
- Bloqueos: cada página y los datos del proyecto (lista y orden de las páginas,
  dispositivos, artículos, diseño del armario, ajustes) se bloquean por separado. Editar
  algo que no se ha bloqueado solicita el bloqueo automáticamente; si lo tiene otro
  usuario, el cambio se deshace. Las páginas bloqueadas por otros son de solo lectura y se
  marcan con 🔒 y el nombre del usuario en la lista de páginas. Los bloqueos terminan
  cuando se liberan, cuando se cierra el proyecto o 90 s después de que el editor dejara de
  enviar su estado (cada 15 s).
- El guardado fusiona en el proyecto del servidor solo lo que usted ha bloqueado; todo lo
  demás procede del servidor. Los cambios de otros usuarios se cargan automáticamente.
  Un guardado nombra la revisión a partir de la cual se editó (`base_revision`; el editor
  lo hace): el servidor entonces también acepta las páginas y los datos del proyecto
  modificados que nadie ha bloqueado (primero los bloquea para usted) y responde con los
  cambios que dejó fuera (`ignored`): bloqueados por otro usuario, modificados por otro
  usuario desde esa revisión o no permitidos. Con `strict` no se guarda nada en ese caso
  (`409`). El servidor mantiene en memoria las últimas 16 revisiones por proyecto; un
  guardado desde una revisión más antigua (o sin `base_revision`) guarda solo lo bloqueado.
- Cada sesión recibe su propio rango de identificadores de objeto, de modo que los usuarios
  nunca crean el mismo identificador.
- **Historial de sesiones**: cada sesión terminada (usuario, proyecto, inicio, última
  actividad, fin, guardados, cómo terminó) se añade a `sessions.jsonl` en el directorio de
  datos, una línea JSON cada una; el servidor conserva las últimas 5000 y escribe el
  archivo de nuevo cuando ha crecido hasta el doble de líneas. Las claves de las sesiones
  nunca se almacenan ni se muestran, solo sus números.
- Los editores prefieren `POST /projects/{name}/snapshot`, ligado a la sesión. Solo con 404
  o 405 usan la instantánea GET anterior seguida del estado de la sesión actual. Ambos
  deben nombrar la misma revisión antes de aplicar la instantánea; el estado aporta las
  identidades de los bloqueos propios. Los fallos de autenticación no desencadenan esta
  alternativa. Un proyecto modificado durante la carga se carga de nuevo.
- En el editor: *Herramientas › Servidor de proyectos…*: introduzca la URL del servidor, su
  nombre de usuario y el token, liste los proyectos y abra uno, o suba el proyecto actual.
  Mientras hay un proyecto compartido abierto, *Guardar* (`Ctrl+S`) guarda en el servidor;
  la ventana y el menú contextual de la lista de páginas bloquean y liberan páginas. La
  barra de estado muestra *Servidor de proyectos: … / Proyecto: …* (*conexión perdida*
  mientras el servidor no responde a su estado); un proyecto restaurado de la última sesión
  o cerrado en el servidor se muestra como *Copia de recuperación local del proyecto del
  servidor: …* y se guarda en un archivo local.
- **Guardado automático:** mientras hay un proyecto compartido abierto, sus cambios
  terminados se guardan solos en el servidor, 30 segundos después de un cambio y luego cada
  30 segundos mientras el trabajo continúa (*Archivo › Ajustes › Servidores* o el panel
  *Servidor de proyectos*: activado / desactivado y el intervalo, de 10 a 600 s). Un
  arrastre o una entrada de texto en curso no se envía; se guarda en cuanto termina. Solo
  se ejecuta un guardado a la vez, un guardado automático no libera ningún bloqueo y no se
  envía nada sin cambios. *Guardar* (`Ctrl+S`) sigue guardando de inmediato. La barra de
  estado y el panel distinguen los estados: *Guardado*, *Aún no guardado*, *Guardando…*,
  *Esperando al servidor* (sin respuesta, sesión terminada, sesión cerrada: los cambios se
  guardan en cuanto la sesión continúa), *Error al guardar* (se reintenta con el siguiente
  intervalo) y *Guardado en parte* (el servidor dejó fuera lo que otro usuario ha bloqueado
  o modificado; su versión se conserva en el panel y no se vuelve a enviar sola), con la
  hora del último guardado que confirmó el servidor. Un proyecto cuenta como guardado solo
  tras la respuesta del servidor. La copia de recuperación local no es un guardado en el
  servidor: durante una interrupción se escribe de inmediato, y el panel lo dice solo
  después de que se haya escrito.
- La lista de proyectos selecciona un proyecto (un doble clic lo abre). *Propiedades…*
  carga sus propiedades desde el servidor (nunca desde la lista) y muestra quién lo creó en
  el servidor y cuándo (*Desconocido* para los proyectos creados antes de que el servidor
  lo guardara; no es el *Creado por* del plano, que editan los usuarios). Quien puede
  editar los datos del proyecto los cambia allí sin abrir el proyecto, salvo que una sesión
  haya bloqueado los datos del proyecto; el servidor sustituye entonces solo la información
  del proyecto, y se niega si se modificó desde que se cargó (*Recargar* muestra la más
  reciente). Para el proyecto compartido abierto, *Propiedades…* es el diálogo habitual,
  que bloquea los datos del proyecto.
- *Copiar proyecto…* (para el proyecto compartido seleccionado o el abierto) muestra de
  inmediato las propiedades de la copia: un nombre libre en el servidor (*… copia*) y el
  título, la descripción, la empresa, el autor, la fecha, la revisión y las propiedades de
  proyecto del original, todo modificable. No existe nada en el servidor hasta *Crear y
  abrir*; *Cancelar* no deja nada, y una copia rechazada (nombre ocupado, sin derecho a
  crearla) conserva las entradas para corregirlas. La copia se hace en el servidor a partir
  del estado guardado: los cambios que nadie ha guardado todavía, tampoco los de otros
  usuarios, no forman parte de ella. Con cambios sin guardar en el proyecto abierto, el
  editor pregunta primero si se guardan. Si el original se modificó en el servidor después
  de que el diálogo lo cargara, la copia no se hace hasta que se confirme de nuevo. Las
  sesiones, los bloqueos de páginas y los derechos de acceso no se copian: la copia se abre
  en una sesión nueva y un usuario con sesión iniciada la gestiona como cualquier proyecto
  que crea. Los comentarios, el marcado, las aprobaciones (con sus firmas), las solicitudes
  de cambio, las revisiones liberadas y las líneas base se dejan fuera salvo que se marque
  *Copiar también revisiones, líneas base y notas de revisión*; los aprobadores del flujo
  de liberación se mantienen como todos los ajustes. El servidor registra quién creó la
  copia, cuándo, y a partir de qué proyecto y revisión.
- *Archivo › Ajustes › Al iniciar* decide lo que abre el editor cuando se inicia: la página
  de inicio con los archivos locales y los proyectos del servidor, el último proyecto (un
  proyecto del servidor se abre de nuevo en el servidor), la lista de proyectos del
  servidor o un archivo o proyecto del servidor elegido. Si el servidor no puede abrir el
  proyecto, un diálogo ofrece reintentar, iniciar sesión, elegir otro proyecto o seguir con
  la copia local; los cambios sin guardar de la copia se guardan o se descartan primero,
  nunca se sobrescriben en silencio.
- API REST (`/api/v1`): `GET /info` (nombre, versión y número de proyectos que puede ver el
  llamante; se responde también sin token), `GET|POST /projects` (`POST` necesita el
  `project`), `GET|PUT|DELETE /projects/{name}`,
  `POST /projects/{name}/open|status|lock|release|close`, `GET|PUT /projects/{name}/access`,
  `GET|PUT /projects/{name}/properties` (la información del proyecto con la revisión, quién
  ha bloqueado los datos del proyecto, los derechos del llamante, quién creó y modificó por última vez el proyecto y de qué se copió; `PUT`
  admite `user`, `revision` e `info`, cuenta como el último cambio, necesita el derecho a editar los datos del proyecto, responde `409`
  mientras los datos del proyecto están bloqueados y `412` si la información cambió desde `revision`),
  `POST /projects/{name}/copy` (`name` y `user` de la copia, opcionalmente la
  `revision` que el origen debe tener todavía – `412` en caso contrario –, su `info` y
  `records`; necesita el derecho a ver el origen y a crear el nombre nuevo, responde como
  al abrir un proyecto),
  `GET|PUT /projects/{name}/inspection?lang=de` (el informe de inspección y ensayo tal como
  está guardado: los datos del informe, la inspección visual, los circuitos con los datos
  del esquema, sus límites, resultados y veredictos, y el veredicto del informe,
  `InspectionSheet`; `PUT` admite `user`, `circuit` o `source` (ambos vacíos: los datos del
  informe y la inspección visual como `check.<item>`), el `rev` de su registro tal como se cargó y los
  `values` por campo, necesita el derecho a guardar revisiones o a editar, responde `409`
  si el registro cambió desde `rev` y cuenta como un cambio del proyecto),
  `GET /projects/{name}/inspection.xlsx?lang=de` (el informe como libro de Excel: un
  resumen, los circuitos y las fuentes),
  `POST /login` (`user` y `password`; `name` se acepta por `user`, como en el servidor de
  artículos), `POST /logout`, `GET /users`, `PUT|DELETE /users/{name}`, `GET|PUT /access`,
  `GET /directories`, `PUT|DELETE /directories/{id}`, `POST /directories/{id}/test`
  (administradores; las contraseñas de las cuentas de servicio nunca se devuelven),
  `GET /sessions?limit=` (administradores: las sesiones abiertas y las últimas terminadas,
  `SessionList`) (véase
  `openecraft_core::collab`). `GET /info` también indica el `user` con sesión iniciada (el
  token del servidor inicia sesión como `admin`), lo que puede hacer el llamante (`access`)
  y si la página web está activada (`web`). Con `--web` además `GET /texts?lang=de` (los
  textos de la página), `GET /projects/{name}/view?lang=de` (páginas, secciones,
  estructuras, dispositivos, bloqueos y los derechos del llamante, de `openecraft_view`) y
  `GET /projects/{name}/pages/{id}/svg` (una página como SVG); ambos necesitan el derecho a
  ver el proyecto y se dibujan a partir de un análisis por proyecto y revisión (se
  conservan los últimos 8 proyectos).

## Servidor de proyectos público

El proyecto mantiene un servidor de proyectos público en
[project-server.openecraft.org](https://project-server.openecraft.org) con su página web
activada. Para trabajar en él se necesita una cuenta: inicie sesión en la página web para
ver los proyectos que puede abrir, e introduzca la URL del servidor
`https://project-server.openecraft.org` en el panel *Servidor de proyectos* del editor e
inicie sesión allí con la misma cuenta. Muestra cómo varios usuarios trabajan en un
proyecto; los proyectos de una empresa deben estar en un servidor de proyectos propio
(véase arriba), detrás de sus propios ajustes de acceso.

## Aviso legal, exención de responsabilidad y aviso de privacidad

Al pie de la página web (`--web`) están los enlaces *Privacidad* y, cuando el operador tiene nombre, *Aviso
legal* (en alemán *Datenschutz* e *Impressum*), cada uno de los cuales abre un diálogo, y
las exenciones de responsabilidad de OpenECraft (sin garantía, los nombres de producto que
menciona pertenecen a sus propietarios). Están ahí sin inicio de sesión, como en el
[sitio web](../website.md#legal-notice), cuyos campos y variables son los mismos:

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
recordados durante poco tiempo; quién trabaja en qué proyecto y cuándo (sesiones, bloqueos, las últimas sesiones terminadas) y quién modificó un proyecto por última vez. Lo que solo sabe el operador (quién aloja el
servidor, qué registra un proxy inverso y durante cuánto tiempo, a quién preguntar, como
pide el artículo 13 del RGPD) se define en `privacy` y se muestra debajo. Mantenga los
textos veraces: un cambio que almacena o envía otra cosa los cambia también (`renderLegal`
de `common.js`, `privacy()` del script de la página). Si un servidor necesita un aviso
legal, y qué debe decir, depende del operador y del país: OpenECraft solo muestra lo que
está definido.
