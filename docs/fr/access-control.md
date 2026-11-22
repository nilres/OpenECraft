# Contrôle d'accès des serveurs

Le serveur d'articles et le serveur de projets partagent un même modèle d'accès : les
utilisateurs se connectent avec un compte local ou avec un annuaire (Active Directory,
OpenLDAP ou un autre serveur LDAP), et ce qu'ils peuvent faire vient de **rôles** qui leur
sont donnés, à eux ou à leurs **groupes**, par des **attributions**, par projet (serveur de
projets) ou par fabricant (serveur d'articles).

- Règles et leur évaluation : `openecraft_parts::access` (partagé avec les clients)
- Annuaires et fichier de paramètres : `openecraft-auth` (`directory.rs`, `config.rs`)
- Droits et rôles intégrés : `openecraft_core::collab::{ProjectRight, ProjectRole}` et
  `openecraft_parts::portal::{PartsRight, Role}`

## Connexions mémorisées au serveur de projets

Dans le volet **Serveur de projets** de l'éditeur, **Rester connecté** est un réglage
explicite et facultatif pour le serveur et le compte sélectionnés. Sous Windows, seul le
jeton de session du compte va dans le Gestionnaire d'identification de Windows. Les mots de
passe ne sont jamais conservés. Les paramètres de l'application ne contiennent que les
métadonnées du serveur/du compte et le choix explicite, indépendamment pour chaque compte,
de sorte qu'une autre fenêtre de l'éditeur ne peut pas désactiver la connexion d'un autre
compte. L'identité complète du serveur comprend le schéma, l'hôte, le port effectif et le
chemin de base ; les identifiants ne passent jamais à un autre serveur. HTTP et HTTPS
restent distincts, et cette fonction ne modifie ni ne chiffre le transport existant.

Au démarrage, `GET /api/v1/session` valide le jeton de compte mémorisé et renvoie
`CurrentAccount { user, access }`, avec les droits actuels et sans jeton ni mot de passe.
Les jetons racine, l'accès anonyme et les jetons de compte invalides ou expirés sont refusés avec
401. Cette requête ne renouvelle pas le jeton et ne touche pas aux verrous de pages. La
session de compte expire actuellement 30 jours après la connexion ; la session de projet
distincte, inactive après 90 secondes, n'est pas concernée. Les serveurs plus anciens sans
ce point d'accès demandent une connexion manuelle ; leur 404 ne supprime pas un identifiant
mémorisé peut-être valide. Les échecs de connexion au serveur conservent aussi le choix
explicite pour un nouvel essai ultérieur.

Si un jeton de compte expire pendant qu'un projet partagé est ouvert, se reconnecter avec
le même serveur et le même compte poursuit ce projet par le processus de récupération de
session. Les modifications non enregistrées et l'historique d'annulation restent
disponibles ; la connexion ne les marque pas comme enregistrées et ne remplace pas le
projet par une nouvelle copie.

**Se déconnecter** retire l'identifiant mémorisé et demande au serveur de révoquer le
jeton de compte une fois que les enregistrements et les fermetures de projets en attente
ont reçu leur réponse.
**Changer d'utilisateur…** libère la session de projet et efface le nom d'utilisateur
sélectionné, mais garde la connexion mémorisée de ce compte pour y revenir. Un projet
partagé ouvert et son historique d'annulation restent comme copie de récupération locale, y
compris les modifications non enregistrées. La déconnexion locale fonctionne hors ligne ;
le volet signale à part si la session du serveur n'a pas pu être terminée ou si le stockage
protégé n'a pas pu être modifié.
Les opérations du système d'exploitation s'exécutent dans l'ordre, y compris la suppression
après un enregistrement déjà en cours ; la fermeture normale de l'application attend leurs
résultats. Les réponses d'une sélection précédente de serveur/de compte ne rétablissent
jamais son identité et ne remplacent jamais le projet ouvert.
Une confirmation d'enregistrement retardée appartient toujours au même projet conservé
pendant la réauthentification ; elle fait avancer la révision enregistrée sans changer la
nouvelle connexion.
La révocation en arrière-plan ne désactive pas la connexion suivante pendant l'attente d'un
serveur.

Le nettoyage d'un jeton invalide compare le jeton rejeté avant de le supprimer. Il n'écrit
jamais un nouveau refus : une autre fenêtre peut déjà avoir mémorisé une connexion plus
récente. Les connexions par annuaire gardent leur choix explicite sous le nom de compte
confirmé par le serveur, y compris lorsque le nom saisi était qualifié par un domaine ou
était un nom d'utilisateur principal.

Le navigateur, macOS et Linux n'offrent actuellement qu'une connexion limitée à la session.
Leur case à cocher est désactivée avec une explication ; il n'y a pas de repli par
jeton/mot de passe dans le stockage local, IndexedDB, les paramètres ordinaires ou les
fichiers de projet. La prise en charge de Windows réutilise le service d'identifiants
existant du système d'exploitation, utilisé aussi pour les PAT EPLAN, avec un espace de
noms séparé. Ces fonctions de session/de plateforme ne modifient ni les commandes de projet
ni l'API des plugins.

Le contrôle isolé en conditions réelles est `scripts/test-project-sign-in.ps1`
(PowerShell 7, Windows). Il utilise un serveur de projets jetable en boucle locale, des
comptes synthétiques, son propre dossier de métadonnées et des clés d'identifiants uniques,
que le test retire ensuite.

## Droits et rôles intégrés

Serveur de projets (la portée d'une attribution est un nom de projet) :

| Droit | Permet |
| --- | --- |
| `view` | voir le projet dans la liste, l'ouvrir en lecture seule |
| `review` | enregistrer des notes de révision (commentaires, annotations, approbations) sans verrous |
| `edit_pages` | verrouiller et modifier des pages |
| `edit_project` | verrouiller et modifier les données du projet (liste des pages, appareils, articles, paramètres) |
| `create` | créer des projets (la portée est le nouveau nom) |
| `delete` | supprimer un projet que personne n'a ouvert |
| `manage_access` | modifier les attributions sur le projet |
| `administer` | utilisateurs, rôles et toutes les attributions (seulement par des attributions sur `*`) |

| Rôle | Droits |
| --- | --- |
| `viewer` | view |
| `reviewer` | view, review |
| `editor` | view, review, edit_pages, edit_project, create |
| `manager` | editor + delete, manage_access |
| `admin` | tout |

Un utilisateur connecté qui crée un projet en devient le `manager` (une attribution sur le
nouveau projet) : les propriétaires de projets gèrent ainsi qui travaille sur leurs projets.

Serveur d'articles (la portée d'une attribution est un fabricant, ou un catalogue comme `catalog:<id>`) :

| Droit | Permet |
| --- | --- |
| `read` | parcourir, rechercher, télécharger des fichiers, favoris (tout le serveur) |
| `edit_parts` | créer et modifier des articles et leurs fichiers, importer |
| `delete_parts` | supprimer des articles |
| `edit_catalogs` | créer, modifier et supprimer des catalogues |
| `administer` | utilisateurs, rôles et attributions (tout le serveur) |

Rôles intégrés : `reader` (read), `manufacturer` et `editor` (read, edit_parts,
delete_parts, edit_catalogs), `admin` (tout). Le rôle propre d'un compte fonctionne toujours
comme auparavant : un utilisateur fabricant n'a le rôle de fabricant que sur son fabricant.

Les droits sur un article ou un catalogue sont ceux sur son fabricant réunis à ceux sur son
catalogue (`catalog:siemens`) : une attribution sur un catalogue permet à des utilisateurs
et à des groupes de modifier le catalogue et ses articles quel que soit leur fabricant, et
une attribution de refus sur l'un ou l'autre retire le droit. Les administrateurs règlent
les attributions d'un catalogue dans *Administration › Catalogues › Autorisations* (`GET` /
`PUT /api/v1/catalogs/{id}/grants`) ; la lecture reste un droit de tout le serveur.

