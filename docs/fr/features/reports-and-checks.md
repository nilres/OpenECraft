# Rapports, formulaires et contrôle du projet

Les rapports en fenêtres, en CSV / Excel ou en pages de rapport dynamiques, les cartouches et les formulaires de rapport (aussi graphiques et dessinés soi-même), et le contrôle du projet avec sa gestion des messages.

## Rapports

Table des matières, liste des appareils, nomenclature (avec articles et prix),
plan de bornes, liste des liaisons, liste d'affectation API, vue d'ensemble des
potentiels, vue d'ensemble des câbles, schéma de câbles – dans une fenêtre,
en export CSV, ou insérés comme pages de rapport dynamiques.

Le filtre du panneau *Rapports* trouve un rapport par son titre, son nom ou les
normes qu'il suit, sans tenir compte de la casse, des espaces et des tirets :
`VDE 0100 600`, `0100-600` ou `IEC 60364-6` trouvent les trois rapports de vérification,
`ISO 13849` les fonctions de sécurité, `60909` le rapport de court-circuit
(`list_reports` avec `search` sur la ligne de commande).

Un câble vendu au mètre a besoin d'une longueur positive pour sa quantité de matériel.
Une longueur manquante est montrée par `?` ; si d'autres câbles de cet article ont des
longueurs, la ligne montre la quantité connue plus `?`. Les prix et la somme du rapport
indiquent aussi la part inconnue. La vérification des câbles nomme chaque câble au mètre
qui a besoin d'une longueur.
Un câble compté à la pièce garde sa quantité d'une pièce quand aucune longueur n'est
fournie.

Les articles de câble peuvent fournir `Outer diameter: 6.3 mm` dans leurs données
techniques. Sélectionner l'article applique cette dimension au cheminement ; un article
sans dimension valide revient à l'estimation d'après ses conducteurs et sa section.
`apply_cable_part` expose la même opération de catalogue par le registre de commandes
partagé.
Le sélecteur de câbles inclut les articles locaux du projet ainsi que le catalogue
configuré ; les métadonnées et les classifications locales priment pour les numéros
d'article répétés.
`list_cable_parts` expose les choix du projet et les choix intégrés aux plugins et à la
CLI/MCP.

## Formulaires

