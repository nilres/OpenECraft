# Símbolos y bibliotecas

Las bibliotecas de símbolos integradas (IEC, NFPA / JIC, GB, GOST, JIS, técnica de fluidos, P&ID, unifilar) y el editor de símbolos para símbolos propios.

## Biblioteca de símbolos IEC 60617

**Biblioteca de símbolos IEC 60617** (~330 símbolos): bobinas de relé (CA, de
enclavamiento, polarizadas, temporizadores, interruptores horarios, contadores, relés de
sobretensión / subtensión), contactos NA/NC/conmutados (adelantados / retardados, de
paso, con retardo a la conexión / a la desconexión), pulsadores e interruptores de
tirador, de llave, de pedal y selectores con retorno por muelle o con enclavamiento y
diagramas de posiciones, paradas de emergencia con rearme por giro / tirón / llave,
interruptores de leva / de rodillo / de presión / de caudal / de nivel / bimetálicos,
sensores inductivos, capacitivos, magnéticos y fotoeléctricos, fusibles (NH),
interruptores con fusibles y seccionadores con fusibles, seccionadores e
interruptores-seccionadores, interruptores automáticos, diferenciales, guardamotores,
relés de sobrecarga, motores y ventiladores CA / CC / EC, lámparas de señalización,
luminarias (L, N, PE), balizas, sirenas, aparatos de medida indicadores, registradores e
integradores, transformadores de corriente, transformadores monofásicos y trifásicos
(grupos de conexión) y autotransformadores, rectificadores y convertidores, bornes,
entradas/salidas de PLC, potenciales, puntos de interrupción, y electrónica:
resistencias, condensadores (también polarizados), diodos, diodos Zener y fotodiodos,
LED, transistores NPN / PNP, MOSFET de canal N, tiristor, triac, termistor NTC,
fotorresistencia, optoacoplador y puente rectificador.

## Símbolos de plano de instalación

**Símbolos de plano de instalación** (IEC 60617-11): interruptores unipolares y
conmutadores, regulador de luz, pulsador, punto de luz, luminaria fluorescente, toma
doble con tierra, caja de derivación, detector de movimiento y cuadro de distribución,
cada uno con un punto de conexión para el cable.

## Editor de símbolos y bibliotecas

Además de los símbolos IEC hay símbolos NFPA / JIC
(30: bobinas, bobinas de temporizador, contactos, contactos temporizados y de
sobrecarga, contactos principales, pulsadores, parada de emergencia, selector,
interruptores de pedal, de final de carrera, de flotador, de presión, de temperatura, de
caudal y de proximidad, interruptores automáticos, seccionadores, fusibles, pilotos,
bocinas, solenoides, transformadores de mando, motores), símbolos de técnica de fluidos
(ISO 1219: bombas, compresor, motor hidráulico, cilindros de simple y de doble efecto,
válvulas 3/2, 4/2 y 5/2, válvulas antirretorno, limitadoras, de bola y reguladoras de
caudal, regulador de presión, unidad de mantenimiento, filtro, acumulador, refrigerador,
manómetro, depósito, silenciador), símbolos unifilares / de vista general (referencias
cruzadas al dispositivo, no forman parte de la lista de redes) y cajas negras / cajas de
PLC. Los juegos nacionales para China (GB), Rusia y la CEI (GOST) y Japón (JIS) llevan
las letras de dispositivo de GB/T 7159 (`KM`, `KA`,
`QF`, `SB`, `HL` …), GOST 2.710 (`KM`, `KK`, `QF`, `SB`, `HL` …) y JIS C 0401
(`MC`, `MCCB`, `PBS`, `PL`, `THR` …), de modo que los dispositivos colocados se numeran
`-KM1`, `-MCCB1`; se dibujan como los símbolos IEC, que GB/T 4728 y JIS C 0617 adoptan
(las proporciones ESKD de los dibujos GOST no se reproducen). Los símbolos propios se
dibujan en el editor de símbolos (líneas, rectángulos, círculos, textos, campos de
identificador / etiqueta, puntos de conexión en la rejilla de 4 mm, función como función
principal, contacto, borne, punto de PLC), o se copian de un símbolo integrado; se
guardan en el proyecto y se intercambian como archivos de biblioteca.
Un símbolo colocado de la biblioteca (uno propio o un símbolo de EPLAN importado) se
puede dibujar más grande o más pequeño: **Escala** en sus propiedades (o `update_symbol`
con `scale`, de 0,1 a 10) escala sus gráficos y sus puntos de conexión, mientras que sus
textos conservan su tamaño, para que los símbolos importados encajen en el dibujo; los
hilos de sus puntos de conexión siguen.
