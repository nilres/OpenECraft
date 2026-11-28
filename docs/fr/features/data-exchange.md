# Sorties et échange de données

Exports PDF, DXF / DWG, SVG et 3D, import DXF / DWG, la couche d'intégration d'entreprise (ERP, PDM, PLM, ECAD, MCAD, achats, machines de câblage), l'échange avec KiCad et FreeCAD et Autodesk Vault.

## Sorties / échange de données

**PDF** de toutes les pages ou d'une sélection de pages (`1-3, 5` ;
également pour le document HTML d'impression ; *Fichier › Imprimer* ouvre toutes les
pages ou la page courante dans le navigateur avec sa boîte de dialogue d'impression)
avec des signets par page, des références croisées et des lignes de rapport cliquables
(les liens vers des pages non incluses sont supprimés) ; les textes au-delà du Latin-1
tels que le polonais, le tchèque, le turc, le grec ou le cyrillique sont écrits avec
un sous-ensemble incorporé de la police Ubuntu, avec recherche et copie possibles,
**DXF** (R12, calques par type) et **DWG** (AutoCAD 2004, le même dessin) par page ou
toutes les pages côte à côte, SVG par page, HTML imprimable, liste de fils pour la
production (CSV avec numéros de fil, extrémités, couleur, section, longueur routée,
longueur de dénudage et terminaison des deux extrémités (embout selon DIN 46228-4,
embout double, cosse à oeil, à fourche, à broche, languette mâle ou clip femelle)
avec leurs articles, un embout double étant nommé avec l'autre fil et listé au premier
des deux, conducteurs de câbles ; fils d'armoire triés par section, couleur et
longueur pour les machines de traitement de fils ; embouts et cosses également dans
la nomenclature ; avec une marque d'ordre d'octets UTF-8, comme les CSV de rapports
de l'éditeur, afin que les tableurs affichent `mm²` et les caractères accentués),
CSV de rapports, tables de symboles API,
données CN des plaques de montage, l'**implantation d'armoire en 3D** en **glTF 2.0**
(visionneuses, web, PLM ; mètres, un noeud par armoire et par élément, couleurs),
**STEP AP214** (MCAD ; solides facettés avec couleurs, millimètres, z vers le haut),
**IGES 5.3** (MCAD ; solides B-rep à faces planes, coques ouvertes pour les maillages
non fermés, couleurs et noms), **IFC4** (BIM ; un tableau de distribution par armoire
avec ses composants sous forme d'éléments triangulés), **JT** (visionneuses PLM ;
maillages en bandes de triangles par élément, structure JT 8.1) et
**Parasolid XT** (MCAD tels que SolidWorks, NX, Solid Edge, Onshape ; texte de
transmission du schéma V13 en mètres, chaque pièce fermée étant un corps solide
facetté dont les triangles coplanaires sont fusionnés en faces planes, les maillages
ouverts un corps surfacique) –
parois, porte, plaque de montage, rails, goulottes, jeux de barres et appareils avec
les modèles 3D de leurs articles.
**Import DXF / DWG** (*Fichier › Import › Importer un DXF / DWG comme page…*) : les anciens dessins comme graphiques
sur une nouvelle page de schéma – lignes, polylignes, cercles, arcs, ellipses,
splines, textes, MTEXT, références de blocs et cotes, dans leurs unités, réduits à la
zone de dessin lorsqu'ils sont plus grands. Les fichiers DWG d'AutoCAD R12 … 2018 sont
lus (et écrits) avec la crate en pur Rust [acadrust](https://crates.io/crates/acadrust),
également dans le navigateur. Les fichiers EDZ de l'EPLAN Data Portal sont lus comme articles (voir [gestion des articles](parts.md)) ; leurs macros de fenêtre 2D deviennent des symboles et leurs macros 3D les
modèles 3D des articles.

## Intégration d'entreprise

**Intégration d'entreprise** (*Outils › Intégration d'entreprise…*) : une couche
d'intégration unique avec des connecteurs pour l'**ERP** (fiches articles et
nomenclature, CSV), le **PDM** (métadonnées de projet et de pages, JSON), le **PLM**
(nomenclature d'ingénierie, JSON), l'**ECAD** (composants et liste des réseaux pour la
conception de circuits imprimés, JSON), le **MCAD** (placement des appareils, CSV),
les **achats** (liste de commande par fournisseur, stock et conditionnements pris en
compte, CSV) et les **machines de câblage** (liste de travaux avec longueur de coupe,
dénudage et embouts, CSV). Tous les connecteurs fonctionnent dans les deux sens et
partagent un vocabulaire de champs neutre : les exports utilisent les codes de champ
comme colonnes / clés, les imports reconnaissent les codes, les noms de colonnes
anglais et allemands courants (`Material`, `Bestand`, `refdes` …) et la
correspondance de colonnes d'un *profil* (p. ex. `MATNR` → `erp_number` pour SAP).
Les imports sont présentés sous forme d'une liste de modifications qui sont examinées
puis reprises : numéros ERP, prix, stock et délais de livraison, identifiants
d'articles PLM, révisions et états de cycle de vie (les articles obsolètes utilisés
sont signalés par le contrôle), numéros et états de documents PDM, affectations
d'articles du système ECAD, positions des appareils du système MCAD, confirmations de
commande, et le retour du traitement des fils sous forme de marques de câblage.
Chaque export enregistre les valeurs exportées, de sorte que les valeurs modifiées
des deux côtés depuis lors sont signalées comme conflits et que les valeurs laissées
inchangées par l'autre système n'écrasent jamais les modifications locales. Un fichier
d'échange neutre (`openecraft.exchange/1`, JSON) contient tout pour les systèmes sans
connecteur ; le journal des échanges est conservé dans le projet.

## KiCad et FreeCAD

OpenECraft échange des données avec les outils libres de conception de circuits imprimés, [KiCad](https://www.kicad.org),
et de conception mécanique, [FreeCAD](https://www.freecad.org), dans leurs propres formats de fichier.

| Quoi | Sens | Comment |
| --- | --- | --- |
| Une carte comme appareil | KiCad → OpenECraft | *Fichier › Import › Importer une carte KiCad comme appareil…* lit la liste d'équipotentielles de la carte (`.net`) |
| Un circuit pour une carte | OpenECraft → KiCad | *Fichier › Export*, *Liste d'équipotentielles KiCad* ; dans l'éditeur de circuits imprimés de KiCad *Fichier › Importer › Netliste* |
| Symboles | les deux | *Fichier › Export*, *Bibliothèque de symboles KiCad* ; *Importer une bibliothèque de symboles…* lit `.kicad_sym` |
| Références et descriptions | KiCad → OpenECraft | connecteur *KiCad* de l'intégration d'entreprise : une liste d'équipotentielles ou une nomenclature (CSV) |
| L'implantation d'armoire en 3D | OpenECraft → FreeCAD | *Fichier › Export*, *Implantation d'armoire en 3D pour FreeCAD* : une macro qui construit l'implantation |
| Positions des appareils | FreeCAD → OpenECraft | connecteur *FreeCAD* de l'intégration d'entreprise : le document enregistré dans FreeCAD (`.FCStd`) |
| Modèles 3D d'articles et de cartes | KiCad, FreeCAD → OpenECraft | fichiers STEP comme modèle 3D d'un article |
| Dessins | les deux | DXF |

**Une carte conçue dans KiCad comme appareil.** Écrivez la liste d'équipotentielles
dans l'éditeur de schémas de KiCad (*Fichier › Exporter › Netliste…*, format KiCad,
ou `kicad-cli sch export netlist`). *Importer une carte KiCad comme appareil…* ajoute
une page de schéma avec une boîte d'appareil pour la carte et un point de connexion
d'appareil pour chaque broche de ses connecteurs (`J1.1`, `J1.2` …), l'équipotentielle
de la carte écrite à chaque broche (`+24V`, `IN1`). La carte se câble alors comme tout
autre appareil et figure dans les listes de connexions, les plans de câbles et la
nomenclature. Les connecteurs sont les composants de références J, P, X, CN, CON, TB
ou XS et ceux des bibliothèques *Connector* et *TerminalBlock* de KiCad, sans les
points de test (TP) ni les cavaliers (JP) ; une carte à
nombreuses broches se poursuit sur d'autres boîtes et pages. La carte reçoit un
article (`PCB.<nom>`) avec le titre, la société et la révision du cartouche et la
liste de ses composants. L'outil `import_kicad` de la ligne de commande et du serveur
MCP prend aussi le fichier de carte (`.kicad_pcb`), dont le contour sur la couche
`Edge.Cuts` donne la taille de l'article, le repère d'appareil et les composants dont
les broches sont les connexions. Le modèle 3D de la carte est le fichier STEP que
KiCad écrit (*Fichier › Exporter › STEP*, `kicad-cli pcb export step`), affecté à
l'article comme tout modèle.

**Le circuit comme liste d'équipotentielles KiCad.** Chaque appareil ayant des
connexions électriques est un composant (référence : le repère d'appareil, valeur :
le numéro de type de son article, champs *Part number*, *Manufacturer* et
*Order number*), chaque potentiel une équipotentielle de points de connexion. Les
empreintes sont affectées dans KiCad ; leurs pastilles portent le nom des points de
connexion (`A1`, `13`). Les composants gardent leur identité d'un export à l'autre :
l'éditeur de circuits imprimés met à jour une implantation au lieu de recommencer.
Une liste d'équipotentielles ou une nomenclature (CSV avec la colonne *Reference*)
qui revient de KiCad affecte références et descriptions par le connecteur *KiCad*,
sous forme de liste de modifications à vérifier.

**Symboles.** L'export écrit les symboles placés dans le projet et ses symboles
utilisateur (avec le paramètre `all` toute la bibliothèque) comme bibliothèque de
symboles KiCad : 4 mm ici valent 2,54 mm (100 mil) là-bas, de sorte que les points de
connexion sont sur la grille de KiCad. Un fichier `.kicad_sym` est lu comme symboles
utilisateur dans une bibliothèque nommée d'après le fichier, 1,27 mm (50 mil)
là-bas valant 4 mm ici : un symbole par unité
(`LM358 A`, `LM358 B`), les noms de broches à l'intérieur, les symboles dérivés avec
le dessin de leur parent, les symboles d'alimentation comme potentiels. Relire une
bibliothèque remplace ses symboles.

**L'implantation d'armoire dans FreeCAD.** L'export est une macro : exécutée dans
FreeCAD (*Macro › Macros…*), elle construit un document avec un conteneur de pièce
par armoire et un objet par composant, rail, goulotte et appareil, nommés par les
repères d'appareil, sous forme de solides (les objets à très nombreux triangles
restent des maillages ; deux réglages en tête de la macro en décident). Chaque
appareil porte son repère, sa référence, son armoire, sa face de montage et sa place
comme propriétés du groupe *OpenECraft*. Déplacez les appareils dans FreeCAD,
enregistrez le document et lisez le fichier `.FCStd` avec le connecteur *FreeCAD* :
les positions reviennent sous forme de liste de modifications, avec la détection de
conflits de tout connecteur. Les modèles conçus dans FreeCAD sont affectés aux
articles comme fichiers STEP.

```bash
openecraft-cli -p plant.oec import_kicad --path relay-io.net --board relay-io.kicad_pcb
openecraft-cli -p plant.oec import_kicad --path Device.kicad_sym
openecraft-cli -p plant.oec export --format kicad_netlist --path plant.net
openecraft-cli -p plant.oec export --format kicad_symbols --path plant.kicad_sym
openecraft-cli -p plant.oec integration_export --connector freecad --path plant.FCMacro
openecraft-cli -p plant.oec integration_import --connector freecad --path plant.FCStd --apply
```

## Autodesk Vault

OpenECraft se connecte à Autodesk Vault Professional 2025.2 ou plus récent par la
Vault Data API (REST, `/AutodeskDM/Services/api/vault/v2`), de sorte que les projets,
documents et modèles 3D sont échangés directement avec le système PDM plutôt que par
des fichiers.

- Dans l'éditeur : *Outils › Autodesk Vault…* – saisissez l'adresse du serveur, le
  coffre-fort de connaissances, le nom d'utilisateur et le mot de passe (ou le jeton
  d'accès d'une session Vault ou d'un compte Autodesk), puis *Se connecter*. Parcourez
  les dossiers et recherchez dans les propriétés de fichier sous le dossier affiché.
- **Import** : un fichier sélectionné est ouvert comme projet (`.oec`, `.oecz`), importé dans
  la base d'articles (CSV, JSON, BMEcat, EPLAN XML, EDZ, paquets d'armoires en ZIP), affecté comme modèle 2D / 3D d'un
  article (SVG, STEP, IGES, Parasolid, JT, IFC, STL, OBJ – p. ex. des modèles de la conception mécanique conservés
  dans le coffre-fort), inséré comme page (DXF, DWG) ou comme page de document (PDF,
  fichiers Office, images), lu avec un connecteur de la couche d'intégration
  d'entreprise, ou enregistré comme fichier local. Le nom du fichier suggère l'usage.