## Comment les droits sont résolus

1. Le rôle intégré du compte (serveur de projets : sur ses projets ; aucun pour les
   utilisateurs venus d'un annuaire).
2. Chaque attribution de la politique dont le sujet correspond à l'utilisateur
   (`{"user": "anna"}`) ou à l'un de ses groupes (`{"group": "corp:Engineering"}` d'un
   annuaire, `{"group": "Wiring"}` défini sur le serveur d'articles). Les noms et les
   portées acceptent les caractères génériques `*` et `?` et ne tiennent pas compte de la
   casse ; `{"user": "*"}` désigne tout utilisateur connecté.
3. Le rôle des requêtes anonymes (`anonymous`), que les utilisateurs connectés ont aussi.
4. **Les attributions de refus l'emportent :** les droits d'une attribution avec
   `"deny": true` sont retirés sur sa portée, quoi que donnent les autres attributions.

Les droits à l'échelle du serveur (`administer`, et `read` sur le serveur d'articles) ne
viennent que d'attributions sur `*`. Le jeton du serveur (`--token`) garde un accès complet.

## Le fichier de paramètres

`access.json` dans le répertoire de données du serveur de projets, `<data>.access.json` à
côté de la base de données d'articles (tous deux modifiables avec `--access <file>`). Un
fichier manquant signifie : pas d'annuaires, pas de rôles définis ni d'attributions ; sur le
serveur d'articles, les requêtes anonymes peuvent lire (`"anonymous": "reader"`). Les
administrateurs modifient les rôles, les attributions et `anonymous` par l'API
(`GET|PUT /api/v1/access`), qui réécrit le fichier, et les annuaires par
`/api/v1/directories` (`GET`, `PUT|DELETE /{id}`, `POST /{id}/test`). Les pages web le font
aussi : celle du serveur d'articles (*Administration › Droits d'accès*, qui les exporte et
les restaure aussi) et celle du serveur de projets lorsqu'il fonctionne avec `--web`
(*Administration › Droits d'accès* et *Annuaires*).

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

Sur le serveur d'articles, `"registration": true` permet aux utilisateurs de s'inscrire
eux-mêmes comme lecteurs (activé dans *Administration › Droits d'accès* ; l'option de
démarrage `--registration` a la priorité, voir [docs/parts-server.md](parts-server.md)) ;
leurs comptes attendent l'approbation d'un administrateur, sauf si
`"registration_approval": false`.
`"dashboard": false` masque le tableau de bord de la page web (*Administration ›
Vue d'ensemble* ; l'option de démarrage `--dashboard` a la priorité).

### Groupes

Le serveur d'articles garde des groupes d'utilisateurs dans `groups` (*Administration ›
Groupes*, `GET /api/v1/groups`, `PUT` / `DELETE /api/v1/groups/{name}`) :

```json
"groups": [{ "name": "Wiring", "description": "Panel shop", "members": ["anna", "bob"] }]
```

Les membres sont des noms d'utilisateur, des utilisateurs locaux comme des utilisateurs
d'un annuaire (par le nom avec lequel ils se connectent). Les attributions à
`{"group": "Wiring"}` s'appliquent aux membres à côté des groupes de leur annuaire ; les
noms de groupe ne peuvent pas contenir `:`, qui sépare les groupes d'annuaire.
Les groupes d'un utilisateur (*Utilisateurs*, `groups` de `PUT /api/v1/users/{name}`) sont
des appartenances : les enregistrer ajoute l'utilisateur aux groupes nommés (les nouveaux
noms deviennent des groupes) et le retire des autres. Les groupes que des comptes locaux
nommaient auparavant sont transformés en groupes du serveur au démarrage. Supprimer un
groupe supprime les attributions qui lui sont faites, supprimer un utilisateur ses
appartenances ; les noms de membres ne peuvent pas être inscrits, et une modification qui
ne laisserait personne pour gérer les utilisateurs est refusée.