Les cartouches sont des listes de lignes, de rectangles, de textes avec des espaces
réservés (`{project}`, `{page}`, `{sheet}`, `{page.previous}` / `{page.next}`,
`{rev.date}`, tableaux de révisions avec `{rev.1.name}`, `{rev.2.name}` …, propriétés
utilisateur `{prop:Name}` …) et d'images, modifiées dans l'éditeur de cartouches et
partagées sous forme de fichiers ; un par projet avec un remplacement par page
(intégrés : Standard, Compact, None). Les textes peuvent être en gras, en italique et
pivotés (par exemple de 90° pour des textes allant vers le haut). Un élément image garde
sa propre image (PNG, JPEG, par exemple le logo de l'entreprise, qui voyage avec le
fichier de cartouche) ou montre le logo du projet de *Propriétés du projet › Logo* (par
exemple celui du client, différent dans chaque projet), ajusté dans son rectangle avec
son rapport d'aspect ; la page de garde montre le logo du projet au-dessus du nom du
projet ou à une place propre (*Propriétés du projet › Logo sur la page de garde*,
`set_cover_logo`). Les formulaires de rapport dessinés prennent les mêmes éléments.
Chaque élément peut être limité aux pages de certains types (par exemple un champ
seulement sur les pages de rapport) ou à un espace réservé rempli (par exemple la
deuxième ligne d'un tableau de révisions seulement avec `{rev.2.name}`) ; les lignes, les
rectangles et les textes reçoivent leur propre couleur, les lignes et les rectangles des
pointillés. Les **codes QR** encodent un texte avec des espaces réservés (par exemple un
lien vers le projet dans la gestion documentaire avec `{project}` et `{page}`). Le
cartouche d'une page est le sien, sinon celui de son type de page (*Cartouche par type de
page*), de son cadre (par exemple un plus étroit pour l'A4 portrait), celui par défaut du
projet. `{rev.status}`,
`{rev.approved_by}` et `{rev.approved_date}` montrent l'état du circuit d'approbation des
révisions et qui a approuvé la dernière révision.
**Tampon** : un grand texte en travers de chaque page qui suit le circuit d'approbation
des révisions, par exemple BROUILLON pendant qu'une révision est préparée, POUR
APPROBATION en relecture, rien (ou TEL QUE CONSTRUIT) une fois validée (*Formulaires ›
Cartouches › Tampon*, `set_stamp`).
**Bibliothèque de l'entreprise** : les cartouches, schémas de rapport et cadres propres
au projet sont publiés sur le serveur d'articles et chargés dans d'autres projets
(*Cartouches › Bibliothèque de l'entreprise*, `parts_company_forms` (`publish`) ; la
publication demande le droit de modifier les catalogues). Les formulaires de rapport
choisissent les colonnes, leur ordre, le titre et les lignes par page de rapport,
l'alignement de chaque colonne, les colonnes totalisées dans une ligne de total sous la
dernière ligne (avec l'unité de leurs valeurs) et une ligne sur deux ombrée.
**Formulaires graphiques** : un formulaire de rapport du plan de bornes, de câbles ou de
connecteurs ou de l'affectation API peut passer à sa forme graphique – le bornier dessiné
avec une case par borne (étages marqués), les butées d'arrêt et les flasques d'extrémité,
les ponts et les cibles externes et internes avec potentiel, texte de fonction et
emplacement lié ; ou chaque câble comme une bande avec ses conducteurs d'un bout à
l'autre (potentiel, numéro de fil, conducteurs de réserve en pointillé) ; le plan des
connecteurs comme un boîtier avec une moitié douille et une moitié broche par broche ;
l'affectation API comme un module par carte avec les entrées arrivant par la gauche et
les sorties partant vers la droite.
**Formulaires dessinés** : tout rapport (aussi ceux des plugins) peut être dessiné avec
un formulaire personnel – des graphiques dessinés une fois par page et un dessin de ligne
répété pour chaque ligne, vers le bas pour les listes ou latéralement (par exemple une
borne à côté de l'autre) ; ses textes prennent les colonnes comme espaces réservés
(`{Device tag}`, `{Part number}` …), `{report}` et les espaces réservés du cartouche.
L'éditeur de formulaires (*Formulaires de rapport › Formulaire dessiné personnalisé*)
part du tableau du rapport et montre la page avec ses lignes réelles pendant que des
lignes, des rectangles et des textes sont ajoutés et glissés.

![La forme graphique du plan de bornes avec les butées d'arrêt, les flasques d'extrémité et les cibles de chaque borne](../../images/terminal-diagram.png)

## Contrôle du projet

Fonctions auxiliaires sans fonction principale, fonctions principales en double,
points de connexion ouverts, points d'interruption non appariés, courts-circuits entre
potentiels, numéros de borne / adresses API en double, appareils sans article, plus de
fils à un point de connexion que sa borne n'en accepte ou fils trop fins ou trop gros
pour lui (vérification *Fils aux points de connexion*, d'après les données de points de
connexion des articles, par exemple issues de fichiers EDZ).
**Gestion des messages** (dans l'esprit des schémas de vérification d'EPLAN) : chaque
message a un code par type (`wires.3f9a1c`, le même dans chaque langue) et une empreinte
par occurrence. Cliquez avec le bouton droit sur un message dans le panneau Messages pour
l'accepter comme vérifié (masqué et non compté jusqu'à ce qu'il change), pour changer le
niveau de son type (erreur, avertissement, remarque, désactivé) ou pour désactiver sa
vérification ; les réglages (⚙) listent les vérifications, les niveaux modifiés et les
messages acceptés (avec note, utilisateur, date et s'ils sont encore signalés). L'onglet
filtre par niveau, page et texte et exporte les messages en CSV.
Les réglages sont enregistrés avec le projet ; les vérifications désactivées et les
niveaux modifiés s'échangent sous forme de fichier de **schéma de vérification** (JSON)
entre projets. Commandes :
`list_checks`, `check_settings`, `set_check`, `set_message_level`, `accept_message` ;
outils `export_check_scheme`, `import_check_scheme` ; `check` montre les codes et les
empreintes.
