# Control de acceso de los servidores

El servidor de artículos y el servidor de proyectos comparten un mismo modelo de acceso:
los usuarios inician sesión con una cuenta local o con un directorio (Active Directory,
OpenLDAP u otro servidor LDAP), y lo que pueden hacer procede de los **roles** que se les
dan a ellos o a sus **grupos** mediante **asignaciones**, por proyecto (servidor de
proyectos) o por fabricante (servidor de artículos).

- Reglas y su evaluación: `openecraft_parts::access` (compartido con los clientes)
- Directorios y el archivo de ajustes: `openecraft-auth` (`directory.rs`, `config.rs`)
- Derechos y roles integrados: `openecraft_core::collab::{ProjectRight, ProjectRole}` y
  `openecraft_parts::portal::{PartsRight, Role}`

## Inicios de sesión recordados del servidor de proyectos

En el panel **Servidor de proyectos** del editor, **Mantener la sesión iniciada** es un
ajuste explícito y opcional para el servidor y la cuenta seleccionados. En Windows, solo el
token de sesión de la cuenta va al Administrador de credenciales de Windows. Las
contraseñas nunca se guardan. Los ajustes de la aplicación contienen solo los metadatos de
servidor y cuenta y la aceptación, de forma independiente por cuenta, de modo que otra
ventana del editor no puede desactivar el inicio de sesión de una cuenta distinta. La
identidad completa del servidor incluye el esquema, el host, el puerto efectivo y la ruta
base; las credenciales nunca pasan a otro servidor. HTTP y HTTPS siguen siendo distintos, y
esta función no cambia ni cifra el transporte existente.

Al iniciar, `GET /api/v1/session` valida el token de cuenta recordado y devuelve
`CurrentAccount { user, access }`, con los derechos actuales y sin token ni contraseña.
Los tokens raíz, el acceso anónimo y los tokens de cuenta no válidos o caducados se rechazan con
401. Esta petición no renueva el token ni toca los bloqueos de páginas. La
sesión de cuenta caduca actualmente 30 días después del inicio de sesión; la sesión de
proyecto inactiva de 90 segundos, que es independiente, no se ve afectada. Los servidores
más antiguos sin este punto de acceso requieren un inicio de sesión manual; su 404 no
elimina una credencial recordada que quizá sea válida. Los fallos de conexión también
conservan la aceptación para un reintento posterior.

Si un token de cuenta caduca mientras hay un proyecto compartido abierto, iniciar sesión de
nuevo con el mismo servidor y la misma cuenta continúa ese proyecto mediante el flujo de
recuperación de sesión. Los cambios sin guardar y el historial de deshacer siguen
disponibles; iniciar sesión no los marca como guardados ni sustituye el proyecto por una
copia nueva.

**Cerrar sesión** elimina la credencial recordada y pide al servidor que revoque el token
de cuenta una vez respondidos los guardados y los cierres de proyecto pendientes.
**Cambiar de usuario** libera la sesión de proyecto y borra el nombre de usuario
seleccionado, pero conserva el inicio de sesión recordado de esa cuenta para volver a ella.
Un proyecto compartido abierto y su historial de deshacer permanecen como copia de
recuperación local, incluidos los cambios sin guardar. El cierre de sesión local funciona
sin conexión; el panel informa por separado si no se pudo terminar la sesión del servidor o
no se pudo cambiar el almacenamiento protegido.
Las operaciones del sistema operativo se ejecutan en orden, incluida la eliminación tras un
guardado ya en curso; el cierre normal de la aplicación espera sus resultados. Las
respuestas de una selección anterior de servidor y cuenta nunca restauran su identidad ni
sustituyen el proyecto abierto.
Una confirmación de guardado retrasada sigue perteneciendo al mismo proyecto conservado
durante la reautenticación; hace avanzar la revisión guardada sin cambiar el nuevo inicio
de sesión.
La revocación en segundo plano no impide el siguiente inicio de sesión mientras se espera a
un servidor.

La limpieza de tokens no válidos compara el token rechazado antes de eliminarlo. Nunca
escribe una nueva renuncia: otra ventana puede haber recordado ya un inicio de sesión más
reciente.
Los inicios de sesión de directorio conservan su aceptación bajo el nombre de cuenta
confirmado por el servidor, también cuando el nombre escrito estaba cualificado con el
dominio o era un nombre principal de usuario.

El navegador, macOS y Linux ofrecen actualmente un inicio de sesión solo para la sesión. Su
casilla está desactivada con una explicación; no hay alternativa de token o contraseña en
el almacenamiento local, en IndexedDB, en los ajustes ordinarios ni en los archivos de
proyecto. La compatibilidad con Windows reutiliza el almacén de credenciales del sistema
operativo existente, usado también para los PAT de EPLAN, con un espacio de nombres
separado.
Estas funciones de sesión y de plataforma no cambian los comandos de proyecto ni la API de
plugins.

