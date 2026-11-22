# Asistentes de cableado y montaje

El diseño del armario como lista de comprobación para el taller, en un ordenador o una tableta.

![El asistente de cableado con la lista de comprobación, el hilo actual con sus datos y el hilo resaltado en 3D](../../images/wiring-assistant.png)

## Asistente de cableado

**Asistente de cableado** (en la línea de EPLAN Smart Wiring): el espacio de trabajo
*Cableado* convierte los hilos tendidos del diseño del armario en una lista de
comprobación cómoda de usar con pantalla táctil para el taller – ordenada por
dispositivo, posición, sección o número de hilo – con el hilo actual resaltado en 3D, su
color, sección, longitud, longitud de pelado y la puntera o el terminal de cada extremo
(dos hilos que comparten una puntera doble, uno tras otro, *engastar junto con el cable
hacia …*), y el progreso en hilos y metros. Los hilos se marcan como cableados o como
problema (con una nota); las marcas se guardan en el proyecto, de modo que la
comprobación señala los hilos modificados en el esquema después de haberse cableado. La
versión web abre el asistente directamente con `…/#wiring` (escritorio: `--wiring`), lo
que convierte una tableta en el taller en un puesto de cableado económico; lista de
comprobación con estado como CSV.

## Asistente de montaje

**Asistente de montaje** (en la línea de EPLAN Smart Mounting): el conmutador *Montaje*
del espacio de trabajo *Cableado* enumera los elementos del diseño del armario en el
orden de montaje – recortes, carriles, canaletas y barras, dispositivos, accesorios y
después puerta y paredes – con el elemento actual resaltado en 3D, su posición, tamaño y
artículo, y el progreso. Los elementos se marcan como montados o como problema (con una
nota); la comprobación señala los elementos movidos o a los que se asignó otro artículo
después de haberse montado. Comandos `mounting_list` y `mark_mounted`; lista de
comprobación con estado como CSV.
