# MCP transcript: an agent draws a DOL motor starter

Written by `docs/launch/ai/mcp_demo.py`; every JSON-RPC message is in `transcript.jsonl`.

## Handshake

`initialize` → OpenECraft 0.1.2, protocol 2025-06-18

> OpenECraft is an electrical CAE (schematic) editor. Open or create a project first (open_project, new_project); the tools change it in memory until save_project writes it, undo / redo revert single tool calls. Pages are A3 (420 x 297 mm, y downwards, 10 columns, 6 rows); positions snap to a 4 mm grid. Use list_symbols for symbol keys and connection points, get_page for positions and item ids, connect to draw wires between connection points (TAG:PIN, e.g. -K1:A1), check to find errors and report / export for the results. Device tags follow IEC 81346 (-K1 contactor, -S1 switch, -X1 terminal strip). Parts come from the project, the built-in catalog or the parts server (parts_* tools).

`tools/list` → 443 tools, e.g. `new_project`, `list_examples`, `open_project`, `save_project`, `project_info`, `set_project_info`, `undo`, `redo`, `index_projects`, `search_projects`, `batch_project_properties`, `archive_project` …

## Project

1. `new_project` `{"name": "Pump P1 – direct-on-line starter", "template": "Empty (IEC 60204-1)", "path": "pump_dol.oec", "overwrite": true}` – an empty project with the IEC 60204-1 potentials (L1 L2 L3 N PE, +24V 0V)
   → {"name": "Pump P1 – direct-on-line starter", "pages": 1, "devices": 0, "errors": 0, "warnings": 0, "file": "pump_dol.oec"}
2. `set_project_info` `{"company": "Demo Ltd.", "creator": "AI agent via MCP", "date": "2026-11-18", "description": "Pump P1, 2.2 kW\ndirect-on-line start\n24 V DC control"}`
   → {"name": "Pump P1 – direct-on-line starter", "pages": 1, "devices": 0, "errors": 0, "warnings": 0, "file": "pump_dol.oec"}
