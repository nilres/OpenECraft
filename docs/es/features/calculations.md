# Dimensionado eléctrico, cálculos y verificación

Dimensionado de conductores, cortocircuito y coordinación de protecciones, cálculo de cargas, CEM, seguridad funcional y conjuntos de reglas – todo ello parte de la comprobación del proyecto – y el informe de inspección y ensayo de la instalación.

## Dimensionado eléctrico

**Dimensionado eléctrico** (IEC 60204-1): los hilos situados detrás de un dispositivo de
protección – desde su lado de carga, a través de contactos principales y relés de
sobrecarga – deben soportar su corriente asignada (`Current` del artículo, el valor
superior de un rango de ajuste; intensidad admisible según la tabla 6, método B2); los
cables reciben su caída de tensión hasta la carga con la mayor corriente asignada en sus
extremos (√3 para tres conductores, cobre a temperatura de servicio), informe
*Dimensionado de cables*; la comprobación notifica los hilos y cables demasiado delgados
(errores) y las caídas de tensión superiores al 5 %. La intensidad admisible se corrige
según la temperatura ambiente, la agrupación de circuitos (IEC 60364-5-52, B.52.17), el
aluminio y el aislamiento de XLPE. Un interruptor-seccionador con fusibles equipado con
cuchillas seccionadoras (subgrupo de artículos `Solid link`, el seccionamiento antes y
después de los transformadores de una medida) solo secciona: no protege nada, no tiene
selectividad propia y la corriente lo atraviesa; la carga de un cable es la corriente
asignada del artículo que aporta la función en su extremo (el mando motorizado de un
interruptor automático, no el interruptor).

## Cálculos eléctricos y verificación

**Cálculos eléctricos y verificación** (*Herramientas › Cálculos eléctricos…*):
- *Suministro*: tensión, esquema de conexión a tierra (TN-S, TN-C, TN-C-S, TT, IT),
  corriente de cortocircuito máxima y mínima en el punto de suministro, corriente
  asignada;
- *Cortocircuito y coordinación de protecciones* (IEC 60909-0 para redes radiales,
  IEC 60364-4-41, -4-43): la fuente a partir de las corrientes de cortocircuito en el
  punto de suministro (con su R/X) o de un transformador (potencia asignada, uk, pérdidas
  en carga, factor de corrección KT) y de la red aguas arriba (S''kQ); los cables desde
  el suministro hasta cada dispositivo de protección y hasta el final de su circuito; los
  motores asíncronos conectados a la red añaden ILR / IrM = 5 (o `Locked rotor current`)
  cuando superan el 1 % de la corriente del suministro. Los dispositivos de protección de
  la red con sus bandas tiempo-corriente (IEC 60898-1 B / C / D / K / Z, protección de
  motores clase 10, fusibles gG / aM, disparadores electrónicos LSI con `Long-time delay`,
  `Short-time pickup`, `Short-time delay`; datos técnicos del artículo `Characteristic`,
  `Breaking capacity`, `Making capacity`, `Instantaneous trip`, `Let-through energy`,
  `Pre-arcing energy`), sustituidas por las curvas del fabricante cuando se indican
  (`Trip curve min` / `Trip curve max`: `1.5: 600; 3: 20; 10: 0.01` en múltiplos de In,
  `Let-through energy: 1 kA: 8000; 6 kA: 25000`). Se comprueba: el poder de corte frente
  a la corriente de cortocircuito y el poder de cierre frente a su valor de cresta (κ a
  partir de R/X), la desconexión con la corriente de defecto mínima al final de cada
  circuito dentro de los tiempos de la tabla 41.1 según U0 (TN 0,8 / 0,4 / 0,2 / 0,1 s,
  TT 0,3 / 0,2 / 0,07 / 0,04 s, circuitos de distribución 5 s / 1 s; TT: se necesita un
  dispositivo diferencial; IT: el segundo defecto, a U o, con neutro distribuido, a U0),
  la resistencia k²S² de los cables, la selectividad con el dispositivo aguas arriba
  (total / parcial hasta una corriente) a partir de la tabla de coordinación del
  fabricante, fusibles gG con una relación de 1,6, la energía de prearco de un fusible
  aguas arriba o las bandas tiempo-corriente; la protección de acompañamiento de la tabla
  de coordinación cubre un poder de corte demasiado bajo. Las tablas de coordinación
  (límites de selectividad y de protección de acompañamiento en kA por número de
  artículo) se editan en la ventana y con `coordination_table`. Informes *Cortocircuito y
  desconexión*, *Selectividad* (con su fundamento);