### Annuaires

Les administrateurs configurent aussi les annuaires dans *Administration › Annuaires* (`GET
/api/v1/directories`, `PUT` / `DELETE /api/v1/directories/{id}`), qui ne montre aucun mot
de passe (un mot de passe vide garde celui qui est stocké) et teste les réglages : *Tester*
se connecte et s'authentifie avec le compte de service ; avec un nom d'utilisateur et un mot
de passe, il connecte l'utilisateur et montre ses groupes
(`POST /api/v1/directories/{id}/test`).

| Clé | Signification |
| --- | --- |
| `id` | nom court ; les groupes apparaissent comme `id:group`, les utilisateurs de l'annuaire le montrent |
| `kind` | `active_directory` (par défaut) ou `ldap` |
| `url` | `ldap://…` ou `ldaps://…` ; `starttls` met à niveau `ldap://` |
| `insecure_tls` | accepter n'importe quel certificat (tests seulement) ; sinon le magasin d'autorités de certification du système est utilisé |
| `bind_dn`, `bind_password` / `bind_password_env` | compte de service pour rechercher les utilisateurs |
| `user_bind` | sans compte de service : nom de liaison de l'utilisateur, `{user}@corp.example` ou `uid={user},ou=…` |
| `base_dn` | où les utilisateurs sont recherchés |
| `user_filter` | par défaut `(&(objectClass=user)(sAMAccountName={user}))` / `(&(objectClass=person)(uid={user}))` |
| `name_attribute` | nom du compte, par défaut `sAMAccountName` / `uid` |
| `nested_groups` | Active Directory : aussi les groupes de groupes (`LDAP_MATCHING_RULE_IN_CHAIN`) |
| `group_filter`, `group_base_dn` | rechercher les groupes (`{dn}`, `{user}`) au lieu de lire `memberOf` |
| `allowed_groups` | seuls les membres de ces groupes peuvent se connecter (vide : tout le monde) |
| `timeout_secs` | délai de connexion et de requête (10 par défaut) |

