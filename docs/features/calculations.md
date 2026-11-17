# Electrical sizing, calculations and verification

Conductor sizing, short circuit and protection coordination, load calculation, EMC, functional safety and rule sets – all part of the project check – and the inspection and test report of the installation.

## Electrical sizing

**Electrical sizing** (IEC 60204-1): the wires behind a protective device – from its
load side through main contacts and overload relays – must carry its rated current
(part `Current`, the upper value of a setting range; current-carrying capacity per
table 6, method B2); cables get their voltage drop to the load with the highest rated
current at their ends (√3 for three cores, copper at operating temperature), report
*Cable sizing*; the check reports too thin wires and cables (errors) and voltage
drops above 5 %. The capacity is corrected for ambient temperature, bunched
circuits (IEC 60364-5-52, B.52.17), aluminium and XLPE insulation. A fuse switch
disconnector fitted with solid links (part subgroup `Solid link`, the isolation before
and after the transformers of a metering) isolates only: it protects nothing, has no
selectivity of its own and the current passes it; the load of a cable is the rated
current of the part providing the function at its end (the motor operator on a
circuit breaker, not the breaker).

## Electrical calculations and verification

**Electrical calculations and verification** (*Tools › Electrical calculations…*):
- *Supply*: voltage, earthing system (TN-S, TN-C, TN-C-S, TT, IT), highest and lowest
  short-circuit current at the supply point, rated current;
- *Short circuit and protection coordination* (IEC 60909-0 for radial networks,
  IEC 60364-4-41, -4-43): the source from the short-circuit currents at the supply
  point (with its R/X) or from a transformer (rated power, uk, load losses, correction
  factor KT) and the network upstream (S''kQ); the cables from the supply to each
  protective device and on to the end of its circuit; asynchronous motors on the
  mains add ILR / IrM = 5 (or `Locked rotor current`) when they exceed 1 % of the
  supply's current. The protective devices of the mains with their time-current bands
  (IEC 60898-1 B / C / D / K / Z, motor protection class 10, gG / aM fuses, electronic
  LSI trip units with `Long-time delay`, `Short-time pickup`, `Short-time delay`; part
  tech data `Characteristic`, `Breaking capacity`, `Making capacity`, `Instantaneous
  trip`, `Let-through energy`, `Pre-arcing energy`), replaced by the manufacturer's
  curves where given (`Trip curve min` / `Trip curve max`: `1.5: 600; 3: 20; 10: 0.01`
  in multiples of In, `Let-through energy: 1 kA: 8000; 6 kA: 25000`). Checked: breaking
  capacity against the short-circuit current and making capacity against its peak
  (κ from R/X), disconnection at the lowest fault current at the end of each circuit
  within the times of table 41.1 by U0 (TN 0.8 / 0.4 / 0.2 / 0.1 s, TT 0.3 / 0.2 /
  0.07 / 0.04 s, distribution circuits 5 s / 1 s; TT: residual current device needed;
  IT: the second fault, at U or with a distributed neutral at U0), k²S² withstand of
  the cables, selectivity with the device upstream (total / partial up to a current)
  from the manufacturer's coordination table, gG fuses at a ratio of 1.6, the
  pre-arcing energy of a fuse upstream or the time-current bands; back-up protection
  from the coordination table covers a too low breaking capacity. The coordination
  tables (selectivity and back-up limits in kA by part number) are edited in the
  window and with `coordination_table`. Reports *Short circuit and disconnection*,
  *Selectivity* (with its basis);
- *Load calculation*: motors, drives, heaters, power supplies and transformers of the
  mains with active / apparent power and current (`Power`, `Current`, `Power factor`,
  `Efficiency`), the load of each protective device, demand with the rated diversity
  factor of IEC 61439-1 (table 101) or a set factor, phase currents and unbalance, and
  the load budget of each 24 V power supply (`Coil power`, `Current consumption`);
  reports *Load calculation*, *Power supply load budget*;
- *EMC*: motor cables of frequency converters shielded at both ends, analogue signals
  in shielded cables, no power and signal conductors in one cable, surge protection;
- *Functional safety* (ISO 13849-1): safety functions of subsystems with their devices,
  category, DCavg, operations per year and CCF; PL by the simplified procedure
  (figure 5) from `MTTFd` / `B10d` of the parts or certified `PL` / `PFHd`, combined by
  table 11 or, where every subsystem has a PFHd, by their sum, with the matching SIL;
  a second channel of other devices is symmetrised (D.2), the DCavg comes from the
  `DC` of the devices when the subsystem gives none (annex E), devices whose T10d
  falls within the mission time of 20 years are named for replacement; report
  *Safety functions*;
- *Rule sets*: built-in libraries (IEC 60204-1, IEC 61439-1, UL 508A / NFPA 79,
  ISO 13849-1, EMC) and customer rule sets (JSON import / export) that tighten the
  limits (voltage drop, disconnection times, minimum cross-sections, load reserve,
  phase unbalance, total selectivity, shielding, surge protection) and add checks
  "every item of a scope (project, pages, devices, parts, cables, connections,
  potentials) that meets these conditions must meet this one"; report *Rule set
  verification*; all results are part of the project check.

