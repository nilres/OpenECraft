# Implantation d'armoire, salles et routage 3D

### Cartes importées sur des rails DIN horizontaux

OpenECraft aligne le clip de l'appareil sur l'axe du rail. Pour les boîtiers d'E/S EL de Beckhoff de 12 × 100 × 68 mm à 8 ou 16 bornes, et le coupleur EK1100 assorti, le centre du rail se trouve à 63,8 mm du haut. Par exemple, un rail centré à Y = 100 mm place le haut à Y = 36,2 mm. La cote de 35,4 mm est l'ouverture du boîtier pour le rail nominal de 35 mm. L'importateur conserve un décalage de centre et une hauteur d'encliquetage de l'article non nuls. Il n'applique le décalage de centre tiré du dessin qu'aux boîtiers correspondants, lorsque le décalage de centre du catalogue est nul. [Borne à 8 broches](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_8_pin.pdf) · [Borne à 16 broches](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Terminal/pdf/ethercat-_busterminal_12mm_16_pin.pdf) · [Boîtier EK1100](https://download.beckhoff.com/download/technical_drawings/Fieldbus_Components/EtherCAT_Coupler/pdf/ethercat-coupler_44mm_2xrj45_8pin.pdf)

Une carte placée ou déplacée à moins de 4 mm d'un bord voisin libre s'aligne bord à bord sur celui-ci, sur la même plaque de montage. L'aperçu de placement montre le résultat. Le dessin de l'armoire utilise d'abord la vue de face affectée, puis une représentation de montage importée. À défaut, une vue d'ensemble du fabricant fournit une illustration ajustée à l'intérieur de l'encombrement physique.

Des armoires avec leurs appareils en taille réelle en 2D et en 3D (dans l'esprit d'EPLAN
Pro Panel), câblage virtuel par les goulottes, données de perçage et CN, climatisation
des armoires, armoires juxtaposées, salles du bâtiment avec leurs équipements posés au
sol et leurs chemins de tuyauterie, baies 19 pouces et routage 3D des câbles, tubes et
tuyaux. Les projets d'exemple `conveyor`, `cabinet_lineup`, `electrical_room` et
`pump_room` les illustrent.

![Les armoires juxtaposées de l'exemple cabinet_lineup en 3D, porte ouverte, avec rails, goulottes, appareils et fils routés](../../images/panel-3d.png)

## Armoires et appareils

- Armoires (à partir d'un article tel que `GEN.ENC-600X800` ou de taille libre) avec
  plaque de montage.
- Rails de montage et goulottes dessinés à la longueur ; sélectionnez-en un et faites
  glisser l'une de ses extrémités pour le redimensionner, ou déplacez-le (les appareils
  suivent leur rail). Les appareils sont placés à partir d'une liste « appareils à
  placer » avec leurs dimensions réelles issues des données d'articles (contacteur +
  bloc auxiliaire s'additionnent en profondeur, les borniers sont aussi longs que
  leurs bornes et leurs réserves) et accrochés sur les rails ; rotation, suppression.
- Couper, copier et coller dans la vue de face : cliquez du bouton droit sur un élément
  pour *Couper*, *Copier*, *Coller*, *Propriétés* et *Retirer* (les appareils
  retournent dans les appareils à placer), ou `Ctrl+X` / `Ctrl+C` / `Ctrl+V`. Les
  rails, goulottes, jeux de barres, découpes et accessoires sont copiés avec les
  éléments montés dessus, les appareils sont laissés de côté (un appareil n'est placé
  qu'une fois) ; les éléments coupés quittent l'implantation, les appareils coupés
  retournent dans les appareils à placer. Les éléments collés suivent le pointeur
  jusqu'à ce qu'un clic les place, sur n'importe quelle face de n'importe quelle
  armoire ; `Esc` ou un clic droit annule. La commande `copy_panel_items` fait de même
  depuis la ligne de commande et pour les agents d'IA.
- Vue de face 2D et vue 3D : faites glisser pour tourner autour, bouton droit / milieu
  ou Maj+glisser pour déplacer la vue, la molette zoome sur le pointeur, toujours plus
  près de la face pleine qui s'y trouve mais sans la traverser (dans le vide elle avance
  jusque dans une salle), un double-clic tourne la vue vers ce qui a été cliqué et s'en
  rapproche, `W` `A` `S` `D` / les touches fléchées et `Q` `E` déplacent dans la vue
  (Maj : plus vite), la barre de la vue propose les vues standard et *Ajuster*
  (`Home`) ; cliquez pour sélectionner. Les murs, plafonds et
  portes vus de l'extérieur sont transparents (*Opacité des murs* dans les paramètres,
  sous Apparence) ; *Masquer les murs de devant* omet les murs entre l'oeil et
  l'intérieur, *Masquer le plafond* les plafonds des salles. Les fils sont dessinés
  dans leur couleur (les fils bicolores tels que vert-jaune avec une bande) et, comme
  les câbles et les tubes, aussi épais que leur diamètre extérieur. Un câble se
  termine un peu avant l'appareil auquel il est raccordé et ses conducteurs s'en
  séparent en courbes douces, chacun vers son point de connexion, du côté par lequel
  ses fils le quittent. À son point de connexion, un fil ou un conducteur se termine
  par sa terminaison : un embout (manchon métallique et collerette dans la couleur
  DIN 46228-4) ou un embout double, une cosse à oeil (son anneau autour du point de
  connexion), une cosse à fourche, à broche, une languette mâle ou un clip femelle
  (avec son manchon isolé rouge, bleu ou jaune ; des cosses tubulaires non isolées à
  partir de 10 mm²), ou le conducteur en cuivre nu sur sa longueur de dénudage. Chaque
  extrémité de fil peut recevoir sa propre terminaison dans les propriétés du fil
  (*Extrémités du fil*), tous les conducteurs d'un câble à l'un de ses appareils dans
  les propriétés du câble (*Extrémités du câble*), et en ligne de commande toutes les
  extrémités à un point de connexion ou à un appareil (`set_termination`) ; les
  extrémités qui n'en ont pas reçoivent un embout lorsque *Extrémités de fil avec
  embouts* est activé, le conducteur nu sinon. Un embout double est partagé par les
  deux fils qui se terminent à un même point de connexion : il n'est proposé que là,
  choisi pour les deux à la fois, dimensionné pour la plus grande section et dessiné
  une seule fois, au premier des deux ; avec *Embouts doubles là où deux fils
  partagent un point de connexion* (ou `set_ferrules`), chaque paire de ce type sans
  terminaison propre en reçoit un. Le contrôle *Embouts doubles* avertit d'un embout
  double avec un seul fil, à un point comptant plus de deux fils ou pour deux
  sections. Un embout ou une cosse sertie ne se courbe pas : le fil ou le conducteur
  sort tout droit de son point de connexion sur toute sa longueur (avec sa collerette
  ou son manchon isolé) et ne se courbe que derrière. Le routage 3D prévoit cette
  sortie droite lorsque l'espace devant le point de connexion est libre ; lorsqu'il ne
  l'est pas (un fil voisin, une goulotte toute proche), le fil se courbe plus serré
  juste derrière l'embout ou la cosse.
  La liste de fils pour la production, l'assistant de
  câblage, les exports pour machines et la nomenclature reprennent les mêmes
  terminaisons ; à travers les presse-étoupes, les traversées de mur et les passages
  de câbles entre armoires, les câbles et les fils sont dessinés d'un seul tenant,
  courbés à cet endroit, plusieurs côte à côte à travers une même bride d'entrée de
  câbles ou une même traversée de mur. Les presse-étoupes sont dessinés avec les
  modèles de leurs articles : à l'extérieur le corps hexagonal, le col et le chapeau
  sur la surface extérieure de leur paroi, à l'intérieur le contre-écrou et
  l'extrémité du filetage sur sa surface intérieure, en laiton nickelé ou en
  polyamide ; les parcours évitent le contre-écrou. Dans une armoire sans goulottes sur sa plaque de
  montage, les fils sont posés par le routage 3D (voir ci-dessous), de point de
  connexion à point de connexion.
- **Jeux de barres** en cuivre (barres dessinées, mètres et supports dans la
  nomenclature) et **découpes** rectangulaires ou rondes dans la plaque de montage. Un
  appareil placé sur un jeu de barres est monté sur ses barres ; les appareils d'un
  même jeu de barres sont reliés par les barres, non par des fils. Les jeux de barres
  d'armoires juxtaposées qui se rejoignent à leur côté commun (le même article à la
  même hauteur, leurs extrémités distantes de 100 mm au plus) forment un seul jeu,
  relié à travers le côté. Les barres sont disposées à l'entraxe `Centre distance` de
  l'article (régulièrement espacées à défaut) le long du jeu, aussi courte que soit
  l'une de ses sections ; la vue 3D dessine chaque barre séparément, sur des supports
  isolants qui les dépassent à peine, aux positions de perçage de la plaque mais à
  côté des appareils insérés dans les barres (les transformateurs que les barres
  traversent), jamais à travers eux
  (`busbars::holder_places`). Un bornier pour jeu de barres placé en travers des barres (`busbars::place_on_bars`)
  met chaque borne debout sur sa propre barre, un bornier par câble d'alimentation ;
  dans le routage 3D, chacune de ces bornes est un solide à part entière (un fil vers
  l'une contourne les autres), et les bornes distantes de moins de 30 mm le long des
  barres sont abordées par l'avant.
  Chaque barre
  est un conducteur : le contrôle `busbars` signale comme court-circuit un jeu dont
  les appareils relient plus de conducteurs qu'il n'a de barres (donnée d'article
  `Bars`), par exemple une borne PEN sur les barres des phases ou les départs situés
  derrière un interrupteur de couplage sur le jeu de barres en amont de celui-ci. Il
  prend le côté alimentation des interrupteurs et des fusibles (le côté départ lorsque
  c'est celui que partagent les autres appareils : une réglette d'arrivée), chaque
  borne comme un conducteur, et suit les conducteurs à travers les primaires des
  transformateurs de courant.
- Pages d'implantation (vue de modèle 2D à l'échelle avec cotes) pour l'impression /
  SVG ; les goulottes, rails de montage et jeux de barres portent une étiquette avec
  leur section et leur longueur de coupe (*60 × 100 mm, L = 940 mm*) sur le plus long
  tronçon qu'aucun appareil ne recouvre, réduite pour tenir, la longueur seule
  (*L = 430 mm*) lorsque l'étiquette entière ne tient pas. Un élément entièrement
  recouvert la reçoit à côté de sa fin (sinon de son début), dans son alignement,
  jusqu'à l'élément suivant ou au bord de la plaque ; elle est omise lorsque même là
  elle serait illisible. Les rails, goulottes et jeux de barres
  figurent dans la nomenclature par longueur totale (ou par barres de stock), avec les
  pièces à couper par armoire et par longueur sous chaque article (sans référence
  d'article propre, de sorte que les exports ERP comptent le matériel une seule
  fois) ; les armoires à la pièce. La légende de l'armoire liste chaque rail, goulotte
  et jeu de barres après les appareils avec sa référence d'article, sa section
  (largeur × hauteur) et sa longueur de coupe.
- Saut entre un appareil du schéma et son emplacement dans l'implantation d'armoire.

## Contrôles de collision et de limites

Chevauchements, hors de la plaque, trop profond pour l'armoire compte tenu d'un
dégagement de porte, monté sur une découpe, à l'intérieur du dégagement dont un
appareil a besoin au-dessus et au-dessous d'après ses données d'article
(`Clearance: 25 mm`) ; lorsqu'un appareil a un modèle 3D, ce sont les **modèles** dont
on vérifie la pénétration, et pas seulement leurs boîtes englobantes.

## Porte de l'armoire