- *Cálculo de cargas*: motores, accionamientos, calefactores, fuentes de alimentación y
  transformadores de la red con potencia activa / aparente y corriente (`Power`,
  `Current`, `Power factor`, `Efficiency`), la carga de cada dispositivo de protección, la
  demanda con el factor de simultaneidad asignado de IEC 61439-1 (tabla 101) o un factor
  fijado, corrientes de fase y desequilibrio, y el balance de carga de cada fuente de
  alimentación de 24 V (`Coil power`, `Current consumption`); informes *Cálculo de
  cargas*, *Balance de carga de las fuentes de alimentación*;
- *CEM*: cables de motor de los convertidores de frecuencia apantallados en ambos
  extremos, señales analógicas en cables apantallados, sin conductores de potencia y de
  señal en un mismo cable, protección contra sobretensiones;
- *Seguridad funcional* (ISO 13849-1): funciones de seguridad formadas por subsistemas
  con sus dispositivos, categoría, DCavg, operaciones por año y CCF; PL por el
  procedimiento simplificado (figura 5) a partir de `MTTFd` / `B10d` de los artículos o
  de `PL` / `PFHd` certificados, combinados según la tabla 11 o, cuando cada subsistema
  tiene un PFHd, por su suma, con el SIL correspondiente; un segundo canal de otros
  dispositivos se simetriza (D.2), el DCavg procede del `DC` de los dispositivos cuando el
  subsistema no indica ninguno (anexo E), los dispositivos cuyo T10d queda dentro del
  tiempo de misión de 20 años se nombran para su sustitución; informe *Funciones de
  seguridad*;
- *Conjuntos de reglas*: bibliotecas integradas (IEC 60204-1, IEC 61439-1, UL 508A /
  NFPA 79, ISO 13849-1, CEM) y conjuntos de reglas de cliente (importación / exportación
  JSON) que endurecen los límites (caída de tensión, tiempos de desconexión, secciones
  mínimas, reserva de carga, desequilibrio de fases, selectividad total, apantallamiento,
  protección contra sobretensiones) y añaden comprobaciones "todo elemento de un ámbito
  (proyecto, páginas, dispositivos, artículos, cables, conexiones, potenciales) que
  cumpla estas condiciones debe cumplir esta otra"; informe *Verificación de reglas*;
  todos los resultados forman parte de la comprobación del proyecto.

## Informe de inspección y ensayo

El **informe de inspección y ensayo** (verificación inicial, IEC 60364-6 / DIN VDE
0100-600) tiene cuatro partes: los datos del informe con la inspección visual, las
fuentes de MBTS, MBTP y de circuitos separados, los circuitos con sus resultados, y el
dictamen con la firma. Se muestra en los informes *Informe de inspección y ensayo*
(los circuitos), *Informe de inspección y ensayo: MBTS, MBTP y separación* e
*Informe de inspección y ensayo: resumen*, se edita en el panel *Herramientas › Informe de inspección y ensayo* del editor (las mismas
secciones que la página web: datos del informe e inspección visual, límites, firma, una
sección por fuente y por circuito, *Exportar Excel*), en *Datos de proyecto › Editar en tabla*
(*Informe de inspección y ensayo*, una fila por circuito, e *Informe de inspección y
ensayo: datos*, una fila por valor del informe y de las fuentes), con el comando
`inspection_report` y en la página web del servidor de proyectos.

**Los circuitos** son los dispositivos de protección alimentados desde la red, en el
orden del esquema, rellenados a partir del esquema:

- circuito (designación de dispositivo), descripción (del dispositivo o, si no, su texto
  de función), fases, los cables detrás del dispositivo con sus tipos, la sección (de los
  cables o, si no, de los hilos);
- dispositivo de protección: característica de disparo y corriente asignada (datos del
  artículo, como para el cálculo de cortocircuito);
- dispositivo diferencial que protege el circuito (el propio o uno aguas arriba) con su
  tipo (`Residual current type` del dispositivo o de su artículo; una `S`, como en `A S`,
  marca uno selectivo) y su corriente diferencial asignada;
- la mayor impedancia de bucle admisible: dos tercios de U0 / Ia (Ia: la corriente que
  desconecta dentro del tiempo de la tabla 41.1; IEC 60364-6, anexo C; también en
  esquemas TT, 411.5.4; IT: el segundo defecto, U / 2 Ia o, con neutro distribuido,
  U0 / 2 Ia, 411.6.4), o, cuando un dispositivo diferencial protege el circuito, U0 / IΔn
  (TN, IT) y 50 V / IΔn (TT: RA × IΔn ≤ UL, 411.5.3; Zs incluye RA).

