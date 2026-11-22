# Ligne de commande, serveur MCP et plugins

Les projets et le serveur d'articles sans fenêtre, pour les scripts et les agents IA.

`openecraft-cli` modifie des projets et gère le serveur d'articles sans fenêtre. Les mêmes
commandes sont proposées aux agents IA par le [Model Context Protocol](https://modelcontextprotocol.io)
(`openecraft-cli mcp`, JSON-RPC sur stdin / stdout) : un agent peut ainsi créer des projets,
placer des symboles, tracer des fils entre des points de raccordement, affecter des articles,
lancer le contrôle, générer des rapports et exporter en PDF / SVG / DXF / DWG.

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

- En ligne de commande, chaque modification est aussitôt réécrite dans `--project`. Dans une
  session MCP, le projet reste ouvert en mémoire : `save_project` l'écrit, `undo` / `redo`
  annulent ou rétablissent des appels d'outils isolés (`help mcp`) ; la ligne de commande
  refuse `undo` / `redo`, car elle rouvre le projet pour chaque commande. Une archive donnée
  comme `--project` reste inchangée : enregistrez le projet comme fichier de projet
  (`save_project --path <file>.oec`) ou convertissez l'archive (`convert_project`).
- Un **projet fractionné** conserve le projet sous forme de nombreux petits fichiers pour la
  gestion de versions (git) : l'index `<name>.oecp` avec, à côté de lui, un dossier `pages`
  (un fichier par page), `data` (un fichier par appareil, article, câble … et par domaine
  des données communes au projet ; les fichiers d'un même type sont répartis sur 16
  dossiers, afin qu'aucun ne devienne trop grand) et `files` (les documents et images
  intégrés comme fichiers ordinaires). Il contient le même projet que le fichier
  `.oec` : `convert_project --path plant.oec --format oecp` écrit
  `plant/plant.oecp`, `--format oec` ou `--format oecz` reconvertissent, et `--project
  plant/plant.oecp` y travaille comme sur un fichier de projet – une modification ne réécrit
  que les fichiers qu'elle concerne. Chaque fichier est une ligne de JSON et ne dépend que
  du contenu du projet, de sorte qu'enregistrer un projet inchangé ne modifie rien. Un
  projet fractionné a besoin d'un dossier à lui ; les autres fichiers qui s'y trouvent (un
  README, `.gitattributes`) ne sont pas touchés. Un dossier qui n'a pas d'index mais
  contient déjà des fichiers JSON dans `pages`, `data` ou `files` est refusé : ils
  appartiennent à quelqu'un d'autre et seraient remplacés ou supprimés.
- L'enregistrement d'un grand projet fractionné coûte ce qui a changé, non ce que le projet
  contient : seules les pages et les données qui diffèrent du dernier enregistrement sont
  écrites (un projet de 500 pages et 20 000 appareils enregistre une page modifiée en bien
  moins d'une seconde). Un enregistrement est une seule étape : les fichiers modifiés sont
  écrits à côté des anciens, un journal (`.oecp-save`) les nomme, et alors seulement ils
  prennent leur place ; après un plantage, le projet est tel qu'il était avant
  l'enregistrement, ou le prochain accès le termine.
- **Git** a besoin de deux choses pour un projet fractionné. Le `.gitattributes` que
  l'enregistrement écrit dans le dossier (s'il n'y en a pas déjà un) empêche git de
  convertir les fins de ligne, ce qui modifierait chaque fichier et endommagerait les
  fichiers intégrés ; un `.gitignore` écrit avec lui laisse de côté le journal et les
  fichiers temporaires d'un enregistrement en cours. Et le pilote de fusion fusionne
  les fichiers JSON enregistrement par enregistrement, de sorte que des modifications de
  symboles différents d'une page, ou de champs différents d'un même symbole, n'entrent pas
  en conflit ; configurez-le une fois par clone :

  ```bash
  git config merge.openecraft.name "OpenECraft split project"
  git config merge.openecraft.driver "openecraft-cli merge_project_file --base %O --ours %A --theirs %B --name %P"
  ```

  Lorsque les deux côtés ont modifié la même valeur, le fichier garde la valeur de la
  branche courante, la commande nomme les endroits et git marque le fichier comme étant en
  conflit : ouvrez le projet, réglez ce qui est juste et faites `git add` sur le fichier.
  Sans le pilote, git fusionne les fichiers comme du texte et y écrit les deux versions ;
  un tel fichier est nommé à l'ouverture du projet : prenez une version
  (`git checkout --ours` ou `--theirs`) et configurez le pilote.
- **Identifiants des nouveaux objets.** Chaque copie d'un projet fractionné fait commencer
  les identifiants de ses nouveaux objets à un endroit aléatoire au-dessus des identifiants
  utilisés, afin que deux personnes qui ajoutent des objets sur leurs branches ne leur
  donnent pas les mêmes identifiants (les identifiants croissent donc par pas, jusqu'à 67
  millions par session d'édition qui ajoute des objets). C'est un garde-fou qui a ses
  limites, non une garantie :
  - La probabilité d'environ 1 sur 30 000 pour deux copies modifiées en même temps vaut tant
    que chacune ajoute moins de 1 024 objets. Une session qui importe une grande macro ou
    copie de nombreuses pages couvre beaucoup des endroits où commencent les identifiants :
    deux copies qui ajoutent chacune environ 50 000 objets se rencontrent à peu près 1 fois
    sur 650.
  - `openecraft-cli -p plant/plant.oecp check_project_data` signale les identifiants utilisés
    plus d'une fois et `--repair` les renumérote : lancez-le après une fusion. Une référence
    à un tel identifiant (une connexion à un objet) ne peut pas dire lequel des deux objets
    elle désignait et peut aboutir au mauvais : regardez donc ce que la réparation nomme.
  - Seule la session qui modifie un projet fait repartir ses identifiants ; le lire (la
    comparaison, un modèle, la base de données des projets) ne le fait pas, et un projet
    créé à partir d'un modèle continue de compter.
- **Modifications d'un autre programme pendant que le projet est ouvert** (un `git pull`, un
  second éditeur) : l'enregistrement n'écrit que les fichiers que ce côté a modifiés et
  garde les autres ; l'éditeur et une session MCP relisent ensuite le projet, de sorte
  qu'il montre les deux. Un fichier modifié des deux côtés n'est pas écrasé :
  l'enregistrement échoue et nomme les fichiers. Enregistrez sous un autre nom, ou rouvrez
  le projet et répétez les modifications. L'éditeur garde les étapes d'annulation
  (l'annulation ne reprend pas ce que l'autre programme a modifié) et mémorise les fichiers
  d'une exécution à l'autre : un pull fait pendant qu'il était fermé est donc traité de la
  même façon.
- Les paramètres s'écrivent `--name value` (aussi `--name=value`, ou tous à la fois avec
  `--json '{…}'`) ; les listes comme `--ids 3,4` et les points comme `--points "0,0 40,0"` ;
  les listes de codes, de repères ou de langues sont séparées par des virgules ou des
  espaces. Les valeurs hors de leur plage sont refusées avec l'indication de la plage
  admise, non ramenées à la limite, et une commande qui échoue laisse le projet inchangé.
  Les nombres des réponses sont arrondis à la précision avec laquelle ils sont stockés.
- Les paramètres renommés depuis des versions antérieures sont encore acceptés sous leurs
  anciens noms : `rename_device` `old` / `new` (désormais `tag` / `new_tag`), `rename` de
  `set_potential`, `set_fluid_group` et `set_pct_loop` (désormais `new_name`),
  `replace_text` `text` / `with` (désormais `find` / `replace`).
- Serveur d'articles : `--server` / `--token`, les variables d'environnement
  `OPENECRAFT_SERVER` / `OPENECRAFT_TOKEN` ou l'outil `parts_connect` (aussi avec `user` /
  `password`) ; par défaut le serveur d'articles public `https://parts-server.openecraft.org`
  ([serveur d'articles public](parts-server.md#serveur-darticles-public)). `parts_connect` avec `kind: Eplan` lit à la
  place l'EPLAN Data Portal (le jeton est le jeton d'accès personnel ; uniquement la
  recherche, `parts_get`, `parts_file` et `parts_check_updates`). Les outils du portail de
  données recherchent avec des facettes (`parts_search`), gèrent les catalogues, les
  fichiers, les favoris et les utilisateurs (`parts_catalog_save`, `parts_attach`,
  `parts_favorites`, `parts_user_save`, …), comparent les articles du projet avec le
  serveur (`parts_check_updates`) et partagent les formulaires de l'entreprise
  (`parts_company_forms` (`publish`)).
- L'aide et les messages suivent `--lang` ou la langue du système ; le serveur MCP répond en
  anglais, sauf si `--lang` est donné.

- `OPENECRAFT_MCP_ROOT` est le dossier dans lequel les fichiers des outils MCP doivent se
  trouver (`path`, `output`, `folder`, `database`, `target` et les fichiers de modèles) ;
  par défaut le dossier de travail du serveur, `*` autorise tous les dossiers. Un agent
  induit en erreur par un texte qu'il lit (un projet, un article, une page web) ne peut pas
  lire ou écraser des fichiers ailleurs, tels que d'autres projets ou des dossiers de
  plugins. Les chemins relatifs se situent dans `OPENECRAFT_MCP_ROOT`, même lorsque le
  client démarre le serveur dans un dossier à lui (Claude Desktop) ; les instructions du
  serveur nomment le dossier à l'agent.
- `OPENECRAFT_MCP_TOOLS` choisit les outils que le serveur MCP propose. Par défaut, il
  propose les outils de base pour les schémas (`core`, environ 50 : projets, pages,
  symboles, fils, câbles, articles, contrôle, rapports, exports ; environ 30 kB) et atteint
  les autres (tous ensemble se décrivent en 260 kB environ) par trois outils à lui :
  `find_tools` (les noms, ou les outils dont le nom ou la description contient les mots
  d'une requête), `describe_tool` (description et paramètres) et `call_tool` (appelle
  n'importe quel outil par son nom). Des noms séparés par des virgules choisissent d'autres
  outils, `parts_*` tous les outils commençant par `parts_`, `*` tous les outils
  (p. ex. `core,panel*,parts_*`).
- Sans serveur d'articles, `parts_search` et `parts_get` utilisent les articles du projet et
  le catalogue intégré (`parts_search --local` toujours), et `assign_part` nomme des
  articles semblables pour un numéro qu'il ne connaît pas.
- `new_project --path` ne remplace pas un fichier existant, sauf si `overwrite` est donné.
  `search_projects` cherche dans le dossier courant, sauf si une base de données ou un
  dossier est donné.

Le serveur MCP est le `openecraft-cli` des paquets : sous Windows le composant d'installation
*Ligne de commande et serveur MCP pour les agents IA* (type d'installation *Installation
complète*, ne fait pas partie de *Éditeur*), dans `%LOCALAPPDATA%\Programs\OpenECraft\`
(ou `C:\Program Files\OpenECraft\` pour tous les utilisateurs) ; sous macOS
`/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli` ; sous Linux
`/usr/bin/openecraft-cli` (deb, rpm), `~/.local/bin` (tar.gz), `<AppImage> cli`,
`flatpak run --command=openecraft-cli io.github.nilres.OpenECraft` ou `openecraft.cli`
(snap). Enregistrez-le auprès d'un agent, p. ex. Claude Code (démarré dans le dossier des
projets) :

```bash
claude mcp add openecraft -- /path/to/openecraft-cli mcp
claude mcp add --scope user -e OPENECRAFT_MCP_ROOT=/home/me/projects openecraft -- openecraft-cli mcp
```

ou dans la configuration `mcpServers` d'autres clients (Claude Desktop :
`claude_desktop_config.json` dans `%APPDATA%\Claude\`, `~/Library/Application Support/Claude/`
ou `~/.config/Claude/` ; Cursor : `~/.cursor/mcp.json` ; VS Code : `.vscode/mcp.json` avec
`servers` au lieu de `mcpServers`) :

```json
{ "mcpServers": { "openecraft": { "command": "openecraft-cli", "args": ["mcp"],
    "env": { "OPENECRAFT_MCP_ROOT": "/home/me/projects" } } } }
```

Sans installation, le serveur existe sous forme de bundle MCP de la version publiée
(`OpenECraft-MCP-<version>-windows-x86_64.mcpb`, `-linux-x86_64.mcpb`,
`-macos-universal.mcpb`) : Claude Desktop et les autres clients qui connaissent le format
l'installent d'un double-clic et demandent le dossier des projets
(`OPENECRAFT_MCP_ROOT`) et les outils (`OPENECRAFT_MCP_TOOLS`). Chaque version inscrit ces
bundles dans le MCP Registry officiel sous `io.github.nilres/openecraft`.

L'agent peut lire et écrire tout fichier auquel le processus du serveur a accès ; démarrez-le
dans un répertoire de travail (et avec un jeton du serveur d'articles) dont l'utilisation par
l'agent vous convient.
[Laissez un agent IA dessiner votre schéma](../launch/ai/README.md) contient un exemple
détaillé avec sa transcription, des exemples de prompts et la configuration d'autres clients.

## Plugins

Les plugins ajoutent des commandes, des contrôles, des rapports et des exports à la fois à
l'éditeur (onglet *Plugins* : *Commandes de plugin* liste chaque commande par plugin avec une
recherche, l'étoile en épingle une sur l'onglet ; onglet *Contrôle*), à la ligne de commande
et au serveur MCP. Les plugins intégrés sont du code Rust dans `crates/openecraft-api` ; les
plugins externes sont des programmes dans n'importe quel langage qui répondent à des
requêtes JSON sur stdin / stdout :

```bash
OPENECRAFT_PLUGINS=examples/plugins openecraft-cli -p pump.oec plugins
openecraft-cli --plugins examples/plugins -p pump.oec report --kind page_notes
```

Voir [docs/plugins.md](plugins.md) pour l'API et le protocole.
