# Éditeur de schémas

Pages, appareils et leur structure, renvois en direct, outils d'édition, navigateurs et enregistrement du projet. L'[interface utilisateur](../user-interface.md) liste les commandes et les touches.

![L'éditeur de schémas avec un circuit de puissance étoile-triangle, le potentiel L1 sélectionné mis en évidence, sa couleur de fil et sa section automatiques](../../images/schematic.png)

## Projets avec pages

Pages de schéma multifilaires (A3, 10 colonnes / 6 lignes, cartouche),
page de garde et pages de rapport générées. Navigateur de pages avec réorganisation.

## Conception orientée appareil

Les repères d'appareil (`-K1`, `-S3`, …) sont attribués automatiquement.
La fonction principale (bobine) et les fonctions auxiliaires (contacts) forment un seul
appareil. Sélectionnez une bobine et placez des contacts : ils lui sont affectés et
numérotés (13/14, 21/22, …).

## Boîtes d'appareil et points de connexion d'appareil

Alimentations, appareils de mesure et modules de commande sans symbole de bibliothèque :
*Insertion › Boîte d'appareil* dessine une boîte en pointillé de taille quelconque comme
nouvel appareil (`-A1`, avec son texte de fonction et ses articles comme tout autre
appareil ; sélectionnée, elle est redimensionnée aux poignées de ses coins et de ses
bords, les points d'un bord déplacé la suivent), *Insertion › Point de connexion d'appareil* place ses connexions un clic après
l'autre. Un point placé sur le contour d'une boîte appartient à l'appareil de la boîte,
part du bord sur lequel il se trouve et est numéroté à la suite (1, 2, 3 … ; `13` est
suivi de `14`) ; sa désignation (`L`, `N`, `PE`, `+24V`, …) se modifie dans ses
propriétés. Chaque point est une connexion à part entière, distincte des bornes : deux
points d'une boîte ne sont jamais reliés par la boîte, ils apparaissent comme `-A1:L`,
`-A1:N` dans les rapports des liaisons et de raccordement des appareils, et la
vérification signale une désignation utilisée deux fois sur un appareil. Les propriétés
d'un point nomment sa boîte d'appareil (une autre boîte l'affecte à cet appareil) ; un
point éloigné de sa boîte montre son repère d'appareil, sur sa boîte c'est la boîte qui
le montre. Sa désignation peut être glissée à n'importe quel endroit et alignée à gauche,
au centre ou à droite. Les propriétés d'une boîte fixent sa largeur et sa hauteur : les
points sur un bord qui se déplace l'accompagnent, leurs fils suivent. Le déplacement, la
rotation, la copie, le couper, la suppression et l'enregistrement comme macro prennent
une boîte avec ses points ; les copies deviennent un nouvel appareil avec leurs propres
points. Commandes : `add_device_box`, `update_device_box`,
`add_device_connections` (plusieurs à une distance donnée, désignations données ou
numérotées à la suite), `update_device_connection`, `list_device_boxes`.

Une boîte peut aussi prolonger un appareil placé ailleurs : donnez-lui le repère de
l'appareil et choisissez son *Rôle* dans les propriétés (`role` de `add_device_box` et
`update_device_box`). Une *Extension* ajoute des points de connexion à l'appareil, pour
un appareil trop grand pour une seule boîte ou un symbole de bibliothèque auquel il
manque des connexions. Ses points sont de nouvelles connexions, et une désignation que
l'appareil possède déjà est signalée comme utilisée deux fois. Une *Vue de connexions*
montre certains points de connexion que l'appareil possède déjà, par exemple seulement
l'alimentation d'une carte API sur la page de puissance ; cochez-les dans les propriétés
(`show_device_pins`) ou placez des points avec leurs désignations. Un point sur une vue
est le point de connexion qu'il montre : les fils qui y sont tracés appartiennent au
potentiel de ce point sur chaque page, les rapports listent la connexion une seule fois,
à partir du point propre de l'appareil, et la vérification signale une vue d'un point que
l'appareil n'a pas. Aucun des deux rôles n'est une seconde fonction principale. La boîte
montre un renvoi vers l'appareil, et la fonction principale de l'appareil vers ses
boîtes ; copiées sans l'appareil, ces boîtes lui restent attachées. Une boîte avec un
rôle mais sans appareil placé est signalée.

## Identifiants de structure (IEC 81346)

Les pages portent une fonction (`=A1`) et un emplacement
(`+CAB1`). Les appareils placés sur une page reçoivent sa structure (`=A1+CAB1-K1`) ; sur
la page, les repères et les renvois ne montrent que ce qui diffère de la structure de la
page (style EPLAN), les repères sont numérotés par structure, copier / coller déplace les
appareils dans la structure cible, modifier la structure d'une page peut entraîner ses
appareils. Le navigateur d'appareils regroupe par structure ; le cartouche montre le nom
complet de la page (`=A1+CAB1/3`).

Les **cadres de structure** (*Insertion › Cadre de structure*) donnent à une zone d'une
page sa propre fonction, son propre emplacement (ou lieu d'installation, produit) :
dessinez un rectangle (redimensionné aux poignées de ses coins et de ses bords quand il
est sélectionné) et réglez dans les propriétés les aspects qu'il remplace ; les
aspects vides sont hérités de la page ou d'un cadre englobant, les cadres imbriqués se
résolvent au plus intérieur. Un appareil appartient au cadre dans lequel se trouve le
point d'insertion de sa fonction principale : il reçoit la structure du cadre
(`=SYS+FIELD-S1` sur une page `=SYS+CAB1`), son repère sur la page omet ce que le cadre
montre, et son repère complet est celui qu'utilisent le navigateur, la numérotation, les
renvois et les rapports. Un aspect d'un repère égal à la structure autour du symbole est
hérité, tout autre est propre à l'appareil : déplacer la fonction principale dans un
cadre ou hors de celui-ci, ou déplacer, redimensionner, modifier ou supprimer le cadre,
renomme l'appareil (toutes ses fonctions) dans ses aspects hérités dans la même étape
d'annulation et conserve les siens ; un repère qui existe déjà est refusé et signalé au
lieu de fusionner deux appareils. Les propriétés du symbole montrent d'où vient chaque
aspect. Les cadres sont copiés, collés et enregistrés dans des macros avec leurs
appareils ; la vérification *Identifiants de structure* signale les cadres qui se
chevauchent sans que l'un soit à l'intérieur de l'autre, et les appareils à l'intérieur
d'un cadre dont le repère a une autre valeur pour un aspect que le cadre fixe.
Commandes `add_structure_box`, `update_structure_box`, `list_structure_boxes`.

## Renvois en direct

Les contacts montrent où se trouve leur bobine (`/3.4`), les bobines montrent une
image de contacts avec tous les contacts et leurs emplacements, les points d'interruption
renvoient à leurs homologues sur d'autres pages. *Liaisons › Renvois…* règle le format
(préfixe, séparateur, ligne : `/3.4`, `3-4`, `/3.4.B`), quand la structure de la page
cible est montrée (quand elle diffère, toujours, jamais), ce à quoi les points
d'interruption renvoient (tous les autres points, en étoile à partir du premier point, ou
par paires dans l'ordre du schéma ; la vérification signale un point sans partenaire) et
si les bobines dessinent une image de contacts. Commande :
`set_cross_references`.

Le nom et le renvoi d'un point d'interruption suivent la flèche : pour les points
horizontaux, le nom au-dessus et le renvoi au-dessous du conducteur, se terminant au
point de connexion, de sorte qu'ils restent à l'intérieur de la page à son bord ; pour
les points verticaux, à côté de la flèche (renvoi sous le nom). *Affichage*, dans les
propriétés, les place à la place au-dessus, au-dessous, à gauche ou à droite de la
flèche, pour tous les points sélectionnés à la fois (à droite d'une flèche vers la
droite : sur une ligne après sa pointe, pour des conducteurs distants de 4 mm) ; avec un
seul point sélectionné, chaque texte peut être glissé séparément sans déplacer le
symbole, et le bouton de réinitialisation revient au placement automatique. Les
positions sont enregistrées avec le projet et utilisées par les exports PDF, SVG et DXF ;
le renvoi reste un lien vers sa page.
Commande : `place_interruption_texts`.

Dans l'éditeur, Ctrl + clic suit un renvoi (la touche qui ouvre les liens,
*Fichier › Paramètres › Touches et gestes*) : la page nommée s'ouvre et, pour un point
d'interruption, son correspondant est sélectionné et centré. Tant que la touche est
enfoncée, le pointeur est une main au-dessus d'un renvoi.

## Connectivité / potentiels

Les fils, les jonctions en T (avec points de connexion), les bornes,
les points de connexion de potentiel (L1, N, PE, +24V, …) et les points d'interruption
sont évalués en réseaux à travers les pages. Sélectionner un fil met en évidence tout son
potentiel.

## Édition

Sélectionner / sélectionner par fenêtre / déplacer avec des fils qui suivent et restent
orthogonaux, pivoter, refléter, copier / couper / coller (par le presse-papiers, les
appareils reçoivent de nouveaux repères), annuler / rétablir, textes, accrochage à la
grille, déplacement de la vue et zoom. Un fil attaché à quelque chose qui se déplace
garde sa direction : un coin ou une jonction à son extrémité éloignée y glisse le long du
fil (une jonction sur un rail de potentiel reste sur le rail), sinon le fil reçoit un
décrochement ou un tracé contournant ce qui gêne, et un fil qui ne peut pas être rendu
orthogonal sans toucher une autre connexion reste en diagonale. Les points de définition
de liaison et les lignes de définition de câble restent sur leurs fils à moins d'être
déplacés eux-mêmes ; une ligne qui ne peut pas rester exactement à travers ses fils reste
où elle est. Avec *Retracer les connexions des éléments déplacés*
(*Liaisons › Croisements de fils…*, désactivé par défaut), un tel fil est retracé avec
les coins qui le suivent, jusqu'au prochain point de connexion, à la prochaine jonction
ou dérivation, sur le chemin de moindre poids : sa longueur en pas de grille, 1,2 par
coude et 1,5 par fil croisé ; une liaison sans un tel chemin est rendue orthogonale comme
auparavant.

Un fil déplacé reste lui aussi connecté : son extrémité sur un point de connexion d'un
symbole qui reste reçoit un nouveau tronçon à partir de ce point (un décrochement à la
place là où le tronçon toucherait un autre point de connexion ou un autre fil), son
extrémité sur un fil traversant glisse le long de ce fil, et les autres fils au point y
restent ; un fil droit garde sa direction et arrive là où il a été glissé, un fil tenu
aux deux extrémités ne se déplace pas le long de lui-même. Les dérivations d'un rail
déplacé ne le suivent que perpendiculairement au rail, elles gardent donc leur place
quand le rail est déplacé le long de lui-même. *Édition › Conserver les connexions*
(activé par défaut) désactive cela ainsi que l'étirement des fils attachés, de sorte que
la sélection se déplace seule et quitte ses connexions ; maintenir Alt au début d'un
glissement inverse le réglage pour ce glissement. Pour déconnecter une extrémité,
sélectionnez le fil et faites glisser la poignée carrée à cette extrémité : seule cette
extrémité se déplace, elle se connecte à ce sur quoi elle est déposée et le fil reste
orthogonal. Commandes :
`move_items` (`keep_connections`), `move_wire_end`.

Aides à l'édition : tailles de grille de 1, 2, 4, 8 et 16 mm (Affichage) ;
les points de connexion et les extrémités de fil hors de cette grille (dessins importés,
boîtes d'appareil de taille quelconque) ajoutent des lignes de grille invisibles qui les
traversent, de sorte que le pointeur s'y accroche là où elles sont plus proches que la
grille, que des fils peuvent partir de tels points et que l'on peut se connecter à de
tels fils ; un symbole en cours de placement ou de déplacement s'accroche à partir de ses
propres points de connexion (et un fil déplacé à partir de ses extrémités), ils arrivent
donc sur de telles lignes et la connexion auto les relie ;
un champ de coordonnées dans la barre d'outils (`x y` absolu ou `@dx dy` relatif au
dernier point, Entrée agit comme un clic de l'outil courant) ; *Accueil › Aligner* (bords
ou centres), *Répartir* (régulièrement, horizontalement ou verticalement), *Déplacer la
sélection de…* et *Copie multiple…* (n copies à une distance fixe, nouveaux repères et
numéros de bornes) ;
les extrémités de fil attachées suivent. Commandes : `align_items`, `distribute_items`,
`copy_items`.

## Textes

Textes libres avec rotation (0 … 359°, par exemple lisibles vers le haut le long d'un
bornier), alignement à gauche / centré / à droite, un ancrage vertical, retour à la ligne
à une largeur fixée, un facteur de largeur des lettres (0,8 : écriture étroite ; dessinée
ainsi à l'écran et dans les exports SVG, PDF et DXF / DWG), gras / italique / souligné,
une bordure et un arrière-plan opaque, et un hyperlien
(Ctrl + clic l'ouvre ; cliquable dans l'export PDF et SVG). Les **textes spéciaux**
montrent des propriétés du projet et de la page n'importe où sur une page : les
espaces réservés tels que `{page}`, `{project}` ou `{prop:Name}` (les mêmes que dans les
cartouches, Insertion › Texte spécial) sont remplis quand la page est dessinée. Un
**texte de fonction de chemin** (Insertion › Texte de fonction de chemin) est le texte de
fonction de toutes les fonctions de sa colonne qui n'en ont pas en propre ; la liste des
appareils, les plans de bornes, de connecteurs et de l'API l'utilisent. Les textes
tournent avec une sélection pivotée ; l'import DXF / DWG conserve la rotation et
l'alignement des textes. Ligne de commande /
MCP : `add_text`, `update_text`.

## Graphiques de schéma

Lignes, rectangles, cercles, ellipses, arcs, polylignes et
polygones (épaisseur, couleur, pointillé, rempli), cotations (alignées / horizontales /
verticales, échelle, unité), images PNG / JPEG (aussi dans l'export PDF et SVG).
**Calques** (navigateur › Calques) : visible, verrouillé, imprimé, couleur ; export DXF /
DWG par calque. *Exporter…* et *Importer…* du volet Calques transmettent les calques avec
leurs paramètres à d'autres projets sous forme de schéma de paramètres (le schéma complet
les contient aussi).

Un élément sélectionné montre des **poignées** qui changent sa taille ou sa forme quand on
les fait glisser, sur la grille : les coins et milieux des bords des rectangles, ellipses,
cadres de structure et boîtes d'appareil, les coins des images (le rapport d'aspect est
conservé), les extrémités et sommets des lignes, polylignes, polygones et cotations,
quatre points d'un cercle pour son rayon et les deux extrémités d'un arc. Une **poignée
de rotation** ronde au-dessus d'une forme, d'une image ou d'un texte sélectionné le fait
pivoter autour de son milieu, par pas de 15° (1° avec Alt enfoncé), l'angle s'affichant
à côté du pointeur ; les rectangles, ellipses et images gardent l'angle (*Rotation* dans
leurs propriétés) et leurs poignées le suivent, les autres formes font pivoter leurs
points. Les propriétés
d'une forme montrent sa taille : la largeur et la hauteur des rectangles et des ellipses,
la longueur des lignes et le rayon des cercles se saisissent, la taille et la longueur du
contour des autres sont affichées. L'**échelle** d'une page (propriétés de la page,
`set_page scale`, 1 : 50 vaut 50) est la longueur réelle par millimètre sur la page : les
tailles des formes y sont affichées et les nouvelles cotations y mesurent, pour des plans
d'implantation et des plans d'étage simples dessinés à l'échelle.

## Croisements de fils et connexions en T

Les fils qui se croisent sans connexion sont dessinés comme un simple croisement, ou le
fil vertical (ou horizontal) saute par-dessus l'autre avec un **arc**, un **pont** carré
ou un **écart** ; les sauts rapprochés sur un fil n'en forment qu'un. Les connexions en T
montrent un **point**, un **gros point** ou **pas de point** (un T simple) ; une
connexion de quatre fils ou plus garde toujours son point, afin de ne jamais ressembler à
un croisement. *Liaisons › Croisements de fils…* règle les deux avec un aperçu ; le
réglage est enregistré avec le projet et dans les schémas de paramètres et s'applique de
la même façon à la zone de dessin et à l'export PDF, SVG et DXF. Ligne de commande / MCP :
`wire_settings` (`jump` : none, arc, bridge, gap ; `jump_wire` : vertical, horizontal ;
`tee` : dot, large_dot, none ; `reroute`).

## Routage

La connexion auto trace des fils entre des points de connexion libres qui se font face
quand des symboles sont placés ou déplacés ; pendant qu'un symbole est placé ou déplacé à
la souris, de fines lignes pointillées montrent les fils qu'elle tracerait ; l'outil fil
peut contourner les symboles et les autres fils
(A) ; les tronçons de fil colinéaires peuvent être réunis. Aussi sous forme de commandes
`route` / `autoconnect`.

## Placement des propriétés

Faites glisser le bloc de repère d'un symbole sélectionné ; placez d'autres propriétés
de l'appareil (description, numéro d'article / de type, fabricant, désignation, points de
connexion, structure) à côté.