Lo que el proyecto no aporta se muestra como *falta*, nunca se supone: ni característica
C, ni 30 mA, ni límite alguno basado en ellos, a diferencia de los cálculos, que sí los
suponen.

Los resultados de un circuito, conservados tal como se introducen (`0,35`, `> 500`) y
comprobados frente a sus límites:

| Resultado | Límite |
| --- | --- |
| Continuidad del conductor de protección (Ω) | sin límite en la norma; junto a ella la resistencia calculada a partir de los cables del circuito (longitud, sección, material del conductor a 20 °C), una indicación por encima de 1,5 veces ese valor |
| Tensión de ensayo de aislamiento (V) | al menos 500 V hasta 500 V nominales, 1000 V por encima (tabla 6.1) |
| Aislamiento L–PE (MΩ; los conductores de línea, o todos los conductores activos unidos entre sí, respecto a PE) | al menos 1 MΩ |
| Aislamiento N–PE y L–N (MΩ, cuando se miden por separado) | al menos 1 MΩ |
| Impedancia de bucle Zs (Ω) | la mayor impedancia de bucle indicada arriba |
| Caída de tensión (%) | el límite de los conjuntos de reglas (5 % si no hay ninguno) |
| Corriente de disparo del diferencial (mA) | ½ IΔn … IΔn |
| Tiempo de disparo del diferencial a IΔn (ms) | como máximo 300 ms; tipo S 130 … 500 ms |
| Tiempo de disparo del diferencial a 5 × IΔn (ms) | como máximo 40 ms; tipo S 50 … 150 ms |
| Tensión de contacto UB a IΔn (V) | como máximo 50 V |
| Polaridad | OK / No conforme |
| Secuencia de fases (circuitos trifásicos) | OK / No conforme |
| Prueba de funcionamiento (aparamenta, enclavamientos, mandos) | OK / No conforme |
| Verificado por, fecha, instrumento de ensayo, observación | se registran |

Un circuito está *No aprobado* cuando un valor queda fuera de su límite o un ensayo ha
fallado, *Aprobado* cuando constan la continuidad, el aislamiento (con su tensión de
ensayo), la impedancia de bucle, la polaridad, la prueba de funcionamiento, (en
trifásico) la secuencia de fases y (con diferencial) el tiempo de disparo a IΔn,
*Incompleto* en los casos intermedios y *No verificado* si no hay valores.

**MBTS, MBTP y separación eléctrica** (6.4.3.4): cada fuente de alimentación y cada
transformador del esquema es una fuente, con su tensión de salida (el `Voltage` del
artículo después de la `/`, como en `230 V / 24 V DC`, u `Output voltage`; CC para las
fuentes de alimentación, CA para los transformadores). El inspector elige su medida de
protección – MBTS, MBTP, separación eléctrica o ninguna propia – e introduce el
aislamiento de sus circuitos respecto a otros circuitos y a tierra con la tensión de
ensayo: MBTS y MBTP al menos 250 V y 0,5 MΩ (tabla 6.1), separación eléctrica como la
red. Una fuente MBTS o MBTP por encima de 50 V CA o 120 V CC no se aprueba.

**La corriente de defecto medida en el origen** se comprueba frente a los poderes de
corte de los dispositivos de protección (datos del artículo, `Breaking capacity`;
434.5.1): un dispositivo alimentado directamente desde el origen cuyo poder de corte es
inferior no se aprueba, uno situado más abajo (donde la corriente de defecto es menor) se
marca para comprobar; la protección de acompañamiento por el dispositivo aguas arriba
(tablas de coordinación) lo cubre.

**Los datos del informe**: cliente, lugar de la instalación, instalador, motivo del
ensayo (instalación nueva, extensión, cambio, reparación), inspector, instrumentos de
ensayo (IEC 61557), los valores medidos en el origen (impedancia de bucle externa Ze,
corriente de defecto prevista, resistencia de la toma de tierra RA, conexión
equipotencial principal, la resistencia de suelos y paredes aislantes de los
emplazamientos no conductores: al menos 50 kΩ hasta 500 V, 100 kΩ por encima,
IEC 60364-4-41, C.1) y la fecha de la próxima inspección. En un esquema TT, RA no debe
superar 50 V / la mayor corriente diferencial asignada de la instalación. En un esquema
IT el informe añade la prueba de funcionamiento del controlador de aislamiento y la
corriente de un primer defecto Id, que no debe superar 50 V / RA (411.6.2). **La
inspección visual** (6.4.2)
responde a 16 puntos con OK, No conforme o No aplicable: protección contra los choques
eléctricos, barreras cortafuegos, conductores, dispositivos de protección, dispositivos
de seccionamiento y de mando, influencias externas, identificación de N y PE,
dispositivos de mando unipolares en los conductores de línea, esquemas y avisos de
advertencia, rotulación, conexiones, conductores de protección y de equipotencialidad,
accesibilidad, protección complementaria mediante diferenciales de 30 mA como máximo,
protección contra sobretensiones, medidas contra las perturbaciones electromagnéticas.
Se añaden puntos propios para la instalación (los puntos de la edición frente a la que se
verifica la instalación, la lista de comprobación de un cliente); intervienen en el
dictamen como los demás.

