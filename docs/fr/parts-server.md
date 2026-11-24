# Serveur d'articles et gestionnaire d'articles

Le portail de données commun à toutes les installations d'OpenECraft et l'application qui le gère.

`openecraft-parts-server` est le portail de données de toutes les installations d'OpenECraft
(bureau et web), dans l'esprit de l'EPLAN Data Portal : des catalogues de fabricants avec une
recherche à facettes, l'historique et le cycle de vie des articles, les fichiers des articles
(fiches techniques, images, modèles, macros), des comptes d'utilisateurs avec des rôles et
une page web pour parcourir le tout. Les détails d'un article montrent sa vue de face 2D et
son modèle 3D (que l'on tourne en le faisant glisser) lorsqu'il en a ; les articles du
catalogue d'exemple ont tous les deux lorsque leurs dimensions sont connues, générés à partir
des dimensions (appareils, bornes, modules d'automate, rails, goulottes, systèmes de jeux de
barres, enveloppes et leurs composants, accessoires). C'est un exécutable séparé qui
fonctionne indépendamment de l'éditeur.

```bash
cargo run --release -p openecraft-parts-server -- --bind 0.0.0.0:8090 --token <secret>
cargo run --release -p openecraft-parts-server -- --storage json --data parts-db.json
cargo run --release -p openecraft-parts-server -- --data parts-db.sqlite --migrate-from parts-db.json
cargo run --release -p openecraft-parts-server -- --install-catalogs all
cargo run --release -p openecraft-parts-server -- --config parts-server.toml
```

En conteneur : `docker run -d -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data
ghcr.io/nilres/openecraft-parts-server` (voir [installation](installation.md)).

- Fichier de configuration (`--config <file>` ou `OPENECRAFT_PARTS_SERVER_CONFIG`) : un
  fichier TOML avec les réglages des options de la ligne de commande sous leurs noms
  (`bind`, `storage`, `data`, `token`, `admin_user`, `install_catalogs`, `access`,
  `registration`, `dashboard` ; voir
  [`parts-server/parts-server.example.toml`](../../parts-server/parts-server.example.toml)).
  Les options de la ligne de commande ou de l'environnement l'emportent ; les chemins
  relatifs sont relatifs au fichier ; des clés inconnues arrêtent le serveur avec une
  erreur. `--migrate-from` n'est qu'une option de la ligne de commande.
- Stockage (`--storage sqlite|json`) : SQLite par défaut (`--data parts-db.sqlite`, un seul
  fichier avec les articles indexés, l'historique, les comptes et les fichiers) ; JSON garde
  tout dans un seul fichier écrit de façon atomique à chaque modification, avec les fichiers
  dans `<name>.files/` à côté de lui. Un fichier de données `.json` sélectionne le stockage
  JSON. Les nouvelles bases de données sont remplies avec le catalogue d'exemple et les
  catalogues de fabricants fournis ; `--migrate-from` copie une base de données existante
  dans une nouvelle.
- Catalogues fournis : `crates/openecraft-parts/catalogs/<id>.json` (un `catalog` et ses
  `parts`, listés dans `openecraft_parts::bundled`, que les projets d'exemple utilisent aussi)
  sont des jeux de départ choisis avec soin d'articles réels de
  fabricants : `abb` (disjoncteurs, interrupteurs différentiels, contacteurs, disjoncteurs
  moteurs manuels), `beckhoff` (coupleur et bornes EtherCAT), `cosmo` (collecteurs de
  circuits de chauffage en acier inoxydable pour 2 - 14 circuits et boîtiers de régulation
  par pièce RTL / TH pour le chauffage par le sol), `dehn` (parafoudres de type 1 - 3, pour les installations photovoltaïques et pour les lignes de données), `gira` (prises de
  courant System 55, mécanismes d'interrupteurs et de boutons-poussoirs, prise réseau Cat.6A
  avec enjoliveurs, caches et cadres), `hager` (disjoncteurs,
  interrupteurs différentiels, contacteurs d'installation, interrupteurs-sectionneurs,
  interrupteurs-sectionneurs à fusibles NH pour jeux de barres de 60 mm et réglettes
  d'interrupteurs-sectionneurs à fusibles pour 185 mm avec transformateurs de courant
  enfichables, disjoncteurs h3+ avec commandes motorisées et déclencheurs, le relais de
  protection de découplage EU400, les armoires univers avec leurs modules de composants,
  compteurs d'énergie), `janitza` (analyseur de réseau UMG 604-PRO), `ledvance` (plafonniers LED SURFACE
  CIRCULAR, downlight DOWNLIGHT SLIM, PANEL 600, réglette DAMP PROOF, hublots et hublots de
  secours), `jinko` (modules photovoltaïques Tiger Neo
  54HL4-B all black 405 - 415 Wc), `longi` (modules photovoltaïques Hi-MO X6 Explorer
  LR5-54HTH 425 - 435 Wc), `loxone` (compteur d'énergie pour
  transformateurs de courant 1 / 5 A sur le bus Tree), `mdt` (actionneurs KNX, interface /
  routeur IP, alimentation, boutons-poussoirs KNX, bouton-poussoir en verre et régulateur de
  température ambiante à encastrer), `mennekes` (prises murales CEE 16 A et 32 A,
  5 pôles), `merten` (boîte de raccordement de cuisinière à encastrer), `phoenix_contact` (blocs de jonction, modules
  relais, alimentations), `pylontech` (modules de batterie LiFePO4 19 pouces), `siemens` (contacteurs SIRIUS, relais de surcharge,
  disjoncteurs moteurs, disjoncteurs SENTRON), `signify` (Philips CoreLine downlight
  DN140B, dalle RC132V, réglette étanche WT120C, applique WL120V), `spelsberg` (boîtes de
  dérivation Abox 025-L à 350-L, placées comme enveloppes de la salle), `striebel_john` (armoires murales ComfortLine A de 5 à 9 rangées
  et de 1 à 5 largeurs de champ avec les panneaux de comptage, de distribution et multimédia de leur hauteur,
  portes, serrures, brides et autres accessoires), `victron` (onduleur /
  chargeur MultiPlus-II, Inverter RS Smart Solar, SmartSolar MPPT, Orion-Tr, Cerbo GX, SmartShunt, BMV-712, Lynx
  Distributor ; surtout des articles pour le montage mural direct), `wago` (bornes de
  raccordement 221 et 2273 pour boîtes de dérivation) et `weidmueller` (bornes,
  module relais, alimentations, goulottes de câblage CD). `--install-catalogs all` (ou des
  identifiants tels que `siemens`) les installe ou les met à jour dans une base de données
  existante : les nouveaux articles sont ajoutés, les articles que personne n'a modifiés
  sont mis à jour, les articles modifiés ou supprimés par des utilisateurs sont conservés.
  Les articles sans modèles en reçoivent, générés à partir de leurs dimensions de montage.
  N'ajoutez que des données qui peuvent être redistribuées (des faits tirés de fiches
  techniques publiques, pas de textes, d'images ou de modèles copiés). Chaque catalogue
  fourni a un logo, `catalogs/logos/<id>.svg`, qu'il reçoit à son installation tant qu'il
  n'a pas de logo : le logo de l'entreprise issu de Wikimedia Commons (domaine public ou
  CC BY-SA 4.0, sources et licences dans `catalogs/logos/README.md` ; les logos sont des
  marques de leurs propriétaires), ou un badge neutre avec le nom lorsque Commons n'en a
  pas sous licence libre (COSMO, Janitza, MDT, Pylontech).
- Logos : chaque catalogue peut avoir le logo de son entreprise (PNG, JPEG, GIF, WebP ou
  SVG, 512 kB au plus), affiché sur les cartes des catalogues, dans la liste des catalogues
  de l'administration et sur les articles du catalogue. Qui peut modifier un catalogue
  change son logo dans *Administration › Catalogues › Édition* (`PUT` / `DELETE
  /api/v1/catalogs/{id}/logo?name=`, `GET` le sert avec une politique de sécurité du contenu
  en bac à sable, de sorte qu'un SVG n'exécute aucun script). Enregistrer un catalogue garde
  son logo ; le fichier du logo est stocké avec les fichiers des articles et fait partie des
  sauvegardes.
- Bibliothèques : des documents JSON communs à tous les projets (`GET` / `PUT /api/v1/libraries/{name}`),
  pour l'instant `forms`, les cartouches, les schémas de rapport et les cadres de
  l'entreprise que l'éditeur (*Formulaires › Cartouches › Bibliothèque de l'entreprise*) et
  la ligne de commande publient et chargent. Quiconque peut lire les articles les lit ; les
  modifier demande le droit de modifier les catalogues pour tout le serveur (éditeurs,
  administrateurs).
- Utilisateurs : les lecteurs gardent des favoris, les fabricants modifient les articles et
  les catalogues de leur propre fabricant, les éditeurs tous les articles, les
  administrateurs gèrent aussi les utilisateurs. Le `--token` (ou `OPENECRAFT_TOKEN`) est le
  jeton d'administrateur ; `--admin-user <name>` (ou `OPENECRAFT_ADMIN_USER`) crée un compte
  d'administrateur avec le mot de passe de `OPENECRAFT_ADMIN_PASSWORD`.
  Les utilisateurs se connectent avec `POST /login` (`name` et `password` ; `user` est pris
  pour `name`, comme sur le serveur de projets) et envoient le jeton de session comme
  `Authorization: Bearer <token>`. Un jeton que le serveur ne connaît pas (une session
  expirée) est refusé avec `401`, pour la lecture aussi, afin que les clients le remarquent
  au lieu de lire anonymement. Les mots de passe des
  comptes locaux (aussi celui de `--admin-user`) ont au moins 8 caractères ; après 5 connexions échouées d'un nom d'utilisateur, chaque tentative suivante attend
  (1 s, en doublant jusqu'à 5 minutes, réponse `429`). Sans jeton, sans comptes et sans annuaires, le serveur
  est en lecture seule.
- Inscription : lorsqu'elle est ouverte, n'importe qui crée un compte local de lecteur sur
  la page web (*S'inscrire* à côté de la connexion, `POST /register` avec `name` et
  `password`) ; les administrateurs donnent davantage de droits. Par défaut, le nouveau
  compte attend l'approbation d'un administrateur (`202`, pas de session) : il ne peut pas
  se connecter tant qu'un administrateur ne l'a pas approuvé dans *Administration ›
  Utilisateurs* (`POST /users/{name}/approval`) ou rejeté (supprimé) ; les comptes en
  attente sont listés en premier et comptés dans la vue d'ensemble. Sans approbation
  (désactivée dans *Administration › Droits d'accès*, stockée comme
  `"registration_approval": false`), le nouvel utilisateur est connecté aussitôt. Approuver
  reste le choix d'un administrateur même lorsque les options de démarrage fixent
  l'inscription. Les comptes qui attendent quand l'approbation est désactivée continuent
  d'attendre jusqu'à leur approbation. Elle est fermée tant qu'elle n'est pas
  activée : par les administrateurs dans *Administration › Droits d'accès* (stockée comme
  `"registration": true` dans le fichier des paramètres d'accès), ou fixée par les options
  de démarrage `--registration true|false`, `OPENECRAFT_REGISTRATION` ou `registration` dans
  le fichier de configuration, qui l'emportent sur l'interrupteur. Les utilisateurs inscrits
  lisent les articles même là où la lecture demande une connexion. Un client peut inscrire
  5 comptes, puis chaque tentative suivante attend (1 s, en doublant jusqu'à 5 minutes) ;
  les noms auxquels des attributions se réfèrent (aussi avec des caractères génériques) et
  `admin` ne sont pas disponibles, afin que personne ne s'inscrive dans les droits d'une
  attribution. Un nom inscrit passe aussi avant un utilisateur d'annuaire de même nom qui
  ne s'est pas encore connecté : sur les serveurs avec des annuaires, attribuez par groupes.
- Contrôle d'accès (`<data>.access.json`, `--access`) : connexion avec des annuaires Active
  Directory ou LDAP à côté des comptes locaux, groupes d'utilisateurs, rôles définis par les
  administrateurs et attributions de rôles à des utilisateurs et à des groupes par fabricant
  ou par catalogue, avec des attributions de refus ; la lecture anonyme peut être
  désactivée. Voir
  [docs/access-control.md](access-control.md).
- Tableau de bord : le *Tableau de bord* de la page web montre le nombre d'articles, de
  catalogues, de fabricants et de fichiers et des graphiques des articles par fabricant,
  groupe de produits, catalogue, groupe principal et cycle de vie, les dernières
  modifications par mois, combien d'articles ont une description, un prix, des dimensions,
  des modèles, des fichiers et des fiches techniques, les utilisateurs par rôle
  (administrateurs) et les articles modifiés en dernier ; ses barres ouvrent les articles
  qu'elles comptent (`GET /api/v1/dashboard`). Les administrateurs le masquent dans
  *Administration › Vue d'ensemble* (`"dashboard": false` dans les paramètres d'accès,
  `PUT /api/v1/dashboard/settings`) ; l'option de démarrage `--dashboard true|false`,
  `OPENECRAFT_DASHBOARD` ou `dashboard` dans le fichier de configuration le fixent.
- Les exports portent la version du serveur, afin qu'une version ultérieure puisse les
  convertir à leur import : `GET /export.json` est un objet avec `export` (`format`,
  `format_version`, `server`, `server_version`, `revision`, `exported`) et `parts`
  (`/import` et les éditeurs le lisent comme auparavant), `GET /export.csv` commence par une
  ligne `# format=openecraft-parts;format_version=1;server_version=…` (l'import CSV saute
  de telles lignes avant l'en-tête), les sauvegardes gardent les champs comme
  `export.<name>` dans leur table `meta`, les catalogues et les paramètres d'accès exportés
  sur la page web ont un objet `export` et le CSV des utilisateurs la même première ligne.
  Les réponses portent aussi l'en-tête `X-OpenECraft-Server-Version`.
- Les articles sont contrôlés avant d'être stockés : les numéros d'article ont au plus 128
  caractères, les autres champs au plus 256 (description, note, caractéristiques
  techniques, homologations et fiche technique 20000), le catalogue qu'un article nomme
  doit exister, et un import refuse un fichier qui contient deux fois un numéro d'article.
- Chaque modification est une nouvelle révision ; chaque article garde sa révision, l'heure
  de la modification et les 20 dernières versions (historique). Les articles ont un cycle
  de vie (actif, en fin de série, arrêté) et un successeur. Les fichiers sont stockés une
  fois par contenu ; un fichier est supprimé dès que ni un article ni une version de
  l'historique ne l'utilise (et au démarrage).
- Groupes de produits : les groupes de produits génériques, groupes de produits et
  sous-groupes d'EPLAN avec leurs numéros d'échange (`openecraft_parts::productgroups`) sont
  proposés dans l'éditeur d'articles à côté des groupes propres à OpenECraft.
- Recherche (`POST /search`) : texte (chaque mot doit apparaître), groupe principal, groupe,
  sous-groupe, fabricant, catalogue, cycle de vie et caractéristiques techniques (tension,
  courant, puissance et les lignes `Name: value` des caractéristiques techniques), triée et
  paginée, avec des facettes qui comptent les articles correspondants par valeur.
- Page web : `http://<server>:8090/` – tableau de bord, recherche avec facettes, détails
  des articles avec fichiers et historique, catalogues, connexion et favoris, en anglais,
  allemand, français et espagnol. Le logo et le titre en haut à gauche ramènent à la page
  d'accueil (tous les articles, aucun filtre). La liste montre 10, 25, 50, 100 ou 200
  articles par page (*Par page*, conservé par le navigateur). L'adresse garde la recherche
  et l'article ouvert (signets, bouton Précédent du navigateur) ; `?lang=de` choisit la
  langue.
- Nouveaux articles sur la page web : les utilisateurs qui peuvent modifier des articles
  d'un fabricant ou d'un catalogue ont *Nouvel article* au-dessus de la liste et *Édition* /
  *Supprimer* sur les articles qu'ils peuvent modifier ; le formulaire contient les données
  générales, techniques, commerciales et de montage, propose les catalogues auxquels
  l'utilisateur peut ajouter des articles et ne remplace jamais un article existant
  (`PUT /parts/{number}` avec `If-None-Match: *` répond `409`).
- Administration sur la page web (*Administration*, montrée aux administrateurs et aux
  utilisateurs qui peuvent modifier des catalogues ou des articles ; connectez-vous avec un
  utilisateur ou, sur un nouveau serveur, avec le jeton d'administrateur) : une vue
  d'ensemble du serveur (stockage, révision, utilisateurs par rôle, paramètres d'accès,
  droits de l'appelant) ; *Utilisateurs* – créer, modifier et supprimer des utilisateurs
  locaux (rôle, fabricant, groupes, nouveau mot de passe), voir leur dernière connexion et
  leurs sessions ouvertes et les déconnecter partout ; *Droits d'accès* – le rôle des
  requêtes sans connexion, les rôles définis avec leurs droits et les attributions à des
  utilisateurs et à des groupes par fabricant ou catalogue ; *Groupes* – des groupes
  d'utilisateurs avec leurs membres ; *Annuaires* – les connexions Active Directory et LDAP
  avec un test de la connexion et d'une ouverture de session ; *Catalogues* – créer,
  modifier et supprimer des catalogues, les autorisations des utilisateurs et des groupes
  sur chaque catalogue, et installer ou mettre à jour les catalogues fournis ; *Importer
  des articles* (CSV,
  JSON, BMEcat, XML EPLAN) ; *Export et sauvegarde* – les articles en CSV ou JSON, les
  catalogues, les utilisateurs (CSV, sans mots de passe) et les paramètres d'accès (JSON,
  que l'on peut restaurer), et une sauvegarde de toute la base de données. La page ne
  propose que ce que l'utilisateur peut faire ; le serveur contrôle chaque modification.
- Sauvegarde (`GET /backup`, administrateurs) : un seul fichier SQLite avec les articles,
  leur historique et leurs fichiers, les catalogues, les utilisateurs avec les hachages de
  leurs mots de passe et les favoris, quel que soit le stockage du serveur. Restaurez-la en
  démarrant un serveur avec `--data <backup>.sqlite`, ou copiez-la dans une nouvelle base
  de données avec `--migrate-from`. Le fichier des paramètres d'accès (annuaires, rôles,
  attributions) n'en fait pas partie : gardez aussi une copie de `<data>.access.json`.
- API REST (`/api/v1`, voir `openecraft_parts::api`) : `GET /info`, `GET /parts`,
  `POST /search`, `GET|PUT|DELETE /parts/{number}`, `GET /parts/{number}/accessories` (ses accessoires
  et, pour un accessoire, les articles auxquels il convient), `GET /parts/{number}/history[/{revision}]`,
  `POST|DELETE /parts/{number}/attachments`, `GET /files/{id}` (les PDF, les images
  matricielles et le texte sont affichés dans le navigateur, les autres fichiers tels que
  SVG et XML sont téléchargés, de sorte qu'aucun fichier n'exécute un script à l'adresse du
  serveur), `POST /import` (CSV,
  JSON ou BMEcat), `GET /export.csv` (tous les articles, ou ceux de la requête dans
  l'adresse : `q`, groupes, fabricant, catalogue, cycle de vie, tri), `GET /export.json` (la
  même chose comme liste d'articles JSON que `/import` lit), `GET /backup`,
  `GET|PUT|DELETE /catalogs/{id}`,
  `GET /bundled`, `POST /bundled/{id}` (installer un catalogue fourni), `POST /login`,
  `POST /register`, `GET|PUT /registration`, `POST /users/{name}/approval`,
  `POST /logout`, `GET|PUT /favorites`, `GET /users` (avec leurs sessions ouvertes et leur
  dernière connexion), `PUT|DELETE /users/{name}`, `DELETE /users/{name}/sessions`
  (déconnecter partout), `GET|PUT /access`, `GET|PUT|DELETE /catalogs/{id}/logo`, `GET|PUT /catalogs/{id}/grants`, `GET /groups`,
  `PUT|DELETE /groups/{name}`, `GET /directories`, `PUT|DELETE /directories/{id}`,
  `POST /directories/{id}/test`, `GET /dashboard`, `GET|PUT /dashboard/settings`.
- Dans l'éditeur : *Données de base › Portail de données* – rechercher sur le serveur, voir
  les détails avec la vue de face et le modèle 3D, ajouter des articles à la base de données locale, enregistrer leurs fichiers, ajouter leurs macros à la bibliothèque de macros et contrôler
  la base de données quant aux versions plus récentes et aux articles en fin de série ou
  arrêtés. Les détails montrent le catalogue de l'article avec son logo (masqué avec
  *Paramètres › Apparence › Portail de données*). Les logos sont dessinés sur une plaque
  blanche ; les logos SVG sont rendus avec `resvg` et la police propre à l'éditeur, de sorte
  qu'ils ont le même aspect sur le bureau et dans le navigateur. *Appareils › Gestion des
  articles › 🌐 Serveur d'articles* télécharge tous les articles ou téléverse des articles.
  Le contrôle du projet avertit des appareils dont les articles sont arrêtés ;
  `use_successor_parts` les fait passer au successeur.
- Le serveur parle en HTTP simple, les jetons et les mots de passe traversent donc le réseau
  sans chiffrement. Gardez-le sur `127.0.0.1` derrière un proxy inverse TLS lorsqu'il est
  accessible depuis d'autres machines, p. ex. avec [Caddy](https://caddyserver.com/), qui
  obtient le certificat :
  `caddy reverse-proxy --from parts.example.com --to 127.0.0.1:8090`.

## Serveur d'articles public

Le projet exploite un serveur d'articles public à l'adresse
[parts-server.openecraft.org](https://parts-server.openecraft.org) avec le catalogue
d'exemple et les catalogues de fabricants fournis. Tout le monde le lit sans compte :
sur sa page web (recherche, catalogues, tableau de bord, les détails d'un article avec sa
vue de face et son modèle 3D), dans l'éditeur (*Données de base › Portail de données*), dans
le gestionnaire d'articles et en ligne de commande ou par des agents IA. C'est le serveur
préréglé de tous (`openecraft_parts::api::PUBLIC_URL`) : l'éditeur et le gestionnaire
d'articles démarrent avec son adresse, la ligne de commande l'utilise sans `--server` /
`OPENECRAFT_SERVER`. Un serveur à vous se saisit à la place comme URL du serveur
(`--server`, ou `parts_connect` avec son `url`) ; l'adresse saisie en dernier est conservée.
C'est un endroit pour essayer le portail de données et pour reprendre des articles dans
votre propre base de données ; une entreprise garde ses propres articles sur un serveur à
elle (ci-dessus), où elle décide qui peut les modifier. Les données des catalogues sont
compilées à partir de fiches techniques publiques : vérifiez les articles par rapport aux
données du fabricant avant de commander.

## Mentions légales, avertissement et notice de confidentialité

Sous la page web se trouvent les liens *Confidentialité* et, lorsque l'exploitant a un nom,
*Mentions légales* (en allemand *Datenschutz* et *Impressum*), qui ouvrent chacun une boîte
de dialogue, ainsi que les avertissements d'OpenECraft (aucune garantie, les noms de
produits qu'il mentionne appartiennent à leurs propriétaires). Ils sont là sans connexion,
comme pour le [site web](../website.md#legal-notice), dont les champs et les variables sont
les mêmes :

| Clé de `[legal]` | Variable | Affiche |
| --- | --- | --- |
| `name` | `OPENECRAFT_LEGAL_NAME` | La personne ou l'entreprise qui exploite le serveur ; vide masque les mentions légales |
| `address` | `OPENECRAFT_LEGAL_ADDRESS` | Son adresse postale |
| `email` | `OPENECRAFT_LEGAL_EMAIL` | L'adresse e-mail, comme lien |
| `phone` | `OPENECRAFT_LEGAL_PHONE` | Le numéro de téléphone |
| `representative` | `OPENECRAFT_LEGAL_REPRESENTATIVE` | Qui représente une entreprise |
| `register` | `OPENECRAFT_LEGAL_REGISTER` | Le tribunal d'immatriculation et le numéro |
| `vat` | `OPENECRAFT_LEGAL_VAT_ID` | Le numéro d'identification à la TVA |
| `responsible` | `OPENECRAFT_LEGAL_RESPONSIBLE` | Qui est responsable du contenu, avec l'adresse |
| `text` | `OPENECRAFT_LEGAL_TEXT` | Texte supplémentaire : avertissements, règlement des litiges |
| `privacy` | `OPENECRAFT_LEGAL_PRIVACY` | Ce que l'exploitant ajoute à la notice de confidentialité : l'hébergement, les journaux, à qui s'adresser |

Réglez-les dans la table `[legal]` du fichier de configuration ou avec les variables, qui
l'emportent ; les lignes de l'adresse et des textes sont séparées par `|` (ou par des sauts
de ligne dans le fichier). `GET /api/v1/legal` les renvoie en JSON. Les libellés suivent la
langue de la page ; les valeurs sont affichées telles qu'elles sont réglées.

La notice de confidentialité dit ce que font la page et le serveur, dans la langue de la
page : pas de cookies, pas de pistage, rien de chargé depuis d'autres serveurs ; la langue,
les réglages de la page et la clé de la session dans le stockage du navigateur ; les comptes
(nom, rôle, groupes, le mot de passe sous forme de hachage) ; les connexions échouées
mémorisées pendant un court moment ; qui a modifié un article et quand (son historique) et les favoris d'un utilisateur. Ce que seul l'exploitant sait (qui héberge le
serveur, ce qu'un proxy inverse journalise et pendant combien de temps, à qui s'adresser,
comme le demande l'article 13 du RGPD) se règle dans `privacy` et s'affiche en dessous.
Gardez les textes exacts : une modification qui stocke ou envoie autre chose les modifie
aussi (`renderLegal` de `common.js`, `privacy()` du script de la page). Qu'un serveur ait
besoin de mentions légales, et ce qu'elles doivent dire, dépend de l'exploitant et du
pays : OpenECraft ne montre que ce qui est réglé.

## EPLAN Data Portal

La gestion des articles se connecte aussi à l'EPLAN Data Portal : choisissez *EPLAN Data
Portal* comme type de serveur dans la fenêtre du portail de données ou dans *Gestion des
articles › 🌐 Serveur d'articles*, gardez l'adresse `https://dataportal.eplan.com/api` et
saisissez un jeton d'accès personnel (créé pour le Data Portal dans le profil de votre
compte EPLAN, ePulse). Le portail fonctionne alors comme un serveur d'articles : recherche
par texte, fabricant et groupe de produits, détails des articles avec les données
commerciales (CSV), les dessins DXF et les images, ajout d'articles à la base de données
locale et contrôle des mises à jour de la base de données.

Le PAT est envoyé comme `Authorization: Bearer PAT:<token>` (les préfixes `PAT:` et
`Bearer ` collés sont acceptés aussi). Le test de connexion lit le catalogue public puis
télécharge les données commerciales du premier article renvoyé pour vérifier l'accès
authentifié au téléchargement. Une recherche publique réussie ne prouve pas à elle seule
l'authentification ;
`/authorization` n'est pas utilisé, car il rejette des PAT acceptés par la passerelle de téléchargement. Les téléchargements de DXF et
de données commerciales envoient par POST l'identifiant de l'article à
`/download/{type}/part`, comme le client web du portail ; les identifiants de pièces jointes existants dans les projets enregistrés restent pris en charge. L'utilisation
du portail et des données d'articles qui en sont tirées est régie par les conditions
d'utilisation d'EPLAN et la licence du compte : vérifiez-les avant de mettre des articles
du portail sur un serveur d'articles partagé. OpenECraft n'est pas affilié à EPLAN.

- `openecraft_client::eplan` enveloppe l'API REST du portail (JSON:API) : il construit les
  requêtes du portail à partir des requêtes du serveur d'articles et transforme les réponses
  en `ServerInfo`, `SearchResult` et `Part`. `openecraft_client::Connection` offre les
  fonctions de lecture des deux types de serveur ; l'éditeur et la ligne de commande
  l'utilisent.
- La connexion est en lecture seule : pas de téléversements, de catalogues, d'utilisateurs,
  de connexion, de favoris, d'historique ni de téléchargement de toute la base de données.
  Les facettes comptent les articles de la page affichée, car le portail ne les compte pas ;
  les catalogues, le cycle de vie et les caractéristiques techniques ne peuvent pas être
  filtrés.
- Le portail trouve les articles par ses propres identifiants, non par numéro : un article
  chargé depuis le portail garde l'identifiant (`portal_id`), de sorte que l'ajout des
  articles marqués et le contrôle des mises à jour les chargent en une seule requête. Le
  contrôle des mises à jour ne compare que les articles venus du portail ; un numéro
  d'article isolé est trouvé par une recherche exacte.
- Les fabricants sont filtrés par leur nom abrégé, les groupes de produits par
  l'identifiant de l'entrée du catalogue EPLAN : (groupe de produits générique + 1) · 10⁸ +
  (groupe de produits + 1) · 10⁴ + (sous-groupe + 1), p. ex. `200050000` pour
  électrotechnique / connecteurs.
- Les coffrets (groupe de produits *Enclosure*) viennent du portail sans dimensions :
  leur taille, leur système de coffret et l'absence de plaque de montage sont lus dans
  la description (`WHD: 800x2000x600 mm`, *baying*, *compact*, *terminal box*,
  *without mounting plate*), de sorte qu'ils se placent dans l'implantation d'armoire comme les
  coffrets des catalogues (testé avec Rittal VX25, TS 8, AX et KX).
- Un article chargé depuis le portail liste ses accessoires par leur numéro ; les
  détails de l'article les montrent avec leur description, leur fabricant et leur
  groupe de produits, de sorte qu'ils s'ouvrent et s'ajoutent comme les autres articles
  et vont avec leur article dans l'implantation d'armoire.
- Les articles au format EPLAN (macros, points de raccordement) viennent comme fichiers EDZ
  de la page web du portail avec une licence EPLAN (un compte gratuit ne télécharge que
  les données commerciales et les dessins DXF) ; importez-les comme d'autres fichiers d'articles (voir
  [gestion des articles](features/parts.md)).
- Seules la version de bureau et la ligne de commande atteignent le portail : il n'autorise
  aucune requête cross-origin des navigateurs, la version web aurait donc besoin d'un proxy.

### PAT du Data Portal partagé sous Windows

L'éditeur et la CLI/MCP peuvent utiliser le même PAT par le Gestionnaire d'identification de
Windows. Saisissez le PAT dans le Data Portal de l'éditeur, sélectionnez **Mémoriser le
jeton sur cet ordinateur** et utilisez **Tester la connexion**. La mémorisation est
désactivée par défaut : sans ce choix explicite, le PAT reste dans la session. Seul un
contrôle réussi du service protégé enregistre un remplacement.
Une erreur de stockage est affichée à part et n'invalide pas la connexion. Le PAT est stocké
pour l'utilisateur Windows courant sur cet ordinateur, non dans un projet, des paramètres
JSON, un fichier d'environnement ou une réponse MCP.

Laissez le champ du PAT vide pour utiliser automatiquement la valeur enregistrée. À la
première requête, l'interface graphique la charge en arrière-plan avant de construire cette
requête ; les requêtes simultanées partagent la recherche. Le fil de l'interface n'attend
jamais la lecture des identifiants. Le résultat, y compris un jeton manquant ou une erreur
de lecture, est mis en cache pour la session.
**Tester la connexion** force une nouvelle recherche après un remplacement ou une
suppression externes ou une erreur de lecture. **Supprimer le jeton enregistré** vide aussi
le cache. Un autre point d'accès ou un jeton explicite utilise un cache séparé, de sorte
qu'une recherche plus ancienne ne peut pas le remplacer.

MCP utilise la valeur enregistrée lorsque
`parts_connect` sélectionne `Eplan` sans jeton explicite. Chaque requête résout de nouveau
les identifiants, de sorte que les remplacements et les suppressions prennent effet sans
redémarrer MCP. Un jeton explicite fourni pour la connexion EPLAN courante a la priorité ;
MCP n'enregistre pas les jetons de session explicites dans le magasin partagé.

Les identifiants sont limités à l'origine HTTPS, au port et au chemin de l'API. Changer de
serveur ne réutilise pas le PAT enregistré d'un autre point d'accès. **Supprimer le jeton
enregistré** ne retire que l'entrée du portail courant et vide son champ dans l'interface
graphique. Les autres plateformes conservent des identifiants limités à la session ; aucune
persistance en clair n'est utilisée comme solution de repli.

Lors d'une mise à niveau depuis une version qui ne gardait les PAT qu'en mémoire, saisissez
le PAT une fois dans l'éditeur mis à jour, sélectionnez **Mémoriser le jeton sur cet
ordinateur** et testez la connexion. Il n'est pas nécessaire de le saisir dans MCP. Les
tests de connexion interrogent le service de jetons protégé du fournisseur de fichiers et
les métadonnées publiques du catalogue ; ils ne téléchargent pas de fichier de données
commerciales et ne dépendent pas du fait qu'un article donné du catalogue en ait un.

## Gestionnaire d'articles

`openecraft-parts-manager` est une application autonome qui gère les articles d'un serveur
d'articles : créer de nouveaux articles, modifier les articles existants (y compris les
modèles 2D / 3D), renommer et supprimer des articles, téléverser des fichiers d'articles
CSV / JSON et exporter tous les articles. Il utilise le même éditeur d'articles que la
gestion des articles de l'éditeur et fonctionne sur le bureau et dans le navigateur.

```bash
cargo run --release -p openecraft-parts-manager        # desktop
cd parts-manager && trunk serve                        # web, http://127.0.0.1:8082
cd parts-manager && trunk build --release              # static files in parts-manager/dist/
```

*Mentions légales et confidentialité* de la barre supérieure montre l'avertissement et la
notice de confidentialité du gestionnaire d'articles et, dans une version web dont le
`config.js` nomme un exploitant, ses mentions légales
([installation.md](installation.md#mentions-légales-dun-éditeur-web)).

Saisissez l'URL du serveur et le jeton d'accès (gardé en mémoire seulement) et cliquez sur
*Connecter*, ou connectez-vous avec un nom d'utilisateur et un mot de passe. Les
modifications sont enregistrées sur le serveur avec *Enregistrer* (`Ctrl+S`) ; l'éditeur
demande avant d'abandonner des modifications non enregistrées et avant de supprimer un
article. *Fichiers et historique* au-dessus de l'éditeur d'articles téléverse et retire les
fichiers d'un article et montre ses modifications ; *Catalogues* gère les catalogues de
fabricants (avec leurs logos : *Choisir un logo…*, *Supprimer le logo*, pour un catalogue
sans modifications non enregistrées) et *Utilisateurs* (administrateurs) les comptes et les
rôles. *Logos* à côté du menu des langues affiche ou masque les logos dans la liste des
catalogues (mémorisé).
