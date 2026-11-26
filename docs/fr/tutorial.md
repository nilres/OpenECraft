# Prise en main : de la première page à la nomenclature

Ce tutoriel parcourt un petit circuit de commande dans OpenECraft : pages, symboles,
fils, appareils avec leurs articles, bornes, un API, le contrôle du projet, les rapports,
l'implantation d'armoire et la sortie. Les commandes sont notées *Onglet › Commande* :
les onglets du ruban, en haut, regroupent les commandes par thème, *Fichier* ouvre la vue
Backstage avec nouveau, ouvrir, enregistrer, import, export et impression. Chaque commande
se trouve aussi par son nom avec *Rechercher une commande* (Ctrl+Shift+P), Alt affiche les
touches d'accès du ruban et F1 ouvre l'aide du panneau utilisé. L'interface utilisateur
est disponible en anglais, allemand, français, espagnol et 16 autres langues
(*Affichage › Langue*) ; les noms ci-dessous sont les noms français.

Le moyen le plus rapide de tout voir d'un coup est *Fichier › Nouveau › Ouvrir le projet
de démonstration* : une commande de convoyeur avec circuits de puissance et de commande,
bornes, un câble, un API, un faisceau et une implantation d'armoire. La plupart des étapes
ci-dessous peuvent y être essayées.

## 1. Un nouveau projet

- *Fichier › Nouveau › Nouveau projet* démarre un projet vide avec une page de schéma ;
  les modèles en dessous partent d'un modèle intégré (cadres de style IEC ou NFPA,
  structure, potentiels) et *À partir d'un fichier modèle…* de n'importe quel projet
  enregistré. La page d'accueil montre les mêmes choix et les projets ouverts en dernier.
- *Données de projet › Propriétés du projet…* contient les données du projet affichées
  dans les cartouches (nom, client, auteur) ainsi que le cadre et le cartouche par défaut.
- Le projet est enregistré dans un seul fichier `.oec` (*Fichier › Enregistrer*, Ctrl+S).
  Dans le navigateur, le projet est conservé dans le stockage du navigateur et téléchargé
  avec *Fichier › Télécharger le projet*.
  Pour la gestion de versions (git), la version de bureau l'enregistre aussi comme projet
  fractionné (*Fichier › Enregistrer comme projet fractionné…*) : un dossier avec l'index
  `<name>.oecp` et un petit fichier par page, appareil et article, de sorte qu'une
  modification ne touche que les fichiers qu'elle concerne.
- *Fichier › Fermer le projet* (Ctrl+Shift+F4) ferme le projet, propose d'enregistrer
  les modifications non enregistrées et affiche la page d'accueil.

## 2. Pages et structure

- Le navigateur *Pages*, à gauche, liste les pages ; *Accueil › Nouvelle page de schéma*
  en ajoute une après la page courante, *Accueil › Nouvelle page* les autres types : page
  de garde, vue d'ensemble, graphiques, document (un PDF ou une image incorporés), pages
  de rapport et d'implantation.
  Les pages ouvertes récemment sont des onglets au-dessus de la page (Ctrl+Tab bascule).
- *Accueil › Propriétés de la page…* (ou un clic droit sur une page) donne à une page sa
  description et sa structure : la fonction (`=A1`) et l'emplacement (`+CAB1`). Les
  appareils placés sur la page prennent cette structure (`=A1+CAB1-K1`) ; sur la page,
  seule la partie qui diffère est affichée.
- Le bouton d'arborescence du navigateur *Pages* regroupe les pages par structure ;
  *Accueil › Renuméroter les pages…* les renumérote. *Accueil › Nouvelle sous-page* insère
  `3.1`, `3.2`, … après la page courante ; la renumérotation les garde comme sous-pages
  de leur page.
