# Faisceaux de câbles

Faisceaux avec topologie 3D, protections et fixations, export KBL, l'interface faisceaux
pour les machines et les plans de clouage (*Liaisons › Faisceaux…*). Le projet d'exemple
`conveyor` comporte un faisceau de porte.

## Les faisceaux et leurs fils

Des liaisons point à point individuelles (`-S1:13 – -K1:A1`) sont affectées à des
faisceaux (`-WH1`), indépendamment des autres liaisons de la même section de
potentiel ; les affectations suivent les appareils renommés, les liaisons manquantes
sont signalées par le contrôle. Le rapport *données de faisceau* liste leurs fils avec
la longueur (routée dans l'implantation d'armoire ou longueur de terrain du faisceau)
et le traitement d'extrémité, la fenêtre des faisceaux montre les connecteurs avec
leur brochage et le diamètre du toron.

## Géométrie 3D du faisceau

Noeuds et segments avec des courbes médianes dérivées des parcours 3D de
l'implantation d'armoire (les connecteurs situés en dehors étant placés autour d'un
noeud commun), ou enregistrées dans le faisceau et modifiées (positions, rayons de
courbure, longueurs sur planche, points de contrôle) ; chaque fil est routé à travers
les segments, chaque segment a son diamètre de toron ; ruban, tubes, gaines annelées,
gaines et gaines thermorétractables sur (des parties de) segments, clips, colliers,
passe-fils et supports à des emplacements ; le contrôle signale les topologies
rompues et les rayons de courbure trop petits ; export du toron en 3D (glTF, JT).

## KBL

VDA 4964, schéma KBL 2.4 SR-1, vérifié par rapport au schéma dans les tests : un
conteneur par faisceau avec les fils généraux et les câbles (conducteurs), les boîtiers
de connecteurs avec emplacements et alvéoles, les contacts, joints et obturateurs
d'alvéoles, les protections de fils, fixations, accessoires, pièces jointes à la
livraison, assemblages et composants, les liaisons entre points de contact, les
liaisons du schéma (potentiels), les groupes de câblage, les options de projet sous
forme de modules, les révisions, approbations, unités, tolérances par défaut et la
topologie 3D (exporteurs `kbl`, `kbl_zip`).

## Interface faisceaux

Pour les machines et les outils ([harness-interface.md](../../harness-interface.md)) :
les données complètes des faisceaux en JSON (`harness_json`, commande `harness_data`)
et la liste de traitement des fils pour les machines de coupe, de dénudage et de
sertissage (`harness_processing`, CSV dans l'ordre de traitement avec longueurs de
coupe, longueurs de dénudage, contacts et joints) ; le plugin *Faisceaux* modifie la
topologie, les protections, les fixations, le traitement d'extrémité et les autres
pièces depuis la ligne de commande et MCP.

## Plans de clouage

Planches à l'échelle : le toron sous forme de branches allant d'un noeud commun aux
connecteurs, chacune aussi longue que l'exigent les longueurs de fil et aussi épaisse
que son toron, avec les clous, les longueurs de branche et le tableau de brochage de
chaque connecteur, sur A3 à la plus petite échelle normalisée qui convient (fenêtre
des faisceaux › *Plan de clouage (SVG)…*, exporteur `nailboard`, commande
`harness_nailboard`).