Les appareils de commande et de signalisation (boutons-poussoirs, voyants …) sont
placés sur la porte (vue de face *Porte*) ; chacun reçoit ses **découpes de porte**
(Ø22,5 par défaut ou d'après les données d'article `Door cutout: Ø30.5` / `92 x 68`,
sinon ses découpes), placées là où l'article les a et tournées avec l'appareil,
vérifiées par rapport au bord libre de la porte (cadre, charnières) et à la face avant
de l'appareil (une découpe qui la dépasse est signalée) et, en 3D, par rapport aux
équipements de la plaque de montage ; leurs fils sont routés par la charnière ; la
page d'implantation montre côte à côte la plaque et chaque face portant des éléments
(porte, parois latérales, paroi arrière, toit, fond).

La porte est articulée à gauche ou à droite (*Charnières de la porte* dans les
propriétés de l'armoire, `door_hinge` de `add_enclosure` / `set_enclosure`, donnée
d'article `Door hinge: right` de l'article d'armoire d'un fabricant) : la vue de la
porte montre les charnières et la poignée de leurs côtés, la vue 3D ouvre la porte
autour de ce bord, et les fils de porte longent la porte jusqu'à ce bord, passent par
la boucle de charnière et entrent dans le bord de la plaque de ce côté.

## Zones interdites

Un article d'armoire peut garder libres des zones de ses faces (*Zones interdites*
dans les données de montage de l'article, importées avec les paquets d'armoires du
configurateur Rittal) : les zones de *placement* ne reçoivent ni appareils, ni rails,
ni goulottes, ni jeux de barres, les zones de *découpe* ni découpes ni trous, p. ex.
les charnières, la serrure et le renfort de la porte, ou les points de fixation de la
plaque de montage. Les faces les montrent hachurées (zones de placement en hachures
montantes, zones de découpe en hachures descendantes), dans l'éditeur et sur les pages
d'implantation, et la vue 3D sous forme de zones translucides (placement en bleu,
découpe en orange) qui accompagnent leur paroi ou la porte. Le contrôle signale un
élément situé dans une zone de placement (*À l'extérieur*) et les découpes et trous
situés dans une zone de découpe (*Fabrication*) ; toucher le bord d'une zone est
permis. L'éditeur d'article les liste sous *Données de montage › Zones interdites*
(type, face, position et taille), où elles sont ajoutées, modifiées et supprimées.

## Modèles d'armoires des fabricants

Une armoire dont l'article est celui d'un fabricant (et non l'un de ceux du catalogue
intégré) avec un modèle 3D à la largeur et à la hauteur de l'armoire, tel que
l'apportent les paquets d'armoires du configurateur Rittal, montre ce modèle dans la
vue 3D et les exports 3D à la place du corps, des parois, de la porte et des plaques
génériques. La porte et ce qui y est monté (lames de charnières, serrures, joints),
tels que la macro du fabricant les marque, sont translucides et s'ouvrent avec la
porte ; un modèle sans cette marque ouvre sa face avant de 20 mm. Les autres triangles
accompagnent la paroi dans le plan de laquelle ils se trouvent (masqués avec elle
lorsque les murs de devant sont masqués) ou restent à l'intérieur (plaque de montage,
plaque passe-câbles, brides). Les socles, kits de juxtaposition et montants 19 pouces
restent génériques. La porte s'ouvre du côté de ses charnières (la Rittal AX est
articulée à droite, voir [Porte de l'armoire](#porte-de-larmoire)).

## Plan de perçage et données CN

Les rails, goulottes et supports de jeux de barres reçoivent des trous taraudés sur
leur longueur, les appareils vissés sur la plaque reçoivent les trous de leurs données
d'article (`Mounting holes: M5: 9,12; 69,72`) ou deux trous par défaut ; **export CN**
par armoire et par face en DXF ou DWG (contour, un calque par diamètre de trou,
découpes, celles qui sont tournées ou arrondies par leur contour) ou CSV (avec la
rotation et le rayon d'angle de chaque découpe).

## Câblage virtuel et taux de remplissage des goulottes

Chaque liaison du schéma entre appareils placés est routée depuis son point de
connexion (appareils tournés, blocs auxiliaires, borne par borne) vers la goulotte à
laquelle la borne fait face, puis à travers le réseau de goulottes (plus court
chemin) → liste de fils avec longueurs, fils affichés en 2D / 3D.

**Taux de remplissage des goulottes** : sections des fils routés (avec isolant), des
câbles et des tubes rapportées à la section intérieure de la goulotte (entre ses
parois, son fond et son couvercle) sur son tronçon le plus rempli, sur la plaque de
montage comme sur les murs, le plafond et le sol d'une salle ;
goulottes colorées en vert / orange / rouge dans la
vue de face, taux de remplissage dans les propriétés de la goulotte, limite dans les
propriétés de l'armoire (70 % par défaut) ; rapport *Taux de remplissage des
goulottes*, le contrôle avertit au-dessus de la limite et donne un avis au-dessus de
80 % de celle-ci ; lorsqu'une goulotte est trop pleine, le câblage est routé de
nouveau, les fils les plus épais d'abord, et les fils font un détour (jusqu'à 10 ×
plus long) autour des tronçons qu'ils rempliraient au-delà de la limite ; les câbles
et tubes routés en 3D font de même autour des tronçons que les fils et les câbles
précédents remplissent déjà ; l'appareil sélectionné montre ses points de connexion.

## Climatisation

IEC/TR 60890, dans les propriétés de l'armoire : puissance dissipée des appareils
placés d'après les données techniques de leurs articles (`Power loss: 7.5 W`) plus une
dissipation supplémentaire, surface effective selon le type d'installation (seule,
dans une rangée, isolée, murale, toit couvert), matériau (coefficient de transmission
thermique), températures ambiante et intérieure admissible
→ température intérieure, débit d'air du ventilateur à filtre (corrigé selon
l'altitude du site), échangeur de chaleur air/air (W/K, lorsque le degré de protection
doit être conservé) ou puissance du climatiseur, puissance de chauffage qui maintient
aussi l'intérieur au-dessus du point de rosée de l'air ambiant (humidité relative) ;
rapport *Climatisation des armoires*, le contrôle avertit lorsqu'un refroidissement
est nécessaire.

La carte thermique de la plaque de montage stratifie la température moyenne et ajoute
un panache au-dessus de chaque appareil ayant des données de puissance dissipée ; les
goulottes horizontales retiennent les panaches (l'air chaud s'accumule dessous), les
ventilateurs et les climatiseurs les atténuent.

![Carte thermique d'une plaque de montage à côté du calcul de climatisation de l'armoire](../../images/heat-map.png)

## Armoires juxtaposées

Des armoires juxtaposées côte à côte avec des passages de câbles entre elles (découpes
dans les parois latérales en regard, ou côtés ouverts l'un vers l'autre) ; la vue 3D
montre tout l'ensemble, les fils passent d'une armoire à la suivante. La vue *salle*
montre toutes les armoires côte à côte à leurs emplacements, chaque ensemble à sa
position et avec sa rotation, fixées à la main ou reprises de l'implantation de
l'installation.

## Salles

*Espace de montage › Nouvelle salle* (plugin `rooms`) : une salle du bâtiment avec son
plan, sa hauteur et son emplacement dans l'installation. Le plan est un rectangle ou
un contour de murs à angle droit (salles en L, en U ou en T) : *Plan* le montre de
dessus, *Dessiner les murs* le dessine en cliquant les coins (sur une grille de 50 mm,
fermé en cliquant le premier coin ou par Entrée), les murs se déplacent en les faisant
glisser ou reçoivent une longueur dans les propriétés de la salle
(commandes `set_room_outline`, `set_room_wall`).

- **Les murs intérieurs** (*Espace de montage › Dessiner › Murs intérieurs*, commandes
  `add_inner_wall`, `set_inner_wall`) sont dessinés dans le plan en cliquant leurs
  extrémités (une chaîne, à angle droit ; une extrémité proche d'un mur s'y accroche),
  se déplacent en les faisant glisser transversalement, reçoivent longueur et
  épaisseur dans les propriétés de la salle et se suppriment avec Suppr ; leurs deux côtés (`inner1a`, `inner1b`) sont des faces comme
  les murs.