Les désignations des points de connexion sont placées à côté du trait de leur point de
connexion, déplacées le long de celui-ci, de l'autre côté ou à côté du fil là où les
graphiques du symbole, des fils, des symboles voisins ou les autres textes du symbole
gênent. Faites-en glisser une d'un symbole sélectionné pour la placer à la main, ou
choisissez gauche, droite, au-dessus ou en dessous pour chacune dans *Placement des
propriétés* ; *Réinitialiser* la ramène au placement automatique. Le placement tourne et
se reflète avec le symbole et ne déplace jamais le point de connexion. Commande :
`place_pin_label`. La désignation d'un symbole à un seul point de connexion (tel qu'un
point de connexion d'appareil) se glisse librement et garde son alignement.

Placé automatiquement, le bloc de repère se trouve à gauche d'un symbole vertical et
au-dessus d'un symbole horizontal. Là où il heurterait un symbole voisin, il passe de
l'autre côté, au-dessus ou au-dessous, selon ce qui est le premier dégagé de symboles et
de fils avec toutes ses lignes, renvois compris. Le repère d'une fonction située juste à
côté d'une fonction du même appareil dans la même rangée (comme des parafoudres dessinés
pôle par pôle) est montré une seule fois, à la première d'entre elles, comme le repère de
bornier d'une rangée de bornes. Un bloc de repère placé à la main reste où il est.

