# Assistants de câblage et de montage

L'implantation d'armoire sous forme de liste de contrôle pour l'atelier, sur un poste de bureau ou une tablette.

![L'assistant de câblage avec la liste de contrôle, le fil courant avec ses données et le fil mis en évidence en 3D](../../images/wiring-assistant.png)

## Assistant de câblage

**Assistant de câblage** (dans l'esprit d'EPLAN Smart Wiring) : l'espace de travail
*Câblage* transforme les fils routés de l'implantation d'armoire en une liste de
contrôle adaptée au tactile pour l'atelier – triée par appareil, position, section ou
numéro de fil – avec le fil courant mis en évidence en 3D, sa couleur, sa section, sa
longueur, sa longueur de dénudage et l'embout ou la cosse de chaque extrémité (deux
fils partageant un embout double l'un après l'autre, *sertir avec le fil vers …*), et
l'avancement en fils et en mètres. Les fils sont marqués comme câblés ou comme
problème (avec une note) ; les marques sont enregistrées dans le projet, de sorte que
les fils modifiés dans le schéma après leur câblage sont signalés par le contrôle. La
version web ouvre l'assistant directement avec `…/#wiring` (bureau : `--wiring`), ce
qui fait d'une tablette dans l'atelier un poste de câblage bon marché ; liste de
contrôle avec état en CSV.

## Assistant de montage

**Assistant de montage** (dans l'esprit d'EPLAN Smart Mounting) : le commutateur
*Montage* de l'espace de travail *Câblage* liste les éléments de l'implantation
d'armoire dans l'ordre de montage – découpes, rails, goulottes et jeux de barres,
appareils, accessoires, puis porte et parois – avec l'élément courant mis en évidence
en 3D, son emplacement, sa taille et son article, et l'avancement. Les éléments sont
marqués comme montés ou comme problème (avec une note) ; les éléments déplacés ou
dotés d'un autre article après leur montage sont signalés par le contrôle. Commandes
`mounting_list` et `mark_mounted` ; liste de contrôle avec état en CSV.
