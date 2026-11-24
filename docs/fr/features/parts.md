# Gestion des articles

La base de données d'articles partagée par tous les projets (style EPLAN) : données
commerciales, techniques et de montage, modèles de fonction, points de connexion,
graphiques 2D / 3D et l'import des données de fabricants. Les articles sont partagés
entre les installations par le
[serveur d'articles](../parts-server.md).

## Base de données d'articles

Une **base de données d'articles** globale partagée par tous les projets (arborescence
groupe principal / groupe / sous-groupe de produits, filtre), avec des données générales,
commerciales (prix, unité de prix, conditionnement, unité de quantité), techniques, de
montage (dimensions, poids), des accessoires et un **modèle de fonction** par article
(par exemple bobine A1/A2, contacts principaux, 13/14 NO). La base de données est
enregistrée dans le stockage de l'application (bureau) / le stockage local (navigateur).

- **Accessoires** qui ne fonctionnent qu'avec d'autres articles (contacts auxiliaires et
  de signalisation, blocs de contacts, onglet *Accessoires*) : un article liste ses
  accessoires, un accessoire peut nommer les articles auxquels il convient (*Compatible
  avec ces articles*) – l'un ou l'autre côté forme la paire, de sorte qu'un accessoire
  sert de nombreux articles et que les données d'un fabricant peuvent en ajouter un sans
  modifier ses articles principaux (le contact de signalisation Hager CZ001 nomme
  l'interrupteur différentiel CCA216D, le CDA440D nomme le CZ001). Un accessoire est
  marqué comme tel et dit où il est monté : en face avant (empilé en profondeur), sur le
  côté gauche ou droit (à côté de l'article sur le rail), sous l'article comme socle sur
  lequel il repose (le module pour composants univers N sous l'interrupteur-sectionneur à
  fusibles NH1 Hager LT153 : l'appareil prend l'encombrement du module, l'article se
  trouve centré devant lui), intégré dans l'article (le déclencheur à minimum de tension
  et le contact auxiliaire dans les logements du disjoncteur h3+ HNT161GR : pas de place
  propre, leurs bornes sont sur le disjoncteur, et leurs bobines dessinées à côté du
  disjoncteur sont des fonctions de l'accessoire, pas une seconde fonction principale) ou
  pas du tout.
  Plusieurs exemplaires d'un même article de base sont placés côte à côte le long du plus
  grand côté de l'article (les transformateurs de courant Hager LVZW150 clipsés derrière
  les trois pôles d'une réglette interrupteur-sectionneur à fusibles NH LVSG), des
  articles de base différents les uns sur les autres. Dans un projet, l'onglet *Articles*
  de l'appareil propose les accessoires qui conviennent à ses articles (*Ajouter un
  accessoire*) et montre l'article sur lequel chacun est monté ; ses fonctions s'ajoutent
  aux contacts de l'appareil ; la vérification `accessories` signale un accessoire sur un
  appareil sans article auquel il convient ; l'implantation d'armoire et la vue 3D le
  dessinent à sa place avec ses propres bornes, et l'appareil devient plus large ou plus
  profond. Ligne de commande / MCP :
  `list_accessories`, `add_accessory` ; serveur d'articles :
  `GET /parts/{number}/accessories` ;
  colonnes CSV `Accessory`, `Accessory mounting`, `Fits` ; références BMEcat des types
  `accessories` et `base_product` ; XML EPLAN `P_ARTICLE_ISACCESSORY`.
- **Ensembles** : un article peut se composer d'articles composants avec des quantités
  (onglet *Accessoires*, colonne CSV `Components` : `NUM:2|NUM2`) ; la nomenclature les
  liste sous l'ensemble (ensembles imbriqués résolus, prix considéré comme inclus).
- **Données de montage typées** par article (onglet *Montage*) : distances (haut, bas,
  gauche, droite, avant, CEM, espace de service), trous de fixation avec filetage,
  découpes de porte et de face (plusieurs par article, chacune ronde ou rectangulaire
  avec son centre à partir du milieu de la vue de face, une rotation et un rayon de
  coin : tableau et aperçu dans l'onglet, forme textuelle
  `197 x 141 @ 0/-6.5 R3 15°; Ø16 @ 80/60`), puissance dissipée, dépassement d'un
  accessoire, fixation, faces, classe de collision et type de borne. Elles pilotent les
  vérifications de l'armoire, le perçage, les découpes et le calcul de climatisation ; le
  CSV a une colonne par valeur (`Power loss`, `Mounting holes`, …). Les articles plus
  anciens avec ces valeurs sous forme de lignes `Key: value` dans les données techniques
  fonctionnent toujours ; les imports et le catalogue intégré déplacent ces lignes dans
  les champs typés, l'éditeur d'articles le propose pour les articles existants.
- **Points de connexion** par article (désignation, position dans la vue de face, la
  direction par laquelle le fil part) ; les articles qui n'en ont pas en propre les
  reçoivent des modèles de fonction selon la convention IEC (désignations impaires en
  haut, paires en bas, appareils de porte par l'arrière, modules API en bas, parafoudres
  avec la terre et le contact de télésignalisation en bas). Les appareils des catalogues
  fournis qui ne sont pas câblés ainsi (Victron, Pylontech, Beckhoff, alimentations
  Phoenix Contact) et les appareils 19 pouces (entrée d'alimentation à l'arrière) ont les
  leurs ; les modèles générés dessinent leurs bornes aux points de connexion. Modifiés
  dans l'éditeur d'articles (onglet *Points de connexion* : tableau, cliquer dans la vue
  de face pour en placer un) ; l'implantation d'armoire câble jusqu'à eux ; colonne CSV
  `Connection points` : `A1:22.5/0/55/up|A2:…`.

## Graphiques et modèles 3D

Vue de face 2D (SVG) et modèle 3D par article :

- **STL**, **OBJ** ;
- **STEP** : solides et modèles surfaciques des fichiers AP203 / AP214 / AP242, placés
  selon leur structure d'assemblage (aussi lorsque les corps d'une pièce dépendent d'une
  autre représentation, comme dans les modèles CAO de CADENAS, par exemple ceux de
  Rittal), tessellés et convertis en mm ;