La comprobación aislada en vivo es `scripts/test-project-sign-in.ps1` (PowerShell 7,
Windows). Usa un servidor de proyectos desechable en la interfaz de bucle local, cuentas
sintéticas, su propia carpeta de metadatos y claves de credenciales únicas, que la prueba
elimina después.

## Derechos y roles integrados

Servidor de proyectos (el ámbito de una asignación es un nombre de proyecto):

| Derecho | Permite |
| --- | --- |
| `view` | ver el proyecto en la lista, abrirlo en solo lectura |
| `review` | guardar registros de revisión (comentarios, marcado, aprobaciones) sin bloqueos |
| `edit_pages` | bloquear y modificar páginas |
| `edit_project` | bloquear y modificar los datos del proyecto (lista de páginas, dispositivos, artículos, ajustes) |
| `create` | crear proyectos (el ámbito es el nombre nuevo) |
| `delete` | eliminar un proyecto que nadie tiene abierto |
| `manage_access` | cambiar las asignaciones del proyecto |
| `administer` | usuarios, roles y todas las asignaciones (solo a partir de asignaciones sobre `*`) |

| Rol | Derechos |
| --- | --- |
| `viewer` | view |
| `reviewer` | view, review |
| `editor` | view, review, edit_pages, edit_project, create |
| `manager` | editor + delete, manage_access |
| `admin` | todo |

Un usuario con sesión iniciada que crea un proyecto pasa a ser su `manager` (una asignación
sobre el proyecto nuevo), de modo que los propietarios de los proyectos gestionan quién
trabaja en ellos.

Servidor de artículos (el ámbito de una asignación es un fabricante, o un catálogo como `catalog:<id>`):

| Derecho | Permite |
| --- | --- |
| `read` | explorar, buscar, descargar archivos, favoritos (todo el servidor) |
| `edit_parts` | crear y modificar artículos y sus archivos, importar |
| `delete_parts` | eliminar artículos |
| `edit_catalogs` | crear, modificar y eliminar catálogos |
| `administer` | usuarios, roles y asignaciones (todo el servidor) |

Roles integrados: `reader` (read), `manufacturer` y `editor` (read, edit_parts,
delete_parts, edit_catalogs), `admin` (todo). El rol propio de una cuenta sigue funcionando
como antes: un usuario fabricante tiene el rol de fabricante solo sobre su fabricante.

Los derechos sobre un artículo o un catálogo son los de su fabricante junto con los de su
catálogo (`catalog:siemens`): una asignación sobre un catálogo permite a usuarios y grupos
modificar el catálogo y sus artículos sea cual sea su fabricante, y una asignación de
denegación sobre cualquiera de los dos quita el derecho. Los administradores fijan las
asignaciones de un catálogo en *Administración › Catálogos › Permisos*
(`GET` / `PUT /api/v1/catalogs/{id}/grants`); la lectura sigue siendo un derecho de todo
el servidor.

## Cómo se resuelven los derechos

1. El rol integrado de la cuenta (servidor de proyectos: sobre sus proyectos; ninguno para
   los usuarios que proceden de un directorio).
2. Cada asignación de la política cuyo sujeto coincide con el usuario (`{"user": "anna"}`)
   o con uno de sus grupos (`{"group": "corp:Engineering"}` de un directorio, `{"group":
   "Wiring"}` definido en el servidor de artículos). Los nombres y los ámbitos admiten los
   comodines `*` y `?` y no distinguen mayúsculas de minúsculas; `{"user": "*"}` es todo
   usuario con sesión iniciada.
3. El rol de las peticiones anónimas (`anonymous`), que también tienen los usuarios con
   sesión iniciada.
4. **Las asignaciones de denegación ganan:** los derechos de una asignación con
   `"deny": true` se quitan en su ámbito, den lo que den las demás asignaciones.

Los derechos de todo el servidor (`administer`, y `read` en el servidor de artículos) solo
proceden de asignaciones sobre `*`. El token del servidor (`--token`) conserva el acceso
completo.

## El archivo de ajustes

`access.json` en el directorio de datos del servidor de proyectos, `<data>.access.json`
junto a la base de datos de artículos (ambos se pueden cambiar con `--access <file>`). Si
falta el archivo, significa: sin directorios, sin roles definidos ni asignaciones; en el
servidor de artículos las peticiones anónimas pueden leer (`"anonymous": "reader"`). Los
administradores cambian los roles, las asignaciones y `anonymous` mediante la API
(`GET|PUT /api/v1/access`), que reescribe el archivo, y los directorios mediante
`/api/v1/directories` (`GET`, `PUT|DELETE /{id}`, `POST /{id}/test`). Las páginas web
también lo hacen: la del servidor de artículos (*Administración › Derechos de acceso*, que
además los exporta y restaura) y la del servidor de proyectos cuando se ejecuta con `--web`
(*Administración › Derechos de acceso* y *Directorios*).

