# Plugins

OpenECraft s'étend par la crate `openecraft-api`. Un plugin ajoute

| Extension | Ce que c'est | Éditeur | Ligne de commande / MCP |
| --- | --- | --- | --- |
| Commande | fonction sur le projet avec des paramètres typés, renvoie du JSON | volet *Commandes de plugin* (onglet *Plugins* : chaque commande par plugin avec une recherche ; les favoris, les commandes utilisées récemment et un menu par plugin externe sur l'onglet ; un formulaire pour les paramètres) et la recherche de commandes | une commande / un outil à part entière |
| Contrôle | ajoute des messages au contrôle du projet | volet *Messages* | `check` |
| Rapport | tableau généré à partir du projet | volet *Rapports*, pages de rapport et blocs de rapports | `report --kind <name>` |
| Exportateur | fichier généré à partir du projet | *Fichier › Export* | `export --format <name>` |

Les noms sont en snake_case et uniques parmi tous les plugins ; les commandes ne doivent pas
prendre le nom d'un outil intégré. Les modifications qu'une commande apporte au projet
forment une seule étape d'annulation. `openecraft-cli plugins` liste les plugins chargés,
*Outils › Gérer les plugins* dans l'éditeur les active et les désactive (l'éditeur s'en
souvient).

Dans *Plugins › Gérer les plugins*, saisissez un dossier de plugin contenant `plugin.json`,
ou un dossier parent dont les sous-dossiers contiennent des plugins, et cliquez sur *Charger
les plugins*. L'éditeur les charge pour la session en cours et mémorise les dossiers
utilisables pour les lancements suivants. Un plugin voisin en échec n'empêche pas de
mémoriser les plugins qui se sont chargés. `OPENECRAFT_PLUGINS` reste disponible pour une
configuration par l'environnement.

## Plugins Rust

Les plugins intégrés se trouvent dans `crates/openecraft-api/src/builtin.rs` et ses
sous-modules et sont compilés dans chaque interface (aussi la version web) : les contrôles,
rapports et exports de projet de `openecraft-core`, les statistiques du projet, les contrôles
de l'implantation d'armoire, la fabrication d'armoires, le cycle de vie des articles, et un
plugin par domaine de commandes de projet (pages, feuilles, dessin, boîtes d'appareil,
appareils, connecteurs, automates, potentiels, génie des procédés, schémas unifilaires,
tableaux, ingénierie, électrotechnique, implantation d'armoire, salles, bâtiments, baies
19 pouces, cheminement, câblage de terrain, câblage, faisceaux, intégration, langues,
structure, propriétés, configurateur, blocs de rapports, gestion de projets). Un domaine
liste ses commandes dans une table `COMMANDS` de `Spec` (`builtin/<area>.rs`) ; la ligne de
commande et le serveur MCP les proposent comme outils à côté des outils de session de
`cli/`. Les contrôles de projet eux-mêmes sont des entrées de registre de
`openecraft_core::checks` (un `Check` par domaine, avec un nom en snake_case), et les types
de registre, les paramètres et les tableaux sont définis dans
`openecraft_core::{ext, params, table}`. Un plugin implémente `Plugin` :

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

Les `keywords` sont des termes de recherche supplémentaires du rapport ajouté en dernier,
non traduits : les normes qu'il suit et d'autres noms. Le volet *Rapports*, `list_reports`
(`search`) et le site web trouvent un rapport par chaque mot de la recherche dans son titre
tel qu'il est affiché, dans son nom ou dans un mot-clé, sans tenir compte de la casse, des
espaces ni de la ponctuation (`Report::matches`) : ainsi `VDE 0100 600` trouve le rapport
de vérification par `DIN VDE 0100-600`.

Les textes donnés comme `&'static str` sont des entrées anglaises du catalogue (`tr_const!`,
voir AGENTS.md) et sont affichés traduits. Les messages d'un contrôle portent soit un texte
fini (`Issue::plain`) soit, comme les contrôles intégrés, une entrée du catalogue avec ses
arguments
(`openecraft_core::issue!(Severity::Warning, target, "Cable {} defined {} times", tag, n)`),
que `Issue::message()` rend dans la langue courante ; l'outil `check` donne le modèle
comme `key` à côté du message.

## Plugins externes

Un plugin externe est un programme dans n'importe quel langage, dans un dossier à lui :

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

Les dossiers contenant des dossiers de plugins sont donnés dans la variable d'environnement
`OPENECRAFT_PLUGINS` (séparés comme `PATH`) pour l'éditeur et la ligne de commande, ou avec
`openecraft-cli --plugins <folder>`. Les plugins externes ont besoin de processus et ne sont
pas disponibles dans le navigateur.

### Protocole

Pour chaque requête, OpenECraft démarre le programme (`run`, dans le dossier du plugin),
écrit un objet JSON sur son stdin et lit un objet JSON sur son stdout, tous deux en UTF-8
(Python est démarré avec `PYTHONUTF8=1` et `PYTHONIOENCODING=utf-8` ; là où `python3` n'est
pas installé, comme sous Windows, `python` est démarré à la place). Chaque requête
a `"api": 1` et une `method`. Une réponse `{"error": "message"}` (ou un code de sortie non
nul, avec le message sur stderr) est une erreur. `project` est le projet tel qu'il est dans
un fichier `.oec`. Un programme qui n'a pas répondu après 5 minutes est arrêté et la requête
échoue. Les plugins s'exécutent avec les droits de l'utilisateur : ne chargez que des
dossiers de plugins auxquels vous faites confiance.

Les requêtes comprennent aussi `project_features`, un tableau annonçant ce que le modèle de
l'hôte en cours d'exécution prend en charge (`PROJECT_FEATURES` dans `openecraft-api`) ;
chaque entrée y est décrite avec les données qu'elle concerne, par exemple
`directional_pin_contacts`, `explicit_wire_junctions`,
`directed_junction_paths`, `fluid_port_roles`, `wire_hide_labels` et
`scoped_connections`. Les importateurs peuvent exiger des fonctionnalités avant de renvoyer
un projet modifié ; utiliser une CLI séparée et plus récente ne donne pas ces
fonctionnalités à un éditeur plus ancien.

| Requête | Réponse |
| --- | --- |
| `{"method": "describe"}` | `{"commands": [{"name", "help", "params": [{"name", "kind", "required", "help"}], "effect"}], "checks": [{"name", "title"}], "reports": [{"name", "title", "keywords": ["…"]}], "exporters": [{"name", "title", "extension", "params"}]}` (toutes les listes sont facultatives, les `params` des exportateurs et les `keywords` des rapports aussi) |
| `{"method": "command", "name", "params": {…}, "project": {…}}` | `{"result": <any JSON>, "project": {…}}` – `project` seulement si la commande l'a modifié |
| `{"method": "check", "name", "project"}` | `{"issues": [{"severity": "error" \| "warning" \| "info", "message", "page": <page index>, "item": <item id>}]}` – `page` / `item` facultatifs ; `page` seul désigne la page, une page que le projet n'a pas ne désigne rien |
| `{"method": "report", "name", "project"}` | `{"headers": ["…"], "rows": [["…"]]}` |
| `{"method": "export", "name", "params": {…}, "project"}` | `{"content": "file content"}` (texte ; `params` contient ce que l'interface a transmis : par convention `page`, le nom d'une page, et `file`, le nom du fichier) |