**Los límites** son ajustes del informe: cada uno tiene un valor predeterminado y su
origen, y un proyecto fija otros valores cuando difiere la edición de la norma frente a
la que se verifica (*Límites* en la tarjeta del informe de la página web, las filas
`limit.<name>` de *Informe de inspección y ensayo: datos*, `limit.<name>` en el `header`
de `inspection_report`). Los valores predeterminados se compararon con fuentes
secundarias públicas (documentos de fabricantes y de formación) en noviembre de 2026, no
con el texto de las normas:

| Límite | Predeterminado | Origen |
| --- | --- | --- |
| Resistencia de aislamiento mínima de la red | 1 MΩ | IEC 60364-6, tabla 6.1 (6A en la edición de 2006) |
| Resistencia de aislamiento mínima y tensión de ensayo de MBTS y MBTP | 0,5 MΩ a 250 V | IEC 60364-6, tabla 6.1 |
| Parte de U0 / Ia que puede alcanzar la impedancia de bucle medida | ⅔ | IEC 60364-6, anexo informativo (C.61.3.6.2 de la edición de 2006); BS 7671 usa 0,8 |
| Límite convencional de la tensión de contacto UL | 50 V | IEC 60364-4-41, 411.5.3 (TT), 411.6.2 (IT) |
| Tiempo de disparo del diferencial a IΔn / 5 × IΔn | 300 ms / 40 ms | IEC 61008-1, IEC 61009-1 |
| Diferenciales selectivos a IΔn / 5 × IΔn | 130 – 500 ms / 50 – 150 ms | IEC 61008-1, IEC 61009-1 |
| Tensión máxima de MBTS y MBTP | 50 V CA, 120 V CC | IEC 60364-4-41, 414.1.1 |
| Suelos y paredes aislantes | 50 kΩ hasta 500 V, 100 kΩ por encima | IEC 60364-4-41, C.1 (las guías británicas usan 0,5 MΩ) |
| Conductor de protección: indicación por encima de la resistencia calculada | 1,5 veces | OpenECraft; la norma no fija ningún límite |

**El dictamen** del informe es *No aprobado* cuando falla un circuito, una fuente, un
punto de la inspección visual o un valor del informe, *Aprobado* cuando todos los
circuitos y fuentes están aprobados, todos los puntos respondidos, el inspector indicado,
(TT) RA medida y (IT) el controlador de aislamiento probado. **La firma** registra quién
firmó y cuándo; cualquier cambio posterior de los datos o de un resultado retira la
firma. El informe se exporta como libro de Excel (en la página web con una hoja de
resumen, los circuitos y las fuentes).

**En obra, con una tableta o un teléfono:** la página web del servidor de proyectos
([servidor de proyectos](../project-server.md)) muestra el informe de cada proyecto en
*Informe de inspección y ensayo*: una tarjeta con los datos del informe, la inspección
visual, el dictamen y *Firmar el informe*, una tarjeta por fuente de MBTS, MBTP o de
circuitos separados, y después una tarjeta por circuito con los datos
del esquema y los campos de los resultados (teclado numérico, el límite junto a cada
campo, los valores fuera de los límites marcados al instante; la tensión de ensayo de
aislamiento, quién verifica, la fecha de hoy y el último instrumento se rellenan para un
circuito nuevo). Quien puede guardar revisiones o editar el proyecto los introduce allí y
guarda cada tarjeta por separado, sin abrir el proyecto y sin bloqueo; los valores aún no
guardados permanecen en el navegador (también si se pierde la conexión). Los editores que
tienen el proyecto abierto los adoptan con la siguiente revisión; sus propios guardados
los conservan (los datos y los resultados se fusionan tarjeta por tarjeta, como los
registros de comentarios). *Excel* descarga el informe.