- **IGES 5.3** : solides et surfaces limitées ;
- **Parasolid** (`.x_t` / `.x_b`) : corps des fichiers de transmission V13 et plus
  récents, plans, cylindres, cônes, sphères, tores et surfaces B tessellés exactement,
  congés et autres faces procédurales remplis à partir de leur contour ;
- **IFC** : corps tessellés, à facettes et extrudés de tous les produits, placés et
  convertis en mm ;
- **JT** : les maillages en bandes de triangles des fichiers JT 8.

Les articles avec des dimensions mais sans modèle en reçoivent un généré d'après ce
qu'ils sont : appareils sur rail DIN avec leurs bornes, leur levier ou leur afficheur,
bornes dans leurs couleurs (PE vert, N bleu), modules API, interrupteurs-sectionneurs à
fusibles, électronique murale avec afficheur, presse-étoupes (corps hexagonal, col et
dôme, en laiton nickelé ou en polyamide selon leur matériau et leur couleur), appareils
19 pouces et batteries en rack, modules PV, pompes, moteurs,
réservoirs, pompes à chaleur, compresseurs, sécheurs frigorifiques, vannes à levier ou à
actionneur, raccords de tuyauterie, collecteurs et unités de traitement d'air. Les
appareils des catalogues fournis sont dessinés d'après leurs photos de produit : les
onduleurs Victron avec leurs coins noirs et leur bande orange, le Cerbo GX et le
SmartShunt, le distributeur Lynx, les batteries en rack Pylontech avec poignées et
bornes, les interrupteurs-sectionneurs à fusibles, compteurs et appareils modulaires
Hager, les parafoudres DEHN avec leurs modules enfichables, les bornes EtherCAT
Beckhoff, les alimentations Phoenix Contact, les disjoncteurs-moteurs et contacteurs
Siemens. Les modèles générés sont peints (une couleur
par triangle, `colors` du maillage) ; les modèles importés prennent la couleur que la vue
3D donne à l'appareil. Les articles posés au sol se tiennent debout sur le sol d'une
pièce.