3. `set_wire_colors` `{"enabled": true}` – draw wires in the colours of their potentials
   → {"potentials": [{"color": "BK", "cross_section": "2.5 mm²", "description": "", "kind": "l", "name": "L1", "voltage": "400 V AC"}, {"color": "BK", "cross_section": "2.5 mm²", "description": "", "kind":
4. `set_wire_numbering` `{"scheme": "page_column"}` – wire numbers by page and column
   → {"digits": 0, "format": "", "prefix": "", "scheme": "page_column", "skip_kinds": [], "start": 1, "step": 1}
5. `list_symbols` `{"query": "motor protection"}` – look up symbol keys and connection points
   → 4 entries
6. `add_page` `{"name": "cover", "kind": "cover", "description": "Cover sheet"}` – a cover sheet, moved in front of the schematic page of the template
   → {"name": "cover"}
7. `set_page` `{"page": "cover", "before": "1"}`
   → {"name": "cover"}
8. `renumber_pages` `{}`
   → {"pages": ["1", "2"]}
9. `set_page` `{"page": "2", "description": "Power circuit pump P1"}`
   → {"name": "2"}

## Power circuit (page 2)

10. `place_symbol` `{"page": "2", "symbol": "pot_l1", "x": 20, "y": 24, "rotation": 270}`
   → id 3, L1: 1 (24, 24)
11. `add_wire` `{"page": "2", "points": [[24, 24], [400, 24]]}`
   → {"ids": [4]}
12. `place_symbol` `{"page": "2", "symbol": "pot_l2", "x": 20, "y": 32, "rotation": 270}`
   → id 5, L2: 1 (24, 32)
13. `add_wire` `{"page": "2", "points": [[24, 32], [400, 32]]}`
   → {"ids": [6]}
14. `place_symbol` `{"page": "2", "symbol": "pot_l3", "x": 20, "y": 40, "rotation": 270}`
   → id 7, L3: 1 (24, 40)
15. `add_wire` `{"page": "2", "points": [[24, 40], [400, 40]]}`
   → {"ids": [8]}
16. `place_symbol` `{"page": "2", "symbol": "pot_n", "x": 20, "y": 48, "rotation": 270}`
   → id 9, N: 1 (24, 48)
17. `add_wire` `{"page": "2", "points": [[24, 48], [400, 48]]}`
   → {"ids": [10]}
18. `place_symbol` `{"page": "2", "symbol": "pot_pe", "x": 20, "y": 56, "rotation": 270}`
   → id 11, PE: 1 (24, 56)
19. `add_wire` `{"page": "2", "points": [[24, 56], [400, 56]]}`
   → {"ids": [12]}
20. `place_symbol` `{"page": "2", "symbol": "disc3", "tag": "-Q1", "x": 60, "y": 72, "function_text": "Main switch"}` – main switch
   → id 13, -Q1: 1 (60, 68), 3 (68, 68), 5 (76, 68), 2 (60, 76), 4 (68, 76), 6 (76, 76)
21. `add_wire` `{"page": "2", "points": [[60, 24], [60, 68]]}` – tap the phase rails
   → {"ids": [14]}
22. `add_wire` `{"page": "2", "points": [[68, 32], [68, 68]]}` – tap the phase rails
   → {"ids": [15]}
23. `add_wire` `{"page": "2", "points": [[76, 40], [76, 68]]}` – tap the phase rails
   → {"ids": [16]}
24. `place_symbol` `{"page": "2", "symbol": "mps3", "tag": "-Q2", "x": 60, "y": 100, "function_text": "Motor protection"}`
   → id 17, -Q2: 1 (60, 96), 3 (68, 96), 5 (76, 96), 2 (60, 104), 4 (68, 104), 6 (76, 104)
25. `place_symbol` `{"page": "2", "symbol": "main3", "tag": "-K1", "x": 60, "y": 136, "function_text": "Pump on"}`
   → id 18, -K1: 1 (60, 132), 3 (68, 132), 5 (76, 132), 2 (60, 140), 4 (68, 140), 6 (76, 140)
26. `connect` `{"page": "2", "from": "-Q1:2", "to": "-Q2:1"}`
   → wire (60, 76) → (60, 96)
27. `connect` `{"page": "2", "from": "-Q2:2", "to": "-K1:1"}`
   → wire (60, 104) → (60, 132)
28. `connect` `{"page": "2", "from": "-Q1:4", "to": "-Q2:3"}`
   → wire (68, 76) → (68, 96)
29. `connect` `{"page": "2", "from": "-Q2:4", "to": "-K1:3"}`
   → wire (68, 104) → (68, 132)
30. `connect` `{"page": "2", "from": "-Q1:6", "to": "-Q2:5"}`
   → wire (76, 76) → (76, 96)
31. `connect` `{"page": "2", "from": "-Q2:6", "to": "-K1:5"}`
   → wire (76, 104) → (76, 132)
32. `place_symbol` `{"page": "2", "symbol": "term", "tag": "-X1", "label": "1", "x": 60, "y": 164}`
   → id 25, -X1: 1 (60, 160), 2 (60, 168)
33. `place_symbol` `{"page": "2", "symbol": "term", "tag": "-X1", "label": "2", "x": 68, "y": 164}`
   → id 26, -X1: 1 (68, 160), 2 (68, 168)
34. `place_symbol` `{"page": "2", "symbol": "term", "tag": "-X1", "label": "3", "x": 76, "y": 164}`
   → id 27, -X1: 1 (76, 160), 2 (76, 168)
35. `place_symbol` `{"page": "2", "symbol": "term_pe", "tag": "-X1", "label": "PE", "x": 84, "y": 164}`
   → id 28, -X1: 1 (84, 160), 2 (84, 168)
36. `connect` `{"page": "2", "from": "-K1:2", "to": "#25:1"}` – terminals of one strip share the tag: #id:pin
   → wire (60, 140) → (60, 160)
37. `connect` `{"page": "2", "from": "-K1:4", "to": "#26:1"}` – terminals of one strip share the tag: #id:pin
   → wire (68, 140) → (68, 160)
38. `connect` `{"page": "2", "from": "-K1:6", "to": "#27:1"}` – terminals of one strip share the tag: #id:pin
   → wire (76, 140) → (76, 160)
39. `update_wire` `{"page": "2", "id": 19, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 19, "cable": ""}
40. `update_wire` `{"page": "2", "id": 20, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 20, "cable": ""}
41. `update_wire` `{"page": "2", "id": 21, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 21, "cable": ""}
42. `update_wire` `{"page": "2", "id": 22, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 22, "cable": ""}
43. `update_wire` `{"page": "2", "id": 23, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 23, "cable": ""}
44. `update_wire` `{"page": "2", "id": 24, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 24, "cable": ""}
45. `update_wire` `{"page": "2", "id": 29, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 29, "cable": ""}
46. `update_wire` `{"page": "2", "id": 30, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 30, "cable": ""}
47. `update_wire` `{"page": "2", "id": 31, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 31, "cable": ""}
48. `add_wire` `{"page": "2", "points": [[84, 56], [84, 160]]}` – protective conductor to the PE terminal
   → {"ids": [32]}
49. `place_symbol` `{"page": "2", "symbol": "motor3", "tag": "-M1", "x": 60, "y": 196, "function_text": "Pump P1\n2.2 kW"}`
   → id 33, -M1: U1 (60, 192), V1 (68, 192), W1 (76, 192), PE (84, 192)
50. `connect` `{"page": "2", "from": "#25:2", "to": "-M1:U1"}`
   → wire (60, 168) → (60, 192)
51. `connect` `{"page": "2", "from": "#26:2", "to": "-M1:V1"}`
   → wire (68, 168) → (68, 192)
52. `connect` `{"page": "2", "from": "#27:2", "to": "-M1:W1"}`
   → wire (76, 168) → (76, 192)
53. `connect` `{"page": "2", "from": "#28:2", "to": "-M1:PE"}`
   → wire (84, 168) → (84, 192)
54. `add_cable_definition` `{"page": "2", "points": [[52, 180], [92, 180]]}` – the four wires to the motor become cores of a new cable
   → {"id": 39, "cable": "-W1"}
55. `apply_cable_part` `{"cable": "-W1", "part": "GEN.CBL-4G1.5"}`
   → {"cable": "-W1"}
56. `update_wire` `{"page": "2", "id": 34, "core": "BN", "color": "BN"}`
   → {"id": 34, "cable": "-W1"}
57. `update_wire` `{"page": "2", "id": 35, "core": "BK", "color": "BK"}`
   → {"id": 35, "cable": "-W1"}
58. `update_wire` `{"page": "2", "id": 36, "core": "GY", "color": "GY"}`
   → {"id": 36, "cable": "-W1"}
59. `update_wire` `{"page": "2", "id": 37, "core": "GNYE", "color": "GNYE"}`
   → {"id": 37, "cable": "-W1"}
60. `table_get` `{"table": "cables"}` – the cable list, for the row key
   → {"columns": [{"editable": true, "name": "Cable"}, {"editable": true, "name": "Type"}, {"editable": true, "name": "Cores"}, {"editable": true, "name": "Cross-section"}, {"editable": true, "name": "Leng
61. `table_set` `{"table": "cables", "key": "38", "column": "Length", "value": "15"}`
   → {"changed": true}
62. `table_set` `{"table": "cables", "key": "38", "column": "Description", "value": "Pump P1"}`
   → {"changed": true}

## 24 V DC supply (page 2)

63. `place_symbol` `{"page": "2", "symbol": "cb1", "tag": "-F1", "x": 300, "y": 64, "function_text": "Control voltage"}`
   → id 40, -F1: 1 (300, 60), 2 (300, 68)
64. `add_wire` `{"page": "2", "points": [[300, 24], [300, 60]]}`
   → {"ids": [41]}
65. `place_symbol` `{"page": "2", "symbol": "psu", "tag": "-T1", "x": 300, "y": 80, "function_text": "24 V DC"}`
   → id 42, -T1: L (300, 76), N (308, 76), PE (316, 76), L+ (300, 96), M (308, 96)
66. `connect` `{"page": "2", "from": "-F1:2", "to": "-T1:L"}`
   → wire (300, 68) → (300, 76)
67. `update_wire` `{"page": "2", "id": 43, "color": "BK", "cross_section": "1.5 mm²"}`
   → {"id": 43, "cable": ""}
68. `add_wire` `{"page": "2", "points": [[308, 48], [308, 76]]}`
   → {"ids": [44]}
69. `add_wire` `{"page": "2", "points": [[316, 56], [316, 76]]}`
   → {"ids": [45]}
70. `place_symbol` `{"page": "2", "symbol": "intr", "label": "+24V", "x": 300, "y": 112}` – interruption point: continues on page 3
   → id 46, +24V: 1 (300, 112)
71. `add_wire` `{"page": "2", "points": [[300, 96], [300, 112]]}`
   → {"ids": [47]}
72. `place_symbol` `{"page": "2", "symbol": "intr", "label": "0V", "x": 324, "y": 112}`
   → id 48, 0V: 1 (324, 112)
73. `add_wire` `{"page": "2", "points": [[308, 96], [308, 104], [324, 104], [324, 112]]}`
   → {"ids": [49, 50, 51]}

## Control circuit (page 3)

74. `add_page` `{"name": "3", "kind": "schematic", "description": "Control circuit pump P1"}`
   → {"name": "3"}
75. `place_symbol` `{"page": "3", "symbol": "intr", "label": "+24V", "x": 20, "y": 24, "rotation": 90}`
   → id 53, +24V: 1 (20, 24)
76. `add_wire` `{"page": "3", "points": [[20, 24], [400, 24]]}`
   → {"ids": [54]}
77. `place_symbol` `{"page": "3", "symbol": "intr", "label": "0V", "x": 20, "y": 236, "rotation": 90}`
   → id 55, 0V: 1 (20, 236)
78. `add_wire` `{"page": "3", "points": [[20, 236], [400, 236]]}`
   → {"ids": [56]}
79. `place_symbol` `{"page": "3", "symbol": "ol_nc", "tag": "-Q2", "pins": "21,22", "x": 40, "y": 40}` – auxiliary contact of the motor protection switch
   → id 57, -Q2: 21 (40, 36), 22 (40, 44)
80. `add_wire` `{"page": "3", "points": [[40, 24], [40, 36]]}`
   → {"ids": [58]}
81. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "1", "x": 40, "y": 56}`
   → id 59, -X2: 1 (40, 52), 2 (40, 60)
82. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "2", "x": 40, "y": 96}`
   → id 60, -X2: 1 (40, 92), 2 (40, 100)
83. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "3", "x": 40, "y": 120}`
   → id 61, -X2: 1 (40, 116), 2 (40, 124)
84. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "4", "x": 40, "y": 160}`
   → id 62, -X2: 1 (40, 156), 2 (40, 164)
85. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "5", "x": 120, "y": 168}`
   → id 63, -X2: 1 (120, 164), 2 (120, 172)
86. `place_symbol` `{"page": "3", "symbol": "term", "tag": "-X2", "label": "6", "x": 120, "y": 216}`
   → id 64, -X2: 1 (120, 212), 2 (120, 220)
87. `connect` `{"page": "3", "from": "-Q2:22", "to": "#59:1"}`
   → wire (40, 44) → (40, 52)
88. `place_symbol` `{"page": "3", "symbol": "pb_nc", "tag": "-S1", "x": 40, "y": 76, "function_text": "Stop"}`
   → id 66, -S1: 11 (40, 72), 12 (40, 80)
89. `connect` `{"page": "3", "from": "#59:2", "to": "-S1:11"}`
   → wire (40, 60) → (40, 72)
90. `connect` `{"page": "3", "from": "-S1:12", "to": "#60:1"}`
   → wire (40, 80) → (40, 92)
91. `connect` `{"page": "3", "from": "#60:2", "to": "#61:1"}`
   → wire (40, 100) → (40, 116)
92. `place_symbol` `{"page": "3", "symbol": "pb_no", "tag": "-S2", "x": 40, "y": 140, "function_text": "Start"}`
   → id 70, -S2: 13 (40, 136), 14 (40, 144)
93. `connect` `{"page": "3", "from": "#61:2", "to": "-S2:13"}`
   → wire (40, 124) → (40, 136)
94. `connect` `{"page": "3", "from": "-S2:14", "to": "#62:1"}`
   → wire (40, 144) → (40, 156)
95. `place_symbol` `{"page": "3", "symbol": "coil", "tag": "-K1", "x": 40, "y": 200, "function_text": "Pump on"}`
   → id 73, -K1: A1 (40, 196), A2 (40, 204)
96. `connect` `{"page": "3", "from": "#62:2", "to": "-K1:A1"}`
   → wire (40, 164) → (40, 196)
97. `add_wire` `{"page": "3", "points": [[40, 204], [40, 236]]}`
   → {"ids": [75]}
98. `place_symbol` `{"page": "3", "symbol": "no", "tag": "-K1", "pins": "13,14", "x": 64, "y": 140}` – self-holding contact in parallel to the start button, both ends in the cabinet
   → id 76, -K1: 13 (64, 136), 14 (64, 144)
99. `add_wire` `{"page": "3", "points": [[40, 108], [64, 108], [64, 136]]}`
   → {"ids": [77, 78]}
100. `add_wire` `{"page": "3", "points": [[64, 144], [64, 176], [40, 176]]}`
   → {"ids": [79, 80]}
101. `place_symbol` `{"page": "3", "symbol": "no", "tag": "-K1", "pins": "53,54", "x": 120, "y": 128}` – lamp contact
   → id 81, -K1: 53 (120, 124), 54 (120, 132)
102. `add_wire` `{"page": "3", "points": [[120, 24], [120, 124]]}`
   → {"ids": [82]}
103. `connect` `{"page": "3", "from": "-K1:54", "to": "#63:1"}`
   → wire (120, 132) → (120, 164)
104. `place_symbol` `{"page": "3", "symbol": "lamp", "tag": "-P1", "x": 120, "y": 196, "function_text": "Pump running"}`
   → id 84, -P1: X1 (120, 192), X2 (120, 200)
105. `connect` `{"page": "3", "from": "#63:2", "to": "-P1:X1"}`
   → wire (120, 172) → (120, 192)
106. `connect` `{"page": "3", "from": "-P1:X2", "to": "#64:1"}`
   → wire (120, 200) → (120, 212)
107. `add_wire` `{"page": "3", "points": [[120, 220], [120, 236]]}`
   → {"ids": [87]}

## Parts, report pages, wire numbers

108. `assign_part` `{"tag": "-Q1", "number": "GEN.MS-3P-40A"}`
   → {"tag": "-Q1"}
109. `assign_part` `{"tag": "-Q2", "number": "GEN.MPS-4-6.3A"}`
   → {"tag": "-Q2"}
110. `assign_part` `{"tag": "-Q2", "number": "GEN.MPS-AUX-1NO1NC"}`
   → {"tag": "-Q2"}
111. `assign_part` `{"tag": "-K1", "number": "GEN.CT-9A-24DC"}`
   → {"tag": "-K1"}
112. `assign_part` `{"tag": "-K1", "number": "GEN.AUX-2NO2NC"}`
   → {"tag": "-K1"}
113. `assign_part` `{"tag": "-M1", "number": "GEN.MOT-2.2KW"}`
   → {"tag": "-M1"}
114. `assign_part` `{"tag": "-X1", "number": "GEN.TB-2.5"}`
   → {"tag": "-X1"}
115. `assign_part` `{"tag": "-X1", "number": "GEN.TB-PE-2.5"}`
   → {"tag": "-X1"}
116. `assign_part` `{"tag": "-X2", "number": "GEN.TB-2.5"}`
   → {"tag": "-X2"}
117. `assign_part` `{"tag": "-F1", "number": "GEN.MCB-1P-C6"}`
   → {"tag": "-F1"}
118. `assign_part` `{"tag": "-T1", "number": "GEN.PS-24DC-5A"}`
   → {"tag": "-T1"}
119. `assign_part` `{"tag": "-S1", "number": "GEN.PB-RD-1NC"}`
   → {"tag": "-S1"}
120. `assign_part` `{"tag": "-S2", "number": "GEN.PB-GN-1NO"}`
   → {"tag": "-S2"}
121. `assign_part` `{"tag": "-P1", "number": "GEN.LMP-GN-24"}`
   → {"tag": "-P1"}
122. `add_page` `{"name": "4", "kind": "terminal_diagram", "description": "Terminal diagram"}`
   → {"name": "4"}
123. `add_page` `{"name": "5", "kind": "cable_overview", "description": "Cable overview"}`
   → {"name": "5"}
124. `add_page` `{"name": "6", "kind": "device_list", "description": "Device list"}`
   → {"name": "6"}
125. `add_page` `{"name": "7", "kind": "bill_of_materials", "description": "Bill of materials"}`
   → {"name": "7"}
126. `renumber_wires` `{}`
   → {"renumbered": 31}

## Check, reports, export

127. `check` `{}` – errors, warnings and hints of the project
   → {"errors": 0, "warnings": 0}
128. `report` `{"kind": "bill_of_materials"}`
   → {"headers": ["Pos", "Qty", "Unit", "Part number", "Type number", "Manufacturer", "Description", "Devices", "Unit price", "Total"], "report": "bill_of_materials", "rows": [["1", "1", "pcs", "GEN.AUX-2N
129. `report` `{"kind": "bill_of_materials", "format": "csv"}` – the same as CSV, which the agent saves for the user
   → Pos;Qty;Unit;Part number;Type number;Manufacturer;Description;Devices;Unit price;Total
130. `report` `{"kind": "terminal_diagram"}` – terminal diagram as a table
   → {"headers": ["Strip", "Terminal", "Level", "Jumper", "External (top)", "Internal (bottom)", "Potential", "Location", "Function text"], "report": "terminal_diagram", "rows": [["-X1", "End bracket", "",
131. `export` `{"format": "pdf", "path": "pump_dol.pdf"}`
   → {"path": "pump_dol.pdf", "bytes": 106910}
132. `export` `{"format": "svg", "page": "2", "path": "power.svg"}`
   → {"path": "power.svg", "bytes": 31814}
133. `export` `{"format": "svg", "page": "3", "path": "control.svg"}`
   → {"path": "control.svg", "bytes": 25712}
134. `save_project` `{}`
   → {"path": "pump_dol.oec"}
135. `convert_project` `{"path": "pump_dol.oec", "format": "oecz"}` – a compressed archive (55 kB) to attach or share
   → {"from": 2, "newer": false, "output": "pump_dol.oecz", "steps": [], "to": 2}

## Result

135 tool calls. Check: 0 errors, 0 warnings. Bill of materials: 21 rows. Files: `pump_dol.oecz`, `pump_dol.pdf`, `power.svg`, `control.svg`, `bill_of_materials.csv`.