## Rechercher

**Rechercher** (*Accueil › Rechercher…*, `Ctrl+F`) : cherche dans tout le projet (noms et
descriptions de pages, repères d'appareil, textes de fonction, textes libres, numéros de
fils, bornes et broches, câbles, numéros d'article, descriptions) avec les options de
casse et de mots entiers, saute aux résultats et remplace du texte dans les textes
libres, les textes de fonction et les descriptions (les pages validées restent telles
quelles). Commandes : `find`, `replace_text`.

## Navigateurs

Pages (au choix regroupées par structure avec les descriptions de la
gestion des identificateurs de structure), appareils (sélectionner sur la page, placer
d'autres fonctions, renommer), liaisons (filtrer, montrer, sélectionner, ajouter une
définition de liaison), câbles (conducteurs utilisés et libres, définitions de câble,
tracer une définition de câble), calques et le contrôle.
L'onglet *Non placés* liste ce que les données du projet possèdent mais pas le schéma :
les canaux des cartes API ayant une description d'appareil qui n'ont pas de symbole
d'entrée / de sortie (avec leur adresse), les fonctions des articles affectés qu'aucun
symbole n'utilise (bobines, contacts avec leurs désignations), et les potentiels définis
dans le projet qu'aucun point de connexion de potentiel ne porte encore ; faites-en
glisser un dans la page (ou cliquez dessus, puis dans la page) pour le placer avec
repère, adresse et désignations. Commande :
`list_unplaced`.

## Langues

Anglais, allemand, français, espagnol, portugais (Brésil), italien, néerlandais,
polonais, tchèque, russe, ukrainien, turc, suédois, danois, finnois, hongrois, roumain,
chinois (simplifié), japonais et coréen (*Affichage › Langue* ; l'éditeur et le
gestionnaire d'articles retiennent le choix et démarrent dans la langue du système / du
navigateur si elle est prise en charge).
Le chinois, le japonais et le coréen sont fournis avec des polices intégrées
(sous-ensembles de Noto Sans SC / JP / KR) ; les textes du projet dans ces écritures
utilisent aussi les polices CJK du système sur le bureau, à l'écran et dans les exports
PDF (incorporées en sous-ensemble).

## Textes du projet en plusieurs langues

Les textes de fonction, les descriptions de page, les textes
libres et les descriptions sont écrits dans une langue source ; le dictionnaire de
traduction du projet (*Données de projet › Langues du projet…*) les traduit ligne par
ligne dans un nombre quelconque de langues cibles, avec export / import CSV pour les
traducteurs et une vérification des textes non traduits. *Données de projet › Exporter en
PDF dans la langue* (ligne de commande : `export` avec `language`) sort la documentation
par langue – les textes du projet à partir du dictionnaire, les rapports et les
cartouches dans cette langue (anglais, allemand, français, espagnol ; les autres langues
reçoivent des titres de rapport en anglais).

## Modèles de projet

**Modèles de projet** (dans l'esprit des projets de base d'EPLAN) : *Fichier › Nouveau ›
Nouveau projet à partir d'un modèle* part d'un modèle intégré (projet IEC 60204-1 vide ;
machine avec page de garde, table des matières, pages de schéma structurées et pages de
rapport ; tableau de distribution avec vue d'ensemble unifilaire, rapports de
climatisation et de court-circuit) ou de n'importe quel fichier de projet ; *Enregistrer
comme modèle de projet…* écrit le projet courant sans révisions, références, révision,
marques de câblage ni messages de vérification acceptés.
Ligne de commande : `new_project --template Machine` (ou un fichier) ; `new_project
--example <name>` part d'un [projet d'exemple](../../../README.md#example-projects).

Les **modèles paramétrés** demandent des valeurs quand un projet est démarré à partir
d'eux et en calculent d'autres par des formules : les modèles fournis *Démarreur moteur
direct*, *Démarreur moteur étoile-triangle* et *Démarreur moteur inverseur* demandent la
machine entraînée, la puissance assignée, la tension du réseau, cos phi, le rendement et
s'il faut ajouter les pages de rapport, et dimensionnent le démarreur à partir d'eux –
courant assigné (7,5 kW sous 400 V : 14,6 A), plage de réglage du disjoncteur moteur
(10 - 16 A), contacteurs (AC-3), section du circuit moteur et articles du catalogue
intégré. Les valeurs remplissent les espaces réservés `{NAME}` dans les textes du projet
(repères, textes de fonction, textes, données d'appareil, références d'article,
descriptions de page) ; une page avec une condition (`REPORTS=yes`, écrite comme dans le
configurateur) est omise quand elle n'est pas remplie. Une formule connaît les nombres,
les `"textes"`, les noms des paramètres et des formules qui la précèdent, `+ - * / ^`,
les comparaisons, `&` (joint des textes) et `sqrt`, `abs`, `round`, `ceil`, `floor`,
`min`, `max`, `if` et `step(x, limite, valeur, …, sinon)`, la valeur de la première
limite que `x` ne dépasse pas.

Le volet *Données de base › Paramètres du modèle* montre le projet ouvert en tant que
modèle : nom, description, paramètres (libellé, unité, valeur par défaut, choix),
formules avec leurs valeurs et la condition de chaque page. *Enregistrer comme modèle de
projet…* enregistre le projet dans le **dossier des modèles**
(`OpenECraft/data/templates` dans le dossier de données de l'utilisateur, ou le dossier
de `OPENECRAFT_TEMPLATES` ; dans le navigateur, le stockage de l'éditeur), où la ligne
de commande le trouve aussi, et *Exporter comme fichier…* écrit un fichier de modèle à
transmettre. Les modèles du dossier figurent à côté de ceux fournis dans *Nouveau projet
à partir d'un modèle* ; un modèle portant le nom d'un modèle fourni prend sa place :
on étend ainsi un modèle fourni en le modifiant (*Modifier le modèle* dans le menu de sa
vignette ouvre le modèle lui-même) et en l'enregistrant sous son nom. Ligne de commande :
`list_project_templates`, `template_values`, `new_project --template "Motor starter
direct on line" --values '{"POWER_KW": 7.5}'`, `new_project --edit`,
`save_project_template`, `delete_project_template` et les commandes `get_template`,
`set_template`, `set_template_parameter`, `set_template_formula`,
`remove_template_parameter` et `set_page --condition`.

## Enregistrement

Fichiers de projet JSON (`.oec`) ; le projet courant et la base de données d'articles
sont aussi enregistrés automatiquement (compressés) dans le dossier de données de
l'application sur le bureau et dans IndexedDB dans le navigateur, dont le stockage local
ne contient qu'environ 5 Mo (jusqu'à ce que la première écriture dans IndexedDB
réussisse, ils sont aussi conservés dans le stockage local).