- **Les portes et fenêtres** (*Espace de montage › Dessiner › Porte / Fenêtre*,
  commandes `add_wall_opening`, `set_wall_opening`) sont des ouvertures dans un mur,
  placées en cliquant le mur dans le plan ou dans la vue du mur : une porte repose sur
  le sol, une fenêtre a sa hauteur d'allège ; taille, position, allège et mode
  d'ouverture se trouvent dans leurs propriétés. *Ouverture* (`opens`) choisit les
  charnières au bord gauche ou droit (vu de l'intérieur de la salle) avec un battant
  s'ouvrant vers l'intérieur ou l'extérieur de la salle, ou un vantail coulissant vers
  la gauche ou la droite le long du mur. Le plan dessine le vantail avec son débattement
  (depuis la surface extérieure du mur lorsqu'il s'ouvre vers l'extérieur) ou le
  vantail coulissant avec sa course, la vue du mur le symbole d'ouverture (sa pointe
  aux charnières, en tirets lorsque le vantail s'ouvre en s'éloignant de
  l'observateur) ou une flèche. Le contrôle garde libres d'armoires le débattement
  d'une porte s'ouvrant vers l'intérieur et la course d'une porte coulissante devant
  le mur.
- **Les armoires** (leurs ensembles juxtaposés) y sont posées à un emplacement du
  plan, où on les fait glisser dans le plan (R tourne l'armoire sélectionnée autour de
  son centre), et se déplacent avec la salle. Une armoire hors de la salle, à cheval
  sur un mur, dans un mur intérieur ou dans le débattement d'une porte est dessinée
  dans la couleur d'avertissement, et le pointeur placé dessus en donne la raison.
- Chaque mur (vu de l'intérieur de la salle) et le plafond portent des rails de
  montage, des goulottes et des appareils : un appareil sur un mur repose sur un rail,
  sauf si son article est marqué *Montage mural* (éditeur d'article › Données de
  montage, un badge `MUR` dans les arborescences d'articles, filtre *Seulement les
  articles pour montage mural direct*), auquel cas il va directement sur le mur
  (*Insertion › Article à montage mural*, également sans repère d'appareil).
- **Prises, interrupteurs et chauffage** (*Insertion › Prises, interrupteurs,
  chauffage*) : prises de courant, prises réseau et d'antenne, interrupteurs,
  boutons-poussoirs, variateurs, thermostats d'ambiance, collecteurs de circuits de
  chauffage et boîtiers de régulation pièce par pièce (RTL / TH), dessinés avec leurs
  modèles dans la vue 3D. Le menu comporte un onglet pour les prises, les
  interrupteurs et le chauffage, une recherche, des filtres par fabricant et par type,
  la liste et l'article choisi avec son modèle 3D (faites glisser pour le tourner), sa
  taille et sa hauteur de pose (*Placer sur le mur* ou un double-clic le place).
  Il liste
  les articles génériques du catalogue (`GEN.SOCKET-F-UP`, `GEN.SW-2WAY-UP`, …,
  collecteurs `GEN.HCM-2` … `GEN.HCM-12`, le nombre de circuits étant choisi dans le
  menu, `GEN.RTL-BOX`) et ceux de la base d'articles et des catalogues fournis `gira`
  (System 55), `mdt` (boutons-poussoirs KNX, bouton-poussoir en verre, régulateur de
  température ambiante) et `cosmo` (collecteurs Standard et Balance, boîtiers de
  régulation) ; un article d'un catalogue fourni entre dans le projet lorsqu'il est
  choisi. Un article ayant une *Hauteur de pose* dans ses données techniques va sur le
  mur à cette hauteur (son milieu au-dessus du sol : prises de courant et prises
  réseau 300 mm, interrupteurs 1050 mm, thermostats 1500 mm, boîtiers de régulation
  250 mm) à l'endroit où le mur est cliqué. Les collecteurs ont un piquage pour le
  départ et le retour et une paire par circuit, vers le bas, pour le routage des
  tuyaux.
- Le **sol** (face `floor`) porte les articles marqués *Montage au sol* (pompes,
  réservoirs, compresseurs ; un badge `SOL`, filtre *Seulement les articles posés au
  sol*) : *Insertion › Article posé au sol*, appareils à placer dans la vue du sol,
  commande `mount_on_wall` avec `face` `floor`. Ils sont dessinés dans le plan, s'y
  déplacent en les faisant glisser et se suppriment avec Suppr.
- **Les traversées de mur** (*Espace de montage › Dessiner › Traversée de mur*,
  commande `add_wall_opening` avec `kind` `penetration` ; 300 x 200 mm, 300 mm sous le
  plafond sauf indication contraire) vont dans les murs extérieurs et mènent à la
  salle située derrière le mur (jusqu'à 1 m de distance, y compris à travers des
  salles tournées l'une par rapport à l'autre) : les câbles, tubes et tuyaux sont
  routés de salle en salle à travers elles. Le plan les dessine, ainsi que le chemin à
  travers le mur, en tirets ; `list_rooms` indique où mène chacune.
- **Les entrées de bâtiment** (*Espace de montage › Dessiner › Entrée de bâtiment*,
  commande `add_wall_opening` avec `kind` `entry` ; 200 x 200 mm, 600 mm au-dessus du
  sol sauf indication contraire) sont les pénétrations de branchement des murs
  extérieurs : le câble d'alimentation, la conduite d'eau principale et les autres
  lignes venant de l'extérieur du bâtiment entrent par elles. Un câble ou un tuyau
  allant d'un appareil de la salle à un appareil situé hors du bâtiment (son lieu
  n'est aucune des salles ni des armoires de l'implantation, par exemple l'armoire de
  distribution de câbles du distributeur) est routé vers l'entrée de bâtiment la plus
  proche.
- **Les chemins de tuyauterie** (*Espace de montage › Dessiner › Chemin de
  tuyauterie*, commande `add_pipe_run`) sont dessinés sur un mur, le plafond ou le sol
  en cliquant leurs coins (à angle droit), à 100 mm devant un mur, 300 mm sous le
  plafond ou 200 mm au-dessus du sol sauf indication contraire : les flexibles, tubes
  et tuyaux du schéma sont routés le long d'eux comme les fils le long des goulottes.
  Le plan les montre en tirets.
- **L'extérieur du bâtiment :** chaque mur d'une salle a aussi sa face extérieure
  (faces `outside1`, `outside2`, …, vues de l'extérieur, *Face extérieure du mur n*
  dans le menu des murs) et le plafond son dessus (face `rooftop`) : les luminaires de
  façade, capteurs et caméras vont sur la face extérieure d'un mur extérieur comme sur
  un mur à l'intérieur (sur un rail ou, s'ils sont marqués *Montage mural*,
  directement), une station météo ou d'autres articles marqués *Montage au sol* sont
  posés sur le toit. Leurs câbles et tuyaux traversent le mur (ou le plafond) juste
  derrière eux pour entrer dans la salle. Le contrôle signale de tels éléments
  lorsqu'une autre salle se trouve juste derrière le mur ou au-dessus du plafond (à
  moins de 1 m).

Le contrôle signale les appareils sur un mur qui ne sont ni l'un ni l'autre, les rails
ou appareils au-dessus d'une porte ou d'une fenêtre, les chemins de tuyauterie en
travers d'une porte ou d'une fenêtre, les éléments au plafond en dehors du plan, les
murs intérieurs qui sortent de la salle, les armoires et articles posés au sol dans le
débattement d'une porte ou dans un mur intérieur, les articles posés au sol à
l'emplacement d'une armoire ou non prévus pour être posés au sol, ainsi que les
armoires et articles posés au sol hors de la salle (ou à cheval sur un mur), les
armoires plus hautes que son plafond, les traversées de mur qui ne mènent à aucune
salle. Les salles n'ont ni calcul de climatisation ni données CN.

**Plusieurs salles** se trouvent à leurs emplacements dans l'installation (*Coin
arrière gauche de la salle* dans ses propriétés, commandes `add_room` / `set_room`
avec `x`, `z` et `angle`). Le plan d'une salle montre en estompé les salles qui
l'entourent avec leurs noms et leurs armoires (celles qui sont à côté, non celles des
niveaux au-dessus et au-dessous) ; un double-clic ouvre l'une d'elles. *Espace de
montage › Afficher › Toutes les armoires* montre chaque salle et chaque armoire à sa
place en 3D, avec les parcours entre elles.

Le navigateur (*Enveloppes et salles*) a le **terrain** à sa racine, au-dessus des
bâtiments avec leurs niveaux et leurs salles et des salles hors des bâtiments. Un clic
sur une salle conserve la vue 3D lorsqu'elle est affichée et montre sinon le plan de
la salle. Un clic sur le terrain montre le plan de masse de tous les bâtiments dans
l'Espace de montage (la molette zoome, le glissement le déplace, un clic sur une salle
ouvre son plan ; dans le mode d'édition *Terrain › Déplacer les bâtiments*, un
bâtiment est glissé à sa place et R le tourne de 90° autour de son milieu, avec ses
salles, armoires, escaliers et toits et les salles qui s'y trouvent sans être sur l'un
de ses niveaux) ou, dans la vue 3D, toute
l'installation avec ses bâtiments, dalles, escaliers et toits ; un clic sur un bâtiment
montre le plan de masse avec ce bâtiment mis en évidence. Un niveau cliqué alors que
la vue 3D est affichée montre toutes les salles de ce niveau côte à côte à leurs
emplacements, avec les armoires qui s'y trouvent, les dalles et les escaliers
atteignant le niveau (sans les toits) ; un second clic montre de nouveau toute
l'installation. Le menu contextuel du terrain ajoute un bâtiment, dessine la limite du
terrain ou place le point de référence, celui d'un bâtiment un niveau au-dessus ou un
sous-sol, celui d'un niveau une salle sur ce niveau.

## Limite du terrain et point de référence

Le **terrain** (*Espace de montage › Terrain*, le menu contextuel du terrain, la
section *Terrain* du volet Bâtiments ; plugin `buildings`) est la parcelle sur
laquelle se trouve l'installation.

- **Limite du terrain** (*Terrain › Limite du terrain*, commande `set_plot`) :
  dessinée dans le plan de masse coin par coin (un clic sur le premier coin, un
  double-clic ou Entrée la ferme, Retour arrière reprend le dernier coin, Échap
  arrête) ; les coins s'accrochent aux coins des salles (à l'intérieur et à
  l'extérieur de leurs murs), des gaines et au point de référence. Ses coins se
  déplacent ensuite dans le plan de masse en les faisant glisser. La limite ne doit
  pas se croiser elle-même ; le plan de masse écrit son nom et sa surface, le volet
  ses coins, sa surface et sa longueur. **Tous les bâtiments doivent rester à
  l'intérieur** : le contrôle *Bâtiments, niveaux et gaines* signale comme erreurs les
  salles dont les murs extérieurs dépassent la limite, et avertit des toits qui la
  dépassent par leur débord et des armoires situées en dehors ; le plan de masse les
  dessine dans la couleur des erreurs et `plot` les liste.
- **Point de référence** (*Terrain › Point de référence*, commande
  `set_reference_point`) : un point géoréférencé de l'installation (sa position dans
  l'installation, ses coordonnées est et nord et son altitude au-dessus du niveau de
  la mer dans un système de coordonnées de référence tel que `EPSG:25832`, la
  direction du nord de la grille dans l'installation et, si elles sont connues, sa
  latitude et sa longitude). Chaque point de l'installation est mesuré à partir de
  lui : le plan de masse montre les coordonnées sous le pointeur (coordonnées est et
  nord, ou la distance à l'est et au nord du point), une flèche du nord, et
  `list_buildings` les coordonnées est et nord et l'altitude de l'origine de chaque
  bâtiment ; `locate` convertit entre l'installation et le système de coordonnées. Les
  exports IFC écrivent le nom du terrain ainsi que la latitude, la longitude et
  l'altitude dans leur site.
- **Mesurer** (*Terrain › Mesurer*, commande `measure`) : deux points cliqués dans le
  plan de masse donnent leur distance, la distance à l'est et au nord du second et sa
  direction par rapport au nord de la grille.

## Bâtiments, niveaux et gaines

*Espace de montage › Bâtiments* (plugin `buildings`) gère les bâtiments de
l'installation dans des onglets : *Terrain* (le terrain, les champs photovoltaïques au
sol, les liaisons entre bâtiments), *Bâtiment* (le bâtiment choisi avec ses niveaux,
gaines et escaliers), *Toits*, *Mise à la terre*, *Plan sur papier* et *Installation
électrique* (ces deux derniers pour un niveau).
Chaque onglet dessine un plan de masse de toutes les salles (un niveau mis en
évidence), des gaines, des escaliers atteignant le niveau affiché (marches et ligne de
foulée), des toits (égouts, faîtages et arêtes en tirets) avec leurs fenêtres de toit,
et des liaisons.

- Un **bâtiment** (commande `add_building`) a un emplacement dans l'installation (x, z
  et une rotation) et des **niveaux** superposés (`add_level`, `set_level` : nom,
  hauteur du sol au-dessus du niveau du terrain du bâtiment, hauteur de sol à sol ;
  une élévation négative est un sous-sol). Un nouveau bâtiment reçoit un niveau `L0`
  au niveau du terrain et se place à côté de ce qui se trouve dans l'installation ;
  les nouveaux niveaux vont au-dessus (`L1`, `L2`, …) ou au-dessous
  (`B1`, `B2`, …).