- *Accueil › Nouvelle section* structure les pages comme des répertoires : une section
  contient des pages et des sous-sections, et les noms des pages commencent par son numéro
  (`2.3.1` est la page 1 de la section 2.3). Dans le navigateur *Pages*, faites glisser
  une page entre deux pages ou sur une section (ou choisissez sa *Section* dans les
  propriétés de la page) pour l'y déplacer, et faites glisser une section entre des
  sections ou sur une autre ; la ligne ou le cadre montre où elle arrive, l'étiquette au
  pointeur le nom qu'elle reçoit, une section repliée s'ouvre quand le glissement est
  maintenu au-dessus d'elle, et l'espace vide sous la liste est le niveau supérieur. Un
  clic droit sur une section ajoute des pages et des sous-sections, la déplace, la
  renumérote ou la supprime, et les pages suivent.
  *Renuméroter les pages…* recompte dans chaque section et peut d'abord numéroter les
  sections 1, 2, 3 … Dès qu'un projet a des sections, les sous-pages portent des lettres
  (`2.3.1.a`).

## 3. Placer des symboles

- Le *Centre d'insertion*, à droite (*Insertion › Centre d'insertion*, Ins), trouve les
  symboles de toutes les bibliothèques (IEC 60617, NFPA, fluidique, P&ID, plan
  d'installation, …), les macros, les appareils issus d'articles et les fonctions non
  placées ; `*` et `?` sont des caractères génériques. Cliquez sur une entrée, puis
  cliquez dans la page pour la placer. Le placement continue jusqu'à Esc ou un clic
  droit. L'étoile garde une entrée parmi les favoris ; les dernières utilisées viennent
  en premier.
- R fait pivoter et M reflète le symbole en cours de placement (ou la sélection). Les
  touches se modifient dans *Fichier › Paramètres › Touches et gestes*, comme les boutons
  de la souris et les touches de modification.
- Les repères d'appareil sont attribués automatiquement d'après la lettre
  d'identification (`-K1`, `-S2`).
  Pour placer les contacts d'un contacteur, sélectionnez d'abord sa bobine : les contacts
  placés ensuite lui appartiennent et reçoivent les numéros libres suivants (13/14,
  21/22, …). Les contacts et les bobines affichent leurs renvois (`/3.4`) en direct.
- Sélectionnez un symbole pour le modifier dans le panneau *Propriétés*, à droite : ses
  onglets contiennent le repère, le texte de fonction et les désignations des points de
  connexion (*Appareil*), le placement de ses textes (*Affichage*), les articles de
  l'appareil (*Articles*), d'autres propriétés et les liaisons avec les renvois. F2 ou un
  double-clic montre la même chose dans une fenêtre ; un clic droit sur la page propose
  les commandes pour la sélection.

## 4. Fils et potentiels

- W (*Insertion › Fil*) lance l'outil fil : cliquez le début, les coins intermédiaires et
  la fin ; Espace change la direction du coin, Entrée ou un double-clic termine le fil,
  Esc ou un clic droit termine l'outil. Un fil qui se termine dans le vide est signalé
  par la vérification *Extrémités de connexion ouvertes*. Avec *Édition › Connexion auto*
  activé, les symboles placés dans l'alignement d'autres sont reliés automatiquement.
- Les potentiels (`L1`, `+24V`, `0V`, …) sont des symboles de potentiel (*Insertion ›
  Potentiel* ou la bibliothèque *Potentiels*) ; *Liaisons › Potentiels* définit leur
  type, leur tension, la couleur et la section des fils. Le navigateur *Non placés* liste
  les potentiels définis qui ne sont encore sur aucune page.
- Les points d'interruption (*Insertion › Point d'interruption*) prolongent un réseau
  d'une page à l'autre ; leurs renvois se règlent dans *Liaisons › Renvois…* (étoile ou
  chaîne).
- *Liaisons › Numérotation des fils…* numérote les liaisons (par potentiel, par page,
  formats tels que `{page}.{n}`) ; un numéro saisi dans les propriétés d'un fil est
  conservé.

## 5. Articles