```json
{
  "directories": [
    {
      "id": "corp",
      "title": "Corporate AD",
      "kind": "active_directory",
      "url": "ldaps://dc1.corp.example",
      "base_dn": "DC=corp,DC=example",
      "bind_dn": "CN=svc-openecraft,OU=Service,DC=corp,DC=example",
      "bind_password_env": "OPENECRAFT_LDAP_PASSWORD",
      "nested_groups": true,
      "allowed_groups": ["OpenECraft *"]
    },
    {
      "id": "lab",
      "kind": "ldap",
      "url": "ldap://ldap.lab.example",
      "starttls": true,
      "user_bind": "uid={user},ou=people,dc=lab,dc=example",
      "base_dn": "ou=people,dc=lab,dc=example",
      "group_filter": "(&(objectClass=groupOfNames)(member={dn}))",
      "group_base_dn": "ou=groups,dc=lab,dc=example"
    }
  ],
  "anonymous": "",
  "roles": [
    { "name": "checker", "description": "Checks drawings", "rights": ["view", "review"] }
  ],
  "grants": [
    { "subject": { "group": "corp:Engineering" }, "role": "editor", "scope": "Plant*" },
    { "subject": { "group": "corp:Quality" }, "role": "checker", "scope": "*" },
    { "subject": { "group": "corp:Contractors" }, "role": "viewer", "scope": "Secret*", "deny": true },
    { "subject": { "group": "corp:OpenECraft Admins" }, "role": "admin", "scope": "*" }
  ]
}
```

En el servidor de artículos, `"registration": true` permite a los usuarios registrarse por
sí mismos como lectores (se activa en *Administración › Derechos de acceso*; la opción de
inicio `--registration` lo anula, véase [docs/parts-server.md](parts-server.md)); sus
cuentas esperan la aprobación de un administrador salvo con
`"registration_approval": false`.
`"dashboard": false` oculta el panel de la página web (*Administración ›
Vista general*; la opción de inicio `--dashboard` lo anula).

### Grupos

El servidor de artículos guarda los grupos de usuarios en `groups` (*Administración › Grupos*, `GET
/api/v1/groups`, `PUT` / `DELETE /api/v1/groups/{name}`):

```json
"groups": [{ "name": "Wiring", "description": "Panel shop", "members": ["anna", "bob"] }]
```

Los miembros son nombres de usuario, tanto de usuarios locales como de usuarios de un
directorio (por el nombre con el que inician sesión). Las asignaciones a
`{"group": "Wiring"}` se aplican a los miembros junto a los grupos de su directorio; los
nombres de grupo no pueden contener `:`, que separa los grupos de directorio.
Los grupos de un usuario (*Usuarios*, `groups` de `PUT /api/v1/users/{name}`) son
pertenencias: al guardarlos se añade el usuario a los grupos nombrados (los nombres nuevos
se convierten en grupos) y se le quita de los demás. Los grupos que las cuentas locales
nombraban antes se convierten en grupos del servidor al iniciar. Eliminar un grupo elimina
las asignaciones a él, eliminar un usuario sus pertenencias; los nombres de los miembros no
se pueden registrar, y un cambio que no dejaría a nadie que gestione usuarios se rechaza.

### Directorios

Los administradores configuran los directorios también en *Administración › Directorios* (`GET
/api/v1/directories`, `PUT` / `DELETE /api/v1/directories/{id}`), que no muestra ninguna
contraseña (una contraseña vacía conserva la almacenada) y prueba los ajustes: *Probar*
se conecta y se enlaza con la cuenta de servicio; con un nombre de usuario y una contraseña
inicia la sesión del usuario y muestra sus grupos (`POST /api/v1/directories/{id}/test`).

