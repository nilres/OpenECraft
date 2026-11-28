# Instalación y compilación

Paquetes para Windows, Linux y macOS, la versión web y la compilación desde el código
fuente.

Las versiones publicadas están en la [página de versiones](https://github.com/nilres/OpenECraft/releases)
con un archivo `SHA256SUMS`; cómo se elaboran se describe en [releasing](../releasing.md).

Compilar desde el código fuente requiere una cadena de herramientas estable y reciente
de Rust (`rustup update stable`); las versiones se compilan con la versión fijada en
`.github/workflows/ci.yml`. Los scripts de instalador que siguen y
`trunk build --release` necesitan además
[cargo-about](https://github.com/EmbarkStudios/cargo-about)
(`cargo install cargo-about@0.9.3 --locked --features cli`) para
`THIRD-PARTY-LICENSES.html`, que cada paquete contiene junto a `LICENSE-MIT` y
`LICENSE-APACHE`.

## Escritorio (Windows / Linux / macOS)

```bash
cargo run --release -p openecraft
```

En Linux se necesitan las bibliotecas habituales de ventanas y diálogos, p. ej. en
Debian/Ubuntu:

```bash
sudo apt install libxkbcommon-dev libgtk-3-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

Un archivo de proyecto indicado en la línea de comandos se abre al iniciar
(`openecraft plant.oec`).

## Instalador de Windows

`OpenECraft-<version>-setup.exe` instala el editor con una entrada en el menú Inicio, sin
derechos de administrador (o para todos los usuarios en Archivos de programa), en inglés,
alemán, francés o español según el idioma de Windows. Opcional: el gestor de artículos,
la línea de comandos / servidor MCP (añadida al `PATH` si se desea), los servidores de
artículos y de proyectos, un acceso directo en el escritorio y la apertura de archivos
`.oec` / `.oecz` / `.oecp` con doble clic. Se desinstala en *Configuración ›
Aplicaciones*; los proyectos y los ajustes se conservan. La CI lo compila en cada push
(artefacto `openecraft-windows-installer`); en local necesita
[Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`):

```bash
powershell -ExecutionPolicy Bypass -File installer/windows/build.ps1   # → target/installer/
```

Los binarios enlazan estáticamente el runtime de C, por lo que no hace falta ningún
paquete redistribuible de Visual C++. Para una instalación desatendida (distribución de
software), ejecute el instalador con `/VERYSILENT /SUPPRESSMSGBOXES`, añada `/ALLUSERS`
para todos los usuarios y seleccione las partes con
`/COMPONENTS="main,partsmanager,cli,servers"` y `/TASKS="fileassoc,addtopath"`.
Con `WINDOWS_SIGN_CERT=<.pfx file>` y `WINDOWS_SIGN_PASSWORD` el script firma los
programas, el instalador y el desinstalador (signtool del Windows SDK); de lo contrario
quedan sin firmar y SmartScreen pide confirmación en la primera ejecución (véase
[releasing](../releasing.md)).
`installer/icon.py` dibuja el icono de la aplicación (`assets/icon.png`,
`assets/icon.ico`, `assets/icon.icns`, `assets/icon.svg`).

## Paquetes de Linux

`installer/linux/build.sh` compila los paquetes para la arquitectura de la máquina de
compilación en `target/installer/`. Cada uno contiene el editor y el gestor de artículos
(con entradas de menú en inglés, alemán, francés y español), la línea de comandos /
servidor MCP y los servidores de artículos y de proyectos. Con deb, rpm, Flatpak y
tar.gz, los archivos `.oec` / `.oecz` / `.oecp` se abren con doble clic.

```bash
bash installer/linux/build.sh                        # tar.gz, deb, rpm, AppImage
bash installer/linux/build.sh flatpak snap           # sandboxed packages
```

| Paquete | Archivo | Instalación | Herramienta de compilación |
| --- | --- | --- | --- |
| Debian, Ubuntu | `openecraft_<version>_amd64.deb` | `sudo apt install ./openecraft_*.deb` | `dpkg-dev` |
| Fedora, RHEL, openSUSE | `openecraft-<version>-1.x86_64.rpm` | `sudo dnf install ./openecraft-*.rpm` | `rpmbuild` |
| Cualquier distribución | `OpenECraft-<version>-x86_64.AppImage` | `chmod +x`, iniciarlo | Debian o Ubuntu, `patchelf`, `zsync`, `appimagetool` y su runtime (descargados, sumas de comprobación verificadas) |
| Cualquier distribución | `OpenECraft-<version>-x86_64.flatpak` | `flatpak install --user OpenECraft-*.flatpak` | `flatpak-builder` |
| Cualquier distribución | `openecraft_<version>_amd64.snap` | `sudo snap install --dangerous openecraft_*.snap` | `snapcraft` |
| Cualquier distribución | `OpenECraft-<version>-linux-x86_64.tar.gz` | `./install.sh` (en `~/.local`) | `tar` |

Los programas son `openecraft`, `openecraft-parts-manager`, `openecraft-cli`,
`openecraft-parts-server` y `openecraft-project-server`. En el AppImage se inician con
`OpenECraft-*.AppImage parts-manager|cli|parts-server|project-server …`, en el Flatpak
con `flatpak run --command=openecraft-cli io.github.nilres.OpenECraft …` y en el snap
como `openecraft.cli`, `openecraft.parts-manager`, … La CI los compila en cada push
(artefactos `openecraft-linux-packages`, `openecraft-flatpak`, `openecraft-snap`); deb,
rpm, AppImage y tar.gz se compilan en Debian 12 y funcionan con glibc 2.36 o posterior.
El AppImage lleva las bibliotecas de ventanas que no están en todos los sistemas
(xkbcommon, Xcursor, Xi: `appimage_libs` en `installer/linux/build.sh`); la compilación
rechaza uno que necesite otra biblioteca o una glibc más reciente que los programas.
Lleva también información de actualización para AppImageUpdate y herramientas similares:
la última versión de GitHub del repositorio que lo compiló (`GITHUB_REPOSITORY`, si no
`nilres/OpenECraft`), cuyo `OpenECraft-<version>-x86_64.AppImage.zsync` escribe la
compilación junto al AppImage (`APPIMAGE_UPDATE_INFO` fija otra información de
actualización, un valor vacío ninguna).
Las entradas de escritorio, los tipos MIME y los datos AppStream
(`io.github.nilres.OpenECraft`) están en `installer/linux/`; una versión nueva necesita
un `<release>` en el archivo metainfo.

## Imagen de disco de macOS

`installer/macos/build.sh` compila `OpenECraft-<version>-macos.dmg` (macOS 11 o
posterior, binarios universales para Apple silicon e Intel) con las apps *OpenECraft* y
*OpenECraft Parts Manager*; arrástrelas a *Aplicaciones*. La línea de comandos y los
servidores están en el paquete de la app:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
bash installer/macos/build.sh                        # → target/installer/
/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli help
```

Sin `MACOS_SIGN_IDENTITY` las apps se firman ad hoc, por lo que Gatekeeper pide
confirmación en el primer inicio (*Ajustes del Sistema › Privacidad y seguridad › Abrir
igualmente*). Con un certificado Developer ID
(`MACOS_SIGN_IDENTITY="Developer ID Application: …"`) y un perfil de llavero de
`notarytool` (`MACOS_NOTARY_PROFILE`, `MACOS_NOTARY_KEYCHAIN` cuando no está en el
llavero de inicio de sesión) la imagen se firma y se notariza.
Los proyectos se abren con *Archivo › Abrir proyecto*; abrirlos con doble clic en el
Finder aún no es posible. La CI compila la imagen en cada push (artefacto
`openecraft-macos-dmg`).

## Actualizaciones

Al iniciarse, el editor pregunta a GitHub en segundo plano por la última versión
(`https://api.github.com/repos/nilres/OpenECraft/releases/latest`); sin conexión sigue
adelante sin ningún mensaje. Una versión más reciente se descarga, se comprueba con el
`SHA256SUMS` de la versión y se instala:

| Instalación | Actualización |
| --- | --- |
| Instalador de Windows | `OpenECraft-<version>-setup.exe` se ejecuta en silencio (`/SILENT`) cuando se cierra el editor, en la carpeta y con los componentes de la instalación; *Reiniciar y actualizar* vuelve a iniciar el editor después |
| AppImage | el archivo AppImage se sustituye de inmediato; la versión nueva se inicia la próxima vez (AppImageUpdate y herramientas similares también lo actualizan) |
| App de macOS | `OpenECraft.app` (y `OpenECraft Parts Manager.app` junto a ella) se sustituyen de inmediato desde la imagen de disco |
| deb, rpm, Flatpak, Snap, tar.gz, compilación propia | la barra de estado solo informa de la versión nueva y enlaza su página de versión |

La barra de estado muestra la descarga y *Reiniciar y actualizar*; *Archivo › Ayuda ›
Buscar actualizaciones* vuelve a mirar e informa de cada resultado. *Ajustes ›
Actualizaciones* desactiva la comprobación al inicio; la variable de entorno
`OPENECRAFT_NO_UPDATE_CHECK` (cualquier valor) lo hace para todos los usuarios de un
equipo. La versión web no tiene actualización propia.

Al primer inicio de una versión nueva, la ventana *Novedades de OpenECraft* muestra los
cambios de las versiones desde la que se usó por última vez, tal como los lista
`CHANGELOG.md` (en inglés). *Archivo › Ayuda › Historial de versiones* los vuelve a
mostrar: una página por versión, agrupadas por serie (`0.1`) en el panel Ayuda.

## Imágenes Docker de los servidores

El servidor de artículos y el servidor de proyectos se publican también como imágenes
Docker en cada versión y en cada push a `master`, compiladas por
`installer/docker/Dockerfile` (Debian 12, `linux/amd64`, ejecutándose como el usuario
`openecraft`, datos en el volumen `/data`, licencias en `/usr/share/doc/openecraft/`):

| Imagen | Puerto | Etiquetas |
| --- | --- | --- |
| `ghcr.io/nilres/openecraft-parts-server` | 8090 | `<version>`, `<major>.<minor>`, `latest` (versiones), `edge` (master), `sha-<commit>` |
| `ghcr.io/nilres/openecraft-project-server` | 8091 | las mismas |

```bash
docker run -d --name parts -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data ghcr.io/nilres/openecraft-parts-server
docker run -d --name projects -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret> -v projects:/data ghcr.io/nilres/openecraft-project-server
```

Los ajustes de cada imagen están en `/etc/openecraft/parts-server.toml` y
`project-server.toml` (`bind = "0.0.0.0:<port>"`, `data` en `/data`); monte allí su
propio archivo o añada opciones de línea de comandos después del nombre de la imagen
(`… openecraft-parts-server --install-catalogs all`). El primer administrador viene
también del entorno: `-e OPENECRAFT_ADMIN_USER=admin -e
OPENECRAFT_ADMIN_PASSWORD=<password>` (`OPENECRAFT_PROJECT_ADMIN_USER` /
`OPENECRAFT_PROJECT_ADMIN_PASSWORD` para el servidor de proyectos) crea la cuenta, o
restablece su contraseña, en cada inicio.
Un montaje de enlace en lugar de un volumen debe ser escribible por el usuario de la
imagen (uid 1000, o inicie el contenedor con `--user`). `installer/docker/compose.yaml`
inicia ambos servidores con Docker Compose. Los servidores hablan HTTP sin cifrar:
manténgalos en `127.0.0.1` detrás de un proxy inverso TLS cuando otras máquinas accedan
a ellos ([servidor de proyectos](project-server.md)). Para compilar las imágenes por su
cuenta:

```bash
docker build -f installer/docker/Dockerfile --target parts-server -t openecraft-parts-server .
docker build -f installer/docker/Dockerfile --target project-server -t openecraft-project-server .
```

La CI compila ambas imágenes en cada push, las inicia una vez y comprueba que responden;
en `master` y para una etiqueta de versión las sube al GitHub Container Registry
(job `docker`), junto con la imagen del editor web (más abajo).

Cada imagen tiene un workflow que la recompila sola, sin el resto de la CI, iniciado a
mano en *Actions* (› Run workflow):

| Imagen | Workflow | Comprobación |
| --- | --- | --- |
| `openecraft-parts-server` | *Docker image of the parts server* (`docker-parts-server.yml`) | `check-server.sh parts` |
| `openecraft-project-server` | *Docker image of the project server* (`docker-project-server.yml`) | `check-server.sh project` |
| `openecraft-web` | *Docker image of the web editor* (`docker-web.yml`) | `check-editor-web.sh` |
| `openecraft-website` | *Docker image of the website* (`docker-website.yml`) | `check-website.sh` |

Los cuatro toman las mismas entradas y comparten sus pasos
(`.github/workflows/docker-image.yml`): una rama, una etiqueta de versión (`v0.1.2`,
también una anterior al workflow) o un commit que compilar (`ref`), etiquetas adicionales
(`tags`), si se sube (`push`) y si se compila sin la caché de compilación (`no-cache`).
La imagen se comprueba como en la CI (los scripts de `installer/docker/`) y se sube con
las etiquetas que la CI da a ese commit (`sha-<commit>`, `edge` para la punta de master,
`<version>`, y `<major>.<minor>` y `latest` solo cuando ninguna versión más reciente las
tiene), más las etiquetas indicadas. Desde la línea de comandos:

```bash
gh workflow run docker-parts-server.yml -f ref=v0.1.2
gh workflow run docker-project-server.yml -f ref=master -f no-cache=true
gh workflow run docker-web.yml -f tags=test
gh workflow run docker-website.yml -f push=false
```

## Web (WebAssembly)

La versión web funciona sin instalación en
[editor.openecraft.org](https://editor.openecraft.org); el sitio web del proyecto,
[openecraft.org](https://openecraft.org), enlaza con ella. Para alojarla por su cuenta:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk@0.22.1 --locked
cd openecraft && trunk serve            # development server on http://127.0.0.1:8080
cd openecraft && trunk build --release  # static files in openecraft/dist/ – host them on any web server
```

Ambas compilaciones web ponen `LICENSE-MIT`, `LICENSE-APACHE` y `OFL-NotoSans.txt` junto
a la página, la compilación de versión también `THIRD-PARTY-LICENSES.html`
(`installer/web/licenses.sh`, en Windows `licenses.ps1`, ejecutado por trunk después de
la compilación); conserve estos archivos al alojar la página.

En el navegador, «Guardar» descarga el archivo de proyecto y «Abrir» usa el selector de
archivos.

### Aviso legal de un editor web

*Ayuda › Aviso legal y privacidad* del editor muestra la exención de responsabilidad de
OpenECraft (sin garantía, los nombres de productos que menciona pertenecen a sus
propietarios) y su aviso de privacidad: qué almacena el editor y dónde, y con qué
servidores se comunica. Un editor web alojado para otros muestra también quién lo opera,
como exigen muchos países (en Alemania el *Impressum* del § 5 DDG), y lo que el operador
añade al aviso de privacidad (el alojamiento, los registros del servidor web, a quién
preguntar). Ambos se fijan en `legal` de `config.js` junto a la página
(`openecraft/config.js`, copiado por la compilación; edítelo después de una compilación)
con los campos del [sitio web](../website.md#legal-notice): `name` (vacío: sin aviso
legal), `address`, `email`, `phone`, `representative`, `register`, `vat`, `responsible`,
`text` y `privacy`; las líneas de la dirección y de los textos se separan con `\n` o
`|`. El editor web que el sitio web trae consigo en `/editor/` muestra el aviso del
sitio web. El editor de escritorio no tiene operador y muestra solo la exención de
responsabilidad y el aviso de privacidad.
El gestor de artículos tiene los mismos textos tras *Aviso legal y privacidad* de su
barra superior, con un aviso de privacidad propio; su compilación web toma el mismo
`config.js`.
Mantenga veraces los textos: un cambio del editor que almacene o envíe otra cosa los
cambia también (`crates/openecraft-ui/src/legal.rs`).

### Imagen Docker del editor web

`ghcr.io/nilres/openecraft-web` (las mismas etiquetas que las imágenes de los
servidores) sirve la compilación de versión del editor web con nginx en el puerto 8080,
como usuario sin privilegios, con los archivos de licencia junto a la página;
`installer/docker/compose.yaml` la inicia con los servidores. A los servidores se accede
desde el navegador, no desde el contenedor: indíquelos en el editor con la dirección que
usa el navegador (`http://127.0.0.1:8090` con el archivo compose); ambos servidores
admiten peticiones de cualquier origen.

```bash
docker run -d --name openecraft-web -p 127.0.0.1:8080:8080 ghcr.io/nilres/openecraft-web
docker build -f installer/docker/Dockerfile --target editor-web -t openecraft-web .
```

Quién opera el editor se fija con las variables `OPENECRAFT_LEGAL_NAME`,
`OPENECRAFT_LEGAL_ADDRESS`, `OPENECRAFT_LEGAL_EMAIL`, `OPENECRAFT_LEGAL_PHONE`,
`OPENECRAFT_LEGAL_REPRESENTATIVE`, `OPENECRAFT_LEGAL_REGISTER`, `OPENECRAFT_LEGAL_VAT_ID`,
`OPENECRAFT_LEGAL_RESPONSIBLE`, `OPENECRAFT_LEGAL_TEXT` y `OPENECRAFT_LEGAL_PRIVACY`
(más abajo, [Aviso legal de un editor web](#aviso-legal-de-un-editor-web)): la imagen
escribe `config.js` a partir de ellas en cada inicio.

La dirección pública del editor (`https://…/`) se fija con `OPENECRAFT_SITE_URL`: con
ella la imagen escribe la dirección canónica y la imagen de vista previa de los
enlaces en la página y nombra un mapa del sitio en `robots.txt`, para los buscadores y
las vistas previas de enlaces.

El sitio de nginx es `installer/docker/editor-web.conf` (monte el suyo en
`/etc/nginx/conf.d/default.conf`): `index.html` y los demás archivos se comprueban en
cada visita, el WebAssembly y su script, nombrados según su hash, se guardan en caché
durante un año. El WebAssembly ocupa 32 MB (una compilación de versión de la página está
optimizada en tamaño: el perfil `web` del `Cargo.toml` raíz, que tarda alrededor de un
cuarto de hora); la compilación de la imagen lo comprime una vez
(`installer/docker/compress.sh`) y nginx envía la copia tal cual: 8 MB con Brotli a los
navegadores que lo piden, lo que solo hacen por HTTPS, y 12 MB con gzip en los demás
casos. Un servidor propio debería enviar también tales copias en lugar de comprimir el
archivo para cada visitante. Ponga delante un proxy inverso TLS para otras máquinas,
como para los servidores. Junto a la página están `worker.js`, un worker que busca el
tendido 3D de un proyecto abierto para que la página siga respondiendo mientras tanto,
`routing/`, los tendidos de los ejemplos procedentes de la compilación, para que un
ejemplo se abra de inmediato, e `i18n/`, los textos de los idiomas distintos del inglés,
el alemán, el francés y el español (`<code>.tsv`): la página obtiene el archivo de un
idioma cuando se usa en lugar de llevarlos todos, y muestra el idioma en inglés si falta
el archivo. Mientras se carga el WebAssembly, la página muestra el progreso
(`loading.js`).

El workflow *Docker image of the web editor* (`.github/workflows/docker-web.yml`)
recompila esta imagen sola, sin el resto de la CI
([más arriba](#imágenes-docker-de-los-servidores)).

### Sitio web

El sitio web del proyecto con sus demostraciones en vivo,
[openecraft.org](https://openecraft.org), es una compilación web propia (`website/`),
desplegada a mano como archivos estáticos o con la imagen Docker
`ghcr.io/nilres/openecraft-website`, que trae consigo el editor web en `/editor/`:
véase [website.md](../website.md).

## Pruebas

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo clippy -p openecraft -p openecraft-parts-manager -p openecraft-website --target wasm32-unknown-unknown
```

Los tres deben pasar sin advertencias (la CI los ejecuta con `-D warnings`).
