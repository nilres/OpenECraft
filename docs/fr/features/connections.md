# Fils, câbles, potentiels et numérotation

Comment les liaisons reçoivent leur câble, leur couleur, leur section et leur numéro, et comment les appareils, les pages et les fils sont numérotés.

## Définitions de potentiel

**Définitions de potentiel** (*Liaisons › Potentiels…*) : type (L, N, PE, DC +/-, signal),
tension, couleur de fil (codes IEC 60757) et section par potentiel ; préréglages
IEC 60204-1 (machines, nouveaux projets) et IEC 60445 (couleurs des phases). Les fils
sans couleur / section propre les prennent du potentiel de leur liaison
(aussi derrière des bornes), sinon de la valeur par défaut du projet pour les liaisons
sans potentiel défini ; les conducteurs de câble gardent les valeurs du câble. Les
propriétés du fil montrent la valeur automatique et d'où elle vient ; les listes de fils,
le câblage de l'armoire, le remplissage des goulottes, les embouts, les faisceaux et le
marquage l'utilisent. La vérification signale le GNYE hors des conducteurs de protection
et le PE dans d'autres couleurs, les valeurs manuelles qui diffèrent de la définition et
les potentiels sans définition. *Affichage › Couleurs des fils* (enregistré avec le
projet) dessine les fils dans leur couleur sur le schéma et dans l'export PDF / SVG ; les
fils bicolores tels que GNYE sont dessinés en pointillé dans leur seconde couleur.

## Câbles

Définitions de câble (repère, type, conducteurs, section, longueur, article du
groupe de produits *Câbles*). Les fils sont affectés à des conducteurs (plusieurs fils à
la fois reçoivent les conducteurs libres dans l'ordre, les fils d'une même liaison
partagent un conducteur) ; les conducteurs et le repère du câble sont montrés sur le
schéma, avec un trait en travers du fil là où une ligne de définition de câble du câble
le croise (sinon à 30 % du fil). La désignation du conducteur et le repère du câble d'un
fil sélectionné peuvent être glissés en place et gardent leur décalage à partir de là ;
*Réinitialiser* dans les propriétés du fil (ou `update_wire` avec un `core_offset` /
`cable_offset` vide) les place de nouveau automatiquement. Rapports vue d'ensemble des
câbles et schéma de câbles, câbles dans la nomenclature selon leur longueur,
vérifications des câbles inconnus, des conducteurs utilisés deux fois ou absents du
câble.

## Définitions de liaison et de câble

Les textes des conducteurs de câble se tiennent automatiquement à l'écart de la ligne de
définition et de leur fil. La mise en page vérifie les zones de texte estimées, les
autres textes de câble, les fils, les graphiques de symboles voisins, les textes de
broche / de propriété / de repère et les textes libres. Elle essaie un ensemble borné de
positions voisines dans un ordre déterministe ; elle ne garantit pas que chaque dessin
chargé ait une place libre. Les conflits restants ont un contour rouge dans l'éditeur.
Les décalages manuels existants gardent leur signification d'origine et ne sont jamais
réinitialisés en silence.

Ce changement de rendu s'applique aussi aux textes automatiques des projets existants,
y compris les pages validées et les exports PDF/SVG. Valider une page protège ses données
contre les modifications ; cela ne fige pas le moteur de rendu. Les décalages manuels
gardent leurs positions.
Pour une trace inchangée d'une validation antérieure, conservez le PDF exporté lors de
la validation.
L'éditeur calcule une mise en page par image et utilise les zones de texte réellement
dessinées pour la sélection comme pour le glissement. Les exports de plusieurs pages
réutilisent leur index de projet.

Survolez un texte de conducteur ou le repère de câble à côté d'un fil pour voir sa cible
de glissement. Faites-le glisser directement pour ne déplacer que le texte, sans d'abord
sélectionner le fil ; cela fonctionne aussi avec une sélection multiple préalable. Les
extrémités de fil restent des poignées de géométrie séparées.
Les pages validées et les projets ouverts en lecture seule ne peuvent pas être modifiés
de cette façon.