## Inspection and test report

The **inspection and test report** (initial verification, IEC 60364-6 / DIN VDE
0100-600) has three parts: the data of the report with the visual inspection, the
circuits with their results, and the verdict with the signature. It is shown in the
reports *Inspection and test report* (the circuits) and *Inspection and test report:
summary*, edited in *Project data › Edit in table* (*Inspection and test report*, one
row per circuit, and *Inspection and test report: data*, one row per value), with the
command `inspection_report` and on the web page of the project server.

**The circuits** are the protective devices fed from the mains, in schematic order,
filled in from the schematic:

- circuit (device tag), description (of the device, otherwise its function text),
  phases, the cables behind the device with their types, the cross-section (of the
  cables, otherwise of the wires);
- protective device: tripping characteristic and rated current (part data, as for the
  short-circuit calculation);
- residual current device protecting the circuit (its own or one upstream) with its
  type (`Residual current type` of the device or of its part; an `S`, as in `A S`,
  marks a selective one) and rated residual current;
- the highest loop impedance that passes: two thirds of U0 / Ia (Ia: the current that
  disconnects within the time of table 41.1; IEC 60364-6, annex C; also in TT systems,
  411.5.4), or where a residual current device protects the circuit U0 / IΔn (TN, IT)
  and 50 V / IΔn (TT: RA × IΔn ≤ UL, 411.5.3; Zs includes RA).

What the project does not give is shown as *missing*, never assumed: no characteristic
C, no 30 mA and no limit resting on them, unlike the calculations, which assume them.

The results of a circuit, kept as entered (`0,35`, `> 500`) and checked against their
limits:

| Result | Limit |
| --- | --- |
| Continuity of the protective conductor (Ω) | recorded |
| Insulation test voltage (V) | at least 500 V up to 500 V nominal, 1000 V above (table 6.1) |
| Insulation resistance (MΩ, the lowest measured) | at least 1 MΩ |
| Loop impedance Zs (Ω) | the highest loop impedance above |
| RCD tripping current (mA) | ½ IΔn … IΔn |
| RCD tripping time at IΔn (ms) | at most 300 ms; type S 130 … 500 ms |
| RCD tripping time at 5 × IΔn (ms) | at most 40 ms; type S 50 … 150 ms |
| Contact voltage UB at IΔn (V) | at most 50 V |
| Function test (polarity, rotation, switching) | OK / Not OK |
| Tested by, date, test instrument, remark | recorded |

A circuit is *Not passed* when a value is out of its limit or the function test
failed, *Passed* when the continuity, the insulation (with its test voltage), the
loop impedance, the function test and (with an RCD) the tripping time at IΔn are
there, *Incomplete* in between and *Not tested* without values.

**The data of the report**: client, installation site, installer, reason for the test
(new installation, extension, change, repair), inspector, test instruments
(IEC 61557), the values measured at the origin (external loop impedance Ze,
prospective fault current, earth electrode resistance RA, main equipotential bonding)
and the date of the next inspection. In a TT system RA must not exceed 50 V / the
largest rated residual current of the installation. **The visual inspection** (6.4.2)
answers 14 items with OK, Not OK or Not applicable: protection against electric shock,
fire barriers, conductors, protective devices, isolating and switching devices,
external influences, identification of N and PE, diagrams and warning notices,
labelling, connections, protective and bonding conductors, accessibility, additional
protection by RCDs of at most 30 mA, surge protection.

**The verdict** of the report is *Not passed* when a circuit, an item of the visual
inspection or RA fails, *Passed* when every circuit passed, every item is answered,
the inspector named and (TT) RA measured. **Signing** records who signed and when;
any later change of the data or of a result takes the signature away. The report
exports as an Excel workbook (on the web page with a summary sheet and a sheet of
the circuits).

Not covered: protection by SELV, PELV or electrical separation, floor and wall
insulation, IT systems (insulation monitoring, first fault), measured voltage drop and
a separate polarity or phase sequence test (part of the function test).

**On site, with a tablet or a phone:** the web page of the project server
([project server](../project-server.md)) shows the report of every project under
*Inspection and test report*: a card with the data of the report, the visual
inspection, the verdict and *Sign the report*, then a card per circuit with the data
of the schematic and the fields of the results (number keyboard, the limit next to
each field, values out of the limits marked at once; the insulation test voltage, who
tests, today and the last instrument are filled in for a new circuit). Who may save
reviews or edit the project enters them there and saves each card on its own,
without opening the project and without a lock; values not yet saved stay in the
browser (also across a lost connection). The editors that have the project open take
them over with the next revision; their own saves keep them (the data and the results
are merged card by card, as the review records). *Excel* downloads the report.