| Clave | Significado |
| --- | --- |
| `id` | nombre corto; los grupos aparecen como `id:group`, los usuarios del directorio lo muestran |
| `kind` | `active_directory` (por defecto) o `ldap` |
| `url` | `ldap://…` o `ldaps://…`; `starttls` eleva `ldap://` |
| `insecure_tls` | aceptar cualquier certificado (solo pruebas); en otro caso se usa el almacén de CA del sistema |
| `bind_dn`, `bind_password` / `bind_password_env` | cuenta de servicio para buscar usuarios |
| `user_bind` | sin cuenta de servicio: nombre de enlace del usuario, `{user}@corp.example` o `uid={user},ou=…` |
| `base_dn` | dónde se buscan los usuarios |
| `user_filter` | por defecto `(&(objectClass=user)(sAMAccountName={user}))` / `(&(objectClass=person)(uid={user}))` |
| `name_attribute` | nombre de cuenta, por defecto `sAMAccountName` / `uid` |
| `nested_groups` | Active Directory: también grupos de grupos (`LDAP_MATCHING_RULE_IN_CHAIN`) |
| `group_filter`, `group_base_dn` | buscar los grupos (`{dn}`, `{user}`) en lugar de leer `memberOf` |
| `allowed_groups` | solo los miembros de estos grupos pueden iniciar sesión (vacío: todos) |
| `timeout_secs` | tiempo de espera de la conexión y de las peticiones (por defecto 10) |

El inicio de sesión prueba primero las cuentas locales; un nombre sin cuenta local pasa a
los directorios por orden (o solo al nombrado en el `provider` de la petición). El primer
directorio que conoce al usuario comprueba la contraseña (un enlace como el usuario; las
contraseñas vacías se rechazan antes de consultar al directorio) y devuelve los grupos. El
servidor guarda una cuenta para el usuario de directorio (sin contraseña; los grupos se
actualizan en cada inicio de sesión), de modo que los administradores lo ven en la lista de
usuarios y los tokens de sesión funcionan como para los usuarios locales.
Un nombre que ya pertenece a una cuenta local no puede iniciar sesión a través de un
directorio.

Las contraseñas de las cuentas locales necesitan al menos 8 caracteres. Tras 5 inicios de
sesión fallidos de un nombre de usuario (local o de directorio) desde una dirección de
cliente, cada intento posterior de ese nombre desde allí espera, 1 s al principio y el
doble tras cada fallo adicional, hasta 5 minutos; tras 30 inicios de sesión fallidos de un
cliente con cualesquiera nombres, sus intentos esperan del mismo modo. Entretanto los
servidores responden `429 Too Many Requests`. Otros no pueden bloquear a un usuario de esta
manera; un inicio de sesión correcto olvida los fallos del nombre (no los del cliente), y
los fallos se olvidan al cabo de una hora. Un nombre sin cuenta tarda tanto en rechazarse
como una contraseña incorrecta, de modo que las respuestas no revelan qué nombres existen.
Los errores de ambos servidores llegan como `{"error": "…"}`, también para rutas
desconocidas y peticiones que no se pueden analizar.

Una sesión (el token de un inicio de sesión) termina al cerrar sesión, cuando se cambia la
contraseña y 30 días después del inicio de sesión; un usuario conserva como máximo 20. Las
sesiones almacenadas por versiones anteriores no tienen inicio y cuentan como terminadas:
sus usuarios inician sesión una vez más. Las peticiones que cambian algo se comprueban
antes de leer su cuerpo, de modo que los llamantes desconocidos no pueden hacer que un
servidor lea cuerpos grandes; el inicio de sesión (y la búsqueda del servidor de artículos)
admiten como máximo 64 kB.

Ambos servidores envían `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
`Referrer-Policy: no-referrer` y una política de seguridad de contenido con cada respuesta.
Los archivos de usuarios y de acceso (`users.json`, `access.json`, una base de datos de
artículos JSON) se escriben legibles solo para su propietario (Unix). Prefiera
`bind_password_env` a `bind_password` (los servidores avisan de una contraseña en el
archivo) y `OPENECRAFT_TOKEN` / `OPENECRAFT_PROJECT_TOKEN` o el archivo de configuración a
`--token`, que los demás usuarios de la máquina ven en la lista de procesos. Ninguno de los
dos servidores habla TLS: ponga un proxy inverso con HTTPS delante de un servidor al que se
accede por la red.

Los valores se escapan para los filtros y los DN de LDAP. Mantenga la cuenta de servicio en
solo lectura.

## Clientes

- Editor, *Herramientas › Servidor de proyectos…*: muestra los derechos sobre el proyecto
  abierto; los lectores y los revisores no pueden modificarlo, los jefes de proyecto editan
  sus asignaciones en la sección *Acceso al proyecto*.
- Gestor de artículos: el editor de usuarios muestra el directorio y los grupos de un
  usuario y edita los grupos del servidor de los que el usuario es miembro.
- Línea de comandos / MCP: `parts_access` (también fija `anonymous`), `parts_grant` (con
  `deny`, o `revoke` para quitar asignaciones), `parts_role` (definir, modificar,
  `delete`), `parts_registration` (`enabled`, `approval`), `parts_user_approve` y
  `parts_user_save` con `groups`.
