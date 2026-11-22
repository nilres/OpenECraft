# Dimensionnement électrique, calculs et vérification

Dimensionnement des conducteurs, court-circuit et coordination des protections, bilan de puissance, CEM, sécurité fonctionnelle et jeux de règles – tous intégrés au contrôle du projet – ainsi que le rapport de vérification de l'installation.

## Dimensionnement électrique

**Dimensionnement électrique** (IEC 60204-1) : les fils situés derrière un appareil de
protection – depuis son côté charge, à travers les contacts principaux et les relais de
surcharge – doivent supporter son courant assigné (`Current` de l'article, la valeur
haute d'une plage de réglage ; courant admissible selon le tableau 6, méthode B2) ;
les câbles reçoivent leur chute de tension jusqu'à la charge de plus fort courant
assigné à leurs extrémités (√3 pour trois conducteurs, cuivre à la température de
service), rapport *Dimensionnement des câbles* ; le contrôle signale les fils et câbles
trop fins (erreurs) et les chutes de tension supérieures à 5 %. Le courant admissible
est corrigé selon la température ambiante, le groupement des circuits
(IEC 60364-5-52, B.52.17), l'aluminium et l'isolation PR. Un interrupteur-sectionneur
à fusibles équipé de barrettes de sectionnement (sous-groupe d'articles `Solid link`,
le sectionnement avant et après les transformateurs d'un comptage) ne fait que
sectionner : il ne protège rien, n'a pas de sélectivité propre et le courant le
traverse ; la charge d'un câble est le courant assigné de l'article qui assure la
fonction à son extrémité (la commande motorisée d'un disjoncteur, non le
disjoncteur).

## Calculs électriques et vérification

**Calculs électriques et vérification** (*Outils › Calculs électriques…*) :
- *Approvisionnement* : tension, schéma de liaison à la terre (TN-S, TN-C, TN-C-S, TT,
  IT), courants de court-circuit maximal et minimal au point d'alimentation, courant
  assigné ;
- *Court-circuit et coordination des protections* (IEC 60909-0 pour les réseaux
  radiaux, IEC 60364-4-41, -4-43) : la source d'après les courants de court-circuit au
  point d'alimentation (avec son R/X) ou d'après un transformateur (puissance assignée,
  uk, pertes en charge, facteur de correction KT) et le réseau amont (S''kQ) ; les
  câbles de l'alimentation jusqu'à chaque appareil de protection, puis jusqu'à
  l'extrémité de son circuit ; les moteurs asynchrones sur le réseau ajoutent
  ILR / IrM = 5 (ou `Locked rotor current`) lorsqu'ils dépassent 1 % du courant de
  l'alimentation. Les appareils de protection du réseau avec leurs bandes
  temps-courant (IEC 60898-1 B / C / D / K / Z, protection moteur classe 10, fusibles
  gG / aM, déclencheurs électroniques LSI avec `Long-time delay`, `Short-time pickup`,
  `Short-time delay` ; données techniques de l'article `Characteristic`, `Breaking
  capacity`, `Making capacity`, `Instantaneous trip`, `Let-through energy`,
  `Pre-arcing energy`), remplacées par les courbes du fabricant lorsqu'elles sont
  données (`Trip curve min` / `Trip curve max` : `1.5: 600; 3: 20; 10: 0.01` en
  multiples de In, `Let-through energy: 1 kA: 8000; 6 kA: 25000`). Sont vérifiés : le
  pouvoir de coupure par rapport au courant de court-circuit et le pouvoir de fermeture
  par rapport à sa valeur de crête (κ d'après R/X), la coupure au courant de défaut
  minimal à l'extrémité de chaque circuit dans les temps du tableau 41.1 selon U0
  (TN 0,8 / 0,4 / 0,2 / 0,1 s, TT 0,3 / 0,2 / 0,07 / 0,04 s, circuits de distribution
  5 s / 1 s ; TT : dispositif différentiel nécessaire ; IT : le second défaut, sous U
  ou, avec un neutre distribué, sous U0), la tenue k²S² des câbles, la sélectivité
  avec l'appareil amont (totale / partielle jusqu'à un courant) d'après la table de
  coordination du fabricant, les fusibles gG dans un rapport de 1,6, l'énergie de
  préarc d'un fusible amont ou les bandes temps-courant ; la protection
  d'accompagnement de la table de coordination couvre un pouvoir de coupure trop
  faible. Les tables de coordination (limites de sélectivité et d'accompagnement en kA
  par référence d'article) se modifient dans la fenêtre et avec `coordination_table`.
  Rapports *Court-circuit et coupure*, *Sélectivité* (avec sa base) ;
- *Bilan de puissance* : moteurs, variateurs, chauffages, alimentations et
  transformateurs du réseau avec puissance active / apparente et courant (`Power`,
  `Current`, `Power factor`, `Efficiency`), la charge de chaque appareil de
  protection, la demande avec le facteur de diversité assigné de l'IEC 61439-1
  (tableau 101) ou un facteur fixé, les courants de phase et le déséquilibre, et le
  bilan de charge de chaque alimentation 24 V (`Coil power`, `Current consumption`) ;
  rapports *Bilan de puissance*, *Bilan de charge des alimentations* ;
- *CEM* : câbles moteur des variateurs de fréquence blindés aux deux extrémités,
  signaux analogiques dans des câbles blindés, pas de conducteurs de puissance et de
  signal dans un même câble, protection contre les surtensions ;
- *Sécurité fonctionnelle* (ISO 13849-1) : fonctions de sécurité composées de
  sous-systèmes avec leurs appareils, catégorie, DCavg, manoeuvres par an et CCF ; PL
  par la procédure simplifiée (figure 5) d'après `MTTFd` / `B10d` des articles ou les
  valeurs certifiées `PL` / `PFHd`, combinés selon le tableau 11 ou, lorsque chaque
  sous-système a un PFHd, par leur somme, avec le SIL correspondant ; un second canal
  composé d'autres appareils est symétrisé (D.2), le DCavg provient du `DC` des
  appareils lorsque le sous-système n'en donne pas (annexe E), les appareils dont le
  T10d tombe dans la durée de mission de 20 ans sont nommés pour remplacement ;
  rapport *Fonctions de sécurité* ;
- *Jeux de règles* : bibliothèques intégrées (IEC 60204-1, IEC 61439-1,
  UL 508A / NFPA 79, ISO 13849-1, CEM) et jeux de règles client (import / export
  JSON) qui resserrent les limites (chute de tension, temps de coupure, sections
  minimales, réserve de charge, déséquilibre de phases, sélectivité totale, blindage,
  protection contre les surtensions) et ajoutent des contrôles
  « chaque élément d'un périmètre (projet, pages, appareils, articles, câbles,
  liaisons, potentiels) qui remplit ces conditions doit remplir celle-ci » ; rapport
  *Vérification des règles* ; tous les résultats font partie du contrôle du projet.

## Rapport de vérification

Le **rapport de vérification** (vérification initiale, IEC 60364-6 / DIN VDE
0100-600) comprend quatre parties : les données du rapport avec l'examen visuel, les
sources des circuits TBTS, TBTP et séparés, les circuits avec leurs résultats, et le
verdict avec la signature. Il est présenté dans les rapports *Rapport de vérification*
(les circuits), *Rapport de vérification : TBTS, TBTP et séparation* et
*Rapport de vérification : synthèse*, modifié dans le panneau *Outils › Rapport de vérification* de l'éditeur (les mêmes
sections que la page web : données du rapport et examen visuel, limites, signature,
une section par source et par circuit, *Exporter en Excel*), dans *Données de projet ›
Modifier dans un tableau* (*Rapport de vérification*, une ligne par circuit, et
*Rapport de vérification : données*, une ligne par valeur du rapport et des sources),
avec la commande `inspection_report` et sur la page web du serveur de projets.

**Les circuits** sont les appareils de protection alimentés par le réseau, dans l'ordre
du schéma, renseignés à partir du schéma :

- circuit (repère d'appareil), description (de l'appareil, sinon son texte de
  fonction), phases, les câbles derrière l'appareil avec leurs types, la section (des
  câbles, sinon des fils) ;
- appareil de protection : caractéristique de déclenchement et courant assigné
  (données d'article, comme pour le calcul de court-circuit) ;
- dispositif différentiel protégeant le circuit (le sien ou un dispositif amont) avec
  son type (`Residual current type` de l'appareil ou de son article ; un `S`, comme
  dans `A S`, désigne un dispositif sélectif) et son courant différentiel assigné ;
- l'impédance de boucle la plus élevée admise : deux tiers de U0 / Ia (Ia : le courant
  qui coupe dans le temps du tableau 41.1 ; IEC 60364-6, annexe C ; également en
  schéma TT, 411.5.4 ; IT : le second défaut, U / 2 Ia ou, avec un neutre distribué,
  U0 / 2 Ia, 411.6.4), ou, lorsqu'un dispositif différentiel protège le circuit,
  U0 / IΔn (TN, IT) et 50 V / IΔn (TT : RA × IΔn ≤ UL, 411.5.3 ; Zs inclut RA).

Ce que le projet ne donne pas est indiqué comme *manquant*, jamais supposé : pas de
caractéristique C, pas de 30 mA et aucune limite qui en découle, contrairement aux
calculs, qui les supposent.

Les résultats d'un circuit, conservés tels que saisis (`0,35`, `> 500`) et vérifiés
par rapport à leurs limites :

| Résultat | Limite |
| --- | --- |
| Continuité du conducteur de protection (Ω) | pas de limite dans la norme ; à côté, la résistance calculée d'après les câbles du circuit (longueur, section, matériau du conducteur à 20 °C), un avis au-delà de 1,5 fois celle-ci |
| Tension d'essai d'isolement (V) | au moins 500 V jusqu'à 500 V nominal, 1000 V au-delà (tableau 6.1) |
| Isolement L–PE (MΩ ; les conducteurs de phase, ou tous les conducteurs actifs reliés entre eux, par rapport à PE) | au moins 1 MΩ |
| Isolement N–PE et L–N (MΩ, lorsqu'ils sont mesurés séparément) | au moins 1 MΩ |
| Impédance de boucle Zs (Ω) | l'impédance de boucle la plus élevée ci-dessus |
| Chute de tension (%) | la limite des jeux de règles (5 % sans jeu de règles) |
| Courant de déclenchement DDR (mA) | ½ IΔn … IΔn |
| Temps de déclenchement DDR à IΔn (ms) | au plus 300 ms ; type S 130 … 500 ms |
| Temps de déclenchement DDR à 5 × IΔn (ms) | au plus 40 ms ; type S 50 … 150 ms |
| Tension de contact UB à IΔn (V) | au plus 50 V |
| Polarité | OK / Non conforme |
| Ordre des phases (circuits triphasés) | OK / Non conforme |
| Essai de fonctionnement (appareillage, verrouillages, commandes) | OK / Non conforme |
| Vérifié par, date, appareil de mesure, remarque | enregistrés |

Un circuit est *Non conforme* lorsqu'une valeur sort de sa limite ou qu'un essai a
échoué, *Conforme* lorsque la continuité, l'isolement (avec sa tension d'essai),
l'impédance de boucle, la polarité, l'essai de fonctionnement, (en triphasé) l'ordre
des phases et (avec un DDR) le temps de déclenchement à IΔn sont présents, *Incomplet*
entre les deux et *Non vérifié* sans valeurs.

**TBTS, TBTP et séparation électrique** (6.4.3.4) : chaque alimentation et chaque
transformateur du schéma est une source, avec sa tension de sortie (la `Voltage` de
l'article après le `/`, comme dans `230 V / 24 V DC`, ou `Output voltage` ; CC pour
les alimentations, CA pour les transformateurs). Le vérificateur choisit sa mesure de
protection – TBTS, TBTP, séparation électrique ou aucune mesure propre – et saisit
l'isolement de ses circuits par rapport aux autres circuits et à la terre avec la
tension d'essai : TBTS et TBTP au moins 250 V et 0,5 MΩ (tableau 6.1), séparation
électrique comme le réseau. Une source TBTS ou TBTP au-delà de 50 V CA ou 120 V CC
n'est pas conforme.

**Le courant de défaut mesuré à l'origine** est vérifié par rapport aux pouvoirs de
coupure des appareils de protection (données d'article, `Breaking capacity` ;
434.5.1) : un appareil alimenté directement depuis l'origine dont le pouvoir de
coupure est inférieur n'est pas conforme, un appareil plus en aval (où le courant de
défaut est plus faible) est marqué à vérifier ; la protection d'accompagnement par
l'appareil amont (tables de coordination) le couvre.

**Les données du rapport** : client, lieu de l'installation, installateur, motif de
la vérification (installation neuve, extension, modification, réparation),
vérificateur, appareils de mesure (IEC 61557), les valeurs mesurées à l'origine
(impédance de boucle externe Ze, courant de défaut présumé, résistance de la prise
de terre RA, liaison équipotentielle principale, la résistance des sols et parois
isolants des locaux non conducteurs : au moins 50 kΩ jusqu'à 500 V, 100 kΩ au-delà,
IEC 60364-4-41, C.1) et la date de la prochaine inspection. En schéma TT, RA ne doit
pas dépasser 50 V / le plus grand courant différentiel assigné de l'installation. En
schéma IT, le rapport ajoute l'essai de fonctionnement du contrôleur permanent
d'isolement et le courant de premier défaut Id, qui ne doit pas dépasser 50 V / RA
(411.6.2). **L'examen visuel** (6.4.2)
répond à 16 points par OK, Non conforme ou Sans objet : protection contre les chocs
électriques, barrières coupe-feu, conducteurs, appareils de protection, appareils de
sectionnement et de commande, influences externes, identification de N et PE,
appareils de commande unipolaires dans les conducteurs de phase, schémas et
avertissements, repérage, connexions, conducteurs de protection et d'équipotentialité,
accessibilité, protection complémentaire par DDR de 30 mA au plus, protection contre
les surtensions, mesures contre les perturbations électromagnétiques. Des points
propres sont ajoutés pour l'installation (les points de l'édition selon laquelle
l'installation est vérifiée, la liste de contrôle d'un client) ; ils participent au
verdict comme les autres.

**Les limites** sont des paramètres du rapport : chacune a une valeur par défaut et sa
source, et un projet fixe d'autres valeurs lorsque l'édition de la norme selon
laquelle il est vérifié diffère (*Limites* sur la carte du rapport de la page web, les
lignes `limit.<name>` de *Rapport de vérification : données*, `limit.<name>` dans le
`header` de `inspection_report`). Les valeurs par défaut ont été comparées à des
sources secondaires publiques (documents de fabricants et de formation) en
novembre 2026, non au texte des normes :

| Limite | Valeur par défaut | Source |
| --- | --- | --- |
| Résistance d'isolement minimale du réseau | 1 MΩ | IEC 60364-6, tableau 6.1 (6A dans l'édition 2006) |
| Résistance d'isolement minimale et tension d'essai TBTS et TBTP | 0,5 MΩ sous 250 V | IEC 60364-6, tableau 6.1 |
| Part de U0 / Ia que l'impédance de boucle mesurée peut atteindre | ⅔ | IEC 60364-6, annexe informative (C.61.3.6.2 de l'édition 2006) ; la BS 7671 utilise 0,8 |
| Tension limite conventionnelle de contact UL | 50 V | IEC 60364-4-41, 411.5.3 (TT), 411.6.2 (IT) |
| Temps de déclenchement DDR à IΔn / 5 × IΔn | 300 ms / 40 ms | IEC 61008-1, IEC 61009-1 |
| DDR sélectifs à IΔn / 5 × IΔn | 130 – 500 ms / 50 – 150 ms | IEC 61008-1, IEC 61009-1 |
| Tension maximale TBTS et TBTP | 50 V CA, 120 V CC | IEC 60364-4-41, 414.1.1 |
| Sols et parois isolants | 50 kΩ jusqu'à 500 V, 100 kΩ au-delà | IEC 60364-4-41, C.1 (les guides britanniques utilisent 0,5 MΩ) |
| Conducteur de protection : avis au-delà de la résistance calculée | 1,5 fois | OpenECraft ; la norme ne fixe pas de limite |

**Le verdict** du rapport est *Non conforme* lorsqu'un circuit, une source, un point
de l'examen visuel ou une valeur du rapport échoue, *Conforme* lorsque chaque circuit
et chaque source est conforme, que chaque point a une réponse, que le vérificateur est
nommé, que (TT) RA est mesurée et que (IT) le contrôleur permanent d'isolement est
essayé. **La signature** enregistre qui a signé et quand ; toute modification
ultérieure des données ou d'un résultat retire la signature. Le rapport s'exporte en
classeur Excel (sur la page web avec une feuille de synthèse, les circuits et les
sources).

**Sur site, avec une tablette ou un téléphone :** la page web du serveur de projets
([serveur de projets](../project-server.md)) montre le rapport de chaque projet sous
*Rapport de vérification* : une carte avec les données du rapport, l'examen visuel,
le verdict et *Signer le rapport*, une carte par source de circuits TBTS, TBTP ou
séparés, puis une carte par circuit avec les données
du schéma et les champs des résultats (clavier numérique, la limite à côté de chaque
champ, les valeurs hors limites marquées aussitôt ; la tension d'essai d'isolement, le
vérificateur, la date du jour et le dernier appareil de mesure sont préremplis pour un
nouveau circuit). Quiconque peut enregistrer des révisions ou modifier le projet les y
saisit et enregistre chaque carte séparément, sans ouvrir le projet et sans verrou ;
les valeurs non encore enregistrées restent dans le navigateur (même en cas de perte
de connexion). Les éditeurs qui ont le projet ouvert les reprennent à la révision
suivante ; leurs propres enregistrements les conservent (les données et les résultats
sont fusionnés carte par carte, comme les enregistrements de révision). *Excel*
télécharge le rapport.