- **Les salles reposent sur des niveaux** (propriétés de la salle › *Bâtiment*,
  *Nouvelle pièce sur ce niveau* / *Placer une pièce ici* dans le volet, commandes
  `place_on_level`, `add_room` avec `building` et `level`) : l'emplacement de la salle
  est alors mesuré dans le plan du bâtiment, elle repose sur le sol du niveau et se
  déplace avec le bâtiment et son niveau. Une nouvelle salle sur un niveau est aussi
  haute que le niveau moins sa dalle de plafond (300 mm) et se place à côté des autres
  salles du niveau. Le navigateur regroupe les salles par bâtiment et par niveau : une
  salle glissée sur un niveau (ou sur l'une de ses salles) est placée sur ce niveau ;
  un niveau qui y est cliqué est sélectionné, et *Nouvelle salle* place alors la
  nouvelle salle dessus (un second clic le désélectionne) ; le menu contextuel d'un
  niveau et de ses salles propose *Nouvelle pièce sur ce niveau*, celui d'une salle
  dans un bâtiment *Retirer du bâtiment*. La suppression d'un niveau ou d'un bâtiment
  laisse ses salles là où elles se trouvent dans l'installation.
- **Les gaines** (`add_shaft`, `set_shaft`) traversent un bâtiment d'un niveau jusqu'à
  un autre, à un emplacement de son plan : les câbles, tubes et tuyaux sont routés à
  travers elles d'une salle vers les salles au-dessus et au-dessous dont la gaine
  traverse le plan, de sous le plafond de l'une jusqu'au-dessus du sol de la suivante.
  Le plan d'une salle montre les gaines qui traversent son niveau.
- **Les escaliers** (*Nouvel escalier* dans le volet, `add_stair`, `set_stair`) mènent
  du sol d'un niveau jusqu'au sol d'un autre : *droit*, *tournant une fois* sur un
  palier d'angle, *tournant deux fois* sur deux paliers d'angle, à *palier
  intermédiaire* (deux volées côte à côte) ou *hélicoïdal* autour d'un noyau, tournant
  à gauche ou à droite. Leur pied est le milieu du bord avant de la marche la plus
  basse dans le plan du bâtiment, avec la direction de la première volée, la largeur
  des volées et le giron d'une marche ; les marches découlent de la différence de
  hauteur (hauteurs de marche proches de 175 mm). Les plans des salles montrent les
  escaliers sur leur niveau (en tirets là où ils montent d'en dessous), la vue 3D
  leurs marches et paliers et la trémie dans la dalle au-dessus.
- **Les toits** (*Nouveau toit* dans le volet, `add_roof`, `set_roof`) : *plat*, *en
  appentis*, *à deux pans*, *à quatre pans* ou *en pavillon* au-dessus d'un rectangle
  du plan du bâtiment (un nouveau toit couvre les salles du bâtiment), avec la hauteur
  de son égout, sa pente, son débord et son épaisseur ; un toit à deux pans ou en
  appentis s'étend le long de x ou de z. La sous-face du toit commence à l'égout
  au-dessus des murs et monte selon la pente. Là où elle est plus basse que le plafond
  d'une salle située en dessous, y compris sur plusieurs niveaux à la fois (égout à
  hauteur de jambette au-dessus du plancher des combles, ou sous le plafond du niveau
  inférieur), elle devient le plafond de cette salle : la salle a des hauteurs
  différentes selon l'endroit. Ses murs s'arrêtent sous le rampant dans la vue 3D, la
  vue d'un mur dessine la ligne où il s'arrête, le plan ombre les parties sous le
  rampant avec leur hauteur la plus basse, et le volet et `list_buildings` donnent le
  plafond le plus bas et le plus haut de chacune de ces salles. Les **fenêtres de
  toit** (`add_roof_window`, `set_roof_window` ; 780 × 1180 mm sauf indication
  contraire) se trouvent dans un versant, à leur milieu dans le plan, avec leur
  largeur le long de l'égout et leur longueur dans le sens de la pente ; elles sont
  visibles de l'extérieur et depuis la salle située en dessous.
- **Les champs photovoltaïques** (*Champ photovoltaïque* sous un toit dans le volet,
  `add_pv_array`, `set_pv_array`) : des rangées de modules solaires sur n'importe
  quelle forme de toit. Sur un versant, ils reposent **parallèlement** à la couverture
  sur des rails fixés par des crochets de toit : deux rails sous chaque rangée de
  modules en portrait, sous chaque colonne en paysage, les crochets espacés au plus de
  la distance entre crochets (1200 mm sauf indication contraire). Sur un toit plat,
  ils sont **inclinés** vers un côté (15° sauf indication contraire, orientés dans la
  direction donnée) ou **est-ouest** par paires sur des supports lestés, les rangées
  assez éloignées pour ne pas se faire d'ombre avec un soleil d'hiver à 17° (ou à une
  distance donnée). Un champ repose sur le versant situé sous son milieu ; son article
  de module (les génériques `GEN.PV-430W` et `GEN.PV-400W-FB`, ou des modules des
  catalogues fournis `longi` et `jinko`) donne la taille des modules et sa puissance
  crête. Le matériel de montage provient de son **système de montage** : un rail ou un
  support de la base d'articles dont les accessoires sont ses crochets de toit,
  raccords de rails, pinces d'extrémité et intermédiaires (celle qui correspond à la
  hauteur du cadre du module), lests et déflecteurs de vent ; le système générique
  `GEN.PV-RAIL-40` / `GEN.PV-BASE-S15` / `GEN.PV-BASE-EW10` complète ce qui manque à
  un système. Les modules (sauf si un appareil, un générateur photovoltaïque, les
  compte) et le matériel de montage vont dans la nomenclature ;
  `list_buildings` donne les modules, la puissance crête et le matériel de chaque
  champ. Le plan de masse dessine les modules, la vue 3D les modules avec leurs cadres
  et leurs boîtes de jonction, les rails sur leurs crochets de toit, les supports et
  les déflecteurs de vent.
  *Afficher › Masquer les modules photovoltaïques* omet les modules pour montrer le
  montage, *Afficher › Modules photovoltaïques transparents* les montre transparents
  avec le câblage en dessous. Le montage
  a un **point de mise à la terre** avec une pince de mise à la terre
  (`GEN.PV-EARTH-CLAMP`, dans la nomenclature) sur chaque rail ou rangée de supports ;
  son **conducteur de mise à la terre** (`earthing` : l'appareil d'une barre de terre
  ou d'équipotentialité) les relie et va jusqu'à cette barre. La vue 3D dessine le
  **câblage** de chaque chaîne : les câbles de chaque module : son câble plus (rouge)
  et son câble moins (noir) sortant de sa boîte de jonction, le câble plus d'un module
  rejoignant le câble moins du suivant à un connecteur sous les modules, et le câble
  moins du premier module et le câble plus du dernier le long du toit, à travers
  celui-ci à côté du champ et, en dessous, jusqu'au **coffret de raccordement du
  générateur** du champ (`combiner`, un appareil tel que `GEN.GAK-2S`)
  et de là un câble principal CC (plus et moins) vers chaque onduleur (sans coffret de raccordement du générateur,
  directement jusqu'à l'onduleur), et le conducteur de mise à la terre (vert-jaune)
  jusqu'à sa barre, pour autant que les appareils soient placés dans l'implantation. Un clic sur un champ dans la vue 3D ouvre ses
  **propriétés** : où il se trouve, ses modules avec leurs données électriques, leur
  puissance crête et leur surface, son montage avec le matériel et la mise à la terre,
  ses chaînes avec leurs tensions, leurs courants et la longueur de leurs câbles, ses
  problèmes, et les réglages du volet Bâtiments. Une salle sous le toit affichée seule
  montre aussi les champs situés au-dessus d'elle et les fenêtres de toit de
  l'extérieur. Un **clic sur un toit** dans la vue 3D montre le toit
  dans les *Propriétés de l'implantation* : sa forme, son égout, sa pente, ses
  fenêtres de toit et ses champs photovoltaïques ; une fenêtre de toit ou un champ
  photovoltaïque ajouté là se place à l'endroit où le toit a été cliqué, et le nouveau
  champ est sélectionné avec ses propriétés.
- **Les champs photovoltaïques au sol** (*Champs photovoltaïques au sol* dans le
  volet : *Champ photovoltaïque* devant ce qui se trouve dans l'installation, *Placer
  sur le plan de masse* à l'endroit cliqué ; `add_pv_array`
  et `set_pv_array` sans `building` ni `roof`) : des champs montés au sol sur le
  terrain, leur milieu dans l'installation. Ils sont disposés en rangées de **tables**
  orientées dans la direction donnée (25° sauf indication contraire) : `stack` modules
  superposés sur une table (2), le bord inférieur à `clearance` au-dessus du sol
  (800 mm), deux rails sous chaque rangée de modules sur des poutres de table, chaque
  poutre sur un pieu battu avant et un pieu battu arrière espacés au plus de la
  distance entre pieux le long de la table (`span`, 2,5 m), les tables assez éloignées
  pour ne pas se faire d'ombre avec un soleil d'hiver à 17° (ou à la distance entre
  rangées donnée). Le système générique est le pieu `GEN.PV-POST-RAM` avec la poutre
  de table `GEN.PV-TABLE-BEAM`, les rails, raccords de rails, pinces de modules et la
  pince de mise à la terre (sur le premier pieu avant de chaque table) comme
  accessoires ; un système de montage de la base d'articles (sous-groupe *PV ground
  post*) apporte les siens. Les modules et le montage vont dans
  la nomenclature au nom du terrain ; le plan de masse et la vue 3D de tout le site
  les dessinent, un clic sur l'un d'eux dans l'un ou l'autre ouvre ses propriétés.
  Leurs chaînes sont celles des champs sur les toits ; leurs câbles (et le conducteur
  de mise à la terre) courent au dos des modules, descendent dans une tranchée de
  câbles de 0,7 m de profondeur et remontent jusqu'au coffret de raccordement du
  générateur, à l'onduleur ou à la barre. `plot` et `list_buildings` les listent avec
  le terrain ; les contrôles signalent les champs au sol qui se chevauchent, qui sont
  sur une salle ou une armoire, et hors de la limite du terrain.
- **Les chaînes photovoltaïques** (*Chaîne* sous un champ photovoltaïque dans le
  volet, `add_pv_string`, `set_pv_string`, `pv_strings`) : les modules d'un champ en
  série sous forme de chaînes, plusieurs chaînes de même type en parallèle, chacune
  sur un tracker MPP d'un onduleur ou d'un régulateur de charge (un appareil dont
  l'article a des données d'entrée photovoltaïque : `Max. PV input voltage`, `MPPT
  voltage range`, `MPP trackers`, `Max. input current per MPPT`, `Max. short-circuit
  current per MPPT` ; les onduleurs génériques `GEN.INV-PV-10KW` et `GEN.INV-PV-40KW`,
  les Victron Inverter RS Smart Solar et SmartSolar MPPT les ont). Des valeurs de la
  fiche technique du module (`Open-circuit voltage`, `Short-circuit current`,
  `Voltage` / `Current at maximum power`, les coefficients de température de Voc, Isc
  et Pmax) et des températures de cellule de l'étude (-10 / 25 / 70 °C sauf indication
  contraire) découlent la **tension la plus élevée** (circuit ouvert à la température
  la plus basse), la tension **moyenne** et la **tension la plus basse** (à la
  puissance maximale, à la température moyenne et à la plus élevée, avec le
  coefficient de la puissance), le courant à la puissance maximale, le courant de
  court-circuit à la température la plus élevée et la puissance. Le rapport *Chaînes
  photovoltaïques* liste chaque chaîne avec ces valeurs, la puissance totale et les
  températures. Le contrôle signale les chaînes dont la tension la plus élevée dépasse
  la plus grande tension d'entrée de leur onduleur ou la tension système de leurs
  modules, les chaînes en dessous ou au-dessus de la plage MPP de leur tracker, les
  trackers que l'onduleur n'a pas, les trackers dont les chaînes dépassent ensemble
  leur courant de court-circuit ou d'entrée ou diffèrent en tension, et les champs
  ayant plus de modules dans des chaînes qu'ils n'en comptent ou des modules dans
  aucune chaîne.