La connexion essaie d'abord les comptes locaux ; un nom sans compte local va aux annuaires
dans l'ordre (ou seulement à celui nommé dans le `provider` de la requête). Le premier
annuaire qui connaît l'utilisateur vérifie le mot de passe (une liaison en tant
qu'utilisateur ; les mots de passe vides sont refusés avant que l'annuaire soit interrogé)
et renvoie les groupes. Le serveur garde un compte pour l'utilisateur d'annuaire (pas de
mot de passe ; groupes actualisés à chaque connexion), de sorte que les administrateurs le
voient dans la liste des utilisateurs et que les jetons de session fonctionnent comme pour
les utilisateurs locaux.
Un nom qui appartient déjà à un compte local ne peut pas se connecter par un annuaire.

Les mots de passe des comptes locaux ont besoin d'au moins 8 caractères. Après 5 connexions
échouées d'un nom d'utilisateur (local ou d'annuaire) depuis une adresse de client, chaque
tentative suivante de ce nom depuis cette adresse attend, 1 s d'abord et deux fois plus
longtemps après chaque nouvel échec, jusqu'à 5 minutes ; après 30 connexions échouées d'un
client avec n'importe quels noms, ses tentatives attendent de la même façon. Les serveurs
répondent `429 Too Many Requests` pendant ce temps. D'autres ne peuvent pas bloquer un
utilisateur de cette façon ; une connexion réussie oublie les échecs du nom (non ceux du
client), et les échecs sont oubliés après une heure. Un nom sans compte met autant de temps
à être refusé qu'un mot de passe erroné, de sorte que les réponses ne disent pas quels noms
existent.
Les erreurs des deux serveurs arrivent comme `{"error": "…"}`, aussi pour les routes
inconnues et les requêtes qui ne s'analysent pas.

Une session (le jeton d'une connexion) prend fin à la déconnexion, lorsque le mot de passe
est modifié et 30 jours après la connexion ; un utilisateur en garde 20 au plus. Les
sessions stockées par des versions antérieures n'ont pas de début et comptent comme
terminées : leurs utilisateurs se connectent une fois de plus. Les requêtes qui modifient
quelque chose sont contrôlées avant la lecture de leur corps, de sorte que des appelants
inconnus ne peuvent pas faire lire de grands corps à un serveur ; la connexion (et la
recherche du serveur d'articles) acceptent 64 kB au plus.

Les deux serveurs envoient `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
`Referrer-Policy: no-referrer` et une politique de sécurité du contenu avec chaque réponse.
Les fichiers d'utilisateurs et d'accès (`users.json`, `access.json`, une base de données
d'articles JSON) sont écrits lisibles par leur seul propriétaire (Unix). Préférez
`bind_password_env` à `bind_password` (les serveurs avertissent d'un mot de passe dans le
fichier) et `OPENECRAFT_TOKEN` / `OPENECRAFT_PROJECT_TOKEN` ou le fichier de configuration
à `--token`, que les autres utilisateurs de la machine voient dans la liste des processus.
Aucun des deux serveurs ne parle TLS : placez un proxy inverse avec HTTPS devant un serveur
accessible par le réseau.

Les valeurs sont échappées pour les filtres et les DN LDAP. Gardez le compte de service en
lecture seule.

## Clients

- Éditeur, *Outils › Serveur de projets…* : montre les droits sur le projet ouvert ; les
  lecteurs et les réviseurs ne peuvent pas le modifier, les chefs de projet y modifient les
  attributions dans la section *Accès au projet*.
- Gestionnaire d'articles : l'éditeur d'utilisateurs montre l'annuaire et les groupes d'un
  utilisateur et modifie les groupes du serveur dont l'utilisateur est membre.
- Ligne de commande / MCP : `parts_access` (règle aussi `anonymous`), `parts_grant` (avec
  `deny`, ou `revoke` pour retirer des attributions), `parts_role` (définir, modifier,
  `delete`), `parts_registration` (`enabled`, `approval`), `parts_user_approve` et
  `parts_user_save` avec `groups`.
