# Installation et compilation

Paquets pour Windows, Linux et macOS, la version web, et la compilation à partir des sources.

Les versions publiées se trouvent sur la [page des versions](https://github.com/nilres/OpenECraft/releases)
avec un fichier `SHA256SUMS` ; leur fabrication est décrite dans [releasing](../releasing.md).

La compilation à partir des sources demande une chaîne d'outils Rust stable récente
(`rustup update stable`) ; les versions publiées sont construites avec la version fixée
dans `.github/workflows/ci.yml`. Les scripts d'installateur ci-dessous et
`trunk build --release` ont aussi besoin de
[cargo-about](https://github.com/EmbarkStudios/cargo-about)
(`cargo install cargo-about@0.9.3 --locked --features cli`) pour
`THIRD-PARTY-LICENSES.html`, que chaque paquet contient à côté de `LICENSE-MIT` et
`LICENSE-APACHE`.

## Bureau (Windows / Linux / macOS)

```bash
cargo run --release -p openecraft
```

Sous Linux, il faut les bibliothèques habituelles de fenêtrage / de dialogue, par exemple
sous Debian/Ubuntu :

```bash
sudo apt install libxkbcommon-dev libgtk-3-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

Un fichier de projet donné sur la ligne de commande est ouvert au démarrage
(`openecraft plant.oec`).

## Installateur Windows

`OpenECraft-<version>-setup.exe` installe l'éditeur avec une entrée dans le menu Démarrer,
sans droits d'administrateur (ou pour tous les utilisateurs dans Program Files), en
anglais, allemand, français ou espagnol selon la langue de Windows. En option : le
gestionnaire d'articles, la ligne de commande / le serveur MCP (ajoutés au `PATH` sur
demande), les serveurs d'articles et de projets, un raccourci sur le bureau, et
l'ouverture des fichiers `.oec` / `.oecz` / `.oecp` par double-clic. Il se désinstalle dans
*Paramètres › Applications* ; les projets et les paramètres sont conservés. La CI le
construit à chaque push (artefact `openecraft-windows-installer`) ; en local, il faut
[Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`) :

```bash
powershell -ExecutionPolicy Bypass -File installer/windows/build.ps1   # → target/installer/
```

Les binaires lient le runtime C de façon statique, aucun redistribuable Visual C++ n'est
donc nécessaire.
Pour une installation sans surveillance (distribution de logiciels), lancez le setup avec
`/VERYSILENT /SUPPRESSMSGBOXES`, ajoutez `/ALLUSERS` pour tous les utilisateurs et
choisissez les parties avec `/COMPONENTS="main,partsmanager,cli,servers"` et
`/TASKS="fileassoc,addtopath"`.
Avec `WINDOWS_SIGN_CERT=<.pfx file>` et `WINDOWS_SIGN_PASSWORD`, le script signe les
programmes, le setup et le désinstalleur (signtool du SDK Windows) ; sinon ils ne sont
pas signés et SmartScreen demande une confirmation à la première exécution (voir
[releasing](../releasing.md)).
`installer/icon.py` dessine l'icône de l'application (`assets/icon.png`, `assets/icon.ico`,
`assets/icon.icns`, `assets/icon.svg`).

## Paquets Linux

`installer/linux/build.sh` construit les paquets pour l'architecture de la machine de
compilation dans `target/installer/`. Chacun contient l'éditeur et le gestionnaire
d'articles (avec des entrées de menu en anglais, allemand, français et espagnol), la ligne
de commande / le serveur MCP et les serveurs d'articles et de projets. Avec deb, rpm,
Flatpak et tar.gz, les fichiers `.oec` / `.oecz` / `.oecp` s'ouvrent par double-clic.

```bash
bash installer/linux/build.sh                        # tar.gz, deb, rpm, AppImage
bash installer/linux/build.sh flatpak snap           # sandboxed packages
```

| Paquet | Fichier | Installation | Outil de construction |
| --- | --- | --- | --- |
| Debian, Ubuntu | `openecraft_<version>_amd64.deb` | `sudo apt install ./openecraft_*.deb` | `dpkg-dev` |
| Fedora, RHEL, openSUSE | `openecraft-<version>-1.x86_64.rpm` | `sudo dnf install ./openecraft-*.rpm` | `rpmbuild` |
| Toute distribution | `OpenECraft-<version>-x86_64.AppImage` | `chmod +x`, puis le lancer | Debian ou Ubuntu, `patchelf`, `zsync`, `appimagetool` et son runtime (téléchargés, sommes de contrôle vérifiées) |
| Toute distribution | `OpenECraft-<version>-x86_64.flatpak` | `flatpak install --user OpenECraft-*.flatpak` | `flatpak-builder` |
| Toute distribution | `openecraft_<version>_amd64.snap` | `sudo snap install --dangerous openecraft_*.snap` | `snapcraft` |
| Toute distribution | `OpenECraft-<version>-linux-x86_64.tar.gz` | `./install.sh` (dans `~/.local`) | `tar` |

Les programmes sont `openecraft`, `openecraft-parts-manager`, `openecraft-cli`,
`openecraft-parts-server` et `openecraft-project-server`. Dans l'AppImage, ils se lancent
avec `OpenECraft-*.AppImage parts-manager|cli|parts-server|project-server …`, dans le
Flatpak avec `flatpak run --command=openecraft-cli io.github.nilres.OpenECraft …` et dans
le snap sous `openecraft.cli`, `openecraft.parts-manager`, … La CI les construit à chaque
push (artefacts `openecraft-linux-packages`, `openecraft-flatpak`, `openecraft-snap`) ;
deb, rpm, AppImage et tar.gz sont construits sous Debian 12 et fonctionnent avec la
glibc 2.36 ou plus récente.
L'AppImage embarque les bibliothèques de fenêtrage qui ne sont pas sur tous les systèmes
(xkbcommon, Xcursor, Xi : `appimage_libs` dans `installer/linux/build.sh`) ; la
construction en refuse une qui a besoin d'une autre bibliothèque ou d'une glibc plus
récente que les programmes. Elle embarque aussi les informations de mise à jour pour
AppImageUpdate et les outils similaires : la dernière version GitHub du dépôt qui l'a
construite (`GITHUB_REPOSITORY`, sinon `nilres/OpenECraft`), dont la construction écrit le
fichier `OpenECraft-<version>-x86_64.AppImage.zsync` à côté de l'AppImage
(`APPIMAGE_UPDATE_INFO` fixe d'autres informations de mise à jour, une valeur vide aucune).
Les entrées de bureau, les types MIME et les données AppStream
(`io.github.nilres.OpenECraft`) sont dans `installer/linux/` ; une nouvelle version a
besoin d'un `<release>` dans le fichier metainfo.

## Image disque macOS

`installer/macos/build.sh` construit `OpenECraft-<version>-macos.dmg` (macOS 11 ou plus
récent, binaires universels pour Apple silicon et Intel) avec les apps *OpenECraft* et
*OpenECraft Parts Manager* ; faites-les glisser dans *Applications*. La ligne de commande
et les serveurs sont dans le bundle de l'app :

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
bash installer/macos/build.sh                        # → target/installer/
/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli help
```

Sans `MACOS_SIGN_IDENTITY`, les apps sont signées ad hoc, si bien que Gatekeeper demande
une confirmation au premier démarrage (*Réglages Système › Confidentialité et sécurité ›
Ouvrir quand même*). Avec un certificat Developer ID
(`MACOS_SIGN_IDENTITY="Developer ID Application: …"`) et un profil de trousseau
`notarytool` (`MACOS_NOTARY_PROFILE`, `MACOS_NOTARY_KEYCHAIN` quand il n'est pas dans le
trousseau de session), l'image est signée et notarisée.
Les projets s'ouvrent avec *Fichier › Ouvrir le projet* ; leur ouverture par double-clic
dans le Finder n'est pas encore prise en charge. La CI construit l'image à chaque push
(artefact `openecraft-macos-dmg`).

## Mises à jour

À son démarrage, l'éditeur demande à GitHub, en arrière-plan, la dernière version
(`https://api.github.com/repos/nilres/OpenECraft/releases/latest`) ; sans connexion, il
continue sans message. Une version plus récente est téléchargée, vérifiée avec le
`SHA256SUMS` de la version et installée :

| Installation | Mise à jour |
| --- | --- |
| Setup Windows | `OpenECraft-<version>-setup.exe` s'exécute en silence (`/SILENT`) quand l'éditeur est fermé, dans le dossier et avec les composants de l'installation ; *Redémarrer et mettre à jour* relance ensuite l'éditeur |
| AppImage | le fichier AppImage est remplacé immédiatement ; la nouvelle version démarre la fois suivante (AppImageUpdate et les outils similaires le mettent aussi à jour) |
| App macOS | `OpenECraft.app` (et `OpenECraft Parts Manager.app` à côté) sont remplacées immédiatement à partir de l'image disque |
| deb, rpm, Flatpak, Snap, tar.gz, compilation propre | la barre d'état signale seulement la nouvelle version et renvoie vers sa page de version |

La barre d'état montre le téléchargement et *Redémarrer et mettre à jour* ; *Fichier ›
Aide › Rechercher des mises à jour* cherche de nouveau et signale chaque résultat.
*Paramètres › Mises à jour* désactive la recherche au démarrage, la variable
d'environnement `OPENECRAFT_NO_UPDATE_CHECK` (valeur quelconque) le fait pour tous les
utilisateurs d'un ordinateur. La version web n'a pas de mise à jour propre.

## Images Docker des serveurs

Le serveur d'articles et le serveur de projets sont aussi publiés comme images Docker à
chaque version et à chaque push sur `master`, construites par
`installer/docker/Dockerfile` (Debian 12, `linux/amd64`, exécutées sous l'utilisateur
`openecraft`, données dans le volume `/data`, licences dans `/usr/share/doc/openecraft/`) :

| Image | Port | Tags |
| --- | --- | --- |
| `ghcr.io/nilres/openecraft-parts-server` | 8090 | `<version>`, `<major>.<minor>`, `latest` (versions publiées), `edge` (master), `sha-<commit>` |
| `ghcr.io/nilres/openecraft-project-server` | 8091 | les mêmes |

```bash
docker run -d --name parts -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data ghcr.io/nilres/openecraft-parts-server
docker run -d --name projects -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret> -v projects:/data ghcr.io/nilres/openecraft-project-server
```

Les paramètres de chaque image sont dans `/etc/openecraft/parts-server.toml` et
`project-server.toml` (`bind = "0.0.0.0:<port>"`, `data` dans `/data`) ; montez-y votre
propre fichier ou ajoutez des options de ligne de commande après le nom de l'image
(`… openecraft-parts-server --install-catalogs all`). Le premier administrateur vient lui
aussi de l'environnement : `-e OPENECRAFT_ADMIN_USER=admin -e
OPENECRAFT_ADMIN_PASSWORD=<password>` (`OPENECRAFT_PROJECT_ADMIN_USER` /
`OPENECRAFT_PROJECT_ADMIN_PASSWORD` pour le serveur de projets) crée le compte, ou
réinitialise son mot de passe, à chaque démarrage.
Un montage lié (bind mount) à la place d'un volume doit être accessible en écriture à
l'utilisateur de l'image (uid 1000, ou démarrez le conteneur avec `--user`).
`installer/docker/compose.yaml` démarre les deux serveurs avec Docker Compose. Les
serveurs parlent HTTP en clair : gardez-les sur `127.0.0.1` derrière un proxy inverse TLS
quand d'autres machines y accèdent ([serveur de projets](project-server.md)). Pour
construire les images vous-même :

```bash
docker build -f installer/docker/Dockerfile --target parts-server -t openecraft-parts-server .
docker build -f installer/docker/Dockerfile --target project-server -t openecraft-project-server .
```

La CI construit les deux images à chaque push, les démarre une fois et vérifie qu'elles
répondent ; sur `master` et pour un tag de version, elle les pousse vers le GitHub
Container Registry (job `docker`), avec l'image de l'éditeur web (ci-dessous).

Chaque image a un workflow qui la reconstruit seule, sans le reste de la CI, lancé à la
main sous *Actions* (› Run workflow) :

| Image | Workflow | Vérification |
| --- | --- | --- |
| `openecraft-parts-server` | *Docker image of the parts server* (`docker-parts-server.yml`) | `check-server.sh parts` |
| `openecraft-project-server` | *Docker image of the project server* (`docker-project-server.yml`) | `check-server.sh project` |
| `openecraft-web` | *Docker image of the web editor* (`docker-web.yml`) | `check-editor-web.sh` |
| `openecraft-website` | *Docker image of the website* (`docker-website.yml`) | `check-website.sh` |

Tous les quatre prennent les mêmes entrées et partagent leurs étapes
(`.github/workflows/docker-image.yml`) : une branche, un tag de version (`v0.1.2`, même
plus ancien que le workflow) ou un commit à construire (`ref`), des tags supplémentaires
(`tags`), s'il faut pousser (`push`) et s'il faut construire sans le cache de construction
(`no-cache`). L'image est vérifiée comme dans la CI (les scripts de `installer/docker/`)
et poussée avec les tags que la CI donne à ce commit (`sha-<commit>`, `edge` pour la tête
de master, `<version>`, et `<major>.<minor>` et `latest` seulement quand aucune version
plus récente ne les porte), plus les tags donnés. Depuis la ligne de commande :

```bash
gh workflow run docker-parts-server.yml -f ref=v0.1.2
gh workflow run docker-project-server.yml -f ref=master -f no-cache=true
gh workflow run docker-web.yml -f tags=test
gh workflow run docker-website.yml -f push=false
```

## Web (WebAssembly)

La version web fonctionne sans installation sur
[editor.openecraft.org](https://editor.openecraft.org) ; le site web du projet
[openecraft.org](https://openecraft.org) y renvoie. Pour l'héberger vous-même :

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk@0.22.1 --locked
cd openecraft && trunk serve            # development server on http://127.0.0.1:8080
cd openecraft && trunk build --release  # static files in openecraft/dist/ – host them on any web server
```

Les deux constructions web placent `LICENSE-MIT`, `LICENSE-APACHE` et `OFL-NotoSans.txt` à
côté de la page, la construction de version aussi `THIRD-PARTY-LICENSES.html`
(`installer/web/licenses.sh`, sous Windows `licenses.ps1`, exécuté par trunk après la
construction) ; conservez ces fichiers quand vous hébergez la page.

Dans le navigateur, « Enregistrer » télécharge le fichier de projet et « Ouvrir » utilise
le sélecteur de fichiers.

### Mentions légales d'un éditeur web

*Aide › Mentions légales et confidentialité* de l'éditeur montre la clause de
non-responsabilité d'OpenECraft (aucune garantie, les noms de produits qu'il cite
appartiennent à leurs propriétaires) et sa déclaration de confidentialité : ce que
l'éditeur enregistre et où, et avec quels serveurs il communique. Un éditeur web hébergé
pour d'autres montre aussi qui l'exploite, comme de nombreux pays l'exigent (en Allemagne
l'*Impressum* du § 5 DDG), et ce que l'exploitant ajoute à la déclaration de
confidentialité (l'hébergement, les journaux du serveur web, à qui s'adresser). Les deux
se règlent dans `legal` de `config.js` à côté de la page (`openecraft/config.js`, copié
par la construction ; modifiez-le après une construction) avec les champs du
[site web](../website.md#legal-notice) : `name` (vide : pas de mentions légales),
`address`, `email`, `phone`, `representative`, `register`, `vat`, `responsible`, `text`
et `privacy` ; les lignes de l'adresse et des textes sont séparées par `\n` ou `|`.
L'éditeur web que le site web apporte sous `/editor/` montre les mentions du site web.
L'éditeur de bureau n'a pas d'exploitant et montre seulement la clause de
non-responsabilité et la déclaration de confidentialité.
Le gestionnaire d'articles a les mêmes textes derrière *Mentions légales et
confidentialité* de sa barre supérieure, avec une déclaration de confidentialité propre ;
sa construction web prend le même `config.js`.
Gardez les textes exacts : une modification de l'éditeur qui enregistre ou envoie autre
chose les modifie aussi (`crates/openecraft-ui/src/legal.rs`).

### Image Docker de l'éditeur web

`ghcr.io/nilres/openecraft-web` (mêmes tags que les images des serveurs) sert la
construction de version de l'éditeur web avec nginx sur le port 8080, sous un utilisateur
sans privilèges, avec les fichiers de licence à côté de la page ;
`installer/docker/compose.yaml` la démarre avec les serveurs. Les serveurs sont atteints
depuis le navigateur, pas depuis le conteneur : dans l'éditeur, indiquez-les avec
l'adresse que le navigateur utilise (`http://127.0.0.1:8090` avec le fichier compose) ;
les deux serveurs autorisent les requêtes de toute origine.

```bash
docker run -d --name openecraft-web -p 127.0.0.1:8080:8080 ghcr.io/nilres/openecraft-web
docker build -f installer/docker/Dockerfile --target editor-web -t openecraft-web .
```

Qui exploite l'éditeur se règle avec les variables `OPENECRAFT_LEGAL_NAME`,
`OPENECRAFT_LEGAL_ADDRESS`, `OPENECRAFT_LEGAL_EMAIL`, `OPENECRAFT_LEGAL_PHONE`,
`OPENECRAFT_LEGAL_REPRESENTATIVE`, `OPENECRAFT_LEGAL_REGISTER`, `OPENECRAFT_LEGAL_VAT_ID`,
`OPENECRAFT_LEGAL_RESPONSIBLE`, `OPENECRAFT_LEGAL_TEXT` et `OPENECRAFT_LEGAL_PRIVACY`
(ci-dessous, [Mentions légales d'un éditeur web](#mentions-légales-dun-éditeur-web)) :
l'image écrit `config.js` à partir d'elles à chaque démarrage.

Le site nginx est `installer/docker/editor-web.conf` (montez le vôtre sur
`/etc/nginx/conf.d/default.conf`) : `index.html` et les autres fichiers sont vérifiés à
chaque visite, le WebAssembly et son script, nommés d'après leur hachage, sont mis en
cache pour un an. Le WebAssembly fait 32 Mo (une construction de version de la page est
optimisée pour la taille : le profil `web` du `Cargo.toml` racine, qui prend environ un
quart d'heure) ; la construction de l'image le compresse une fois
(`installer/docker/compress.sh`) et nginx envoie la copie telle quelle : 8 Mo avec Brotli
aux navigateurs qui le demandent, ce qu'ils ne font qu'en HTTPS, 12 Mo avec gzip sinon.
Un serveur à vous devrait aussi envoyer de telles copies au lieu de compresser le fichier
pour chaque visiteur. Placez un proxy inverse TLS devant lui pour les autres machines,
comme pour les serveurs. À côté de la page se trouvent
`worker.js`, un worker qui cherche le routage 3D d'un projet ouvert pour que la page
reste réactive pendant ce temps, `routing/`, les routages des exemples issus de la
construction, afin qu'un exemple s'ouvre immédiatement, et `i18n/`, les textes des
langues autres que l'anglais, l'allemand, le français et l'espagnol (`<code>.tsv`) : la
page récupère le fichier d'une langue quand elle est utilisée au lieu de les embarquer
toutes, et montre la langue en anglais si le fichier manque. Pendant le chargement du
WebAssembly, la page montre la progression (`loading.js`).

Le workflow *Docker image of the web editor* (`.github/workflows/docker-web.yml`)
reconstruit cette image seule, sans le reste de la CI
([ci-dessus](#images-docker-des-serveurs)).

### Site web

Le site web du projet avec ses démonstrations en direct,
[openecraft.org](https://openecraft.org), est une construction web à part (`website/`),
déployée à la main sous forme de fichiers statiques ou avec l'image Docker
`ghcr.io/nilres/openecraft-website`, qui apporte l'éditeur web sous `/editor/` :
voir [website.md](../website.md).

## Tests

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo clippy -p openecraft -p openecraft-parts-manager -p openecraft-website --target wasm32-unknown-unknown
```

Les trois doivent passer sans avertissement (la CI les exécute avec `-D warnings`).