- **Les prises de terre** (*Mise à la terre et protection contre la foudre* dans le
  volet, `add_earth_electrode`, `set_earth_electrode`) : une **prise de terre à fond
  de fouille** dans le radier du niveau le plus bas, une boucle fermée à 100 mm à
  l'intérieur des murs extérieurs, divisée par des feuillards transversaux en mailles
  de 20 × 20 m au plus (`GEN.ER-STRIP-FE30`, ses écarteurs tous les 2 m et ses
  raccords en croix), une **prise de terre en boucle** dans le sol à un mètre à
  l'extérieur des murs et à un demi-mètre de profondeur (`GEN.ER-STRIP-V4A30`), et des
  **piquets de terre** à un emplacement du plan, enfoncés à partir d'un demi-mètre de
  profondeur (éléments `GEN.ER-ROD-1500`, pointe et collier). Chaque prise de terre a
  un **matériau** (acier galvanisé à chaud, acier inoxydable V4A, cuivre, acier
  cuivré) et un **diamètre** (d'un piquet, ou d'un fil rond à la place d'un
  feuillard) ; son article est celui de la base d'articles ou du catalogue ayant ce
  matériau et ce diamètre (d'après les données techniques `Material` et `Diameter`),
  sauf si un article est choisi. Les murs extérieurs sont le rectangle que couvrent les toits,
  sinon celui des salles. Une prise de terre reliée à une barre principale de terre
  (un appareil) monte jusqu'à elle avec une patte de raccordement. Tout cela va dans
  la nomenclature.
- **La protection contre la foudre** (`set_lightning_protection`, `add_air_rod`,
  `set_air_rod`) : de classe I à IV, un **dispositif de capture** formé de conducteurs
  sur les faîtages et les arêtes des toits en pente, ainsi qu'en périphérie et en
  maillage (5, 10, 15 ou 20 m selon la classe) sur les toits plats et en appentis, et
  des **pointes de capture** qui y sont reliées ; des **conducteurs de descente**
  autour des murs extérieurs, espacés au plus de 10, 10, 15 ou 20 m (au moins deux, ou
  autant qu'indiqué), chacun de sous l'égout jusqu'à son **joint de contrôle** à un
  mètre au-dessus du sol, puis dans le sol jusqu'à la prise de terre en boucle (sinon
  la prise de terre à fond de fouille ou les piquets de terre), le conducteur de
  toiture de l'égout, par-dessus le toit, jusqu'au dispositif de capture. Le
  **matériau** et le **diamètre** des conducteurs et de chaque pointe déterminent
  leurs articles, la **quincaillerie de montage** est constituée des supports de
  conducteur sur le toit et sur les murs (supports de toiture, de faîtage ou de toit
  plat, supports muraux avec ou sans entretoise) à une distance qui leur est propre,
  une pointe se tient dans un support de pointe ou sur un socle en béton ; la hauteur
  des joints de contrôle se règle aussi. Le fil rond, ses supports, les pointes avec
  leurs supports ou socles, les joints de contrôle, les entrées de terre et les
  colliers vont dans la nomenclature. Un clic sur une prise de terre, un conducteur ou
  une pointe dans la vue 3D ou le plan de masse ouvre ses **propriétés** avec tous ces
  réglages. Les champs photovoltaïques sur les toits doivent se trouver sous sa
  protection selon l'**angle de protection** de la classe (70 - 79° jusqu'à 2 m
  au-dessus de la surface, jusqu'à 23° au rayon de la sphère fictive). La vue 3D et le
  plan de masse dessinent les prises de terre et le système. *Terrain › Mise à la
  terre et protection contre la foudre* les ajoute à un bâtiment : prises de terre à
  fond de fouille et en boucle, piquets de terre et pointes de capture cliqués dans le
  plan de masse, protection contre la foudre avec sa classe.
- **Les liaisons entre bâtiments** (`add_site_link`, `set_site_link`) : une tranchée
  de câbles, un fourreau ou un pont de câbles allant d'une entrée de bâtiment d'une
  salle à une entrée de bâtiment d'une salle d'un autre bâtiment, par des coins dans
  l'installation. Les parcours les empruntent comme une traversée de mur ; leur
  longueur est donnée ou découle du chemin (jusqu'à 800 mm sous le sol pour une
  tranchée ou un fourreau, à la hauteur de l'entrée la plus haute pour un pont).

Le contrôle *Bâtiments, niveaux et gaines* signale les salles sur des niveaux
disparus, les salles plus hautes que leur niveau ou qui se chevauchent sur un niveau,
les niveaux qui se chevauchent, les gaines ne reliant pas deux salles, les armoires et
articles posés au sol qui se trouvent dans une gaine, les liaisons dont les entrées de
bâtiment ont disparu, les escaliers sans niveaux valides, avec une hauteur de marche
hors de 140 - 200 mm ou enfreignant la règle du pas (deux fois la hauteur de marche
plus le giron 590 - 650 mm), les fenêtres de toit hors de leur versant ou à cheval sur
un faîtage ou une arête, les salles dans lesquelles le toit descend jusqu'au sol, les
armoires plus hautes que le plafond rampant à leur emplacement et les éléments sur les
murs ou le plafond au-dessus du rampant, les champs photovoltaïques hors de leur toit
ou à cheval sur un faîtage, une arête ou son bord, inclinés sur un toit en pente,
au-dessus d'une fenêtre de toit ou les uns sur les autres, les champs photovoltaïques
au sol qui se chevauchent, qui sont sur une salle ou hors de la limite du terrain, et
les champs photovoltaïques dont l'appareil manque ou compte un autre nombre de modules
ou dont la barre de terre manque, les prises de terre dont la barre manque, les
piquets de terre à l'intérieur des murs, une protection contre la foudre sans prise de
terre ou avec moins de conducteurs de descente que sa classe n'en exige, les pointes
de capture hors des toits et les champs photovoltaïques hors de la protection du
dispositif de capture. L'exemple 15 (`plant_buildings`) montre un bâtiment principal
avec deux niveaux et une gaine, un local de pompage relié par une tranchée de câbles
dans laquelle est routé un câble moteur, des luminaires de façade et une station météo
sur le toit ; l'exemple 16 (`office_building`) ajoute un immeuble de bureaux avec des
combles sous un toit à deux pans, un escalier tournant sur un palier d'angle, des
fenêtres de toit, un champ photovoltaïque de dix modules LONGi sur rails et crochets
de toit dans le versant arrière avec son onduleur et la barre d'équipotentialité de
son montage sur le mur pignon des combles, une prise de terre à fond de fouille sur la
barre principale de terre, une prise de terre en boucle et un système de protection
contre la foudre de classe III avec une pointe de capture, des prises et interrupteurs
Gira et MDT et un collecteur de circuits de chauffage COSMO.

**En 3D**, *Espace de montage › Afficher › Bâtiment entier* montre le bâtiment de la
salle affichée avec tous ses niveaux et salles, leurs dalles, les escaliers et les
toits ; *Toutes les armoires* montre ainsi chaque bâtiment de l'installation. Les
toits et les murs pignons sont vus de l'extérieur comme les murs (translucides, ou
omis avec *Masquer les murs de devant*) ; *Masquer le plafond* omet les plafonds et
les toits pour regarder dans le bâtiment. L'installation en 3D dessine la limite du
terrain sur le sol et le point de référence sous forme de poteau.

![Plan de l'exemple electrical_room : salle en L, mur intérieur, porte, fenêtre et les deux armoires](../../images/room.png)

## Plans sur papier

Les plans d'installation électrique sont dessinés sur le dessin du bâtiment fourni par
l'architecte. Un **plan sur papier** se place sous un niveau (volet *Bâtiments*, onglet
*Plan sur papier*, choisir le niveau du plan de masse, *Plan…* ; ligne de commande
`import_floor_plan`) : une image (PNG, JPEG,
TIFF, GIF, WebP, BMP) ou une page d'un fichier PDF, un scan (y compris ceux des
scanners de bureau, qui découpent une page en une image de fond et des masques de
télécopie en noir et blanc) ou un dessin (ses lignes et ses surfaces remplies ; les
textes ne sont pas dessinés). Pour un fichier PDF de plusieurs pages, la page se
choisit dans le volet. Le plan de masse le montre sous les salles du niveau, et le
plan de chaque salle du niveau le montre sous la salle à sa place
(*Afficher › Plan sur papier* dans le ruban de l'espace de montage, activé par
défaut ; le sol de la salle est alors dessiné transparent) ; l'image est conservée dans le projet sous forme de PNG en niveaux de gris (la crate `openecraft-scan` lit les
fichiers, `openecraft_core::panel::floorplan` contient le reste).

- **Échelle et emplacement** (*Échelle*, *Résolution*, *Coin supérieur gauche*,
  *Rotation*, *Affiché* ; commande `set_floor_plan`) : l'échelle du dessin (1:100 pour
  un nouveau) et la résolution du scan (tirée de la page PDF, 150 dpi pour une image)
  donnent la taille d'un pixel dans le bâtiment. *Mesurer* (commande
  `calibrate_floor_plan`) fixe l'échelle à partir de deux points cliqués dans le plan
  de masse et de leur distance, une cote du dessin. Le coin supérieur gauche de
  l'image se trouve à un point du plan du bâtiment, l'image étant tournée autour de
  lui comme une salle.
- **Zone** (*Choisir*, `area` de `set_floor_plan`) : la partie de l'image dans
  laquelle les salles sont recherchées, sans cartouche, coupes ni autres dessins de la
  feuille ; affichée en tirets.
- **Détecter les pièces** (commande `detect_floor_plan`) trouve les murs, portes,
  fenêtres et salles dans le plan et les montre dans le plan de masse : les salles
  entourées, les portes dans la couleur des avertissements, les fenêtres en bleu, le
  débattement d'une porte sous forme d'un point à sa charnière. Rien ne change encore.
- **Créer les pièces** (commande `trace_floor_plan`) crée sur le niveau une salle pour
  chaque salle trouvée (`+ROOM1`, …) : son contour suit les faces intérieures de ses
  murs, elle est aussi haute que le niveau moins sa dalle, à sa place dans le
  bâtiment ; une porte va dans la salle vers laquelle elle s'ouvre (son débattement
  étant dessiné) ou dans la plus petite des deux salles, articulée du côté dessiné ;
  une fenêtre dans la salle située derrière elle. Les salles dont le contour n'est pas
  un contour de murs à angle droit sont signalées et laissées de côté. Le plan reste
  sous le niveau pour y comparer les salles et dessiner dessus ; *Retirer* l'enlève,
  les salles restent.

**Comment les salles sont trouvées** (`openecraft_core::wallscan`, pur traitement
d'image, sans modèle) : des murs dessinés pleins, ombrés ou hachurés (comme on dessine
les murs coupés), à angle droit par rapport aux bords de la feuille.

1. *Encre* : un pixel nettement plus sombre (10 niveaux) que le papier qui l'entoure
   (le niveau le plus clair dans l'épaisseur du mur le plus épais, mesuré sur la
   feuille réduite ; les scans pâlissent et jaunissent de façon inégale), après un
   filtre médian 3 × 3 qui supprime les taches.
2. *Murs* : l'encre soumise à une ouverture morphologique de la taille du mur le plus
   mince (90 mm) : le texte, les lignes de cote, les débattements de portes et les
   lignes des fenêtres disparaissent, les murs pleins restent. De ce qui reste, seules
   les grandes parties éloignées des bords de la feuille sont des murs.
3. *Tronçons droits* : des segments en travers d'un mur (une colonne d'un mur le long
   de x) aussi minces qu'un mur, qui gardent leur position sur au moins deux fois leur
   épaisseur ; les segments qui débordent d'un tronçon (un mur qui le rejoint, une
   tache de crayon sur son côté) le prolongent.
4. *Ouvertures* : depuis l'extrémité libre d'un tronçon (non là où un autre tronçon le
   rejoint), un rayon dans son prolongement, sur les lignes de son milieu, jusqu'au
   mur suivant : aussi long qu'une porte ou une fenêtre (0,5 … 5 m) sur la plupart des
   lignes. S'il aboutit à un mur transversal (une porte à côté d'un coin), il mesure
   au plus 1,5 m ; il n'est plus large que jusqu'au tronçon suivant du même mur (une
   fenêtre entre deux tronçons). Une ouverture dont les deux côtés sont de toute façon
   reliés (une tache dans une salle) n'en est pas une.