Sélectionnez une définition de câble et choisissez **Aligner les textes des
conducteurs…** dans son menu contextuel, dans le ruban Liaisons ou dans ses propriétés.
Choisissez au-dessus/à gauche ou au-dessous/à droite et une distance minimale. La page
montre un aperçu ; rien ne change avant **Appliquer**.
Les textes des conducteurs croisés de ce câble sur cette page sont alignés ensemble, avec
une distance commune augmentée au besoin pour éviter les obstacles. L'application
remplace délibérément leurs positions manuelles en une seule étape d'annulation. Les
positions des repères de câble, la géométrie des fils, les affectations câble/conducteur
et les autres pages restent inchangées. Si aucun placement dégagé n'est trouvé, changez
les options ou placez les textes un par un. Les positions enregistrées survivent à
l'enregistrement et à la réouverture et utilisent le même chemin de rendu pour l'export
SVG/PDF.

Pour renommer un câble, cliquez avec le bouton droit sur sa ligne de définition ou son
texte et choisissez **Renommer le câble…**. La même action se trouve dans le navigateur
de câbles et dans les propriétés ouvertes avec F2 ou un double-clic. La boîte de dialogue
sélectionne le repère entier : saisissez le nouveau repère et appuyez sur Entrée, ou
appuyez sur Échap pour annuler. Les références sur toutes les pages suivent le
renommage, qui peut être annulé en une étape. Les repères en double et les modifications
de pages validées sont refusés. Les projets partagés acquièrent les verrous du projet et
des pages concernées avant d'appliquer la modification. L'annulation ne libère que les
verrous nouvellement demandés par la boîte de dialogue, une fois leur demande terminée ;
les verrous qui protègent d'autres modifications locales restent.
L'action se trouve aussi dans *Liaisons › Câbles*. L'automatisation peut utiliser
`rename_cable` avec `cable` (repère complet actuel) et `tag` (nouveau repère complet).
Les serveurs de projets plus anciens sont pris en charge : si le point d'accès de
l'instantané de session est absent, l'éditeur combine l'ancien instantané avec l'état des
verrous de la session à la même révision. Une révision plus récente pendant le chargement
entraîne une nouvelle tentative au lieu d'un mélange d'états.

Le tracé d'une définition de câble donne un aperçu des fils croisés et des conducteurs
libres. Ses propriétés montrent les nombres d'affectés/non affectés et l'ordre
d'affectation, en commençant au premier point de la ligne. Inverser la ligne inverse
l'ordre pour les conducteurs libres ; les affectations existantes gardent leur
conducteur. Les conducteurs déjà utilisés ailleurs dans le projet ne sont pas
disponibles. Le texte du câble (par exemple `4x1.5`) décrit le câble saisi, pas le nombre
de croisements.

S'il y a trop peu de conducteurs libres ou si un fil croisé a déjà une affectation
contradictoire, **aucun des fils en attente n'est affecté**. La ligne reste dans le plan
avec un avertissement de vérification et une indication de correction dans ses
propriétés. Modifiez la liste des conducteurs ou sélectionnez un autre câble, puis
choisissez **Affecter les fils croisés**. Les affectations existantes doivent être
modifiées explicitement dans les propriétés du fil ; un autre câble n'est jamais écrasé
en silence. La liste physique câble/conducteurs n'est jamais agrandie automatiquement
pour faire disparaître une incohérence.

Quand la création d'un nouveau câble est bloquée par une affectation contradictoire,
seule la ligne de définition est conservée : aucun câble inutilisé n'est créé.
Sélectionnez un câble dans ses propriétés après avoir corrigé le conflit. Une désignation
de conducteur importée sans câble est aussi un conflit ; effacez-la ou modifiez-la
explicitement dans les propriétés du fil.
Le bouton **Affecter les fils croisés** désactivé explique les manques et les conflits
au survol. Les aperçus interactifs réutilisent l'index d'affectation de la session ;
l'application vérifie toujours de nouveau le projet actuel.

L'automatisation utilise `preview_cable_definition` avec `page`, deux `points` et un
`cable` facultatif pour un aperçu en lecture seule. `apply_cable_definition` prend `page`
et l'`id` de la définition, et rapporte `changed` ainsi que l'`assignment` qui en résulte.
`add_cable_definition` garde ses champs de réponse existants et ajoute `assignment`,
avec les nombres, les conducteurs libres, les conflits et les candidats fil/conducteur
ordonnés.

