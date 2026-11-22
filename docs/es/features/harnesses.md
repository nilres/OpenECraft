# Mazos de cables

Mazos con topología 3D, protecciones y fijaciones, exportación KBL, la interfaz de mazos
para máquinas y planos de clavos (*Conexiones › Mazos…*). El proyecto de ejemplo
`conveyor` tiene un mazo de puerta.

## Los mazos y sus hilos

Las conexiones individuales punto a punto (`-S1:13 – -K1:A1`) se asignan a mazos
(`-WH1`), con independencia de las demás conexiones del mismo tramo de potencial; las
asignaciones siguen a los dispositivos renombrados, y la comprobación notifica las
conexiones que faltan. El informe *Datos de mazos* enumera sus hilos con la longitud
(tendida en el diseño del armario o la longitud de campo del mazo) y el tratamiento de
los extremos; la ventana de mazos muestra los conectores con su asignación de pines y el
diámetro del haz.

## Geometría 3D del mazo

Nodos y segmentos con curvas de eje derivadas de los recorridos 3D del diseño del armario
(los conectores situados fuera de él se colocan alrededor de un nodo común), o guardadas
en el mazo y editadas (posiciones, radios de curvatura, longitudes en el tablero, puntos
de control); cada hilo se tiende por los segmentos, cada segmento tiene su diámetro de
haz; cinta, tubos, tubos corrugados, fundas y termorretráctil sobre (partes de)
segmentos, clips, bridas, pasacables y soportes en posiciones concretas; la comprobación
notifica las topologías rotas y los radios de curvatura demasiado pequeños; exportación
del haz en 3D (glTF, JT).

## KBL

VDA 4964, esquema KBL 2.4 SR-1, comprobado frente al esquema en las pruebas: un
contenedor por mazo con hilos generales y cables (conductores), carcasas de conector con
ranuras y cavidades, terminales, juntas y tapones de cavidad, protecciones de hilos,
fijaciones, accesorios, piezas suministradas sueltas, conjuntos y componentes, las
conexiones entre puntos de contacto, conexiones de esquema (potenciales), grupos de
cableado, opciones de proyecto como módulos, revisiones, aprobaciones, unidades,
tolerancias predeterminadas y la topología 3D (exportadores `kbl`, `kbl_zip`).

## Interfaz de mazos

Para máquinas y herramientas ([harness-interface.md](../../harness-interface.md)): los
datos completos del mazo como JSON (`harness_json`, comando `harness_data`) y la lista de
procesado de hilos para máquinas de corte, pelado y engaste (`harness_processing`, CSV en
el orden de procesado con longitudes de corte, longitudes de pelado, terminales y
juntas); el plugin *Mazos* edita la topología, las protecciones, las fijaciones, el
tratamiento de los extremos y otros artículos desde la línea de comandos y MCP.

## Planos de clavos

Tableros de montaje a escala: el haz como ramas desde un nodo común hasta los conectores,
cada una tan larga como exigen las longitudes de los hilos y tan gruesa como su haz, con
clavos, longitudes de rama y la tabla de pines de cada conector, en A3 a la menor escala
normalizada en la que cabe (ventana de mazos › *Plano de clavos (SVG)…*, exportador
`nailboard`, comando `harness_nailboard`).
