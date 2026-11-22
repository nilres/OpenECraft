# Fluidique, procédé et ingénierie d'installation

Flexibles et tuyaux dans leurs propres réseaux, liens électro-fluidiques, boucles CCP, la préplanification de l'installation et l'ingénierie unifilaire / multifilaire.

## Fluidique

Les fils entre les orifices des symboles fluidiques et P&ID sont des flexibles / tuyaux
dans leurs propres réseaux (non mélangés aux potentiels électriques ; un orifice
fluidique sur un réseau électrique est une erreur), numérotés séparément des fils
(`H1`, `H2` …), avec fluide, diamètre nominal, couleur et longueur, listés dans la
*liste des flexibles*.

- *Liste des réseaux fluidiques* : chaque orifice reçoit sa fonction d'après sa
  désignation ISO 1219 / ISO 5599 (alimentation, pression P/1, travail A/B/2/4,
  échappement R/S/3/5, retour T, pilotage X/Y/Z/12/14, fuite L, en ligne) et chaque
  ligne de flexible son type (ligne de pression, de travail, de retour /
  d'échappement, de pilotage …) avec ses dérivations (tés).
- *Configuration des flexibles* : par flexible, tube ou tuyau, le type de ligne,
  l'article (au mètre), les raccords d'extrémité, la pression maximale et le rayon de
  courbure ; la longueur de coupe ajoute la surlongueur de coupe du projet par
  extrémité. Les tubes entre des appareils placés sur la même plaque de montage sont
  routés par ses goulottes comme les fils, ce qui donne la longueur lorsqu'aucune
  n'est fixée. La *liste des tubes / tuyauteries* totalise les pièces, les longueurs
  de coupe, les coudes et les raccords par type, diamètre et article.
- *Groupes d'appareils fluidiques* : terminaux de distributeurs, unités de traitement,
  blocs de distribution et groupes hydrauliques avec leurs appareils dans l'ordre des
  emplacements et le nombre de places.
- *Électro-fluidique* : les appareils fluidiques sont liés aux appareils électriques
  qui les actionnent ou les surveillent (bobines d'électrovannes, détecteurs de
  position et pressostats) ; les deux schémas affichent des références croisées l'un
  vers l'autre, et *Synchroniser* lie les électrovannes à la bobine de même numéro
  (`-QM3` → `-MB3`) et copie les textes de fonction.
- Contrôles : lignes de pression sur un retour / échappement, fluides ou diamètres
  différents sur un même flexible ou un même appareil, flexibles prévus pour une
  pression inférieure à la pression de service ou plus courts que leur parcours,
  électrovannes sans bobine, bobines sans vanne, liens erronés ou manquants, groupes
  pleins ou en double ; les échappements ouverts des appareils pneumatiques
  s'évacuent à l'atmosphère au lieu d'être des points de connexion ouverts.
- Rapports : *liste des réseaux fluidiques*, *configuration des flexibles*, *liste
  des tubes / tuyauteries*, *groupes d'appareils fluidiques*, *renvois
  électro-fluidiques*.

![Circuit pneumatique de l'exemple pneumatic_press avec vérins, distributeurs 5/2, régleurs de débit et références croisées vers les bobines d'électrovannes](../../images/pneumatics.png)

## Procédé (P&ID)

Les symboles P&ID (ISO 10628 : réservoir, pompes, vannes, échangeur de chaleur,
instruments) servent à la préplanification : les instruments portent des lettres de
fonction et un numéro de boucle (`TIC 101`), les boucles CCP de la préplanification
(*Outils › Fluide et procédé…*) listent leurs appareils, et la *vue d'ensemble des
boucles CCP* indique si chaque boucle est planifiée, en cours d'étude ou étudiée dans
le schéma.

## Préplanification

**Préplanification** (*Appareils › Préplanification…*) : l'installation sous forme
d'arborescence de segments (installation, zone, unité, …) avec la fonction et le lieu
hérités le long de l'arborescence et les appareils qui y sont planifiés (repère, type,
description, puissance, tension, article, macro de page) ; la puissance planifiée par
segment, des modèles de segments, l'état de chaque appareil planifié par rapport au
schéma (planifié, étudié, article différent ; également un contrôle et le rapport
*Préplanification*), des pages de schéma générées à partir des macros de page des
appareils non encore étudiés, et l'échange en CSV / Excel (commandes
`preplanning`, `add_segment`, `plan_device`, … ; outils `import_preplanning`,
`generate_preplanning_pages`).

## Ingénierie unifilaire / multifilaire

**Ingénierie unifilaire / multifilaire** (*Appareils › Unifilaire / multifilaire…*) :
les appareils sont planifiés avec des symboles unifilaires (disjoncteur,
interrupteur-sectionneur, disjoncteur-moteur, fusible, relais de surcharge,
contacteur, moteur, transformateur) sur des pages de synoptique et étudiés dans le
schéma multifilaire ; les deux partagent le repère d'appareil. La fenêtre liste chaque
appareil comme planifié, étudié, absent du schéma unifilaire ou non concordant, et
génère le schéma d'une page unifilaire (symboles de puissance tripolaires,
conducteurs, bobines de contacteurs) ou un schéma unifilaire des appareils de
puissance du schéma (disposés de l'alimentation vers les charges). Le contrôle signale
les appareils planifiés, les symboles non concordants, les liaisons du schéma
unifilaire absentes du schéma et les textes de fonction différents ; *Synchroniser les
textes de fonction* remplit les textes vides à partir de l'autre représentation.
