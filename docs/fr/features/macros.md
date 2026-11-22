# Macros, configurateur et édition en tableaux

Réutilisation de circuits sous forme de macros avec variantes et espaces réservés, pages générées par le configurateur, et modifications en masse dans des tableaux et des tableurs.

## Macros

Enregistrez une sélection (macro de fenêtre) ou une page entière (macro de page) dans
une bibliothèque de macros (enregistrée avec l'application, import / export JSON). Les
macros de fenêtre se placent à la souris (aperçu fantôme, les appareils reçoivent de
nouveaux repères), les macros de page insèrent une page.
Les macros copiées emportent les définitions de symboles personnalisés, les affectations
d'appareils et les articles de catalogue référencés. Les aperçus résolvent ces
définitions embarquées avant l'insertion. L'insertion dans un autre projet préserve les
entrées de catalogue existantes et les modifications locales de symboles.
Les espaces réservés tels que `{MOTOR}` dans les repères, les textes, les textes de
fonction et les désignations sont remplis à partir de **jeux de valeurs** lors de
l'insertion. **Variantes** : une macro peut contenir la même fonction en plusieurs
versions (A, B, … – par exemple démarrage direct et étoile-triangle) ; enregistrez une
sélection sous le nom d'une macro existante avec une autre variante, choisissez la
variante lors de l'insertion. **Boîtes de macro** : les macros de fenêtre placées
retiennent leur macro, leur variante, les valeurs des espaces réservés et leurs éléments
(cadre en pointillé à l'écran, *Affichage › Boîtes de macro*) ;
*Macros › Boîtes de macro › Mettre à jour les macros modifiées* remplace le contenu des
boîtes dont la macro a changé dans la bibliothèque, une boîte modifiée sur la page est
réenregistrée dans la bibliothèque (commande `list_macro_boxes`, outil
`update_macro_boxes`).

## Configurateur

**Configurateur** (*Données de projet › Configurateur…*) : des options avec leurs choix
(par exemple `STARTER` = DOL / SD) et des règles qui insèrent une macro de page dans une
variante avec des valeurs d'espaces réservés (`NAME=Pump; POWER={POWER}`) quand leur
condition est remplie (`STARTER=DOL & LINE!=2`, `LINE=1|2`). *Générer les pages* remplace
les pages générées auparavant (marquées dans les propriétés de la page, à désactiver pour
en garder une) ; les valeurs des options remplissent les espaces réservés `{OPTION}` ;
les problèmes (macro, variante ou option inconnue) l'arrêtent.

## Édition en tableaux

**Édition en tableaux** (*Données de projet › Modifier dans un tableau…*) : appareils
(repère, texte de fonction, description, article), liaisons (numéro de fil, couleur,
section, câble, conducteur), bornes (bornier, numéro, texte de fonction) et le rapport de
vérification (les résultats, une ligne par circuit ; ses données et l'examen visuel, une
ligne par valeur) sous forme de lignes ; filtrer, modifier les cellules sur place,
remplir une colonne ou rechercher et remplacer dans les lignes filtrées ; export / import
CSV pour les modifications dans un tableur (lignes par clé, seules les cellules modifiées
sont appliquées) ; les lignes sur des pages validées restent inchangées. Renommer un
appareil entraîne ses données de bornier, de connecteur et de carte API.
L'aller-retour fonctionne aussi avec des fichiers **Excel** (`.xlsx` : exporter,
modifier, importer) ; les tableaux de rapport s'exportent comme classeurs Excel depuis la
fenêtre Rapports et avec `report --format xlsx --path …` (ligne d'en-tête en gras et
figée avec filtre, nombres comme nombres, codes tels que `007` conservés comme texte).