Un point de définition de liaison sur un fil fixe
le numéro, la couleur et la section de toute la liaison et le conducteur de son fil ; une
ligne de définition de câble tracée à travers des fils en fait des conducteurs d'un câble
(nouveau câble à conducteurs numérotés, ou un câble choisi). La vérification signale les
points à côté des fils, les points contradictoires ou non appliqués, et les lignes de
câbles inconnus.

## Symboles de liaison : pièces en T, coins, croisements et ponts

Trois appareils sur un même potentiel peuvent être câblés A–B et A–C ou A–B et B–C ; la
liste des réseaux seule ne permet pas de les distinguer. Une **pièce en T** (*Insertion ›
Liaisons › Pièce en T*, placée là où trois fils se rencontrent) enregistre ce choix avec
le projet, dans l'esprit des symboles de liaison d'EPLAN : ses trois accès sont le pied
(la direction de la pièce : gauche, droite, haut ou bas) et les deux bras de la barre en
travers. Quatre variantes par direction désignent l'accès *source* et l'ordre des deux
*cibles* :

| Variante | Source | Première cible | Seconde cible |
| --- | --- | --- | --- |
| 1 | bras gauche / supérieur | l'autre bras (tout droit) | le pied |
| 2 | bras droit / inférieur | l'autre bras (tout droit) | le pied |
| 3 | pied | bras gauche / supérieur | bras droit / inférieur |
| 4 | pied | bras droit / inférieur | bras gauche / supérieur |

Les deux fils physiques de la liaison vont de l'appareil à la source à l'appareil à la
première cible et à l'appareil à la seconde cible. L'appareil d'un accès est le premier
point de connexion le long de son fil, aussi à travers des points d'interruption ; là où
le fil atteint une autre pièce en T, celle-ci transmet ses cibles (quand le fil arrive à
sa source) ou sa source. La liste des liaisons, les schémas de liaisons, le câblage de
l'armoire, les faisceaux et tout autre consommateur des liaisons utilisent ce résultat.
Les liaisons sans pièce en T gardent l'ordre du schéma comme auparavant, et une pièce qui
ne convient pas à ses fils (pas trois fils, aucun dans sa direction, un point de
connexion à cet endroit) est ignorée et signalée par la vérification *Symboles de
liaison*, de même que deux symboles sur un point et une liaison que ses symboles câblent
en boucle (plus de fils que ses appareils n'en demandent, par exemple les sources de deux
pièces en T qui se font face de part et d'autre d'un fil).

Le symbole est une courte diagonale de la source à la seconde cible (la première cible
est tout droit) ; *Représentation › Point* dessine à la place le point de connexion
habituel et garde le cheminement. Le panneau des propriétés montre les quatre variantes
sous forme d'aperçus avec les cibles numérotées, la direction, et les fils de la liaison.
Faire pivoter ou refléter le dessin tourne la pièce avec ses fils. Les symboles de
liaison suivent leurs fils : quand un appareil ou une dérivation se déplace le long du
rail, la pièce en T glisse avec la dérivation ; quand le rail se déplace, ses dérivations
s'étirent et la pièce reste dessus ; quand tout le point de jonction se déplace, la pièce
se déplace avec lui.

Tant qu'un outil de symbole de liaison est actif, le panneau Propriétés (réaffiché par
l'outil quand il était fermé) contient les cheminements du prochain symbole parmi
lesquels choisir, Tab les parcourt et R (Pivoter) tourne le symbole. Un symbole qui vient
d'être placé n'est pas sélectionné, ces choix n'agissent donc que sur les symboles encore
à placer ; un symbole placé se modifie en le sélectionnant. Là où trois fils se
rencontrent déjà, une pièce en T prend sa direction de ceux-ci et du cheminement choisi.

Un **coin** (*Insertion › Liaisons › Coin*) est le deuxième symbole de liaison : deux
fils qui se rencontrent à angle droit, avec quatre orientations (Tab ou R choisit la
suivante pendant le placement). Il
ne change aucun cheminement ; son utilité est le câblage automatique : avec *Connexion
auto* activé, placer un coin (ou une pièce en T) trace des fils depuis les points de
connexion qui font face tout droit à ses bras libres (aussi depuis une pièce en T sur un
fil qui le traverse), et un symbole placé ou déplacé de sorte qu'un point de connexion
fasse face à un bras libre y est câblé. Ainsi, un coin au point de rencontre de deux
liaisons les câble toutes les deux. Un coin qui convient à ses fils n'est dessiné que
comme le coude ; un coin sans ses deux fils montre ses bras dans la couleur d'erreur et
est signalé par la vérification.

