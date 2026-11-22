# Serveur de projets

Des projets partagés pour une équipe : plusieurs utilisateurs travaillent sur le même projet, chaque page est verrouillée par l'un d'eux.

`openecraft-project-server` stocke des projets pour une équipe. Plusieurs utilisateurs
ouvrent le même projet en même temps et travaillent sur des pages différentes ; une page est
modifiée par un seul utilisateur à la fois.

```bash
cargo run --release -p openecraft-project-server -- --bind 0.0.0.0:8091 --data projects --token <secret>
cargo run --release -p openecraft-project-server -- --config project-server.toml
```

En conteneur : `docker run -d -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret>
-v projects:/data ghcr.io/nilres/openecraft-project-server` (voir [installation](installation.md)).

- Fichier de configuration (`--config <file>` ou `OPENECRAFT_PROJECT_SERVER_CONFIG`) : les
  réglages des options de la ligne de commande en TOML (`bind`, `data`, `token`,
  `admin_user`, `access`, `web` ; voir
  [`project-server/project-server.example.toml`](../../project-server/project-server.example.toml)),
  comme pour le serveur d'articles.
- **Page web** (facultative : `--web`, `--web true`, `OPENECRAFT_PROJECT_WEB=true` ou
  `web = true` dans le fichier de configuration ; désactivée par défaut) :
  `http://<server>:8091/` liste les projets que l'utilisateur peut voir (avec les pages, la
  révision, qui les a créés et modifiés en dernier, qui y travaille et les droits de l'utilisateur) et montre chacun d'eux en lecture seule : les pages par section ou
  groupées par structure, les appareils avec les endroits où ils sont représentés, et les
  pages dessinées comme sur le papier (zoom, glisser pour déplacer, touches fléchées ou
  Page up / Page down pour la page suivante). Les pages verrouillées par d'autres
  utilisateurs sont marquées par 🔒 et l'utilisateur ; la page consulte le projet toutes les
  20 s et suit les nouvelles révisions. Les utilisateurs se connectent avec leur compte ou
  le jeton du serveur ; sans connexion, la page montre ce que les requêtes anonymes peuvent
  voir (les paramètres d'accès). Les chefs de projet modifient les attributions d'un projet
  (*Autorisations*). Les administrateurs disposent d'une zone *Administration* : une vue
  d'ensemble (projets en cours d'utilisation, sessions ouvertes, utilisateurs par rôle,
  paramètres d'accès), les sessions (celles qui sont ouvertes avec leur utilisateur, leur
  projet, leur début, leur dernière activité, leurs enregistrements et leurs pages
  verrouillées, et l'historique de celles qui sont terminées avec leur durée et la façon
  dont elles ont pris fin : fermée, expirée, déconnectée ou utilisateur supprimé), les
  utilisateurs (rôle, projets, groupes,
  mot de passe), les droits d'accès (accès sans connexion, rôles définis, attributions) et
  les annuaires (Active Directory, LDAP, avec un test de la connexion et d'une ouverture de
  session). *Rapport de vérification* d'un projet montre son rapport pour la vérification
  sur site, conçu pour une tablette ou un téléphone : les réviseurs et les éditeurs y
  saisissent les données du rapport, l'examen visuel et les résultats mesurés de chaque
  circuit, sans ouvrir le projet et sans verrou, signent le rapport et le téléchargent
  comme classeur Excel
  ([rapport de vérification](features/calculations.md#rapport-de-vérification)).
  La page existe dans toutes les langues de l'application ; elle utilise l'API ci-dessous,
  le serveur contrôle donc chaque requête comme pour l'éditeur. Sans `--web`, le serveur
  répond à `/` par un court texte et n'a pas de routes de page.
- **Créé par, modifié en dernier par, dernière modification** : le serveur garde pour
  chaque projet qui l'a créé et qui y a enregistré la dernière modification, et quand ; la
  liste des projets de l'éditeur (volet *Serveur de projets*), la page web et
  `GET /api/v1/projects` (`created_by`, `created_at`, `changed_by`, `changed_at` ; secondes
  depuis 1970, UTC) les montrent à tous ceux qui peuvent voir le projet. Le nom est celui
  de l'utilisateur connecté ; sur un serveur sans comptes (ou avec le jeton du serveur),
  c'est le nom d'utilisateur avec lequel la session a été ouverte. Il est indépendant de
  *Créé par* dans les propriétés du projet, que les utilisateurs modifient. Seul un
  enregistrement qui modifie le projet compte : ouvrir ou lire un projet, les requêtes
  d'état qui gardent une session en vie, les verrous et l'enregistrement d'un contenu
  inchangé le laissent tel quel. Les projets de versions antérieures du serveur montrent
  *Inconnu* jusqu'à leur prochaine modification (la création reste inconnue), et un projet
  téléversé comme copie d'un autre est un nouveau projet de l'utilisateur qui l'a téléversé.
- Les noms de projets qui ne diffèrent que par la casse sont un seul nom (Windows et macOS
  ne distinguent pas leurs fichiers) : les requêtes trouvent un projet quelle que soit la casse de son nom ; les noms de périphériques de Windows (`CON`, `NUL`, `COM1` …) et les noms se terminant
  par un point sont refusés.
- Chaque projet est stocké comme projet fractionné dans un dossier à lui
  (`<name>/<name>.oecp` avec un petit fichier par page, appareil et article ; il s'ouvre
  dans l'éditeur de bureau et peut être placé sous gestion de versions) à côté de
  `<name>.state.json` (révision, compteur d'identifiants, qui a créé et modifié en dernier
  le projet) dans le répertoire de données. Un enregistrement n'écrit que les fichiers qui ont changé. Les fichiers de projet de versions antérieures (`<name>.oec`) sont convertis au
  premier démarrage et conservés comme `<name>.oec.bak`.
- Avec `--token` (ou `OPENECRAFT_PROJECT_TOKEN`), chaque requête a besoin de
  `Authorization: Bearer <token>` ; sans lui, sans comptes d'utilisateurs et sans annuaires,
  quiconque atteint le serveur peut modifier ses projets, mais seuls les clients de la
  propre machine du serveur peuvent créer des utilisateurs et modifier les paramètres
  d'accès (ainsi personne d'autre ne s'empare du serveur en créant le premier
  administrateur).
- Le serveur parle en HTTP simple, les jetons et les mots de passe traversent donc le réseau
  sans chiffrement. Gardez-le sur `127.0.0.1` derrière un proxy inverse TLS lorsqu'il est
  accessible depuis d'autres machines, p. ex. avec [Caddy](https://caddyserver.com/), qui
  obtient le certificat :
  `caddy reverse-proxy --from projects.example.com --to 127.0.0.1:8091`. L'éditeur
  utilise alors `https://projects.example.com`.
- **Comptes d'utilisateurs** (`users.json` dans le répertoire de données, hachages de mots
  de passe Argon2) : les lecteurs ouvrent les projets en lecture seule, les réviseurs
  enregistrent aussi des commentaires, des annotations et des approbations, les éditeurs
  verrouillent, modifient, enregistrent et créent des projets, les chefs de projet
  suppriment aussi des projets et gèrent qui peut les utiliser, les administrateurs gèrent
  aussi les utilisateurs ; le rôle d'un utilisateur peut être limité à certains projets
  (noms avec `*` / `?`). `--admin-user <name>`
  (ou `OPENECRAFT_PROJECT_ADMIN_USER`) crée le premier administrateur avec le mot de passe de
  `OPENECRAFT_PROJECT_ADMIN_PASSWORD` (au moins 8 caractères, comme tout mot de passe). Les
  utilisateurs se connectent depuis *Outils › Serveur de projets…* (nom d'utilisateur, mot de passe, *Se connecter*) ; le serveur utilise
  le nom du compte sur les verrous, quoi que dise la requête, et pour les commentaires, les
  réponses, les annotations et les approbations qu'un utilisateur connecté ajoute (les
  approbations des autres utilisateurs restent telles quelles). Les appelants anonymes ne
  peuvent pas utiliser le nom d'un compte. Les responsables d'un projet peuvent y ajouter
  des refus, mais seuls les administrateurs les lèvent. Les noms d'utilisateur ont au plus
  64 caractères et aucun caractère de contrôle ; un nouvel utilisateur a besoin d'un rôle,
  de groupes d'annuaire ou d'une attribution dans les paramètres d'accès. Se déconnecter
  ferme les projets ouverts avec cette connexion et libère leurs verrous ; supprimer un
  utilisateur ferme tous les siens. Une connexion échouée répond
  `wrong user name or password`, quel que soit celui des deux qui était faux.
- **Contrôle d'accès** (`access.json` dans le répertoire de données, `--access`) : les
  utilisateurs se connectent avec un compte local ou avec des annuaires Active Directory /
  LDAP ; les rôles (intégrés ou définis par les administrateurs) sont attribués à des
  utilisateurs et à des groupes d'annuaire par projet, les attributions de refus retirent
  des droits. Quiconque crée un projet le gère ; les chefs de projet modifient ses
  attributions dans la section *Accès au projet* de la fenêtre du serveur de projets. Voir
  [docs/access-control.md](access-control.md).
- Verrous : chaque page et les données du projet (liste et ordre des pages, appareils,
  articles, implantation d'armoire, paramètres) sont verrouillées séparément. Modifier
  quelque chose que vous n'avez pas verrouillé demande automatiquement le verrou ; si un
  autre utilisateur le détient, la modification est annulée. Les pages verrouillées par
  d'autres sont en lecture seule et marquées par 🔒 et le nom de l'utilisateur dans la liste
  des pages. Les verrous prennent fin lorsqu'ils sont libérés, lorsque le projet est fermé,
  ou 90 s après que l'éditeur a cessé d'envoyer son état (toutes les 15 s).
- L'enregistrement ne fusionne dans le projet du serveur que ce que vous avez verrouillé ;
  tout le reste vient du serveur. Les modifications des autres utilisateurs sont chargées
  automatiquement.
  Un enregistrement nomme la révision à partir de laquelle il a été modifié
  (`base_revision` ; l'éditeur le fait) : le serveur reprend alors aussi les pages et les
  données de projet modifiées que personne n'a verrouillées (il les verrouille d'abord pour
  vous) et répond avec les modifications qu'il a laissées de côté
  (`ignored`) : verrouillées par un autre utilisateur, modifiées par un autre utilisateur
  depuis cette révision ou non autorisées. Avec `strict`, rien n'est enregistré dans ce cas
  (`409`). Le serveur garde en mémoire les 16 dernières révisions par projet ; un
  enregistrement à partir d'une révision plus ancienne (ou sans `base_revision`)
  n'enregistre que ce qui est verrouillé.
- Chaque session reçoit sa propre plage d'identifiants d'objets, de sorte que les
  utilisateurs ne créent jamais le même identifiant.
- **Historique des sessions** : chaque session terminée (utilisateur, projet, début,
  dernière activité, fin, enregistrements, façon dont elle a pris fin) est ajoutée à
  `sessions.jsonl` dans le répertoire de données, une ligne JSON chacune ; le serveur garde
  les 5000 dernières et réécrit le fichier lorsqu'il a atteint deux fois plus de lignes.
  Les clés des sessions ne sont jamais stockées ni montrées, seulement leurs numéros.
- Les éditeurs préfèrent le `POST /projects/{name}/snapshot` lié à la session. Sur 404 ou
  405 seulement, ils utilisent l'ancien instantané par GET suivi de l'état de la session
  courante. Les deux doivent nommer la même révision avant que l'instantané soit
  appliqué ; l'état fournit les identités des propres verrous. Les échecs
  d'authentification ne déclenchent pas ce repli. Un projet modifié pendant le chargement
  est chargé de nouveau.
- Dans l'éditeur : *Outils › Serveur de projets…* – saisir l'URL du serveur, votre nom d'utilisateur et le
  jeton, lister les projets et en ouvrir un, ou téléverser le projet courant. Tant qu'un
  projet partagé est ouvert, *Enregistrer* (`Ctrl+S`) enregistre sur le serveur ; la fenêtre
  et le menu contextuel de la liste des pages verrouillent et libèrent les pages. La barre
  d'état montre *Serveur de projets : … / Projet : …* (*connexion perdue* tant que le
  serveur ne répond pas à son état) ; un projet restauré de la dernière session ou fermé sur
  le serveur apparaît comme *Copie de récupération locale du projet du serveur : …* et est
  enregistré dans un fichier local.
- **Enregistrement automatique :** tant qu'un projet partagé est ouvert, ses modifications
  terminées sont enregistrées d'elles-mêmes sur le serveur, 30 secondes après une
  modification puis toutes les 30 secondes tant que le travail continue (*Fichier ›
  Paramètres › Serveurs* ou le volet *Serveur de projets* : activé / désactivé et
  l'intervalle, de 10 à 600 s). Un glissement ou une saisie de texte en cours n'est pas
  envoyé ; il est enregistré dès qu'il est terminé. Un seul enregistrement s'exécute à la
  fois, un enregistrement automatique ne libère aucun verrou et rien n'est envoyé sans
  modifications. *Enregistrer* (`Ctrl+S`) enregistre toujours immédiatement. La barre
  d'état et le volet distinguent les états :
  *Enregistré*, *Pas encore enregistré*, *Enregistrement…*, *En attente du serveur* (pas de
  réponse, session terminée, déconnecté : les modifications sont enregistrées dès que la
  session reprend), *Échec de l'enregistrement* (nouvel essai à l'intervalle suivant) et
  *Enregistré en partie* (le serveur a laissé de côté ce qu'un autre utilisateur a
  verrouillé ou modifié ; votre version est gardée dans le volet et n'est pas renvoyée
  d'elle-même), avec l'heure du dernier enregistrement confirmé par le serveur.
  Un projet ne compte comme enregistré qu'après la réponse du serveur. La copie de
  récupération locale n'est pas un enregistrement sur le serveur : pendant une panne, elle
  est écrite aussitôt, et le volet ne le dit qu'après son écriture.
- La liste des projets sélectionne un projet (un double clic l'ouvre). *Propriétés…* charge
  ses propriétés depuis le serveur (jamais depuis la liste) et montre qui l'a créé sur le
  serveur et quand (*Inconnu* pour les projets créés avant que le serveur ne garde cette
  information ; ce n'est pas le *Créé par* du dessin, que les utilisateurs modifient). Qui
  peut modifier les données du projet les y change sans ouvrir le projet, sauf si une
  session a verrouillé les données du projet ; le serveur ne remplace alors que les
  informations du projet, et refuse si elles ont été modifiées depuis leur chargement
  (*Recharger* montre les plus récentes). Pour le projet partagé ouvert, *Propriétés…* est
  la boîte de dialogue habituelle, qui verrouille les données du projet.
- *Copier le projet…* (pour le projet sélectionné ou le projet partagé ouvert) montre
  aussitôt les propriétés de la copie : un nom libre sur le serveur (*… copie*) et le titre,
  la description, l'entreprise, l'auteur, la date, la révision et les propriétés de projet
  de l'original, tous modifiables. Rien n'existe sur le serveur avant *Créer et ouvrir* ;
  *Annuler* ne laisse rien derrière lui, et une copie refusée (nom déjà pris, pas de droit
  de la créer) garde les saisies à corriger. La copie est faite sur le serveur à partir de
  l'état enregistré : les modifications que personne n'a encore enregistrées, aussi celles
  des autres utilisateurs, n'en font pas partie. Avec des modifications non enregistrées
  dans le projet ouvert, l'éditeur demande d'abord s'il faut les enregistrer. Si l'original
  a été modifié sur le serveur après que la boîte de dialogue l'a chargé, la copie n'est
  faite qu'après une nouvelle confirmation. Les sessions, les verrous de pages et les
  droits d'accès ne sont pas copiés : la copie est ouverte dans une nouvelle session et un
  utilisateur connecté la gère comme tout projet qu'il crée. Les commentaires, les
  annotations, les approbations (avec leurs signatures), les demandes de modification, les
  révisions diffusées et les références sont laissés de côté, sauf si *Copier aussi les
  révisions, les références et les notes de révision* est coché ; les approbateurs du flux
  de diffusion restent, comme tous les paramètres. Le serveur consigne qui a créé la copie,
  quand, et à partir de quel projet et de quelle révision.
- *Fichier › Paramètres › Au démarrage* décide de ce que l'éditeur ouvre à son démarrage :
  la page d'accueil avec les fichiers locaux et les projets du serveur, le dernier projet
  (un projet du serveur est rouvert sur le serveur), la liste des projets du serveur ou un
  fichier ou un projet du serveur choisi. Si le serveur ne peut pas ouvrir le projet, une
  boîte de dialogue propose de réessayer, de se connecter, de choisir un autre projet ou de
  continuer avec la copie locale ; les modifications non enregistrées de la copie sont
  d'abord enregistrées ou abandonnées, jamais écrasées en silence.
- API REST (`/api/v1`) : `GET /info` (nom, version et nombre de projets que l'appelant peut
  voir ; répond aussi sans jeton), `GET|POST /projects` (`POST` a besoin du
  `project`), `GET|PUT|DELETE /projects/{name}`,
  `POST /projects/{name}/open|status|lock|release|close`, `GET|PUT /projects/{name}/access`,
  `GET|PUT /projects/{name}/properties` (les informations du projet avec la révision, qui a
  verrouillé les données du projet, les droits de l'appelant, qui a créé et modifié en dernier le projet et de quoi il a été copié ; `PUT`
  prend `user`, `revision` et `info`, compte comme la dernière modification, a besoin du droit de modifier les données du projet, répond `409`
  tant que les données du projet sont verrouillées et `412` si les informations ont changé
  depuis `revision`),
  `POST /projects/{name}/copy` (`name` et `user` de la copie, facultativement la
  `revision` que la source doit encore avoir – `412` sinon –, ses `info` et `records` ;
  a besoin du droit de voir la source et de créer le nouveau nom, répond comme l'ouverture
  d'un projet),
  `GET|PUT /projects/{name}/inspection?lang=de` (le rapport de vérification tel qu'il est
  enregistré : les données du rapport, l'examen visuel, les circuits avec les données du
  schéma, leurs limites, résultats et verdicts, et le verdict du rapport,
  `InspectionSheet` ; `PUT` prend `user`, `circuit` ou `source` (les deux vides : les
  données du rapport et l'examen visuel comme `check.<item>`), la `rev` de son enregistrement telle qu'elle a été chargée et les
  `values` par champ, a besoin du droit d'enregistrer des révisions ou de modifier, répond
  `409` si l'enregistrement a changé depuis `rev` et compte comme une modification du
  projet),
  `GET /projects/{name}/inspection.xlsx?lang=de` (le rapport comme classeur Excel : un
  résumé, les circuits et les sources),
  `POST /login` (`user` et `password` ; `name` est pris pour `user`, comme sur le serveur
  d'articles), `POST /logout`, `GET /users`, `PUT|DELETE /users/{name}`, `GET|PUT /access`,
  `GET /directories`, `PUT|DELETE /directories/{id}`, `POST /directories/{id}/test`
  (administrateurs ; les mots de passe des comptes de service ne sont jamais renvoyés),
  `GET /sessions?limit=` (administrateurs : les sessions ouvertes et les dernières
  terminées, `SessionList`) (voir
  `openecraft_core::collab`). `GET /info` indique aussi le `user` connecté (le jeton du
  serveur se connecte comme `admin`), ce que l'appelant peut faire (`access`) et si la page
  web est activée (`web`). Avec `--web` aussi `GET /texts?lang=de` (les textes de la page),
  `GET /projects/{name}/view?lang=de` (pages, sections, structures, appareils, verrous et
  droits de l'appelant, de `openecraft_view`) et `GET /projects/{name}/pages/{id}/svg`
  (une page en SVG) ; les deux ont besoin du droit de voir le projet et sont dessinés à
  partir d'une seule analyse par projet et révision (les 8 derniers projets sont gardés).

## Serveur de projets public

Le projet exploite un serveur de projets public à l'adresse
[project-server.openecraft.org](https://project-server.openecraft.org) avec sa page web
activée. Y travailler demande un compte : connectez-vous sur la page web pour voir les
projets que vous pouvez ouvrir, saisissez l'URL du serveur
`https://project-server.openecraft.org` dans le volet *Serveur de projets* de l'éditeur et
connectez-vous-y avec le même compte. Il montre comment plusieurs utilisateurs travaillent
sur un même projet ; les projets d'une entreprise ont leur place sur un serveur de projets
à elle (ci-dessus), derrière ses propres paramètres d'accès.

## Mentions légales, avertissement et notice de confidentialité

Sous la page web (`--web`) se trouvent les liens *Confidentialité* et, lorsque l'exploitant
a un nom, *Mentions légales* (en allemand *Datenschutz* et *Impressum*), qui ouvrent chacun
une boîte de dialogue, ainsi que les avertissements d'OpenECraft (aucune garantie, les noms
de produits qu'il mentionne appartiennent à leurs propriétaires). Ils sont là sans
connexion, comme pour le [site web](../website.md#legal-notice), dont les champs et les
variables sont les mêmes :

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
mémorisées pendant un court moment ; qui travaille sur quel projet et quand (sessions, verrous, les dernières sessions terminées) et qui a modifié un projet en dernier. Ce que seul l'exploitant sait (qui héberge le
serveur, ce qu'un proxy inverse journalise et pendant combien de temps, à qui s'adresser,
comme le demande l'article 13 du RGPD) se règle dans `privacy` et s'affiche en dessous.
Gardez les textes exacts : une modification qui stocke ou envoie autre chose les modifie
aussi (`renderLegal` de `common.js`, `privacy()` du script de la page). Qu'un serveur ait
besoin de mentions légales, et ce qu'elles doivent dire, dépend de l'exploitant et du
pays : OpenECraft ne montre que ce qui est réglé.
