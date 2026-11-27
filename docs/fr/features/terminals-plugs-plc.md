# Borniers, connecteurs, API et marquage

Les borniers avec bornes à étages et accessoires, les connecteurs, les cartes API avec l'échange vers les outils de programmation d'API, et les repères pour l'atelier.

## Éditeur de borniers

Ordre des bornes (monter / descendre, trier), renumérotation 1 … n,
**ponts** (les bornes pontées partagent leur potentiel), bornes de réserve (dans le plan
de bornes et la nomenclature) ;
chaque borne a son article : les bornes PE prennent l'article du bornier dont les données
techniques contiennent `Terminal type: PE`, toutes les autres son article de passage
(nomenclature, longueur du bornier dans l'armoire, points de connexion par borne) ;
**accessoires automatiques** d'après les données techniques des articles de borne
(`End bracket: …`, `End plate: …`,
`Jumper: …` avec des articles de pont `Poles: n` ; un article de butée d'arrêt dans la
liste d'articles du bornier, groupe de produits `End bracket`, remplace celui des données
techniques) : une butée d'arrêt aux deux
extrémités, un flasque d'extrémité après la dernière borne de chaque article, des ponts
enfichables de la plus courte longueur qui convient (coupés s'ils sont plus longs, dents
cassées aux bornes intermédiaires) – dans la nomenclature, le plan de bornes,
l'implantation d'armoire (2D / 3D) et la vérification ;
la fenêtre des borniers montre le bornier sur le rail avec ses ponts et possède un
commutateur pour les accessoires automatiques.
**Bornes à étages** : une borne à l'étage 2 ou supérieur se trouve sur le bloc de la
borne qui la précède (un article par bloc, données techniques `Levels: 2`) ; le plan de
bornes liste chaque étage, la disposition sur le rail et la nomenclature comptent les
blocs, et la vérification signale plus d'étages que l'article n'en a. **Types de pont** :
ponts enfichables (données techniques `Jumper`), ponts à cavalier (`Saddle jumper`) et
ponts filaires (n'importe quel étage, sans article) ; les ponts enfichables et à cavalier
doivent rester sur un seul étage. Commandes :
`list_strips`, `get_strip`, `set_terminal`, `set_jumper_kind`, `set_strip`,
`renumber_strip`.

## Connecteurs

**Connecteurs** (*Appareils › Connecteurs…*, groupe de symboles *Connecteurs*) : broche
et douille (paire accouplée, deux points de connexion), broche mâle et douille femelle
(un chacune) ; repère = connecteur (`-X10`), désignation = broche. Les broches mâles et
femelles de même désignation sont accouplées : le potentiel passe, les deux côtés sont
des liaisons distinctes (numéros de fils, câbles). Les broches sont numérotées par
connecteur lors du placement ; la fenêtre des connecteurs et le rapport *plan des
connecteurs* listent les broches dans l'ordre du connecteur avec le côté douilles, le
côté broches et le potentiel ; les articles avec des fonctions `Plug pin` donnent le
nombre de broches (les broches inutilisées sont listées comme réserve). La vérification
signale les broches placées deux fois et plus de broches que les articles du connecteur
n'en ont ; l'export KBL nomme l'article de boîtier des connecteurs avec toutes les
alvéoles de l'article.

## Brochages des connecteurs et câbles plats

**Brochage** (fenêtre des connecteurs, export *Brochages des connecteurs (SVG)*
`pinouts`, commande `get_pinout`) : les faces d'accouplement du côté broches (mâle) et du
côté douilles (femelle, l'image miroir) avec chaque broche de l'article du connecteur à
sa place, remplie de la couleur du fil ou du conducteur de câble à la broche (réserves
vides), le détrompeur et un tableau des broches avec potentiel, cibles et conducteurs de
câble ; le survol d'une broche montre ses détails, un clic montre son symbole. La face
vient des données techniques de l'article de connecteur :
`Pin layout: circle 4` (broches sur un cercle, broche 1 à droite du détrompeur en haut,
comptées dans le sens antihoraire sur la face mâle comme dans l'IEC 61076-2-101),
`circle 7+1` (broches centrales en dernier), `circle 9+3` / `circle 11+5+1` (un anneau
intérieur, puis le centre), `row 8`,
`zigzag 10` (deux rangées, broches impaires au-dessus des broches paires : IDC),
`grid 2x5` (numérotées rangée par rangée), `cols 20x2` (numérotées colonne par colonne),
`dsub 9`, chacune au choix avec `+PE` ; pour toute autre face, les centres des broches en
mm : `points -3.5,2 3.5,2 0,-2` (face rectangulaire) ou
`round 0,9 -8,-3 8,-3` (face circulaire avec détrompeur). `Pitch`, `Coding` et `Gender`
(`male`, `female`, `hermaphroditic`) la complètent. Les broches numérotées d'un article
plus petit se trouvent à leur numéro (M8 à 3 broches : 1, 3, 4) ; les connecteurs sans
disposition sont montrés en une rangée.

Le catalogue contient, chacun en mâle et en femelle :

| Famille | Articles |
| --- | --- |
| Capteur / actionneur | M5, M8 (3 – 8 broches), M12 A, B (PROFIBUS), D, X (Ethernet 10 Gbit/s) et les codages de puissance S, K, T, L, 7/8" (3 – 6 broches) |
| Circulaire | M16 selon IEC 61076-2-106 (3 – 24 broches), M23 signal (12, 17, 19 broches), connecteurs de puissance M23 et M40 pour servomoteurs, connecteurs aviation GX12 / GX16 / GX20 |
| Vannes | DIN EN 175301-803 formes A, B et C, 2 + PE et 3 + PE |
| Rectangulaire | inserts pour connecteurs industriels lourds des séries A, D, E et Q avec PE, D-sub 9 – 37, DIN 41612 type C (32, 64, 96), connecteurs frontaux d'API (20, 40) |
| Carte | connecteurs femelles IDC et embases à boîtier 10 – 64, simple rangée 2,5 mm, double rangée 3,0 et 4,2 mm, borniers enfichables 3,5 / 5,08 / 7,62 mm |
| Données | RJ11, RJ12, RJ45, BNC |
| Engins mobiles | type DT étanche (2 – 12 broches) et système 1,5 mm (1 – 6 broches) |
| Puissance | CEE IEC 60309 16 / 32 / 63 A à 3, 4 et 5 pôles, connecteurs d'appareils IEC 60320 C5 – C20, Schuko, BS 1363, NEMA 5-15, L14-30, L21-30, connecteurs d'installation (3 – 5 pôles), connecteurs PV, connecteurs de batterie (hermaphrodites, 50 – 175 A), connecteurs banane de sécurité, connecteurs pour thermocouples |

**Câbles plats** (`Cable form: Flat` dans l'article de câble : câbles en nappe de 10 – 64
conducteurs au pas de 1,27 mm, câbles plats pour systèmes à festons) :
`connect_flat_cable` fait des fils d'un côté d'un connecteur des conducteurs du câble, la
broche n prenant le conducteur n. Les câbles en nappe (câbles plats avec un `Pitch`)
montrent des conducteurs gris avec le conducteur 1 en rouge sur les faces. La
vérification *Brochages des connecteurs et câbles plats* signale les broches que
l'article du connecteur n'a pas, les conducteurs d'un câble plat sur un connecteur IDC
(`Termination: IDC`) qui arrivent sur une autre broche et les câbles plats dont le nombre
de conducteurs diffère du nombre de broches du connecteur IDC.

## Navigateur de borniers et de connecteurs

**Navigateur de borniers et de connecteurs** (onglet de navigateur *Borniers /
connecteurs*) : les borniers avec leurs bornes dans l'ordre du rail (étages des blocs à
étages) et leurs réserves, les connecteurs avec leurs broches et leurs cibles et les
broches inutilisées des articles de connecteur ; un clic montre un élément placé, les
réserves et les broches inutilisées se glissent dans la page.

## Navigateur API

**Navigateur API** (onglet de navigateur *API*) : les cartes avec leurs canaux dans
l'ordre des adresses, ceux qui sont placés avec le texte de fonction et les appareils
raccordés (un clic montre le symbole), les canaux libres des cartes ayant une description
d'appareil à glisser dans la page.

## Vue d'ensemble API

Sélectionnez une fois la carte importée sous **Centre d'insertion › Appareils**, puis cliquez sur la feuille pour chaque canal successif. Pour une EL1008, les clics un à huit partagent un repère de carte. Le neuvième clic commence le canal 1 sur une autre carte. Échap termine le placement. Dépliez **Représentations** quand une variante de câblage particulière est requise.

Un EMA isolé avec des définitions de canaux reconnues a une entrée sous **Symboles** et utilise la même séquence de placement. Importez son XML d'article pour obtenir un article de catalogue, l'affectation à la nomenclature et les dimensions pour l'armoire.

Les macros analogiques Beckhoff avec un signal identifiable et ses bornes associées produisent un seul point API. Sur l'EL3174, l'entrée de tension `U1` utilise la borne 1 pour le signal et la borne 9 pour sa référence. Les deux bornes peuvent être câblées séparément. Les représentations en mode courant sont des alternatives pour les quatre mêmes canaux. Utilisez le Centre d'insertion pour sélectionner une représentation particulière. La vue d'ensemble API indique la borne du signal, et le réadressage de la carte laisse intactes les étiquettes des canaux analogiques. Les déclarations de variables TwinCAT conservent le type de données fourni. Les canaux sans type de données déclaré restent disponibles sur la feuille et dans la vue d'ensemble.

Pour les Beckhoff EL1008 et EL2008, importez le XML d'article avec l'EMA correspondant à côté. Placez un article sur le schéma pour utiliser son prochain canal disponible, ou choisissez un canal multifilaire particulier dans le Centre d'insertion. Les canaux de la même famille de macros partagent un repère de carte. Une fois les huit placés, le prochain dépôt par défaut commence une autre carte. Le navigateur API liste les canaux restants après le premier placement. La vue d'ensemble API modifie les adresses et les textes de fonction. Le réadressage utilise l'ordre des canaux importé même quand les points sont placés dans le désordre. Par exemple, le canal 2 utilise la borne physique 5 et le bit 1. Fixez un autre octet de départ pour une autre carte dans le même espace d'adresses. Le coupleur EK1100 conserve ses connexions d'alimentation et de bus.

Toutes les entrées / sorties par carte avec les appareils raccordés, adresses
et textes de fonction modifiables, octet de départ / châssis / emplacement de la carte,
réadressage (des points numériques ; les points analogiques gardent leurs adresses de
mot), **export / import de la table de symboles** (CSV : nom, type de données, adresse,
commentaire) pour les outils de programmation d'API, la table des variables de
**Siemens TIA Portal** (feuille Excel *PLC Tags*), l'import des tags de
**Rockwell Studio 5000** (alias des canaux de module), la table de symboles de
**Siemens STEP 7 classic** (SDF), une liste de variables globales pour
**Beckhoff TwinCAT 3** (TcGVL, `AT %I*` / `AT %Q*`), des variables pour
**Schneider Control Expert** (XSY, `EBOOL` avec adresses topologiques), des commentaires
d'appareil pour **Mitsubishi GX Works** (CSV, `X` / `Y` en octal) et des variables
globales IEC 61131-3 (CODESYS) ; les adresses
utilisées sur plusieurs cartes sont signalées.

## Marquage

**Marquage** (*Liaisons › Marquage…* : texte des repères de fil, matériel par type avec
nombres de pièces et export ; *Fichier › Export › Exporter le marquage (CSV)…*, rapport
*Marquage*) : repères d'appareil (porte-étiquettes pour les appareils de porte ;
l'emplacement de l'armoire omis du repère), repères de borne dans l'ordre du rail avec le
repère du bornier, repères de fil (numéro de fil aux deux extrémités, ou marquage de
destination : chaque extrémité montre l'autre extrémité), repères de câble ; regroupés
par matériel de marquage dans l'ordre de pose, en CSV pour les logiciels de marquage
(UTF-8, `;`, en-tête `Material;Category;Text;Count;Enclosure;Group`) ; le matériel figure
dans la nomenclature. Les textes et les matériels sont des paramètres du projet
(`marking`).
Les **schémas de repérage** choisissent les repères, les colonnes et leur ordre, le
séparateur, l'en-tête et une ligne par étiquette imprimée, pour l'export CSV / texte et
Excel ; tables d'import intégrées pour Phoenix Contact CLIP PROJECT, Weidmüller M-Print
PRO, Brady, Murrplastik et Partex (points de départ – à copier et à adapter à la version
du programme), les schémas personnels sont enregistrés dans le projet (commandes
`marking_schemes`,
`set_marking_scheme`, exporteurs `marking` / `marking_xlsx` avec `scheme`).