## Articles dans le projet

- **Articles du projet** : les articles sont copiés dans le projet lors de l'affectation ;
  synchroniser le projet à partir de la base de données, transférer du projet vers la
  base de données, supprimer les articles du projet inutilisés.
- Boîte de dialogue **Sélection d'article** avec *sélection d'appareil* (seulement les
  articles qui fournissent la fonction principale de l'appareil) ; plusieurs articles par
  appareil avec quantité (contacteur + bloc auxiliaire).
- Les modèles de fonction pilotent les **contacts de réserve** dans l'image de contacts,
  « Appliquer les désignations » (numéros des points de connexion issus de l'article) et
  le contrôle du projet (plus de fonctions placées que les articles n'en fournissent,
  articles manquants dans le projet).

## Import et export

- **CSV** (séparé par `;`, noms d'en-tête) et **JSON**, import et export.
- Catalogues **BMEcat** (1.2 / 2005, tels que publiés par les fabricants et les
  grossistes, par exemple depuis l'EPLAN Data Portal ou les bases de données ETIM /
  eCl@ss) : identification, description, prix, fiche technique, la classe ETIM / eCl@ss
  et chaque caractéristique comme donnée technique ; les dimensions, le poids, le courant
  / la tension assignés et la puissance dissipée vont dans leurs champs (par nom de
  caractéristique, ou par les codes ETIM de la largeur, de la hauteur et de la
  profondeur).
- **XML d'articles EPLAN** : l'export de la gestion des articles d'EPLAN (`<part>` avec
  attributs `P_ARTICLE_*` et variantes, textes multilingues, groupes de produits par
  leurs numéros d'échange) est importé comme les autres formats, et les articles
  s'exportent en XML d'articles EPLAN pour le chemin inverse (gestion des articles,
  `parts_export --format eplan_xml`). Les modèles de fonction deviennent des modèles de
  fonction (entrées et sorties API comme fonctions API), les définitions de points de
  connexion des points de connexion (points de connexion sur des connecteurs comme
  `X10.1`) avec les fils que leurs bornes acceptent (plus petite et plus grande section
  en mm², converties depuis l'AWG là où seul celui-ci est donné, et le plus grand nombre
  de fils), les plans de perçage des trous de fixation : trous traversants (`Ø3.3`) ou
  taraudés (`M4`), tandis qu'une position rectangulaire ou un trou plus grand qu'une vis
  est une découpe de l'article, avec sa place et sa rotation (le `197 x 141` d'un pupitre
  IHM, 6,5 mm au-dessus du milieu de sa face avant) ; les enregistrements de perçage sont
  conservés tels quels avec l'article (leur sous-type et leur troisième dimension ne sont
  pas encore interprétés). Les données de rail d'EPLAN (hauteur d'encliquetage, décalage
  du centre) ou un texte nommant le rail
  (*DIN rail*, *Hutschiene*, *TS 35*, *TS 15*, *35 x 15*, *G 32*) font de l'article un
  article pour un rail de montage de cette taille (*Rail de montage 35 mm*).