5. *Pièces* : les surfaces libres délimitées par les murs avec les ouvertures fermées
   (celles qui atteignent le bord de la feuille sont à l'extérieur), d'au moins 1 m²
   (*Plus petite pièce*, paramètre de commande `min_room`, conservé avec le plan : un WC ou un placard est plus petit). Leur contour : découpé le long
   des lignes où la bordure court sur au moins 150 mm, les cellules de ces découpes
   situées en majeure partie dans la salle composent le plan (murs à angle droit,
   aucun décrochement plus court que 75 mm).
6. *Portes et fenêtres* : une ouverture entre deux salles est une porte ; une
   ouverture vers l'extérieur est une fenêtre lorsque des lignes la traversent le long
   du mur (vitrage, cadre, appui), sinon une porte. Un quart de cercle d'encre partant
   d'une extrémité d'une porte vers une salle est son débattement.

Sur les plans scannés d'une maison de 1972 (1:50, 150 dpi, crayon), il trouve les
salles du rez-de-chaussée avec leurs portes et fenêtres en une demi-seconde ; les murs
dessinés par deux traits sans remplissage, les murs obliques ou courbes et les feuilles
scannées de travers ne sont pas encore trouvés. Les salles trouvées sont un point de
départ à vérifier et à corriger : des salles reliées par une ouverture sans porte
dessinée forment une seule salle, et les escaliers ou cheminées sont des murs.

## Installation : prises, interrupteurs, luminaires et leurs circuits

Le plan d'installation d'un niveau se dessine dans le volet *Bâtiments* (onglet
*Installation électrique*, choisir le niveau du plan de masse) sur son plan ; plugin
`installation`, `openecraft_core::installation`. Tout ce qui le compose se trouve dans
le schéma : les appareils, leurs liaisons et leurs câbles ; les circuits sont lus à
partir de là, et les longueurs de leurs câbles sont celles du routage 3D.

- **Insertion des appareils** (*Insérer* avec un appareil de protection et un article
  choisis ; commande `place_installation`) : un clic dans une salle du plan de masse y
  place une prise de courant, un interrupteur, un luminaire ou un élément chauffant
  pour l'appareil de protection (un disjoncteur, un disjoncteur différentiel ou un
  fusible du réseau jusqu'à 32 A dans le schéma ; la liste marque ceux qui sont en
  réserve, commande `list_breakers`) : un plafonnier (`GEN.LAMP-CEILING-LED`,
  `GEN.LAMP-DOWNLIGHT-LED`, articles avec *Ceiling mounting: yes*) au plafond à
  l'endroit cliqué, tout le reste sur le mur le plus proche du clic (à moins de 0,8 m)
  à sa hauteur de pose (prises 300 mm, interrupteurs 1050 mm, appliques 2000 mm). Il
  devient un appareil du schéma dans sa salle (`+ROOM1-XD1` prises de courant, `-SF1`
  interrupteurs, `-EA1` luminaires, `-EB1` éléments chauffants) avec l'article,
  dessiné sur la **page d'installation** de l'appareil de protection : une page de
  schéma après la page de l'appareil de protection, créée avec son premier appareil,
  dont le point de connexion de départ reçoit le point d'interruption `L-F5` vers elle
  (pour un appareil tripolaire, `L1-F5` … `L3-F5` à ses trois points de connexion de
  départ). La page dessine les conducteurs de phase à partir de ces points
  d'interruption et les conducteurs de neutre et de protection à partir des potentiels
  `N-F5` et `PE-F5` sous forme de lignes le long de son bord supérieur, chaque
  appareil dans une colonne en dessous.
  Chaque appareil reçoit un câble `-W<n>` depuis l'appareil qui le précède (le premier
  depuis l'appareil de protection, un luminaire depuis le premier interrupteur de sa
  salle ou le dernier luminaire derrière lui), ou depuis l'appareil ou la boîte de
  dérivation choisi (*Câble depuis* ; paramètre `from`) : un circuit s'y ramifie. Ses
  fils sont les conducteurs (BN, BU, GNYE ; en triphasé BN, BK, GY, BU, GNYE) d'un
  `NYM-J 3x1.5` (`5x`), ou plus gros lorsque l'appareil de protection ou un appareil
  du circuit l'exige (courant admissible dans les conditions d'installation). Le câble
  conserve sa source et sa cible (`cable.source`, `cable.target`) : les liaisons du
  circuit, les tronçons de la liste et les extrémités des parcours 3D les suivent, les
  colonnes de la page suivent l'arborescence des câbles. Dans un réseau dont les fils
  appartiennent à plusieurs câbles, chaque liaison prend le câble des fils à ses
  propres points de connexion.
- **Circuits triphasés** : un appareil de protection tripolaire alimente des prises
  CEE (`GEN.SOCKET-CEE-16-5P`, `-32-5P`), des boîtes de raccordement pour cuisinière
  (`GEN.CONN-BOX-COOKER-UP`) et des éléments chauffants triphasés
  (`GEN.HEATER-SAUNA-9000`) ; un appareil avec L2 ou L3 est refusé sur un circuit
  monophasé.
- **Boîtes de dérivation et boîtes de raccordement d'appareils** (`GEN.JBOX-UP`,
  `GEN.JBOX-5-UP`, `GEN.CONN-BOX-UP` ; onglet *Boîtes*) : une boîte de dérivation est
  un appareil du circuit d'où partent les câbles d'autres appareils ; une boîte de
  raccordement d'appareil alimente un équipement fixe. Une boîte de dérivation n'est
  pas une charge dans le dimensionnement des câbles : un câble qui s'y termine porte
  la plus grande charge des câbles qui en partent (d'après leur source et leur cible,
  que `place_installation` enregistre ; un câble dessiné à la main n'en transmet
  aucune), calculée en monophasé pour une charge monophasée derrière une alimentation
  à cinq conducteurs.
- **Circuits va-et-vient** : un va-et-vient (`GEN.SW-2WAY-UP`) placé `from` un autre
  est dessiné debout dans la colonne suivante, les deux navettes de leur câble allant
  de l'un à l'autre ; pour davantage de points de commande, des permutateurs
  (`GEN.SW-INTER-UP`) s'intercalent entre eux, chacun `from` l'interrupteur précédent,
  dessiné sur sa page (refusé lorsque cette page est pleine). Un luminaire sans `from`
  est raccordé au dernier interrupteur, qui reprend aussi les luminaires placés sur
  les interrupteurs précédents ; les autres appareils alimentés par eux restent. Un
  va-et-vient placé après une paire fermée (deux va-et-vient) commence un circuit
  propre, même avec `from` l'un de la paire. Les câbles entre les
  interrupteurs ont quatre conducteurs : les deux navettes, et le neutre et le
  conducteur de protection prolongés jusqu'aux luminaires. Un va-et-vient fonctionnant
  seul a sa navette libre dans *Points de connexion non utilisés*
  (`function.unused_pins`) : un point de connexion qui y est nommé n'est pas signalé
  comme ouvert. Les navettes d'un permutateur sans dernier interrupteur restent
  ouvertes. Un interrupteur de volets roulants (`GEN.SW-BLIND-UP`) a deux sorties
  réelles et n'est pas un va-et-vient.
- **Sans circuit** : les prises réseau et d'antenne (`GEN.DATA-RJ45-2X-UP`,
  `GEN.TV-SAT-UP`) et les détecteurs de fumée (`GEN.SMOKE-ALARM`, au plafond) sont
  seulement placés dans les salles (`place_installation` sans `breaker` ; avec `tag`
  comme cet appareil), affichés dans le plan de masse et listés avec les éléments des
  salles.
- **À l'extérieur du bâtiment** : un point hors de la salle place l'article sur la
  face extérieure de son mur le plus proche à sa hauteur de pose (une prise
  extérieure, un luminaire de façade) ; les appareils
  du jardin vont dans
  une zone extérieure, une salle placée hors du bâtiment (`place_on_level` sans
  bâtiment).
- **Ce que `place_installation` accepte en outre** : le repère d'appareil (`tag`), la
  hauteur de pose (`height`, mm), le repère, le type, l'article, le nombre de
  conducteurs, la section et une longueur connue du câble (`cable`, `cable_type`,
  `cable_part`, `cores`, `cross_section`, `length`) ; une longueur connue est
  conservée (*Longueur connue*, `cable.fixed_length`) et prise à la place du parcours
  3D. `place_cabinet_device` place les appareils de protection dans le tableau de
  distribution, dont les circuits ont besoin pour leurs longueurs.
- **Dans la vue 3D et les vues de murs** (*Insertion › Prises, interrupteurs,
  chauffage*) : le menu choisit l'appareil de protection en haut (ou *Aucun : pas dans
  le schéma*) ; l'article choisi est placé par un clic sur un mur ou le plafond de la
  salle dans la vue 3D (sur un mur à sa hauteur de pose ; Échap arrête), ou sur le mur
  dans une vue de mur, et va dans le schéma comme avec *Insérer*. La liste du menu est
  aussi haute que ses articles, au moins autant que l'article affiché à côté, au plus
  un peu plus de la moitié de la fenêtre.
- **Le plan de masse** dessine les appareils avec les symboles des plans
  d'installation (IEC 60617-11) dans la couleur de leur circuit, ainsi que les
  liaisons des circuits ; en 3D, ce sont les modèles des articles (des modèles générés
  pour les articles génériques et les articles sans modèle propre : prises et
  interrupteurs dans leurs cadres, plafonniers ronds avec leur diffuseur bombé, spots
  encastrés avec collerette et diffuseur, luminaires rectangulaires muraux, à panneau
  et réglettes, blocs de secours avec leur pictogramme, convecteurs avec grille et
  thermostat, sèche-serviettes avec leurs barreaux, panneaux chauffants infrarouges).
  Le menu 3D *Insertion › Prises, interrupteurs, chauffage* a aussi un onglet
  *Luminaires*.
- **Les éléments chauffants** s'insèrent de la même façon : convecteurs
  (`GEN.HEATER-PANEL-1000`, `GEN.HEATER-PANEL-2000`, 400 mm) et sèche-serviettes
  (`GEN.HEATER-TOWEL-500`, 900 mm) sur le mur, panneaux chauffants infrarouges
  (`GEN.HEATER-IR-600`) au plafond ; articles des sous-groupes *Room heater*, *Towel
  radiator*, *Infrared heating panel* ou ayant une fonction d'élément chauffant pour
  murs ou plafonds (les chauffages d'armoire n'en sont pas) (*Insertion › Prises,
  interrupteurs, chauffage*, onglet *Chauffage*).