- *Appareils › Gestion des articles* ouvre la base de données d'articles (groupes de
  produits, filtre, éditeur d'articles avec données commerciales, techniques et données
  de montage typées, modèles de fonction, points de connexion, modèles 2D / 3D,
  ensembles). Les articles s'importent depuis CSV, BMEcat, XML d'articles EPLAN, les
  fichiers EDZ de l'EPLAN Data Portal (*Fichier › Import*) ou le portail de données
  (*Données de base › Portail de données*).
- Affectez les articles dans l'onglet *Articles* des propriétés d'un appareil (*Ajouter
  un article…*) ; la sélection d'appareil ne propose que les articles qui fournissent la
  fonction principale de l'appareil. *Appliquer les désignations* reprend de l'article
  les désignations des points de connexion. La section *Appareils* du Centre d'insertion
  place un nouvel appareil à partir d'un article avec des fonctions.
- Un article peut aussi être glissé depuis l'arborescence de la *Gestion des articles* ou
  depuis les résultats du *Portail de données* sur une page de schéma : le symbole de sa
  fonction principale y est placé comme nouvel appareil avec l'article affecté. Un article
  qui n'est pas encore dans la base de données d'articles est d'abord importé (depuis le
  catalogue d'exemple, ou téléchargé depuis le serveur d'articles). La ligne de commande
  fait de même avec `place_part`.
- Le navigateur *Non placés* liste alors les fonctions des articles affectés qui ne sont
  pas encore dans le schéma (par exemple les contacts de réserve d'un contacteur) ;
  faites-en glisser une dans la page pour la placer avec son repère et ses désignations.

## 6. Bornes et connecteurs

- Les bornes se placent comme les autres symboles (bibliothèque *Bornes*) ; les bornes de
  même repère forment un bornier. *Appareils › Borniers* les ordonne, ajoute des
  réserves, des ponts (enfichables, à cavalier, filaires), des butées d'arrêt et des
  flasques d'extrémité ainsi que des blocs à étages.
- Le navigateur *Borniers / connecteurs* montre chaque bornier dans l'ordre du rail et
  chaque connecteur avec ses broches ; un clic montre une borne ou une broche placée, les
  réserves et les broches inutilisées se glissent dans la page.
- *Liaisons › Câbles* définit les câbles ; sélectionnez des fils et utilisez *Affecter au
  câble*, ou tracez une ligne de définition de câble à travers eux (*Insertion ›
  Définition de câble*).

## 7. API

- Les entrées et sorties API se placent depuis la bibliothèque *API* avec le repère de
  leur carte et leur adresse. *Appareils › Matériel automate et topologie de bus* ajoute
  des stations, des châssis et des modules à partir de descriptions d'appareil (GSDML,
  ESI) ; *Appareils › Vue d'ensemble API* liste tous les points avec les appareils qui y
  sont raccordés et réadresse les cartes.
- Le navigateur *API* montre les cartes avec leurs canaux ; les canaux libres des cartes
  ayant une description d'appareil se glissent dans la page.
- *Fichier › Export* écrit la table de symboles pour l'outil de programmation de l'API :
  TIA Portal, Studio 5000, STEP 7 classic, TwinCAT 3, Control Expert, GX Works ou
  IEC 61131-3.

## 8. Le contrôle du projet