Types de paramètres : `str`, `int`, `num`, `bool`, `ids` (liste d'identifiants d'éléments),
`points` (liste de `[x, y]` en mm), `object` (objet JSON). `required` vaut `true` par défaut.

Une commande qui répond par une erreur laisse le projet inchangé, dans l'éditeur et le
serveur MCP aussi.

Chaque page de `project` a un type `kind` : `"Schematic"`, `"Cover"`, `"Overview"`,
`"Graphic"` ou `"Document"`, et pour une page qui montre quelque chose du projet, ce
qu'elle montre : `{"Report": "terminal_diagram"}` (un rapport par son nom dans
`list_reports`, aussi celui d'un plugin), `{"Mounting": 12}` et `{"Drilling": 12}`
(l'identifiant de l'enveloppe). Un projet qu'OpenECraft ne peut pas lire est refusé avec
l'endroit de l'erreur (`pages[2].kind.Report: invalid type: unit variant, expected newtype variant`).

Une commande qui construit une série de pages remplit la page de départ d'un nouveau
projet (sa seule page, une page de schéma sans contenu ;
`openecraft_core::ops::blank_page` en Rust) et garde son nom, au lieu d'ajouter une
page à côté. Les pages qu'elle ajoute prennent des noms que le projet n'a pas encore :
la vérification signale un nom de page utilisé deux fois.

`effect` d'une commande dit ce qu'elle fait : `read` (elle ne fait que lire : le projet
reste tel quel), `add` (elle ne fait qu'ajouter au projet) ou `change` (elle peut modifier,
remplacer ou supprimer ce qui existe ; la valeur par défaut). Le serveur MCP le transmet
comme annotations de l'outil (`readOnlyHint`, `destructiveHint`), d'après lesquelles un
client décide de demander ou non à son utilisateur avant un appel ; un plugin en Rust le
dit avec `.effect(Effect::Read)` après `reg.command(…)`. Le titre d'un outil est son nom
en mots (`set_spares` : *Set spares*).