- **Circuits du schéma** (lecture seule ; commande `list_circuits`, rapport *Circuits
  d'installation*) : chaque appareil de protection du réseau jusqu'à 32 A avec les
  prises de courant, interrupteurs, luminaires et éléments chauffants situés derrière
  lui : sur les réseaux derrière l'appareil de protection, à travers les
  interrupteurs, contacts, bornes et contacts principaux des contacteurs, mais pas à
  travers d'autres appareils de protection ou des charges. Un appareil compte d'après
  son article (un article pour murs ou plafonds) ou, sans articles, d'après son
  symbole (prises de courant, éléments chauffants, lampes, luminaires, symboles
  d'installation ; contacts d'un appareil dont la lettre repère commence par `S`) ;
  les appareils placés dans une armoire sont laissés de côté. La liste montre pour
  chacun les appareils placés, ses câbles, la longueur et les chutes de tension ; un
  clic sur un circuit choisit son appareil de protection.
- **Placement des appareils du schéma** (*Tout placer* ou un clic sur un appareil du
  circuit choisi ; commande `place_circuit_device`) : les appareils dessinés à la main
  dans le schéma sont placés par un clic dans une salle comme un article (avec leur
  propre article, l'article choisi s'il est de leur type, ou l'article générique de
  leur type, qui devient leur article) ; un clic sur une prise de courant, un
  interrupteur, un luminaire ou un élément chauffant placé auparavant sans repère
  d'appareil fait de celui-ci l'appareil. Un appareil placé de nouveau se déplace ;
  l'appareil suivant non encore placé suit.
- **Longueur de câble** : chaque liaison des conducteurs de phase d'un circuit entre
  deux appareils, un câble (une fois) ou un fil isolé, a la longueur de son parcours
  3D (routage dans les salles, à travers les murs et entre les niveaux,
  `crate::routing3d`). Tant qu'elle n'est pas routée, elle est estimée (affichée avec
  `~`) : du haut du tableau à l'appareil dans la zone d'installation horizontale à
  300 mm sous le plafond (DIN 18015-3), le long de x et de z de l'installation, en
  montant et en descendant dans les zones verticales des appareils, en montant ou en
  descendant entre les niveaux, 0,3 m pour chaque boîte. *Écrire les longueurs de
  câble* (`write_circuit_cables`) écrit les longueurs (arrondies au demi-mètre
  supérieur) dans la liste des câbles.
- **Chute de tension depuis l'alimentation** (*Depuis l'alimentation* ;
  `total_drop_percent` de `list_circuits`) : par liaison
  ΔU = k · I · L · (ρ / A · cos φ + X' · sin φ) (k = 2 en monophasé, √3 en triphasé,
  ρ du cuivre à sa température de service des paramètres électriques,
  X' = 0,08 mΩ/m, cos φ 0,95), additionnée le long des liaisons de l'appareil de
  protection jusqu'à l'appareil le plus éloigné. Les câbles du circuit portent le
  courant d'emploi de son appareil de protection (bilan de puissance) ou son courant
  assigné ; les câbles d'alimentation allant de l'alimentation au tableau de
  distribution (ceux que suit le calcul de court-circuit, avec leurs longueurs dans la
  liste des câbles) le courant d'emploi de l'appareil de protection amont ou son
  courant assigné. La somme est vérifiée par rapport à la limite des jeux de règles
  (*Chute de tension*) ; l'infobulle indique la part de l'alimentation.
- **Contrôle** *Circuits d'installation* : circuits dont l'appareil de protection ne
  se trouve dans aucun tableau de distribution de l'implantation, câbles plus fins que
  ne l'exige leur appareil de protection (courant admissible dans les conditions
  d'installation), chutes de tension supérieures à la limite, appareils d'un circuit
  placés dans aucune salle (information) ; prises de courant, interrupteurs,
  luminaires et éléments chauffants des salles qui ne sont pas dans le schéma
  (information).

## Coffrets de branchement

Le système d'armoire *Coffret de branchement* (`house_connection`, commande
`add_enclosure` avec `system` `house_connection`) est le coffret du distributeur à
l'entrée de bâtiment : polyester renforcé de fibres de verre (RAL 7035, IP54) avec un
couvercle articulé et plombable et une plaque de montage, en deux tailles standard
(`GEN.HAK-255X415X144` pour NH00, `GEN.HAK-410X720X185` pour NH2). Son couvercle
s'ouvre dans la vue 3D comme la porte de n'importe quelle armoire. À l'intérieur vont
le socle de fusibles NH (`GEN.NH2-BASE-3P-400A`, alimenté par le bas : le câble
d'alimentation monte tout droit jusqu'à son côté amont à travers un presse-étoupe dans
le fond, le câble principal sort tout droit par le haut) et le bloc de bornes PEN
(`GEN.TB-PEN-240-V`). L'exemple `building_services` en accroche un au mur au-dessus de
l'entrée de câbles, avec la barre principale de terre (`GEN.MET-BAR-10`) et la patte
de la prise de terre à fond de fouille (`GEN.FE-LUG-V4A-10`, symbole *Prise de terre*)
à côté.

## Armoires de distribution à modules fonctionnels

L'article d'armoire d'un fabricant livré avec son socle l'indique dans ses données
(`Plinth: 200`) : une nouvelle armoire de cet article repose sur le socle, qui est
dessiné et exporté mais n'est pas un article à part dans la nomenclature (l'armoire
Hager univers `HAG.FR25S2`, 1850 mm + 200 mm de socle, cinq champs, classe de
protection II).
Sa plaque de montage est placée par `Mounting origin: 25/12.5/20` (le coin supérieur
gauche de l'avant de la plaque depuis la gauche, le haut et l'arrière de l'armoire,
mm ; la plaque garde la même distance à droite et en bas), comme le donnent les
paquets d'armoires du configurateur Rittal ([articles](parts.md)). Une plaque qui n'est pas centrée
donne aussi sa taille (`Mounting size: 500 x 575` ; `mounting_size` de
`add_enclosure` / `set_enclosure`). L'article d'armoire d'un fabricant
qui nomme son système (`Enclosure system: compact`) reçoit les parois et les données
de ce système mais reste l'article de l'armoire ; seules les armoires du catalogue
intégré suivent leur taille avec le corps du système.
Les systèmes de distribution construisent l'armoire à partir de modules fonctionnels :
un module sur lequel repose l'appareil est un accessoire monté *sous l'article*
(`HAG.UE31E1B` sous l'interrupteur-sectionneur à fusibles NH1 `HAG.LT153`,
`HAG.UE31SS400` sous les transformateurs de courant), de sorte que l'appareil prend
l'encombrement et la profondeur du module et que l'article se trouve devant lui.
L'exemple `pv_transformer_metering` construit une telle armoire champ par champ
d'après une armoire réelle : zone du point de raccordement et parafoudre, comptage de
la consommation avec sectionnement NH1 sur le jeu de barres, module de transformateurs
au-dessus, circuits de tension et interrupteur-sectionneur (la puissance circule vers
le haut à travers le champ : les conducteurs vont d'un appareil tout droit dans le
suivant), la protection de découplage (NA), le comptage de la production sous
l'interrupteur de couplage et les jeux de barres des départs ; des anneaux de levage
dans le toit, les câbles des consommateurs à travers le fond, l'alimentation par le
bas ; les deux armoires de comptage du gestionnaire du réseau de distribution avec
leurs blocs de bornes d'essai à côté. Les brides d'entrée de câbles
(sous-groupe d'articles `Cable entry`, une membrane recevant plusieurs gros câbles,
comme l'univers FZ402) sont des accès pour le routage 3D comme les presse-étoupes.
L'exemple `industrial_main_distribution` juxtapose deux armoires univers, ouvertes
l'une vers l'autre, pour former la distribution principale d'un site industriel : les
câbles d'alimentation en aluminium sur des bornes pour jeu de barres et à travers une
bride d'entrée dans le fond, les transformateurs du comptage dans le jeu de barres
d'alimentation, un jeu de barres de 185 mm traversant les deux armoires avec des
réglettes verticales d'interrupteurs-sectionneurs à fusibles NH, les transformateurs
de courant encliquetés derrière ceux qui sont mesurés, les interrupteurs de couplage
de la protection de découplage (NA) alimentés depuis des bornes pour jeu de barres.
Aucune des deux n'a de goulottes : chaque conducteur y est posé par le routage 3D, de
sorte que les forts courants passent par les barres et que seul ce qui peut être
courbé avec son rayon est un fil.

## Baies 19 pouces

Le système d'armoire *Baie 19 pouces* (`rack19`, *Espace de montage › Nouvelle
enveloppe*, commande `add_rack` ou `add_enclosure` avec `system` `rack19`) est un
châssis avec une porte perforée et deux paires de montants 19 pouces (IEC 60297) à la
place d'une plaque de montage ; ses tailles standard vont d'une baie murale de 12 U à
47 U, et les montants sont un article de la nomenclature (`GEN.RK-RAIL-42U`). Ses
unités de hauteur (1 U = 44,45 mm) sont numérotées à partir du bas à côté des montants
dans la vue de face et sur la page d'implantation.

- **Les pièces 19 pouces** sont aussi larges qu'un panneau 19 pouces (482,6 mm) et
  hautes d'un nombre entier d'unités (donnée technique `Rack units: 2`) : le catalogue
  comporte un commutateur réseau, un panneau de brassage, une PDU, un serveur, un
  onduleur, des panneaux obturateurs, une étagère et un panneau passe-câbles.
  *Insertion › Pièce 19 pouces* en monte une dans les unités libres les plus hautes de
  la baie affichée (commande `mount_in_rack` avec `unit`, comptée à partir du bas) ;
  placés ou glissés, les appareils et les pièces s'accrochent sur des unités entières,
  centrés, leur face avant sur les montants avant, et sont tenus par les montants (pas
  de trous dans une plaque).
- Les unités occupées deux fois sont des collisions ; le contrôle signale les pièces
  19 pouces hors des unités et, à titre d'avis, les autres articles sur le plan de
  montage (ils ont besoin d'une étagère ou d'une plaque de montage). Le rapport *Plan
  d'occupation de baie 19 pouces* (et `list_racks`) liste les pièces de chaque baie à
  partir du haut avec leurs unités et leurs profondeurs, ainsi que les unités libres.
- Les baies se juxtaposent en rangées et se placent dans des salles comme les autres
  armoires.

## Routage 3D

Les câbles, les tubes et (en option) les fils sont routés à travers les armoires
juxtaposées : dans les goulottes, le long de chemins de routage dessinés au-dessus de
la plaque de montage, ou librement autour des appareils ; ségrégation des classes
puissance, commande, signal et fluide, réservations de routage, rayons de courbure,
torons de câbles avec leurs dérivations, reprises de blindage (rail de colliers de
blindage, queue de cochon) et leurs contrôles ; les parcours peuvent être figés pour
la production et restent tels quels jusqu'à ce qu'ils deviennent invalides.

Les fils d'une armoire ayant des goulottes sur sa plaque de montage passent par les
goulottes (le paramètre d'implantation *Router les fils en 3D* les route plutôt comme
des câbles). Dans une armoire sans goulottes, et dans une salle, il n'y a rien pour
les y poser : chaque fil est routé en 3D de point de connexion à point de connexion.
Il quitte chaque point de connexion tout droit dans la direction que donne le point,
sur la longueur de son rayon de courbure, et arrive à l'autre à l'opposé de la
direction que donne celui-ci, sans jamais le dépasser pour revenir
(à travers
la boîte de son propre appareil lorsque le point se trouve à l'intérieur, comme les
bornes d'un interrupteur sur son module, jamais à travers un autre appareil placé
juste devant le point), garde la moitié de son épaisseur libre de tout, ne se courbe
qu'avec son
rayon, et reste à l'écart de tout autre parcours et de lui-même (lorsque ses
extrémités sont trop proches pour son rayon de courbure, il fait une boucle qui ne
recoupe ni ne longe jamais son propre chemin, les coudes à ses coins et le piquage
dont il sort ; lorsqu'une telle boucle n'existe pas, il n'est pas routé) ; les
parcours ne se rejoignent que là où ils arrivent au même appareil, et là ils sont côte
à côte : un parcours ne croise jamais un autre parcours qu'il rejoint (il ne le peut
que sur sa propre sortie d'un point de connexion, ou là où il arrive à l'appareil où
l'autre se termine, et il le contourne là aussi lorsque cela coûte peu). La sortie
droite de chaque point de connexion et de chaque entrée de câbles reste libre jusqu'à
ce que son propre parcours soit posé, afin qu'aucun autre parcours ne passe en travers
d'une borne, et seul un parcours sortant d'un point de connexion juste à côté la
partage. Parmi les fils d'une armoire de même épaisseur, le plus court est routé en
premier : entre deux appareils, les conducteurs intérieurs sont au plus près d'eux,
les extérieurs les contournent. Un fil qui ne trouve pas un tel chemin n'est pas
dessiné : il est signalé comme erreur (*… ne peut pas être routé : aucun chemin qui
garde la moitié de son épaisseur libre de tout et se courbe avec son rayon de … mm*).
Les causes habituelles sont un conducteur trop épais pour l'espace entre ses
extrémités (une section de jeu de barres dessinée comme un fil : montez plutôt les
appareils sur un jeu de barres), des points de connexion qui se tournent le dos, un
bornier juste sous un interrupteur, ou deux extrémités plus proches l'une de l'autre
que ne l'exigent les coudes entre elles (une entrée de câbles juste sous la boîte dans
laquelle elle mène, deux raccords l'un au-dessus de l'autre avec leurs orifices sur
les côtés). Le panneau des messages saute à un point de connexion du
fil ou du câble, et la vue 3D montre une note rouge avec la raison à chacune de ses
extrémités à la place du parcours.