- **Export** : le fichier de projet, chaque export de *Fichier › Export* (PDF,
  DXF / DWG, l'implantation d'armoire en 3D en STEP, IGES, Parasolid, JT, glTF ou IFC, rapports, tables API, exports de
  plugins) et les exports des connecteurs vont dans le dossier affiché. Un fichier du
  même nom qui s'y trouve reçoit une nouvelle version (extrait, téléversé par
  morceaux, archivé avec le commentaire ; l'extraction est annulée en cas d'échec),
  sinon le fichier est ajouté.
  Un projet ouvert depuis le coffre-fort est de nouveau archivé avec *Fichier › Enregistrer (Archiver … dans Vault)*.
- **Éléments** : *Mettre à jour les liens d'articles depuis les éléments Vault* trouve
  l'élément Vault de chaque article du projet (par numéro d'élément ou par sa
  propriété *Part Number*) et présente les numéros d'élément, révisions et états de
  cycle de vie comme modifications du connecteur PLM à examiner, avec la même
  détection de conflits que les imports de fichiers.
- Ligne de commande / MCP : `vault_connect`, `vault_browse`, `vault_items`, `vault_download`,
  `vault_open`, `vault_upload`, `vault_sync_items`. Une session MCP se connecte une
  fois avec `vault_connect` ; la ligne de commande démarre une session par appel et
  prend le serveur, le jeton et l'identifiant du coffre-fort dans
  `OPENECRAFT_VAULT_URL`, `OPENECRAFT_VAULT_TOKEN` et `OPENECRAFT_VAULT_ID`
  (`vault_connect` affiche le jeton et l'identifiant du coffre-fort d'une connexion).

```bash
openecraft-cli vault_connect --url https://vault.example.com --user anna --password …
export OPENECRAFT_VAULT_URL=https://vault.example.com OPENECRAFT_VAULT_TOKEN=V:… OPENECRAFT_VAULT_ID=101
openecraft-cli vault_browse --query "Plant 1"
openecraft-cli -p plant.oec vault_upload --folder 42 --format step --comment "Panel layout"
```

- Les mots de passe ne sont pas enregistrés et le jeton n'est conservé qu'en mémoire.
  La version navigateur nécessite un serveur Vault (ou un proxy inverse placé devant)
  qui autorise les requêtes d'origines croisées depuis l'adresse de l'éditeur.