- Le panneau *Messages*, sous la page, exécute les vérifications en continu (appareils,
  liaisons, bornes, câbles, API, implantation d'armoire, …) et liste les messages sous
  forme de tableau ; un clic sur une colonne trie, un clic sur un message saute à son
  emplacement (F4 affiche le panneau).
- Cliquez avec le bouton droit sur un message pour l'accepter comme vérifié, pour changer
  le niveau de son type ou pour désactiver sa vérification ; *Données de projet ›
  Réglages de vérification* les liste et échange des schémas de vérification avec
  d'autres projets.

## 9. Rapports

- *Données de projet › Rapports* montre chaque rapport (liste des appareils,
  nomenclature, plan de bornes, schéma de câbles, plan des connecteurs, affectation API,
  liste des liaisons, …) sous forme de tableau avec export en CSV et Excel.
- *Insérer comme page de rapport* (ou *Accueil › Nouvelle page › Nouvelle page de
  rapport*) insère un rapport sous forme de pages du projet ; elles se mettent à jour
  quand le projet change. *Données de base › Formulaires* choisit les colonnes d'un
  formulaire de rapport ou fait passer les plans de bornes, de câbles et de connecteurs
  et l'affectation API à leur forme graphique.
- La nomenclature totalise tous les articles des appareils, bornes, câbles, faisceaux, du
  matériel d'implantation d'armoire, des embouts et du matériel de marquage ; les
  ensembles listent leurs composants.
- *Données de projet › Modifier dans un tableau* modifie en masse les appareils,
  liaisons, bornes, câbles, broches de connecteur, points API, pages et articles (filtre,
  remplissage, rechercher et remplacer, CSV) ; les navigateurs l'ouvrent pour leurs
  lignes.

## 10. Implantation d'armoire

- L'espace de travail *Implantation d'armoire* (*Affichage › Espaces de travail* ou le
  sélecteur de la barre d'accès rapide) montre l'onglet *Espace de montage* du ruban :
  armoires, plaques de montage, rails, goulottes, jeux de barres, découpes et les
  appareils du schéma ; la vue 3D montre l'armoire avec la porte, les fils routés et le
  remplissage des goulottes.
- L'espace de travail *Câblage* est l'assistant de câblage pour l'atelier (les fils dans
  l'ordre avec couleur, section, longueur et traitement des extrémités) ; l'espace de
  travail *Montage* est l'assistant de montage (les éléments dans l'ordre de montage avec
  position, taille et article). *Câblage › Mode tablette* agrandit les textes et les
  boutons pour un écran tactile.
- Les plans de perçage (*Espace de montage › Données CN*) et l'implantation d'armoire en
  3D (STEP, glTF, JT, IFC, *Fichier › Export*) sont exportés pour la fabrication.

## 11. Sortie

- *Fichier › Export* liste tous les exports : PDF (toutes les pages ou une sélection de
  pages telle que `1-3, 5`, avec signets et renvois cliquables), HTML pour l'impression,
  DXF / DWG, SVG, la liste de fils pour la production, le marquage (avec des schémas de
  repérage pour les logiciels de marquage), le KBL et les plans de clouage des faisceaux,
  les tables API, AutomationML et les formats 3D. Les exports avec options les demandent
  d'abord.
- *Fichier › Imprimer* ouvre toutes les pages ou la page courante dans le navigateur avec
  sa boîte de dialogue d'impression.
- *Données de projet › Exporter en PDF dans la langue* écrit le PDF dans l'une des
  langues du projet (*Données de projet › Langues du projet*).

## 12. Modifications et révision

- *Données de projet › Révisions* crée des révisions ; les modifications depuis la
  dernière révision validée sont marquées sur les pages et listées avec les anciennes et
  les nouvelles valeurs dans la vue d'ensemble des révisions. Les pages validées sont
  verrouillées.
- L'espace de travail *Révision* et *Insertion › Commentaire* / *Annotation* contiennent
  les commentaires et les annotations en rouge / en vert ; *Importer des commentaires
  PDF…* reprend les commentaires qu'un relecteur a faits dans un export PDF.

## 13. Automatisation

Tout ce qui précède est aussi disponible sans l'interface utilisateur : la ligne de
commande (`cargo run -p openecraft-cli -- help`) et le serveur MCP pour les agents d'IA
(`openecraft-cli mcp`) offrent les mêmes commandes, vérifications, rapports et exports, et
les plugins ajoutent les leurs (voir [plugins.md](plugins.md)) ; dans l'éditeur, ils se
trouvent dans *Plugins › Commandes de plugin*, où l'étoile épingle une commande à
l'onglet *Plugins*.