- **EDZ** (archives de l'EPLAN Data Portal, 7-Zip ou ZIP comme ABB les publie) : le XML d'articles qu'elles
  contiennent est lu comme ci-dessus, avec les distances, la puissance dissipée et le
  lien vers la fiche technique. Les macros de fenêtre 2D incorporées deviennent des
  symboles du projet. L'image du produit est conservée avec l'article sous forme de petit
  JPEG (192 px au plus) et montrée dans les détails de l'article avec ses données de
  montage (montage, trous, découpe, distances) et les fils que ses points de connexion
  acceptent.
  Une macro 3D de l'article (EPLAN Pro Panel, `…_3D.ema`, `P_ARTICLE_MACRO`) lui donne
  son modèle 3D quand il n'en a pas, aussi pour le XML d'articles d'un fichier EDZ
  décompressé ou d'un dossier de fabricant ; une macro dont les corps sont dans une forme
  que cette version ne lit pas est signalée dans les diagnostics EPLAN de l'article.
  Le gestionnaire d'articles et `parts_import` enregistrent aussi les fichiers d'origine
  comme pièces jointes. Les enregistrements d'article du projet et du serveur portent les
  définitions converties et les diagnostics.
  Le XML d'article d'un fichier EDZ décompressé (`items/partxml/…part.xml`) est lu de la
  même façon : avec les fichiers de points de connexion et de plans de perçage à côté de
  lui et les macros et images sous `items/`.
- **Paquets d'armoire** (archives ZIP de configurateurs d'armoires tels que le
  configurateur Rittal) : la nomenclature (`bill-of-material.csv`) donne les articles, le
  premier est l'armoire configurée (préfixe du fabricant et nom d'après la macro EPLAN,
  par exemple `RIT.1360000`, taille d'après *WHD: 600x600x350 mm*, GTIN, identifiant de
  configuration et système de coffrets compacts dans ses données techniques) avec les
  autres lignes comme composants. L'aperçu de la macro EPLAN Pro Panel devient son image.
  Les corps de la macro (enveloppe, porte, plaque de montage, plaque passe-câbles,
  charnières et serrure) deviennent son modèle 3D, dressé debout avec la face avant vers
  +z ; une macro sans corps laisse ce rôle au modèle STEP, sans ses zones restreintes de
  découpe et de placement (gestionnaire d'articles et éditeur ; `parts_import` ne fait
  que joindre le fichier STEP). La
  face avant de la plaque de montage (ou sinon la plus grande surface tournée vers
  l'avant dans la moitié arrière de l'enveloppe) donne les données d'article
  `Mounting origin: 25/12.5/20` et
  `Mounting size: 550 x 575`, le côté des blocs de charnière à côté de la porte
  `Door hinge: right`, de sorte
  qu'une armoire de cet article reçoit cette plaque (voir
  [implantation d'armoire](panel-layout.md)).
  Les zones restreintes de la macro deviennent les *Zones interdites* de l'article
  (données de montage, `placing plate 0/0 550 x 20; cutout left 10/20 30 x 40` : type,
  face, place et taille dans les coordonnées de la face), chacune sur la face dans le
  plan de laquelle elle se trouve ; celles sur le cadre autour de la porte sont omises.
  L'implantation d'armoire les dessine et les vérifie (voir
  [implantation d'armoire](panel-layout.md#zones-interdites)). La macro, le projet EPLAN
  (`.zw1`), le fichier STEP et les documents PDF sont enregistrés comme pièces jointes.
- **Un article déjà connu :** un fichier EDZ, un XML d'articles EPLAN ou un paquet
  d'enveloppe dont l'article est déjà dans la base d'articles, sur le serveur
  d'articles ou dans un catalogue livré avec OpenECraft est fusionné avec cet article au
  lieu de le remplacer (l'éditeur, Parts Manager et `parts_import`). L'article connu est
  celui du même numéro, sinon celui de la même référence de commande du même fabricant
  (ou le seul article de cette référence) ; il garde son numéro, et les fichiers et
  références de l'import le suivent. Les valeurs de l'import l'emportent ; ce qui lui
  manque reste, comme les données d'article `Fields`, `Rows` et `Module spaces` des
  armoires et champs de Striebel & John. Le groupe et le sous-groupe restent, les
  données techniques sont fusionnées ligne par ligne (le `Clé: valeur` de l'import pour
  la même clé), les accessoires sont réunis. Les fichiers CSV, JSON et BMEcat
  remplacent toujours les articles de leurs numéros. Parts Manager et `parts_import`
  envoient un gros fichier en plusieurs requêtes ; un fichier qu'il contient plus grand
  que ce qu'accepte le serveur d'articles (32 Mo, comme certaines macros 3D d'ABB)
  n'est pas enregistré et est nommé, l'article garde le modèle qui en a été lu.

### Placer un symbole EPLAN

Ouvrez **Importer des articles** et sélectionnez l'EDZ ou le XML d'article du fabricant ; une boîte en bas à droite annonce le résultat (le numéro et la description de l'article, ou le nombre d'articles) et se ferme après quelques secondes ou sur un clic. Sur le bureau, le XML d'article peut charger un EMA associé depuis le même répertoire. Sous **Centre d'insertion › Appareils**, cherchez l'article, sélectionnez-le, puis sélectionnez la variante avant de le placer. Chaque ligne a un aperçu et liste les propriétés ou les références de bibliothèque non résolues. Un EMA isolé peut aussi être ouvert avec **Importer une bibliothèque de symboles**. Les fenêtres à plusieurs appareils explicitement repérés peuvent être placées comme macros natives.

Le décodeur lit les macros de fenêtre PXF, les blocs de dessin incorporés, les transformations affines, les graphiques vectoriels de base, le texte et certains champs de propriétés. Ses correspondances SPECIAL intégrées couvrent les contacts d'API et d'appareil 13, 17 et 20, les formes de contact de connecteur et de bus 34, 35 et 37, la forme bilatérale 22 avec une désignation commune, et la forme de vue d'ensemble 40. Ces formes utilisent les dessins de contacts de la vue d'ensemble des symboles IEC d'EPLAN de 2007, p. 1–2. Les points de connexion physiques XYZ continuent d'appartenir au modèle de l'armoire.

L'importateur lit les références locales d'emplacement de broche et les références de transfert de potentiel connecteur/désignation entre guillemets. Les références réciproques et sans ambiguïté entre des contacts statiques mis en correspondance sur le même appareil créent des ponts natifs. Les règles à sens unique, les cibles manquantes, les identités d'appareil incertaines et les comportements d'appareil non pris en charge restent des diagnostics. L'importateur conserve les coupures de connexion auto, les métadonnées de câble/blindage et les points de définition de liaison séparément des segments de fil. Le XML source ne contient pas nécessairement une liste de fils routés de la source à la cible. [Logique des points de connexion EPLAN](https://www.eplan.help/en-us/Infoportal/Content/Plattform/2022/Content/htm/xfctdefbrowsergui_d_anschlusslogik.htm)

Dépliez **Placements non résolus** pour une variante d'article afin de voir le maître, l'objet source et les désignations de connecteur et de borne. Pour un fichier ou tout un dossier de fabricant, `cargo run -p openecraft-edz --example inspect -- PATH --references` produit en flux des lignes JSON avec les broches natives, les références en attente, les contrôles de liaison et l'état de transfert par représentation. Les coordonnées des placements non résolus sont des points d'insertion. Une définition native vérifiée de clé `user:eplan-lib:LIBRARY:NUMBER:VARIANT` fournit des ancrages de broche explicites. Les dessins PDF non enregistrés peuvent être conservés sous `user:pdf-review:LIBRARY:NUMBER` pour comparaison et ne participent pas à la mise en correspondance.

Les variantes partielles peuvent être inspectées et placées. Les symboles maîtres manquants, les propriétés non prises en charge, les états d'espace réservé et les autres objets non traités sont signalés. Les réglages de calque incorporés ainsi que les épaisseurs de plume explicites et les couleurs prédéfinies sont repris. Les palettes personnalisées, les motifs de tirets exacts et les remplissages peuvent différer d'EPLAN. La reconstruction automatique du câblage d'un projet EPLAN complet et la conversion de modèles 3D sont hors du champ du décodeur 2D. La réimportation actualise les graphiques non modifiés dont les extrémités sont compatibles. Les modifications locales et les géométries de broche existantes priment sur une révision en conflit.
