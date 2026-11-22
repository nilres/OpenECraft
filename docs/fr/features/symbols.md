# Symboles et bibliothèques

Les bibliothèques de symboles intégrées (IEC, NFPA / JIC, GB, GOST, JIS, fluidique, P&ID, unifilaire) et l'éditeur de symboles pour les symboles personnels.

## Bibliothèque de symboles IEC 60617

**Bibliothèque de symboles IEC 60617** (~330 symboles) : bobines de relais (AC, à
accrochage, polarisées, temporisations, interrupteurs horaires, compteurs, relais à
maximum / minimum de tension), contacts NO/NF/inverseurs (avancés / retardés, de passage,
temporisés à la fermeture / à l'ouverture), boutons-poussoirs, boutons à tirer,
interrupteurs à clé, à pédale et sélecteurs à rappel par ressort ou à accrochage et
diagrammes de positions, arrêts d'urgence à réarmement par rotation / traction / clé,
interrupteurs à came / à galet / de pression / de débit / de niveau / à bilame, capteurs
inductifs, capacitifs, magnétiques et photoélectriques, fusibles (BT HPC),
interrupteurs-fusibles et sectionneurs-fusibles, sectionneurs et
interrupteurs-sectionneurs, disjoncteurs, DDR, disjoncteurs-moteurs, relais de surcharge,
moteurs et ventilateurs AC / DC / EC, voyants de signalisation, luminaires (L, N, PE),
balises, sirènes, appareils de mesure indicateurs, enregistreurs et intégrateurs,
transformateurs de courant, transformateurs monophasés et triphasés (couplages) et
autotransformateurs, redresseurs et convertisseurs, bornes, entrées/sorties API,
potentiels, points d'interruption, et électronique : résistances, condensateurs (aussi
polarisés), diodes, diodes Zener et photodiodes, LED, transistors NPN / PNP, MOSFET à
canal N, thyristor, triac, thermistance CTN, photorésistance, optocoupleur et pont
redresseur.

## Symboles de plan d'installation

**Symboles de plan d'installation** (IEC 60617-11) : interrupteurs unipolaires et
va-et-vient, variateur, bouton-poussoir, sortie d'éclairage, luminaire fluorescent, prise
de courant double avec PE, boîte de dérivation, détecteur de mouvement et tableau de
distribution, chacun avec un point de connexion pour le câble.

## Éditeur de symboles et bibliothèques

Outre les symboles IEC, il existe des symboles NFPA / JIC
(30 : bobines, bobines de temporisation, contacts, contacts temporisés et de surcharge,
contacts principaux, boutons-poussoirs, arrêt d'urgence, sélecteur, interrupteurs à
pédale, de fin de course, à flotteur, de pression, de température, de débit et de
proximité, disjoncteurs, sectionneurs, fusibles, voyants, avertisseurs, électrovannes,
transformateurs de commande, moteurs), des symboles fluidiques (ISO 1219 : pompes,
compresseur, moteur hydraulique, vérins à simple et double effet, distributeurs 3/2, 4/2
et 5/2, clapets anti-retour, limiteurs de pression, vannes à boisseau sphérique et
régulateurs de débit, régulateur de pression, unité de conditionnement, filtre,
accumulateur, refroidisseur, manomètre, réservoir, silencieux), des symboles unifilaires
/ de vue d'ensemble (renvois vers l'appareil, hors de la liste des réseaux) et des boîtes
noires / boîtes API. Des jeux nationaux pour la Chine (GB), la Russie et la CEI (GOST) et
le Japon (JIS) portent les lettres d'appareil de GB/T 7159 (`KM`, `KA`,
`QF`, `SB`, `HL` …), GOST 2.710 (`KM`, `KK`, `QF`, `SB`, `HL` …) et JIS C 0401
(`MC`, `MCCB`, `PBS`, `PL`, `THR` …), de sorte que les appareils placés sont numérotés
`-KM1`, `-MCCB1` ; ils sont dessinés comme les symboles IEC, que GB/T 4728 et JIS C 0617
adoptent (les proportions ESKD des dessins GOST ne sont pas reproduites). Les symboles
personnels se dessinent dans l'éditeur de symboles (lignes, rectangles, cercles, textes,
champs de repère / d'étiquette, points de connexion sur la grille de 4 mm, fonction telle
que fonction principale, contact, borne, point API), ou se copient à partir d'un symbole
intégré ; ils sont enregistrés dans le projet et échangés sous forme de fichiers de
bibliothèque.
Un symbole placé de la bibliothèque (un symbole personnel ou un symbole EPLAN importé)
peut être dessiné plus grand ou plus petit : **Échelle** dans ses propriétés (ou
`update_symbol` avec `scale`, 0,1 à 10) met à l'échelle ses graphiques et ses points de
connexion, tandis que ses textes gardent leur taille, afin que les symboles importés
s'accordent au dessin ; les fils à ses points de connexion suivent.
