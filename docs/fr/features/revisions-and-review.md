# Révisions, revue et gestion des modifications

Révisions avec pages verrouillées et modifications marquées, le circuit d'approbation, les références avec comparaison et fusion, les commentaires et annotations redline, et les demandes de modification.

## Révisions

Une révision enregistre un instantané de toutes les pages et peut les valider
(verrouiller) ; les pages verrouillées sont en lecture seule. Les modifications depuis
la dernière révision diffusée (symboles, fils, textes, graphiques, cotes, images,
définitions de liaisons et de câbles, pages et leurs propriétés ajoutés / modifiés /
supprimés) sont listées, marquées sur les pages par un triangle de révision et
rapportées dans la *vue d'ensemble des révisions* ; les éléments modifiés nomment les
propriétés modifiées avec l'ancienne et la nouvelle valeur
(`Device tag: -K1 › -K2; Position`).

## Processus de diffusion

Lorsque des approbateurs sont définis (*Données de projet › Révisions… › Circuit
d'approbation* : noms d'utilisateur et nombre d'approbations nécessaires), une nouvelle
révision commence comme brouillon, est soumise à la revue, approuvée ou rejetée par
les approbateurs (avec enregistrement de l'utilisateur, de la date et d'une note) et
diffusée une fois approuvée. Les pages modifiées de quelque manière que ce soit après
la soumission bloquent la diffusion jusqu'à ce que la révision soit soumise de nouveau
(les approbations antérieures deviennent caduques).

## Références et comparaison / fusion

Une référence enregistre le projet entier (pages, appareils,
articles, câbles, implantation d'armoire, paramètres) ; la diffusion d'une révision en
enregistre une automatiquement.
*Données de projet › Comparer et références…* compare le projet à une référence, aux
pages d'une révision ou à un autre fichier de projet : pages ajoutées / supprimées /
modifiées (avec les éléments modifiés), appareils, articles, potentiels, câbles,
faisceaux, armoires, borniers, connecteurs, cartes API et sections de paramètres. Les
différences sélectionnées sont reprises.

## Revue : commentaires et annotations redline

Des commentaires à une position de page avec état (ouvert, en
cours, résolu, rejeté), auteur, date et réponses ; des annotations redline (à
modifier) et greenline (modifié tel que construit) sous forme de ligne, rectangle,
nuage de révision ou texte. Ils sont dessinés sur un calque propre (affiché à l'écran,
absent des exports), listés dans l'onglet *Révision* du navigateur et peuvent être
ajoutés aux pages validées et aux pages qu'un autre utilisateur a verrouillées. Sur le
serveur de projets, ils sont fusionnés enregistrement par enregistrement, sans verrous.
Les **commentaires PDF** qu'un relecteur a faits dans un export PDF (Acrobat ou une
autre visionneuse : notes, zones de texte, surlignages, formes, réponses) sont importés
dans le calque de revue (*Révision › Importer des commentaires PDF…*, commande
`import_pdf_comments`) ; les rectangles et les lignes deviennent aussi des annotations
redline, les commentaires déjà importés sont ignorés.

## Demandes de modification

Des demandes venant de l'intérieur ou de l'extérieur du projet (numéro, titre,
description, demandeur, origine, priorité, état, pages et appareils concernés,
révision de mise en oeuvre) avec des commentaires liés (*Données de projet › Demandes
de modification…*). Échange avec d'autres systèmes en CSV
(`Number;Title;Description;Requester;Origin;Date;Priority;
Status;Pages;Devices;Revision`) ou JSON ; l'import met à jour les demandes dont le
numéro est connu.
