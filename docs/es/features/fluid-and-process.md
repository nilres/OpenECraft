# Técnica de fluidos, proceso e ingeniería de plantas

Mangueras y tuberías en redes propias, vínculos electro-fluidos, lazos PCT, la preplanificación de la planta y la ingeniería unifilar / multifilar.

## Técnica de fluidos

Los hilos entre las conexiones de los símbolos de técnica de fluidos y de P&ID son
mangueras / tuberías en redes propias (no se mezclan con los potenciales eléctricos; una
conexión de fluido en una red eléctrica es un error), numeradas aparte de los hilos
(`H1`, `H2` …), con medio, diámetro nominal, color y longitud, y enumeradas en la *Lista
de mangueras*.

- *Lista de redes de fluidos*: cada conexión recibe su función a partir de su designación
  ISO 1219 / ISO 5599 (alimentación, presión P/1, trabajo A/B/2/4, escape R/S/3/5,
  retorno T, pilotaje X/Y/Z/12/14, fuga L, en línea) y cada línea de manguera su tipo
  (línea de presión, de trabajo, de retorno / escape, de pilotaje …) con sus derivaciones
  (tes).
- *Configuración de mangueras*: por manguera, tubo flexible o tubería, el tipo de línea,
  el artículo (por metros), los racores de los extremos, la presión máxima y el radio de
  curvatura; la longitud de corte añade el suplemento de corte del proyecto por extremo.
  Los tubos entre dispositivos colocados en la misma placa de montaje se tienden por sus
  canaletas como los hilos, lo que da la longitud cuando no se ha fijado ninguna. La
  *Lista de tubos / tuberías* suma piezas, longitudes de corte, curvas y racores por
  tipo, tamaño y artículo.
- *Grupos de dispositivos de fluidos*: terminales de válvulas, unidades de mantenimiento,
  bloques de distribución y grupos hidráulicos con sus dispositivos en el orden de las
  posiciones y el número de plazas.
- *Electro-fluidos*: los dispositivos de fluidos se vinculan con los dispositivos
  eléctricos que los accionan o los supervisan (bobinas de electroválvula, detectores de
  posición y presostatos); ambos esquemas muestran referencias cruzadas entre sí, y
  *Sincronizar* vincula las electroválvulas con la bobina del mismo número (`-QM3` →
  `-MB3`) y copia los textos de función.
- Comprobaciones: líneas de presión en un retorno / escape, medios o tamaños distintos en
  una manguera o en un dispositivo, mangueras con presión asignada inferior a la presión
  de servicio o más cortas que su recorrido, electroválvulas sin bobina, bobinas sin
  válvula, vínculos erróneos o ausentes, grupos llenos o duplicados; los escapes abiertos
  de los dispositivos neumáticos descargan a la atmósfera en lugar de ser puntos de
  conexión abiertos.
- Informes: *Lista de redes de fluidos*, *Configuración de mangueras*, *Lista de tubos /
  tuberías*, *Grupos de dispositivos de fluidos*, *Referencias cruzadas electro-fluidos*.

![Circuito neumático del ejemplo pneumatic_press con cilindros, válvulas 5/2, reguladores de caudal y referencias cruzadas a las bobinas de las electroválvulas](../../images/pneumatics.png)

## Proceso (P&ID)

Los símbolos de P&ID (ISO 10628: depósito, bombas, válvulas, intercambiador de calor,
instrumentos) dan soporte a la preplanificación: los instrumentos llevan letras de
función y número de lazo (`TIC 101`), los lazos PCT de la preplanificación
(*Herramientas › Fluidos y proceso…*) enumeran sus dispositivos, y el *Resumen de lazos
PCT* muestra si cada lazo está planificado, en ingeniería o diseñado en el esquema.

## Preplanificación

**Preplanificación** (*Dispositivos › Preplanificación…*): la planta como árbol de
segmentos (planta, área, unidad, …) con la función y el lugar heredados árbol abajo y los
dispositivos planificados en ellos (designación, tipo, descripción, potencia, tensión,
artículo, macro de página); la potencia planificada por segmento, plantillas de
segmentos, el estado de cada dispositivo planificado frente al esquema (planificado,
diseñado, artículo distinto; también una comprobación y el informe *Preplanificación*),
páginas de esquema generadas a partir de las macros de página de los dispositivos aún no
diseñados, y el intercambio como CSV / Excel (comandos `preplanning`, `add_segment`,
`plan_device`, …; herramientas `import_preplanning`, `generate_preplanning_pages`).

## Ingeniería unifilar / multifilar

**Ingeniería unifilar / multifilar** (*Dispositivos › Unifilar / multifilar…*): los
dispositivos se planifican con símbolos unifilares (interruptor automático,
interruptor-seccionador, guardamotor, fusible, relé de sobrecarga, contactor, motor,
transformador) en páginas de vista general y se diseñan en el esquema multifilar; ambos
comparten la designación de dispositivo. La ventana enumera cada dispositivo como
planificado, diseñado, no presente en el esquema unifilar o no coincidente, y genera el
esquema de una página unifilar (símbolos de potencia de 3 polos, conductores, bobinas de
contactor) o un esquema unifilar de los dispositivos de potencia del esquema (dispuestos
desde la alimentación hasta las cargas). La comprobación notifica los dispositivos
planificados, los símbolos que no coinciden, las conexiones del esquema unifilar que
faltan en el esquema y los textos de función distintos; *Sincronizar los textos de
función* rellena los vacíos a partir de la otra representación.