`describe` est demandé une fois au chargement du plugin ; les contrôles s'exécutent après
chaque modification du projet dans l'éditeur, ils doivent donc répondre vite. Les paramètres
de contrôle du projet s'appliquent aussi aux messages des plugins : un contrôle peut être
désactivé par son nom, et ses messages en texte simple partagent le nom du contrôle comme
code (les changements de niveau s'appliquent alors à tous) et peuvent être acceptés un par
un.

Les articles peuvent régler `symbol` sur la clé d'une définition de symbole du projet. La
logique commune de placement des articles utilise cette définition dans l'éditeur, la CLI et
MCP. Une clé vide conserve la sélection existante fondée sur les modèles. `source_data` sur
les articles et les définitions de symboles peut conserver en JSON des identifiants
d'importateur et des propriétés du fabricant.
`Project.import_sources` associe des identifiants de source stables à des jeux de données
JSON, y compris des enregistrements qui n'ont pas de surface d'édition native. Ces champs
sont facultatifs pour les fichiers plus anciens.

Les clips de macro natifs portables peuvent porter `symbol_defs`, un tableau de définitions
de symboles du projet. L'insertion d'un clip installe les définitions manquantes et renomme
les clés entrantes en conflit, en préservant les modifications locales et en mettant à jour
les instances entrantes. La table facultative `devices` et le tableau `parts` portent les
affectations d'appareils et les enregistrements de catalogue.
L'insertion renomme les affectations d'appareils avec leurs instances et installe les
articles manquants sans écraser les enregistrements de catalogue locaux. `Part.symbol` suit
les définitions renommées.
Les aperçus de macros résolvent les définitions fournies avant de consulter le projet de
destination.
La
commande `insert_macro` accepte une macro native sous forme de chaîne JSON ainsi que `page`,
`x` et `y`. L'éditeur, la CLI et MCP exposent la même commande. Les grands objets de
paramètres de la CLI peuvent être fournis comme `--json @parameters.json` pour éviter les
limites d'arguments du shell.
Les broches de symboles utilisateur peuvent régler `internal: true` pour des noeuds de
rattachement de cavaliers. Elles participent toujours aux ponts locaux mais n'apparaissent
pas comme extrémités physiques d'appareil. Un fil qui traverse une broche interne s'y
connecte ; les autres broches ont besoin d'une extrémité de fil à cet endroit.
`list_nets` expose l'appartenance native par identifiant de symbole et indice de broche à
partir de zéro ; son tableau `pins` contient les extrémités physiques d'appareil. Le tableau
supplémentaire `connection_points` comprend les points de rattachement de jonctions et de
ponts internes qui appartiennent à un réseau câblé ou physique, sans ajouter d'extrémités
d'appareil ni de réseaux vides. Les importateurs peuvent vérifier les chemins aboutissant à
ces points par la même commande dans l'éditeur, la CLI et MCP.
`bridge_terminals` vaut true par défaut et peut être désactivé pour des tronçons de fil
particuliers.
`list_connections` renvoie les cibles physiques de fil de broche à broche comme objets
`from` et `to` contenant un identifiant de symbole et un indice de broche à partir de zéro,
avec le potentiel, le numéro de conducteur, la couleur, la section, le câble et l'âme. Les
ponts internes d'appareil conservent la continuité électrique sans créer de fils externes à
fabriquer.
Les symboles de jonction utilisateur peuvent régler `directed_junction: true` : leurs
`bridges` décrivent alors aussi des chemins cibles physiques entre des ports espacés.
`set_symbol_junction_paths` accepte une clé de symbole et
`paths: {"links": [[0, 1], [0, 2]], "directed": true}` ;
l'éditeur de symboles expose les mêmes chemins. La propriété de page
`page.use_drawn_wire_targets` préserve les dérivations explicites aux points de raccordement
des appareils et suit les fils à travers les ponts internes des appareils. Sur de telles
pages et sur les pages à jonctions explicites, les broches d'un même symbole à la même
position comptent comme un seul rattachement.
Les symboles utilisateur peuvent fournir `pin_directions`, une table associant des indices
de broche à partir de zéro à des masques de direction locaux (haut 1, droite 2, bas 4,
gauche 8). Les entrées omises autorisent tous les côtés ; zéro désactive les contacts
latéraux automatiques. Les directions suivent la rotation et la mise en miroir de
l'instance. Les extrémités de fil enregistrées se connectent toujours.
`set_pin_connection_directions` accepte `symbol`, `pin` et `directions` (`"up,right"`,
`"all"` ou `"none"`) ; l'éditeur de symboles expose les mêmes directions. Les nouveaux fils
automatiques utilisent ces directions.
La propriété de page `page.explicit_wire_junctions` rend non conducteurs les croisements
d'une extrémité avec l'intérieur d'un fil, sauf si une jonction les relie, et les ports
d'appareil ne se connectent qu'aux extrémités de fil dessinées. Les rattachements internes
de barre et les ports de jonction conservent les contacts latéraux. Les points de jonction
et les contrôles d'extrémités ouvertes utilisent les mêmes règles.
`device_box_roles` ajoute `SymbolInst.box_role` aux boîtes d'appareil : `extension` ajoute des points de raccordement à
l'appareil du repère de la boîte, `view` montre des points de raccordement qu'il a déjà et les relie d'une
page à l'autre par appareil et désignation ; la valeur par défaut `device` est omise du fichier.
Une instance d'interruption peut stocker `function.connection_scope` dans ses `values`.
Les points ne se relient et ne se renvoient qu'à d'autres de même portée et de même
libellé ; une portée vide conserve le comportement existant à l'échelle du projet. Le
libellé reste le nom affiché. Cette propriété est modifiable par la base de données des
propriétés dans l'éditeur et par les commandes de propriétés communes à la CLI et à MCP.
Les symboles utilisateur peuvent fournir `fluid_ports`, une table associant des indices de
broche à partir de zéro à
`supply`, `pressure`, `working`, `exhaust`, `return`, `pilot`, `suction`, `leakage`,
`measuring` ou `passage`. Une entrée omise conserve la déduction fondée sur la désignation.
Ces rôles survivent aux copies de bibliothèque, aux macros portables et à la sérialisation
du projet ; `list_symbols` les expose et `set_fluid_port_role` modifie ou efface une entrée
(`role: "automatic"`) par le volet Plugins de l'éditeur, la CLI et MCP. L'éditeur de
symboles propose aussi le rôle sur le point de raccordement fluidique sélectionné.
`FieldKind::TagName` affiche le nom de l'appareil sans la structure héritée tout en restant
lié au repère de l'instance. Les cadres peuvent régler `visible: false` lorsque le dessin
importé comprend sa propre bordure et son propre cartouche ; la géométrie du papier reste
disponible. `ZeroBasedNumbers` est pris en charge pour les étiquettes de colonnes des
dessins source.
Les images de dessin PNG, y compris les images à palette et la transparence, sont exportées
en PDF avec un masque de transparence d'image.
`Func::Graphic` garde le dessin décoratif hors de l'identification des appareils et de
l'évaluation de la liste des réseaux, tout en conservant des primitives de symbole
modifiables.

Les définitions de symboles du projet peuvent contenir une primitive de dessin `Image`. Elle
porte un `name` de fichier, des `data` encodées en base64 et quatre `corners` en coordonnées
locales du symbole, dans l'ordre haut gauche, haut droite, bas droite, bas gauche.
L'éditeur et la sortie SVG/PDF appliquent à ces coins la rotation et le miroir du symbole.
Les représentations existantes des primitives de dessin sont prises en charge à côté des
images.

Les symboles utilisateur peuvent indiquer un `domain` `Electrical`, `Fluid` ou `Process`.
Les fichiers de symboles plus anciens prennent `Electrical` par défaut. Les points de
raccordement fluidiques et de procédé participent aux réseaux fluidiques, et leur connexion
à une broche électrique est signalée comme un réseau mixte.

Les primitives `Field` acceptent un `angle` facultatif en degrés, selon la même convention
que le texte de symbole. L'omettre conserve le comportement historique d'étiquette
horizontale. Leur `kind` peut aussi être `{"Pin": 0}` (indice de broche à partir de zéro),
qui suit la désignation de raccordement de l'instance placée. La désignation affichée d'une
borne utilise `Label`.

La fonction `Auxiliary` représente une fonction d'appareil supplémentaire avec des broches
électriques et un renvoi vers sa fonction principale, sans inventer un type de contact.
`Junction` n'a pas de repère d'appareil et ne relie que les chemins listés dans ses ponts.
Ses points de raccordement sont des noeuds de routage locaux et sont exclus des extrémités
d'appareil dans la liste des réseaux.

Les fils peuvent régler `hide_labels: true` lorsque leurs numéros de fil et leurs étiquettes
de câble/d'âme sont déjà dessinés comme annotations séparées. Les propriétés électriques et
les rapports restent disponibles. Le panneau des propriétés du fil et la commande
`update_wire` peuvent rétablir les étiquettes automatiques ; les nouveaux fils affichent
les étiquettes par défaut.

Un exemple complet se trouve dans [`examples/plugins/page_notes`](../../examples/plugins/page_notes).

### Coordonnées des dessins CAO

Les outils CLI/MCP `import_dxf` et `import_dwg` renvoient `source_bounds` (points `min` et
`max` en millimètres du dessin décodé, Y vers le haut) et `page_origin` (la position sur la
page de la limite supérieure gauche de la source), à côté de `scale`. Pour un point de page
importé `p`, ses coordonnées source sont `source_bounds.min.x +
(p.x - page_origin.x) / scale` et `source_bounds.max.y -
(p.y - page_origin.y) / scale`. Cela comprend la conversion d'unités du dessin et permet aux
importateurs externes d'appliquer une fenêtre définie par l'auteur. Le placement existant
sur la page et les valeurs par défaut des paramètres restent inchangés.

### Macros de fenêtre EPLAN

Le plugin de dessin accepte le XML de macro de fenêtre PXF avec `import_ema` (paramètre `xml`). Son résultat liste les définitions de symboles natives, les types de fonction, les identifiants de variantes, les macros à plusieurs appareils et les références non résolues. Une réimportation préserve les définitions modifiées localement. La commande `place_part` accepte une clé de symbole `variant` facultative. L'outil de fichiers `import_eplan_symbols` de la CLI et de MCP lit un EMA, un EDZ, un XML d'article d'accompagnement ou un dossier de fabricant avec un numéro d'article sélectionné.

`product_designation_fields` annonce le type `{"Product":N}` de `Prim::Field`. Il affiche les N derniers niveaux de la désignation de produit du repère de l'instance courante, sans les aspects fonction et emplacement ; zéro se comporte comme un. Le champ se met à jour après un renommage et survit à la sérialisation de la bibliothèque de symboles et du projet. L'éditeur de symboles expose le nombre de niveaux ; les plugins utilisent la même primitive sérialisée par les définitions de symboles. Les hôtes sans cette fonctionnalité ne peuvent pas charger ces variantes de champ.

`function_text_fields` annonce le type `"FunctionText"` de `Prim::Field` et le `default_function_text` facultatif des symboles utilisateur. Le placement commun initialise le texte de fonction natif de l'instance à partir de la valeur par défaut de la bibliothèque. Les champs rendent la valeur courante de l'instance, y compris le texte multiligne transformé ; le bloc de repère automatique omet un texte de fonction en double lorsqu'un champ explicite existe. `list_symbols` renvoie la valeur par défaut de la bibliothèque.

`function_part_selection` annonce des références d'articles affectés propres à une fonction. La propriété `function.part_index` sélectionne une position à partir de un dans les articles de l'appareil et consigne son numéro d'article dans `SymbolInst.values["function.part_number"]`. La suppression d'articles antérieurs ne change pas cette référence. Un article sélectionné supprimé donne une sélection vide, la position zéro et un message de projet ; régler une position existante la répare, tandis qu'effacer la position rétablit la sélection automatique du premier article. Les propriétés `part.*` d'une fonction ne lisent que son article sélectionné ; les propriétés d'appareil conservent leur comportement agrégé. `function.manufacturer` prend en charge une valeur de remplacement explicite et modifiable, avec repli sur l'article sélectionné, et `function.selection_text` conserve le texte de sélection saisi indépendamment des références de type du catalogue. La commande enregistrée `list_function_parts` renvoie l'article sélectionné et les valeurs d'attributs modifiables de chaque fonction pour une inspection par lots.

`property_blocks` annonce `Prim::PropertyBlock` avec des lignes ordonnées `Property(key)`, `Field(FieldKind)` et `Text(value)`, des vecteurs locaux `pos` et `step`, une `size` de police, un `anchor` horizontal, un `angle` de texte, `align` (`Start`, `Middle`, `End`) et une `color` RVB facultative. Le moteur de rendu commun au canevas et aux exports résout les propriétés sur la fonction d'origine au repère complet, saute les valeurs vides et dispose chaque ligne avant d'appliquer les transformations du symbole. Les `default_values` des symboles utilisateur initialisent les propriétés d'instance modifiables lors du placement commun ; `list_symbols` expose les valeurs par défaut et les blocs. L'éditeur de symboles prend en charge la création et la modification des blocs et des propriétés par défaut.

La fonctionnalité de projet `wire_type` préserve la constitution du conducteur dans
`Wire.wire_type`. `update_wire` accepte `wire_type` ; `list_connections` le renvoie pour
chaque connexion physique. Le tableau modifiable des connexions ajoute Type de conducteur
après ses neuf colonnes d'origine ; les rapports de connexions comprennent la même valeur.
Vide signifie non précisé.

La fonctionnalité `physical_wire_routes` annonce un suivi des conducteurs qui tient compte
des contacts. `list_connections` comprend `route_traced` et `wires` (identifiant de page
natif et identifiant de fil). Lorsque `route_traced` vaut false, la liste des fils est la
portée de propriétés existante à l'échelle du réseau, non un chemin démontré de point à
point. Les pages à jonctions explicites gardent séparés les sommets propres à chaque fil
jusqu'à ce qu'une extrémité valide, une jonction ou un contact directionnel de broche les
réunisse. Ces règles s'appliquent aussi lorsqu'un croisement appartient à un même réseau par
un autre chemin.

`trace_wire_routes` accepte `routes: {"pairs": [{"from": {"symbol": 12, "pin": 0},
"to": {"symbol": 34, "pin": 0}}]}` et renvoie un résultat ordonné pour chaque paire.
Il suit les extrémités dessinées locales exactes, y compris les points d'interruption
virtuels, sans sauter entre des noms de potentiel ou d'interruption égaux. Chaque résultat
a `route_traced` et `wires` avec les identifiants natifs de page/de segment et leurs
propriétés de conducteur. Cela vérifie une dérivation particulière même lorsque la liste de
câblage physique choisit un autre chemin valide à travers un réseau nommé. Une extrémité
invalide ou non connectée n'a pas de chemin suivi.

### Voies complètes de modules d'automate

La fonctionnalité de projet `multi_channel_plc` annonce `UserSymbol.plc_channels`, exposé aussi par `list_symbols`. Chaque définition de voie contient une `key` stable, un `index` directionnel à partir de zéro, une direction `input` facultative, `analog`, `data_type`, `designation`, des `pins` physiques à partir de zéro et une `signal_pin` facultative. Les métadonnées `plc_channel` existantes à une seule voie restent prises en charge.

Les valeurs d'instance des voies utilisent les clés de propriété à partir de un `function.plc_address[1]`, `function.plc_symbol[1]`, `function.plc_comment[1]` et `function.plc_data_type[1]`. Les définitions de bibliothèque devraient laisser vides les adresses configurées et les noms de signaux. Les blocs de propriétés peuvent référencer ces clés pour afficher la configuration courante. `get_plc_points` renvoie une ligne par voie avec ses indices de broches physiques ; `set_plc_channel` modifie une voie à l'aide d'un identifiant de symbole et d'un indice de voie à partir de zéro. Les clés de ligne des tableaux d'automate utilisent `symbol_id:channel_index` pour les modules complets et conservent les identifiants de symbole numériques pour les anciens points uniques. Les voies vides et sans direction sont exclues des exports de variables d'automate.

### Représentations liées de voies d'automate

La fonctionnalité `linked_plc_channels` ajoute `SymbolInst.plc_bindings`, une table associant des indices de voie à partir de zéro à `{group, primary}`. Un groupe non vide identifie la même voie logique entre les représentations de même repère d'appareil complet. Réglez `primary` sur la représentation électrique dont les broches physiques et l'emplacement doivent servir à la navigation des E/S. Des repères d'appareil complets différents gardent les appareils copiés indépendants.

La vue d'ensemble de l'automate, les décomptes de matériel, les tableaux et les exports de variables utilisent les voies logiques. Les modifications de voie, les modifications de propriétés de voie indexées, le réadressage de cartes et l'import de tables de symboles mettent à jour toutes les vues liées ; une vue diffusée rejette toute la modification de la voie. Les valeurs saisies existantes restent stockées sur chaque vue. Les champs affichés vides peuvent se replier sur une autre vue liée. `get_plc_points` renvoie par défaut les lignes d'E/S logiques ; `representations:true` renvoie toutes les représentations du schéma avec leurs propres valeurs stockées, leurs indices de broches physiques et les métadonnées de liaison, pour une vérification d'import sans perte.

La fonctionnalité `plc_channel_indices` ajoute `ChannelBinding.index` facultatif, le numéro de voie matériel relatif à la carte. Stockez le numéro matériel de la vue d'ensemble du module sur chaque vue liée, même lorsqu'une vue électrique ne contient qu'une seule voie locale. `get_plc_points.hardware_index` expose ce numéro séparément de l'indice `channel` local au symbole qu'utilise `set_plc_channel`. Le réadressage et la désignation de voie logique utilisent le numéro matériel. Les liaisons plus anciennes sans numéro explicite se replient sur la définition de module liée la plus complète, ce qui garde utilisables les projets poursuivis après un réordonnancement des feuilles.

Les articles représentant des modules d'automate complets utilisent un `FunctionTemplate` par voie, avec `PlcInput`, `PlcOutput`, `PlcAnalogInput` ou `PlcAnalogOutput` et les désignations de broches physiques de cette voie. Laissez vides les descriptions de signaux réutilisables. L'utilisation native des fonctions et les calculs de réserve comptent une seule fois les voies de schéma liées et excluent les graphiques d'armoire. `list_device_descriptions` comprend les capacités de fonctions analogiques ; `list_unplaced` reconnaît les voies de module occupées par leur identité matérielle, indépendamment du texte d'adresse configuré.

`set_plc_card` préserve les adresses de voie saisies pour les modifications qui ne touchent que les métadonnées. Fournir `start_byte`, `output_byte`, `readdress:true` ou un `slot` Rockwell reconstruit les adresses à partir de la configuration. Une description de tête importée avec des ports vides et sans bus configuré garde une liste de ports vide ; elle n'acquiert pas d'interface PROFINET déduite.

La conversion du schéma de station comprend les voies numériques des modules complets et leurs représentations liées. Elle conserve chaque décalage d'adresse configuré et chaque nom de signal. Un texte d'adresse personnalisé qui n'est pas une adresse numérique prise en charge reste inchangé.

La fonctionnalité `explicit_panel_mounts` prend en charge `EnclosureKind::Assembly`, un canevas de montage ouvert sans géométrie implicite de coque ou de plaque, et `ItemKind::MountingPlate`, un élément explicite avec une position, une taille et une épaisseur définies par l'auteur. Les éléments de plaque explicites suppriment aussi la plaque par défaut d'une armoire. Les coordonnées restent dans le repère commun de l'enveloppe. `PanelItem.mounted_on` référence un autre élément de la même enveloppe ; `mounting_explicit:true` distingue un élément explicitement libre de l'ancien placement sur rail déduit. `rail_mount_override:true` conserve la référence de l'instance à travers le chargement natif et l'actualisation des articles. `rail_vertical:true` mesure la référence transversale au rail le long de x au lieu de y.

Utilisez `add_enclosure` / `set_enclosure` avec `assembly:true`, `place_mounting_plate`, et `set_panel_item` avec `mounted_on`, les dimensions, `rail_offset`, `clip_on`, `rail_vertical` et `rail_mount_override`. Les déplacements et les changements de profondeur du parent emportent une seule fois les descendants, y compris les supports imbriqués. Les éléments explicitement détachés gardent leur position. Les listes d'éléments exposent les champs de montage. Les copies portables et en motif réaffectent les références de montage internes ; la suppression d'un support détache les enfants qui subsistent. Les rails horizontaux comme verticaux prennent en charge l'accrochage et la disposition. Les données de projet d'origine sans ces champs facultatifs conservent l'ancien comportement.

La fonctionnalité `individual_panel_terminals` ajoute `PanelItem.terminal` : une désignation non vide lie une empreinte et son propre article à une borne d'un bornier. Vide conserve l'implantation du bornier entier. Les points physiques, les maillages, le câblage virtuel et les extrémités de câble 3D résolvent chaque borne séparément ; des placements répétés ambigus ne sélectionnent pas un élément arbitraire. Les articles sélectionnés propres à une fonction pilotent aussi l'implantation des borniers. `place_panel_device` crée un appareil ou une borne aux dimensions explicites ; `set_panel_item` peut modifier sa désignation de borne. `get_panel_points` expose les points physiques générés ou exacts par l'API commune.

La fonctionnalité `explicit_panel_geometry` conserve `PanelItem.single_part` : l'empreinte ne représente que son propre article de catalogue, même sans désignation de borne. Des dimensions de catalogue manquantes peuvent générer des points de raccordement non exacts à l'aide de l'empreinte de l'instance, sans réécrire le catalogue. Plusieurs vues physiques ne se résolvent par l'article sélectionné que s'il est unique. `Enclosure.no_implicit_plate` supprime la plaque par défaut, tandis que `mounting_origin: [x,y,z]` donne une référence explicite de face de montage en coordonnées de l'enveloppe. Les ensembles ouverts prennent par défaut la référence zéro et toute l'étendue du canevas ; leurs espaces de cheminement n'ont pas de parois d'armoire. Les commandes d'ajout/de réglage d'enveloppe exposent `implicit_plate`, `mounting_origin` (objet avec x/y/z ; vide réinitialise) `mounting_size` (objet avec width/height pour une plaque qui n'est pas centrée ; vide réinitialise) et `door_hinge` (`left` ou `right`, le bord où la porte est articulée, vue de face), et les commandes de placement/de modification d'appareil exposent `single_part`. Les plaques explicites apportent leur propre article à la nomenclature.


La fonctionnalité `cached_impossible_routes` ajoute `layout.routing.reuse_failed_searches` (false par défaut), exposé aussi par la commande `routing_rules`. Un projet peut explicitement conserver les résultats de recherches anticipées impossibles après des changements de remplissage des goulottes, au lieu de répéter des recherches exhaustives. Les projets ordinaires gardent le comportement de nouvelle tentative existant. Les chemins réussis mis en cache exigent toujours les contrôles de capacité et de distance.

La fonctionnalité `literal_part_quantities` ajoute `PartRef.total`. Une valeur true fait de
`count` la quantité littérale pour tout l'appareil/le bornier (y compris un zéro explicite),
au lieu de multiplier les articles de borne par les positions du bornier. La valeur false
par défaut préserve l'ancien comptage, y compris l'ancienne valeur par défaut qui prend zéro
pour un. Les références automatiques primaires et les références littérales supplémentaires
peuvent utiliser le même numéro d'article ; leurs quantités s'additionnent, de sorte que les
bornes nouvellement dessinées augmentent toujours le matériel primaire sans multiplier les
réserves. Les appareils littéraux de matériel seul apparaissent aussi dans `list_devices`
et dans la nomenclature.
Les commandes communes `add_device_part` et `set_device_part_quantity` exposent à la fois
le nombre et le mode ; la seconde utilise un indice d'affectation à partir de 1. L'éditeur
des articles d'appareil expose aussi l'indicateur de quantité totale. Les références
littérales à zéro n'apportent ni capacité de fonction, ni pièces, ni poids, ni puissance
dissipée, ni quantité de matériel.

`Enclosure.device_tag` lie un corps physique à son affectation d'appareil primaire, comme
le `tag` sur un élément structurel d'armoire. Leur article automatique primaire est compté
par l'implantation physique, les articles littéraux supplémentaires étant comptés à part.
Changer l'article physique par `set_enclosure`, `set_panel_item` ou un remplacement de
profilé met à jour son affectation primaire et les liaisons d'articles de fonction tout en
conservant les réserves littérales. Les commandes d'ajout/de réglage d'enveloppe exposent
`device_tag` ; les modifications d'éléments exposent `part`.

La fonctionnalité `material_ownership` ajoute `owner_device` et `bom_excluded` aux éléments
d'armoire et aux enveloppes, exposés par les commandes d'interrogation et de modification
de l'armoire. La propriété consigne l'appareil qui rend compte d'un article et ne change
jamais `mounted_on`. Une exclusion de placement ne supprime que son entrée de matériel
physique ; les propriétés d'appareil modifiables
`device.material_owner` et `device.exclude_from_bom` conservent les appareils de matériel
seul et peuvent supprimer tout le matériel affecté à un appareil. Les références de
propriétaire suivent le renommage des appareils. Les macros d'armoire réutilisables effacent
les repères de propriétaires externes et conservent l'exclusion de placement explicite. Les
interrogations de l'armoire exposent `mounting_height` au-dessus du support explicite
(null si inconnu) ; les entretoises générées des rails sur un support explicite utilisent
cette hauteur et exigent
`layout.hardware`, au lieu de traiter l'épaisseur du support comme une entretoise.

Les liaisons d'appareils d'armoire participent aussi à `list_devices` et à la nomenclature
lorsqu'aucune fonction de schéma n'est dessinée, indépendamment de tout article
supplémentaire. Une borne placée individuellement et pas encore représentée dans le schéma
apporte son article primaire une fois par désignation de borne. Des vues physiques répétées
et le fait de dessiner ensuite la même borne n'ajoutent pas une autre pièce. Le placement
existant du bornier entier garde son développement des bornes du schéma.

La fonctionnalité `portable_macro_materials` garantit que l'insertion de macros portables
préserve les valeurs de propriétés d'appareil et les références automatiques et littérales
distinctes au même article, y compris zéro. Lors de l'extension d'un
appareil existant, les modes de référence et les propriétés manquants sont complétés tandis
que les quantités et les valeurs de propriétés locales restent inchangées. Les articles que
l'appareil n'utilise pas ne sont ajoutés que s'il n'en a encore aucun ou si une fonction du clip les sélectionne, sauf demande explicite décrite ci-dessous. Les propriétaires de matériel à l'intérieur d'un clip
suivent la structure de la page cible et le renommage en cas de collision ; les
propriétaires externes sont effacés. Les repères d'appareil entrants sont réservés avant
d'attribuer des remplacements, y compris les repères existants de matériel seul. Les vues
auxiliaires conductrices suivent leur repère maître.
Les espaces réservés des propriétés d'appareil se résolvent avec les repères de macro et
les numéros d'article.

La fonctionnalité `explicit_macro_part_additions` ajoute `Clip.device_part_additions` facultatif, une table associant des repères d'appareil à des tableaux de numéros d'article. Elle demande explicitement d'ajouter les références **littérales** d'accessoires manquantes déjà présentes dans les `devices[tag].parts` correspondants du clip. Elle ne peut pas ajouter un article non affecté ou une référence automatique, remplacer un article existant ni écraser une quantité locale. Sans cette table, le collage et l'insertion de macros gardent les règles ci-dessus pour les appareils existants. Les repères et les numéros prennent en charge les paramètres de macro, la structure cible et le renommage en cas de collision. C'est une intention d'insertion, non une instruction de modifier les valeurs par défaut ordinaires de copier/coller du projet.

La fonctionnalité `per_wire_junctions` ajoute `Wire.explicit_junctions` facultatif : omis,
il hérite de `page.explicit_wire_junctions`, true exige des contacts de jonction dessinés,
et false permet des contacts en T implicites. Les liaisons implicites d'un fil avec
l'intérieur d'un autre exigent que les deux conducteurs les permettent ; les jonctions
explicites et les extrémités enregistrées se connectent toujours. Les rattachements du côté
des broches utilisent la règle du conducteur concerné. Un fil explicite
active aussi les cibles physiques dessinées sur son réseau électrique.
Les commandes communes add_wire, connect, route et update_wire acceptent
`junction_policy: inherit|explicit|implicit`. update_wire renvoie cette règle ;
les interrogations de page et les clips portables conservent le champ natif facultatif.
L'éditeur propose les mêmes choix dans les propriétés du fil. L'insertion d'une macro
explicite ne change jamais le réglage de la page ni les autres fils. Le cache d'analyse suit
les modifications de règle, et le nettoyage des fils comme la fusion des fils colinéaires
préservent les règles et les propriétés différentes.

`apply_cable_part` (`cable`, `part`) applique un article du projet ou du catalogue intégré à
un câble existant. Il utilise la même opération que l'éditeur, y compris le type, les noms
d'âmes fournis et la section, et `Outer diameter` des caractéristiques techniques en
millimètres. Des diamètres manquants ou invalides remettent la valeur forcée à zéro, de
sorte que le cheminement utilise son estimation. Il renvoie les champs de câble configurés
et `diameter_mm` (zéro signifie estimé), et rejette les noms de câble ou d'article inconnus
sans changer l'état.

`list_cable_parts` ne prend aucun paramètre et renvoie les articles de câble disponibles
parmi les articles du projet et le catalogue intégré, triés par numéro et sans doublons.
Les définitions du projet ont la priorité, y compris une modification locale du groupe de
produits. Les lignes comprennent le numéro, le type, la description, le fabricant, les âmes
fournies et la section, et un `diameter_mm` déclaré positif ou null. L'éditeur applique la
même priorité avec sa base de données d'articles configurée.

La fonctionnalité `portable_cables` transporte les inventaires de câbles et les articles
locaux qu'ils référencent dans `Clip.cables`. Le copier/coller et l'insertion de macros
attribuent de nouveaux identifiants de câble et des repères sans collision dans la
structure de destination, en réaffectant de façon cohérente les symboles marqueurs, les
fils, les définitions de connexion et les lignes de définition de câble. Les affectations
d'âmes ne sont conservées que pour les câbles appartenant au clip. Les anciens clips sans
définitions de câble continuent d'effacer les références de fil/d'âme. Les champs de texte
de câble prennent aussi en charge les espaces réservés de macro natifs. Ce comportement est
commun à toutes les interfaces.

Une référence d'article de fonction explicite peut résoudre l'article de câble
correspondant lorsque le même repère a aussi un enregistrement ordinaire de matériel
d'appareil. L'article sélectionné doit correspondre à une affectation réelle ; les
références inconnues restent invalides. Le matériel de câble reste dans l'inventaire des
câbles au lieu d'être dupliqué dans la nomenclature ordinaire des appareils.

Les résultats de connexions physiques peuvent contenir plus d'une fois la même paire
d'extrémités lorsque des âmes de câble séparées ont des chemins dessinés simples et
distincts. Chaque connexion conserve ses propres identifiants de fil et ses propriétés de
câble/d'âme pour les rapports et le cheminement. Un tronçon en cul-de-sac n'est pas traité
comme une connexion complète supplémentaire. Les commandes de connexion existantes et le
registre des rapports exposent ce comportement dans chaque interface.

La commande commune `apply_cable_part` met à jour les références explicites de fonctions
placées qui n'appartenaient qu'à l'article de câble précédent. Les références à un article
d'appareil ordinaire sous le même repère, et les références explicites invalides ou sans
rapport, sont préservées. L'éditeur de câbles utilise la même opération de référence d'âme
pour la sélection dans le catalogue et les modifications directes du numéro d'article. La
longueur et les affectations de conducteurs sont conservées lors de l'application des
données de câble du catalogue.

La fonctionnalité d'hôte `macro_definition_parameters` étend les espaces réservés de macro
natifs aux lettres des définitions de symboles portables, aux libellés/textes de
fonction/propriétés par défaut, aux libellés de broches, aux primitives de texte et aux
lignes littérales des blocs d'attributs. Les clés de symbole, les clés de propriété, les
images encodées et les métadonnées source brutes conservent leur identité. La résolution
opère sur une copie du clip avant l'installation et est commune au bureau, à la CLI et à
MCP. Les importateurs qui convertissent des paramètres de modèles source devraient exiger
cette fonctionnalité, afin que des hôtes plus anciens ne laissent pas des graphiques
statiques incohérents avec les instances résolues.

`insert_macro` accepte des `values` facultatives, un objet associant des noms d'espaces
réservés à des chaînes. Il résout ensemble les champs d'instance et de définition portable
avant le collage ; les espaces réservés inconnus restent littéraux. Des valeurs invalides
qui ne sont pas des chaînes échouent avant toute modification. Omettre le paramètre
préserve le texte de modèle non résolu existant. Cette option est disponible par le
registre commun dans chaque interface.

La fonctionnalité d'hôte `parallel_cable_cores` confirme que les consommateurs de
connexions physiques conservent des chemins d'âmes parallèles distincts tels qu'ils ont été
dessinés. Les importateurs doivent vérifier l'hôte en cours d'exécution ainsi que toute CLI
de vérification séparée avant d'accepter des circuits contenant des âmes de câble
affectées ; un vérificateur plus récent ne peut pas mettre à jour le moteur de rapports
d'un éditeur plus ancien.

La fonctionnalité `page_property_blocks` résout les propriétés de projet/de page dans les
blocs d'attributs de symbole à l'aide de leur contexte de dessin placé. Les autres champs
conservent la sélection de fonction, y compris l'article sélectionné. `page.local_name`, en
lecture seule, renvoie le numéro de page sans son préfixe de section ;
`page.section_pages` compte les pages de la même section (les pages de premier niveau
forment la section zéro). Les deux se mettent à jour après des modifications.

La fonctionnalité `page_template_values` conserve la table `values` d'un modèle de page
lors de son enregistrement, de son échange et de son insertion. Les anciennes tables vides
restent compatibles. La commande commune list_page_templates renvoie ces valeurs par
défaut, et chaque interface les applique par la même opération d'insertion. Les
importateurs devraient vérifier les deux fonctionnalités avant de fournir des cadres source
réutilisables avec des champs de contexte modifiables.

La fonctionnalité `page_frame_fields` ajoute les propriétés calculées
`page.section_last_number`, `page.previous_local_name` et `page.next_local_name`. Le
dernier numéro utilise l'ordre naturel ; les pages adjacentes suivent l'ordre modifiable de
leur section, avec des valeurs vides à chaque extrémité. Pour les feuilles 11 à 23, le
nombre de pages est 13 alors que le dernier numéro est 23. Les insertions, les suppressions
et la renumérotation mettent à jour ces champs. `page.function_designation` et
`page.location_designation` comprennent les signes d'aspect `=` et `+`, y compris les
signes seuls pour un aspect non configuré.
Ces propriétés en lecture seule sont disponibles par le registre de propriétés commun, les
blocs d'attributs de symbole, les espaces réservés de texte libre et les formulaires de
cartouche.


La fonctionnalité d'hôte `function_cable_references` fournit la propriété de fonction
modifiable `function.cable`, qui contient le repère de câble complet pour un marqueur placé.
Renommer le câble par le tableau des câbles ou l'éditeur met à jour ces références, les
repères de marqueurs correspondants, les affectations de fils, les points de raccordement
et les lignes de définition, y compris les options de modèle inactives. Les symboles
d'appareil ordinaires qui partagent un repère restent séparés. Supprimer l'entrée
d'inventaire efface son association explicite de marqueur et ses affectations d'âmes,
retire son chemin et ses définitions d'accessoires propres, et conserve les emplacements
d'extrémité des appareils ordinaires. Le contenu portable de macro/de presse-papiers transporte le câble référencé
et fait correspondre l'association au nouveau repère du câble collé. Utilisez
`set_properties` sur `function:<symbol id>` pour modifier l'association ; les opérations de
registre existantes s'appliquent de la même façon au bureau, à la CLI et à MCP.


Les cellules de repère de câble (`table_set`, tableau `cables`, colonne `Cable`) et le
gestionnaire de câbles utilisent la même opération atomique `ops::rename_project_cable`.
Elle supprime les espaces en trop du repère saisi, valide sa désignation, refuse les
doublons et les modifications de dessins diffusés, et ne met à jour les références qu'après
validation. Le gestionnaire de câbles garde le texte provisoire hors du projet jusqu'à
Enter ou la perte du focus ; Escape annule la saisie provisoire.

Une fonction avec une référence `function.cable` explicite résout son article primaire
sélectionné et affecté par cette entrée de l'inventaire des câbles. Un article de câble
modifié, effacé ou manquant se reflète immédiatement, y compris pour les modifications
directes des propriétés du câble et du tableau. Les sélections d'articles de fonction mises
en cache ne masquent pas cette association. La sélection ordinaire d'articles de fonction
garde ses règles existantes de référence explicite.


La fonctionnalité `cable_property_blocks` résout les lignes de la classe Câble dans les
blocs d'attributs de symbole par rapport à la référence `function.cable` explicite du
symbole. Une cible explicite manquante reste non résolue ; les symboles sans cette
référence conservent l'ancien contexte du câble de même repère. Ce contexte s'applique
aussi aux propriétés de câble placées et est commun au canevas et aux exports. L'éditeur de
symboles propose les propriétés de câble.
Les propriétés placées de la classe Article résolvent l'article de catalogue sélectionné de
la fonction, y compris l'article de câble lié courant et les affectations d'appareil non
primaires.

Les hôtes qui annoncent `placed_wire_labels` conservent `Wire.labels` même lorsque
`hide_labels` supprime les étiquettes automatiques. Chaque disposition contient `kind`
(`Core`, `Cable`, `Number` ou `Color`), la fraction de rattachement `along` (0..1 de a à b),
`offset` en mm du dessin, `size`, `anchor`, `angle` et une `color` RVB facultative.
Le texte vient du fil propriétaire courant. `update_wire` accepte
`labels: {"items": [...]}` pour remplacer les dispositions, ou un tableau items vide pour
les retirer. Des dispositions invalides rejettent toute la commande. Les étiquettes suivent
leur fil pendant l'édition et sont redistribuées une fois lorsque le routage crée de
nouveaux segments.

Les marqueurs d'âme non rattachés peuvent conserver un texte modifiable indépendant dans
les propriétés indexées `function.cable_core[1]` à `[64]`. Les importateurs devraient
utiliser les étiquettes de fil placées pour les marqueurs qui référencent un fil physique
réel, afin que les modifications de l'âme du fil changent automatiquement le texte affiché.

`rename_symbol_device` accepte un `id` de symbole placé et `new_tag`. Une référence
`function.cable` explicite correspondant à son repère affiché sélectionne l'inventaire des
câbles ; les appareils ordinaires de même repère restent indépendants. Les cibles de câble
manquantes, les repères de câble en double et les références de dessins diffusés rejettent
l'opération. Le renommage de la fonction sélectionnée dans l'éditeur utilise la même
opération. `rename_device` suit l'inventaire des câbles lorsqu'aucune identité d'appareil
ordinaire ne partage ce repère. Les références de propriétaire de matériel ne suivent les
modifications de câble que tant que l'ancien repère n'a pas d'identité d'appareil ordinaire.
Renommer un appareil ou un câble peut conserver un préfixe importé inchangé contenant des
espaces ; le nouveau nom final de l'appareil suit toujours les règles normales de
validation des repères.

### Orientation explicite dans l'armoire

Les hôtes qui annoncent `explicit_panel_rotations` préservent les valeurs numériques
facultatives 0, 90, 180 et 270 de `PanelItem.rotation` (dans le sens horaire dans le plan de
la face). Les multiples négatifs de 90 sont acceptés et normalisés. Les valeurs manquantes
préservent l'ancienne déduction à partir des dimensions ; un zéro explicite supprime cette
déduction. Les `x`, `y`, `w`, `h` enregistrés restent l'empreinte déjà orientée. Les
importateurs ne doivent pas faire pivoter ces dimensions une seconde fois. Les graphiques,
les maillages, les positions et les directions de sortie des points de raccordement, les
accessoires de montage et les plans de perçage exacts suivent l'angle.

`set_panel_item(rotation: ...)` change l'orientation sans déplacer ni redimensionner
l'empreinte. `infer_rotation: true` efface l'angle explicite ; fournir les deux est une
erreur. Les interrogations d'éléments renvoient `rotation` et `effective_rotation`.
`arrange_panel_items(action: "rotate")` fait tourner les éléments sélectionnés de 90 degrés
autour de leur centre et échange largeur et hauteur, comme l'action Pivoter de l'éditeur.

Les paramètres booléens facultatifs proposent dans l'éditeur Utiliser la valeur par défaut
du plugin, Activé et Désactivé. Le premier omet l'argument, de sorte que les plugins
externes conservent leurs valeurs par défaut documentées ; un false explicite reste
distinct de l'omission dans chaque interface.

### Révisions des définitions de symboles

`merge_symbol_definitions` accepte `definitions: {"items": [<UserSymbol>, ...]}`.
Le lot complet est désérialisé et validé avant toute modification. Il renvoie
`keys`, une clé résolue par entrée dans l'ordre de la source, et `added`, le nombre de
nouvelles définitions. Les définitions typées identiques réutilisent leur clé existante. Les
conflits essaient `<original>_import_1`, `_import_2`, et ainsi de suite, en réutilisant une
révision existante égale. Les définitions existantes et les instances des pages restent
inchangées. Deux entrées partageant une clé d'origine peuvent renvoyer des clés de révision
différentes ; les importateurs devraient donc faire correspondre les références à
l'intérieur de chaque clip entrant plutôt que de construire une seule table globale des
clés d'origine.

L'éditeur, la CLI, MCP et les plugins externes utilisent tous la même commande enregistrée.
Le typage natif normalise les champs par défaut et la précision numérique avant la
comparaison ; les importateurs devraient utiliser cette opération au lieu de comparer des
objets JSON bruts enregistrés. C'est la même opération de base que celle utilisée pour
coller des définitions de macros portables.

### Cadres dessinés par l'auteur

Les hôtes qui annoncent `plot_frame_artwork` acceptent `forms.frames[].artwork`, un tableau
de primitives de dessin de symbole en millimètres du papier. Un dessin non vide remplace la
bordure et le cartouche générés ; `visible` commande le dessin complet du cadre.
Les blocs de propriétés résolvent les propriétés actuelles du projet et de la page pour
chaque page qui l'utilise.
Le dessin appartient au cadre réutilisable et n'est pas un contenu de circuit sélectionnable. Les primitives qui ont une `color` (chemins, cercles, textes, blocs de propriétés : un logo en couleur) la gardent ; les autres sont dessinées dans la couleur du cadre.
La commande commune `set_frame` accepte un `artwork: {"items": [...]}` JSON facultatif ; un tableau items vide
rétablit le cadre généré. `list_frames` comprend le dessin. Le canevas, l'aperçu du cadre
et les exports utilisent le même moteur de rendu. Les fichiers de projet plus anciens
gardent les cadres générés.

### Images de cartouche et styles de texte

Les hôtes qui annoncent `title_block_images` acceptent des éléments `Image` dans les
cartouches (`forms.title_blocks[].items`) et dans les formulaires de rapport dessinés
(`layout.header`, `layout.row`) :
`{"type": "Image", "a": {…}, "b": {…}, "picture": {"file": {"name", "data"}, "px": [w, h]}}`
avec un PNG ou un JPEG en base64 ajusté dans le rectangle `a`..`b` en gardant ses
proportions. Sans `picture`, l'élément montre `info.logo` (même forme que `picture`), le
logo du projet que la page de garde montre aussi : au-dessus du nom du projet, ou ajusté
dans `info.cover_logo` (`x`, `y`, `width`, `height` en mm depuis le coin supérieur gauche
de la zone de dessin ; commande `set_cover_logo`). Les éléments `Text` prennent `bold`,
`italic` et `angle` facultatifs (degrés, dans le sens antihoraire). Les espaces réservés
`{page.previous}`, `{page.next}` et
`{rev.N.name|date|author|description}` (N-ième dernière révision diffusée) sont remplis
comme les autres. Les commandes communes `list_title_blocks`, `set_title_block` (éléments
sous la forme `items: {"items": [...]}`), `delete_title_block`, `set_default_title_block` et
`set_page` (`title_block`) les modifient ; la ligne de commande ajoute
`add_title_block_image` et `set_project_info` (`logo`) pour les fichiers d'image.

### Conditions, styles, codes QR et tampons des formulaires

Les hôtes qui annoncent `form_customization` acceptent ces champs supplémentaires de
`forms` (tous facultatifs, les fichiers et plugins plus anciens les omettent) :

- Chaque élément d'un cartouche ou d'un formulaire dessiné prend `show: {"filled": "…",
  "page_types": [...]}` : il n'est dessiné que lorsque `filled` (avec des espaces réservés)
  n'est pas vide et seulement sur les pages des types donnés (`schematic`, `cover`,
  `report`, `mounting`, `drilling`, `overview`, `graphic`, `document`). `Line`, `Rect` et `Text`
  prennent `color: [r, g, b]`, `Line` et `Rect` `dashed`.
  `{"type": "Qr", "a": {…}, "size": mm, "text": "…"}` dessine un code QR (mode octets,
  niveau M) du texte rempli.
- `forms.page_type_title_blocks` associe des types de page à des noms de cartouche et
  `forms.frames[].title_block` nomme le cartouche d'un cadre. Une page utilise son propre
  cartouche, sinon celui de son type, de son cadre, celui par défaut du projet, Standard.
- `forms.stamp` (`draft`, `review`, `approved`, `released`, `size`, `angle`, `color`)
  est un texte dessiné en travers de chaque page selon l'état du flux de révision ; les
  espaces réservés `{rev.status}`, `{rev.approved_by}`, `{rev.approved_date}` et
  `{rev.N.approved_by|approved_date}` donnent l'état et qui a approuvé une révision.
- Les formulaires de rapport prennent `totals` (colonnes additionnées sous la dernière
  ligne), `shade_rows` et `align` (`{column: "Left" | "Center" | "Right"}`) ; les pages de
  rapport figées les gardent dans `totals`, `shade_rows` et `align` de leur tableau.

Commandes : `set_stamp`, `set_default_title_block` (`page_type`), `set_frame`
(`title_block`) ; `set_title_block` prend les nouveaux champs d'élément. La bibliothèque de
formulaires (fichier et bibliothèque `forms` du serveur d'articles, `GET` /
`PUT /api/v1/libraries/forms`) porte aussi `frames` ; la ligne de commande la charge et la
publie avec `parts_company_forms` (avec `publish`).

### Champs de renvois de fonction dynamiques

Les hôtes qui annoncent `function_cross_references` résolvent la propriété en lecture seule
`function.cross_references` à partir de l'index natif courant des appareils/interruptions
et des paramètres de renvois. Les lignes de blocs de propriétés conservent le placement et
la disposition définis par l'auteur tandis que leur texte suit les pages et sections
renommées et les cibles d'appareil modifiées.
Leurs liens par ligne vers la page cible survivent à l'export PDF ; le texte du canevas et
du SVG utilise le même résolveur. Un champ explicite remplace
le texte de renvoi automatique ; les symboles ordinaires conservent les annotations
automatiques.
Les aperçus de bibliothèque/fantômes sans contexte de projet laissent vide la ligne de
renvoi.

La commande commune `list_cross_references` renvoie pour chaque fonction `id`, `page`,
`tag`, le `text` multiligne et les `references` (texte et page cible, null pour les cibles
manquantes). Les filtres facultatifs `id` et `page` utilisent les identifiants de symbole
natifs et les noms de page.
La propriété est aussi disponible par `get_properties` et les sélecteurs de propriétés ;
`set_properties` rejette les tentatives de figer sa valeur calculée.