Le routage travaille sur une grille (25 mm dans une armoire, 50 mm dans une salle) ;
un point de connexion se trouve là où il se trouve. La vue 3D dessine un parcours tout
droit à la sortie de son point de connexion : son premier coin se trouve sur cette
ligne, le tronçon suivant s'incline ou revient sur la grille par un segment oblique,
les courts tronçons intermédiaires sont décalés en conséquence. Lorsque la sortie
droite d'un point est barrée et que le parcours part de la cellule voisine, il lui
faut là de la place pour un coude en S de son rayon vers cette ligne (ou pour
s'incliner vers elle), sinon il n'est pas routé. Les chemins qui courent ensemble sont
côte à côte sur des voies ; un chemin change de voie sur un segment oblique d'un
tronçon, jamais par un décrochement à un coin, et un décalage trop court pour les
coudes de ses deux coins est dessiné comme une courbe en S du rayon de courbure. Un
câble se termine sur le coude du coin précédant la séparation de ses conducteurs
lorsqu'il y est coupé ; les conducteurs se séparent à l'intérieur de la boîte de leurs
extrémités, et un conducteur allant à une borne située derrière d'autres de son
bornier (l'une au-dessus de l'autre sur les barres) les dépasse par l'avant de
l'appareil. Lorsque deux parcours se croisent encore juste à un appareil auquel ils
arrivent tous deux, la vue fait passer l'un par-dessus l'autre plus brusquement que
leur rayon de courbure ne le permet.

Un câble entre deux armoires qui ne se trouvent dans aucune salle commune est routé
dans chacune d'elles jusqu'à l'une de ses entrées de câbles (une note indique que le
chemin entre elles ne fait pas partie de l'implantation) ; chaque câble prend sa
propre entrée lorsqu'il y en a assez qui lui conviennent. Les câbles passant par une
même bride d'entrée de câbles traversent sa membrane (donnée d'article `Cutout`) côte
à côte, chacun à sa propre place, espacés de l'épaisseur du câble plus un
centimètre : la place libre la plus proche de la direction que prend le câble, le
milieu lorsqu'aucune n'est libre.

Entre les salles, un parcours emprunte les traversées de mur, gaines et liaisons
entre bâtiments qui les relient, en en traversant le moins possible : à travers la
première salle jusqu'à une traversée, à travers les salles intermédiaires, à travers
la dernière salle jusqu'à son extrémité (ses tronçons par salle et par armoire
figurent dans la liste de routage) ; sa longueur inclut les chemins à travers les
murs, gaines et liaisons. Sans une telle traversée, le parcours
reste non routé (*aucune traversée de mur ne mène d'une salle à l'autre*). Un parcours
allant d'un appareil d'une salle vers l'extérieur du bâtiment se termine à son entrée
de bâtiment la plus proche, y compris depuis un appareil situé dans une armoire posée
dans la salle (à travers le presse-étoupe de l'armoire et la salle) ; parmi plusieurs
traversées de mur reliant les deux mêmes salles, le parcours prend celle dont le
chemin est le plus court.

Dans une salle, les parcours passent par son espace libre : autour de ses murs, de ses
murs intérieurs, des armoires qui s'y trouvent et du débattement de ses portes, de
préférence par ses goulottes et ses chemins de tuyauterie. Passer horizontalement dans
l'espace ouvert d'une salle, à plus de 300 mm de ses murs, de son plafond et de ce qui
s'y trouve, coûte dix fois plus : les tuyaux et les câbles longent les murs et le
plafond et descendent tout droit vers un appareil posé dans la salle.
Les câbles et les tuyaux ne tournent qu'après la moitié de leur rayon de courbure en
ligne droite (lorsqu'un tel chemin existe) ; parmi les chemins de coût à peu près
égal, un parcours prend celui dont les coudes gardent leurs distances (un court
tronçon en premier ou en dernier). Un parcours entre une armoire et un appareil de sa
salle (ou deux armoires d'une même salle) se fait par tronçons : à travers l'armoire
jusqu'à un presse-étoupe sur son toit, ses côtés ou son arrière (d'abord un de sa
propre armoire, sinon de la rangée ; à défaut, par le milieu du toit, ce qui est
signalé), puis à travers la salle. Un presse-étoupe reçoit un seul câble : celui qui
est libre et dont la plage de serrage convient au câble, le plus proche de la
direction que prend celui-ci ; une bride d'entrée de câbles en reçoit autant qu'elle a
de places le long de sa membrane.
Un câble qui trouve tous les presse-étoupes de son armoire occupés n'est pas routé et
est signalé comme erreur (*… chaque presse-étoupe de … est occupé par un autre câble.
Ajoutez un presse-étoupe pour lui*).
Une boîte de dérivation d'un fabricant marquée comme armoire (donnée d'article
`Junction box enclosure: yes`, l'Abox de Spelsberg) est placée sur un mur comme une
armoire de la salle, son arrière sur le mur, son milieu à la hauteur de pose, avec à
l'intérieur une borne de raccordement WAGO 221-415 qui devient l'appareil du schéma
(les câbles de son circuit entrent dans la boîte). Dans la vue 3D, un clic sur
l'intérieur d'un mur ou sur le plafond d'une salle y place un article : à moins de
200 mm de sa hauteur de pose, à cette hauteur (sa zone d'installation), plus loin à la
hauteur cliquée ; l'article est montré en transparence là où il irait pendant que le
pointeur se déplace.
Avec *Ajouter des presse-étoupes pour les nouveaux câbles* (propriétés de l'armoire,
`set_auto_glands` ; activé par défaut), après chaque modification des câbles ou du
schéma, un câble qui quitte une armoire pour un appareil situé hors de celle-ci et de
sa rangée et ne trouve aucun presse-étoupe libre qu'il puisse emprunter en reçoit un :
en polyamide, le plus petit dont la plage de serrage accepte le câble, à la première
place libre à partir du milieu de la face. Un câble vers un appareil de la salle où se
trouve l'armoire le reçoit dans le toit (il ne peut sortir que par le toit, les côtés
et l'arrière, voir ci-dessous ; pour une armoire accrochée à un mur, dans le fond), un
câble vers le terrain dans le fond. Lorsque cette face est pleine, le presse-étoupe va
dans la face suivante par laquelle le câble peut sortir : le fond ou le toit, puis les
côtés gauche et droit et l'arrière
(pas un côté contre l'armoire suivante de la rangée, ni les côtés d'une armoire de
moins de 150 mm de profondeur, dont les fils ne trouveraient aucun chemin jusqu'à
eux ; pas l'arrière d'une armoire accrochée au mur ou adossée à un mur de sa salle, et
son fond, pour un câble vers la salle, seulement en direction d'un emplacement situé
sous le fond de l'armoire avec son socle, comme le routage le quitte : l'appareil dans
la salle ou dans une armoire qui s'y trouve, sinon la première traversée de mur sur le
chemin de sa salle ; un appareil sur le toit est au-dessus). Un
câble qui ne trouve de place libre sur aucune d'elles n'en reçoit pas et est signalé (par *Ajouter ceux qui manquent* dans la barre
d'état, comme `glands_missing` d'une commande et `missing` de `add_cable_glands`,
chacun avec l'armoire et la raison) : faites de la place ou ajoutez une bride d'entrée
de câbles. Un presse-étoupe supprimé à la main reste supprimé jusqu'à ce que les
câbles changent ; *Ajouter ceux qui manquent* (`add_cable_glands`) ajoute un
presse-étoupe pour chaque câble qui n'en a pas, même avec l'option désactivée. Une
armoire avec une bride d'entrée de câbles n'en reçoit aucun. Une
armoire accrochée à un mur (son fond au-dessus du sol) se quitte aussi par un
presse-étoupe dans son fond : sous chacun de ces presse-étoupes, un puits descendant
jusqu'au sol et ressortant vers l'avant de l'armoire reste libre pour les parcours.
Figé, un tel parcours conserve ses tronçons.

**Encastré dans les salles.** Chaque câble, fil et tube est posé dans les salles *en
saillie* (comme ci-dessus), *encastré dans le mur* ou *encastré dans le sol* : comme
le disent les règles de routage (*Pose dans les salles*, commande `routing_rules` avec
`laying`) ou comme défini pour lui (propriétés du parcours › *Pose dans les salles*,
commande `set_laying` avec sa clé). Encastré, il traverse la salle dans des saignées
de ses murs, à la *Profondeur de saignée* (30 mm sauf indication contraire,
`slot_depth`) derrière leur surface moins la moitié de son épaisseur, le long des
lignes médianes des **zones d'installation** (règles de routage › *Zones
d'installation*, commande `installation_zones`) : des zones horizontales au-dessus du
sol et sous le plafond, coupées là où se trouvent les portes et les fenêtres et
reliées autour des coins de la salle, des zones verticales à côté des coins, des
portes et des fenêtres. Un appareil sur un mur est alimenté tout droit vers le haut ou
vers le bas depuis la zone horizontale la plus proche (ou latéralement depuis une zone
verticale dans laquelle il se trouve) ; une extrémité libre (un presse-étoupe sur le
dessus d'une armoire) va d'abord au mur le plus proche. Dans le sol, la ligne descend
tout droit le long du mur (ou, lorsqu'elle est basse au-dessus du sol, tout droit vers
le bas) puis, à la *Profondeur dans le sol* (60 mm, `floor_depth`) sous sa surface,
suit la zone dans le sol. Les zones
sont celles de la DIN 18015-3:2016 sauf modification (*Zones allemandes
(DIN 18015-3)* les réinitialise) :
ZW-o 150 - 450 mm sous le plafond, ZW-u 150 - 450 mm au-dessus du sol, ZW-m 1000 -
1300 mm au-dessus du sol pour les pièces avec plans de travail (désactivée), ZS-t /
ZS-f / ZS-e 100 - 300 mm à côté des portes, des fenêtres et des coins, ZD-r 200 -
500 mm des murs dans le plafond (affichée, non empruntée par le routage) et ZF, la
même dans le sol (la norme n'a pas de zones dans le sol). Un parcours que les zones ne
relient pas reste non routé (*aucun chemin par les zones d'installation*). *Afficher ›
Zones d'installation* montre les zones en transparence sur les murs en 3D et dans les
vues des murs, celle du sol dans le plan ; *Afficher › Ouvrir murs et sols* rend
transparents les murs et les dalles en 3D, comme *Ouvrir les goulottes*, pour montrer
les lignes encastrées qui s'y trouvent.
Les flexibles, tubes et tuyaux se terminent aux piquages des modèles générés de leurs
appareils (le piquage d'aspiration et celui de refoulement d'une pompe, les piquages
d'un réservoir, les deux extrémités d'une vanne ou d'un raccord, les sorties d'un
collecteur) : le parcours arrive à un point situé hors du piquage à deux fois
l'épaisseur du tuyau (au moins 30 mm) et continue tout droit dedans. Une vanne, un
compteur ou un clapet anti-retour dans un tuyau est considéré comme monté dans le sens
qui rapproche ses deux tuyaux de l'endroit d'où ils viennent.
L'exemple `pump_room` montre des pompes posées au sol, un chemin de tuyauterie et les
parcours.

Dans un schéma, le menu contextuel d'un appareil placé le montre dans l'implantation
d'armoire (la plaque de montage ou la porte de son armoire, le mur ou le plan de sa
salle) ou dans la vue 3D avec la caméra près de lui (*Afficher dans l'implantation
d'armoire*, *Afficher dans la vue 3D*).

Les parcours font partie de l'analyse du projet. À l'ouverture d'un projet, son
analyse est construite en arrière-plan : l'éditeur montre une fenêtre avec l'étape en
cours (listes de réseaux, liaisons, câblage d'armoire, routage 3D avec le parcours en
cours, remplissage des goulottes, contrôles) et montre le projet une fois qu'il est
prêt.

L'armoire routée est l'entrée des [assistants de câblage et de montage](workshop.md)
et des exports 3D (glTF, STEP, IGES, IFC, JT, Parasolid ; voir
[sorties et échange de données](data-exchange.md)).