Un **croisement** (*Insertion › Liaisons › Croisement*) relie les deux fils qui se
croisent en son point, lesquels restent non reliés sans lui. Ce sont deux pièces en T
réunies : chaque extrémité de sa ligne droite (verticale ou horizontale) est câblée tout
droit vers l'autre extrémité d'abord et en travers vers un bras ensuite ; les deux
variantes décident quelle extrémité prend le bras gauche / supérieur. Dans l'exemple
d'EPLAN, un croisement vertical de variante 2 mène une liaison venant du haut d'abord
vers le bas puis vers la gauche, et une liaison venant du bas d'abord vers le haut puis
vers la droite. Un **pont** (*Insertion › Liaisons › Pont*) a lui aussi quatre fils et
forme une étoile du côté d'où vient la liaison vers les trois autres : le côté opposé
d'abord, puis le gauche ou le supérieur, puis le droit ou l'inférieur. Les deux peuvent
être dessinés comme un point, gardent alors leur cheminement, et tournent avec le dessin.
Chaque symbole est une liste de liens entre ses bras ; là où un fil atteint un autre
symbole, ce symbole transmet les appareils des bras liés à celui par lequel le fil
arrive.

Commandes : `add_t_junction` (direction d'après les fils présents quand elle n'est pas
donnée), `add_corner` (bras d'après les fils présents, par exemple `down left`),
`add_crossing` (`line` vertical ou horizontal, `variant` 1 ou 2), `add_bridge`
(`direction` d'où vient la liaison), toutes avec `autoconnect` pour tracer les fils vers
les points de connexion en face, `update_connection_symbol`,
`list_connection_symbols` (type, cheminement, liens et les fils de chacun).
`autoconnect` prend aussi les identifiants de symboles de liaison.

## Numéros de fils

Un numéro par liaison (tous ses segments de fil ; les deux côtés d'une
borne sont des liaisons distinctes). Numérotation automatique : séquentielle,
page/colonne (`3.4.1`) ou par nom de potentiel (`L1`, `+24V`), avec préfixe et valeur de
départ ; les numéros fixés à la main sont conservés. Montrés sur le schéma, dans la liste
des liaisons et la liste de fils de l'armoire ; vérifications des numéros contradictoires
et en double.

## Numérotation

**Numérotation** (dans l'esprit de la numérotation hors ligne d'EPLAN) : *Appareils ›
Numérotation des appareils…* renumérote les repères d'appareil par structure et lettre
d'identification dans l'ordre du schéma (page, colonne, de haut en bas) selon un format :
`{letter}{n}` (`-K1`),
`{letter}{page}.{n}` (`-K3.1`), `{letter}{page}{column}` (`-K34`) ou style NFPA
`{page}{column}{letter}` (`-34K`), avec valeur de départ, pas, lettres et portée, et un
aperçu des nouveaux repères ; tout ce qui se réfère à un repère suit (articles, borniers,
connecteurs, données API, implantation d'armoire, faisceaux) ; les repères d'appareils
qui ne sont pas placés (liste des appareils, implantation d'armoire) ne sont pas donnés à
d'autres. *Accueil › Renuméroter les pages…* numérote les pages avec départ et pas, en
recommençant par structure et en gardant les sous-pages (`3.a` ; *Accueil › Nouvelle page
› Nouvelle sous-page* insère `3.1`, `3.2`, … après la page courante). Les sections de
pages contiennent des pages et des sous-sections comme des répertoires (*Accueil ›
Nouvelle section*, faites glisser des pages dessus dans le navigateur *Pages*) ; les noms
des pages portent les numéros de section (`2.3.1`), suivent quand une section est
renumérotée ou déplacée, et la renumérotation compte par section. La numérotation des
fils reçoit un schéma *Format* avec des espaces réservés (`{page}`, `{column}`, `{row}`,
`{potential}`, `{kind}`, `{color}`, `{cross_section}`, compteur `{n}` compté par
valeur du reste), le nombre de chiffres du compteur et le pas, et les types de potentiel
qui ne reçoivent pas de numéro (par exemple PE). Commandes : `renumber_devices`,
`renumber_pages`, `set_wire_numbering`.
