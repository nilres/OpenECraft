# Línea de comandos, servidor MCP y plugins

Proyectos y el servidor de artículos sin ventana, para scripts y agentes de IA.

`openecraft-cli` edita proyectos y mantiene el servidor de artículos sin ventana. Los mismos
comandos se ofrecen a los agentes de IA mediante el [Model Context Protocol](https://modelcontextprotocol.io)
(`openecraft-cli mcp`, JSON-RPC por stdin / stdout), de modo que un agente puede crear proyectos,
colocar símbolos, trazar hilos entre puntos de conexión, asignar artículos, ejecutar la
comprobación, generar informes y exportar PDF / SVG / DXF / DWG.

```bash
cargo run --release -p openecraft-cli -- help                 # all commands
cargo run --release -p openecraft-cli -- help connect         # parameters of a command
cargo run --release -p openecraft-cli -- --version             # also -V or version
openecraft-cli new_project --name "Pump control" --path pump.oec
openecraft-cli new_project --example star_delta_starter --path star_delta.oec   # see list_examples
openecraft-cli -p pump.oec place_symbol --page 1 --symbol pb_no --x 200 --y 100   # -S1
openecraft-cli -p pump.oec place_symbol --page 1 --symbol coil --x 200 --y 160    # -K1
openecraft-cli -p pump.oec place_part --page 1 --part GEN.LMP-GN-24 --x 240 --y 160   # -P1 with its part
openecraft-cli -p pump.oec connect --page 1 --from -S1:14 --to -K1:A1
openecraft-cli -p pump.oec check
openecraft-cli -p pump.oec export --format pdf --path pump.pdf
openecraft-cli --server http://127.0.0.1:8090 --token <secret> parts_import --path parts.csv
openecraft-cli --server http://127.0.0.1:8090 --token <secret> parts_import --path 6ED1052-1CC08-0BA3.edz
openecraft-cli --project drawing.oec import_eplan_symbols --path drive.edz
openecraft-cli --project drawing.oec import_eplan_symbols --path Beckhoff --part BEC.EK1100
```

- En la línea de comandos cada cambio se escribe de inmediato en `--project`. En una sesión
  MCP el proyecto permanece abierto en memoria: `save_project` lo escribe, `undo` / `redo`
  revierten llamadas individuales a herramientas (`help mcp`); la línea de comandos rechaza
  `undo` / `redo`, ya que abre el proyecto de nuevo para cada comando. Un archivo comprimido
  indicado como `--project` no se modifica: guarde el proyecto como archivo de proyecto
  (`save_project --path <file>.oec`) o convierta el archivo comprimido (`convert_project`).
- Un **proyecto dividido** guarda el proyecto en muchos archivos pequeños para el control
  de versiones (git): el índice `<name>.oecp` y, junto a él, una carpeta `pages` (un
  archivo por página), `data` (un archivo por dispositivo, artículo, cable … y por área de
  los datos de todo el proyecto; los archivos de una clase se reparten en 16 carpetas, para
  que ninguna crezca demasiado) y `files` (los documentos e imágenes incrustados como
  archivos normales). Contiene el mismo proyecto que el archivo `.oec`:
  `convert_project --path plant.oec --format oecp` escribe `plant/plant.oecp`,
  `--format oec` o `--format oecz` lo convierten de vuelta, y `--project
  plant/plant.oecp` trabaja con él como con un archivo de proyecto: un cambio reescribe
  solo los archivos a los que afecta. Cada archivo es una línea de JSON y depende solo del
  contenido del proyecto, de modo que guardar un proyecto sin cambios no modifica nada. Un
  proyecto dividido necesita una carpeta propia; los demás archivos que haya en ella (un
  README, `.gitattributes`) no se tocan. Una carpeta que no tiene índice pero ya contiene
  archivos JSON en `pages`, `data` o `files` se rechaza: son de otro y se sustituirían o
  eliminarían.
- Guardar un proyecto dividido grande cuesta lo que ha cambiado, no lo que contiene el
  proyecto: solo se escriben las páginas y los datos que difieren del último guardado (un
  proyecto de 500 páginas y 20 000 dispositivos guarda una página modificada en mucho
  menos de un segundo). Un guardado es un solo paso: los archivos modificados se escriben
  junto a los antiguos, un diario (`.oecp-save`) los nombra, y solo entonces ocupan su
  sitio; tras un fallo el proyecto está como antes del guardado, o el siguiente acceso lo
  termina.
- **Git** necesita dos cosas para un proyecto dividido. El `.gitattributes` que el guardado
  escribe en la carpeta (salvo que ya exista uno) impide que git convierta los finales de
  línea, lo que cambiaría todos los archivos y dañaría los incrustados; un `.gitignore`
  escrito con él deja fuera el diario y los archivos temporales de un guardado en curso. Y el controlador de fusión fusiona
  los archivos JSON registro a registro, de modo que los cambios en símbolos distintos de
  una página, o en campos distintos de uno, no entran en conflicto; configúrelo una vez por
  clon:

  ```bash
  git config merge.openecraft.name "OpenECraft split project"
  git config merge.openecraft.driver "openecraft-cli merge_project_file --base %O --ours %A --theirs %B --name %P"
  ```

  Donde ambos lados han cambiado el mismo valor, el archivo conserva el valor de la rama
  actual, el comando nombra los lugares y git marca el archivo como en conflicto: abra el
  proyecto, ponga lo que sea correcto y haga `git add` del archivo. Sin el controlador, git
  fusiona los archivos como texto y escribe en ellos ambas versiones; un archivo así se
  nombra al abrir el proyecto: tome una versión (`git checkout --ours` o `--theirs`) y
  configure el controlador.
- **Identificadores de los objetos nuevos.** Cada copia de un proyecto dividido empieza los
  identificadores de sus objetos nuevos en un lugar aleatorio por encima de los
  identificadores en uso, para que dos personas que añaden objetos en sus ramas no les den
  los mismos identificadores (por eso los identificadores crecen a saltos, de hasta
  67 millones por sesión de edición que añade objetos). Es una protección con límites, no
  una garantía:
  - La probabilidad de aproximadamente 1 entre 30 000 para dos copias editadas al mismo
    tiempo vale mientras cada una añada menos de 1 024 objetos. Una sesión que importa una
    macro grande o copia muchas páginas abarca muchos de los lugares en los que empiezan
    los identificadores: dos copias que añaden cada una unos 50 000 objetos coinciden
    aproximadamente 1 vez de cada 650.
  - `openecraft-cli -p plant/plant.oecp check_project_data` informa de los identificadores
    usados más de una vez y `--repair` los renumera: ejecútelo después de una fusión. Una
    referencia a un identificador así (una conexión a un objeto) no puede saber a cuál de
    los dos objetos se refería y puede acabar en el equivocado, así que revise lo que
    nombra la reparación.
  - Solo la sesión que edita un proyecto empieza sus identificadores de nuevo; leerlo (la
    comparación, una plantilla, la base de datos de proyectos) no lo hace, y un proyecto
    creado a partir de una plantilla sigue contando.
- **Cambios de otro programa mientras el proyecto está abierto** (un `git pull`, un segundo
  editor): el guardado escribe solo los archivos que este lado ha cambiado y conserva los
  demás; el editor y una sesión MCP leen entonces el proyecto de nuevo, de modo que muestra
  ambos. Un archivo cambiado en ambos lados no se sobrescribe: el guardado falla y nombra
  los archivos. Guarde con otro nombre, o abra el proyecto de nuevo y repita los cambios.
  El editor conserva los pasos de deshacer (deshacer no revierte lo que cambió el otro
  programa) y recuerda los archivos entre ejecuciones, de modo que un pull hecho mientras
  estaba cerrado se trata del mismo modo.
- Los parámetros son `--name value` (también `--name=value`, o todos a la vez con `--json '{…}'`);
  las listas como `--ids 3,4` y los puntos como `--points "0,0 40,0"`; las listas de
  códigos, identificadores de dispositivo o idiomas se separan con comas o espacios. Los
  valores fuera de su rango se rechazan indicando el rango permitido, no se recortan, y un
  comando que falla deja el proyecto sin cambios. Los números de las respuestas se
  redondean a la precisión con la que se almacenan.
- Los parámetros renombrados desde versiones anteriores se siguen aceptando con sus nombres
  antiguos: `rename_device` `old` / `new` (ahora `tag` / `new_tag`), `rename` de
  `set_potential`, `set_fluid_group` y `set_pct_loop` (ahora `new_name`), `replace_text`
  `text` / `with` (ahora `find` / `replace`).
- Servidor de artículos: `--server` / `--token`, las variables de entorno
  `OPENECRAFT_SERVER` / `OPENECRAFT_TOKEN` o la herramienta `parts_connect` (también con
  `user` / `password`); por defecto el servidor de artículos público
  `https://parts-server.openecraft.org`
  ([servidor de artículos público](parts-server.md#servidor-de-artículos-público)). `parts_connect` con `kind: Eplan` lee en su lugar el EPLAN
  Data Portal (el token es el token de acceso personal; solo búsqueda, `parts_get`,
  `parts_file` y `parts_check_updates`). Las herramientas del portal de datos buscan con
  facetas (`parts_search`), gestionan catálogos, archivos, favoritos y usuarios
  (`parts_catalog_save`, `parts_attach`, `parts_favorites`, `parts_user_save`, …), comparan
  los artículos del proyecto con el servidor (`parts_check_updates`) y comparten los
  formularios de la empresa (`parts_company_forms` (`publish`)).
- La ayuda y los mensajes siguen `--lang` o el idioma del sistema; el servidor MCP responde
  en inglés salvo que se indique `--lang`.

- `OPENECRAFT_MCP_ROOT` es la carpeta en la que deben estar los archivos de las
  herramientas MCP (`path`, `output`, `folder`, `database`, `target` y los archivos de
  plantilla); por defecto la carpeta de trabajo del servidor, `*` permite todas las
  carpetas. Un agente engañado por un texto que lee (un proyecto, un artículo, una página
  web) no puede leer ni sobrescribir archivos en otro lugar, como otros proyectos o
  carpetas de plugins. Las rutas relativas están en `OPENECRAFT_MCP_ROOT`, también cuando
  el cliente inicia el servidor en una carpeta propia (Claude Desktop); las instrucciones
  del servidor indican la carpeta al agente.
- `OPENECRAFT_MCP_TOOLS` elige las herramientas que ofrece el servidor MCP. Por defecto
  ofrece las herramientas básicas para esquemas (`core`, unas 50: proyectos, páginas,
  símbolos, hilos, cables, artículos, comprobación, informes, exportaciones; unos 30 kB) y
  llega a las demás (todas juntas se describen en unos 260 kB) mediante tres herramientas
  propias: `find_tools` (nombres, o las herramientas cuyo nombre o descripción contiene las
  palabras de una consulta), `describe_tool` (descripción y parámetros) y `call_tool`
  (llama a cualquier herramienta por su nombre). Los nombres separados por comas eligen
  otras herramientas, `parts_*` todas las herramientas que empiezan por `parts_`, `*` todas
  (p. ej. `core,panel*,parts_*`).
- Sin servidor de artículos, `parts_search` y `parts_get` usan los artículos del proyecto y
  el catálogo integrado (`parts_search --local` siempre), y `assign_part` nombra artículos
  parecidos para un número que no conoce.
- `new_project --path` no sustituye un archivo existente salvo que se indique `overwrite`.
  `search_projects` busca en la carpeta actual salvo que se indique una base de datos o una
  carpeta.

El servidor MCP es el `openecraft-cli` de los paquetes: en Windows el componente del
instalador *Línea de comandos y servidor MCP para agentes de IA* (tipo de instalación
*Instalación completa*, no forma parte de *Editor*), en
`%LOCALAPPDATA%\Programs\OpenECraft\` (o `C:\Program Files\OpenECraft\` para todos los
usuarios); en macOS `/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli`; en Linux
`/usr/bin/openecraft-cli` (deb, rpm), `~/.local/bin` (tar.gz), `<AppImage> cli`,
`flatpak run --command=openecraft-cli io.github.nilres.OpenECraft` u `openecraft.cli`
(snap). Regístrelo en un agente, p. ej. Claude Code (iniciado en la carpeta de los
proyectos):

```bash
claude mcp add openecraft -- /path/to/openecraft-cli mcp
claude mcp add --scope user -e OPENECRAFT_MCP_ROOT=/home/me/projects openecraft -- openecraft-cli mcp
```

o en la configuración `mcpServers` de otros clientes (Claude Desktop:
`claude_desktop_config.json` en `%APPDATA%\Claude\`, `~/Library/Application Support/Claude/`
o `~/.config/Claude/`; Cursor: `~/.cursor/mcp.json`; VS Code: `.vscode/mcp.json` con
`servers` en lugar de `mcpServers`):

```json
{ "mcpServers": { "openecraft": { "command": "openecraft-cli", "args": ["mcp"],
    "env": { "OPENECRAFT_MCP_ROOT": "/home/me/projects" } } } }
```

Sin instalación, el servidor existe como paquete MCP de la versión publicada
(`OpenECraft-MCP-<version>-windows-x86_64.mcpb`, `-linux-x86_64.mcpb`,
`-macos-universal.mcpb`): Claude Desktop y otros clientes que conocen el formato lo
instalan con un doble clic y preguntan por la carpeta de proyectos
(`OPENECRAFT_MCP_ROOT`) y las herramientas (`OPENECRAFT_MCP_TOOLS`). Cada versión inscribe
estos paquetes en el MCP Registry oficial como `io.github.nilres/openecraft`.

El agente puede leer y escribir cualquier archivo al que tenga acceso el proceso del
servidor; inícielo en un directorio de trabajo (y con un token del servidor de artículos)
que le parezca bien que use el agente.
[Deje que un agente de IA dibuje su esquema](../launch/ai/README.md) contiene un ejemplo
desarrollado con su transcripción, peticiones de ejemplo y la configuración de más clientes.

## Plugins

Los plugins añaden comandos, comprobaciones, informes y exportaciones al editor (pestaña
*Plugins*: *Comandos de plugin* lista cada comando por plugin con una búsqueda, la estrella
fija uno en la pestaña; pestaña *Revisión*), a la línea de comandos y al servidor MCP al
mismo tiempo. Los plugins integrados son código Rust en `crates/openecraft-api`; los
plugins externos son programas en cualquier lenguaje que responden a peticiones JSON por
stdin / stdout:

```bash
OPENECRAFT_PLUGINS=examples/plugins openecraft-cli -p pump.oec plugins
openecraft-cli --plugins examples/plugins -p pump.oec report --kind page_notes
```

Véase [docs/plugins.md](plugins.md) para la API y el protocolo.
