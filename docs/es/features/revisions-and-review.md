# Revisiones, comentarios y gestión de cambios

Revisiones con páginas bloqueadas y cambios marcados, el flujo de aprobación, líneas base con comparación y fusión, comentarios y marcado en rojo, y solicitudes de cambio.

## Revisiones

Una revisión guarda una instantánea de todas las páginas y puede liberarlas (bloquearlas);
las páginas bloqueadas son de solo lectura. Los cambios desde la última revisión liberada
(símbolos, hilos, textos, gráficos, cotas, imágenes, definiciones de conexión y de cable,
páginas y sus propiedades añadidos / modificados / eliminados) se enumeran, se marcan en
las páginas con un triángulo de revisión y se recogen en el *Resumen de revisiones*; los
elementos modificados indican las propiedades modificadas con el valor antiguo y el nuevo
(`Device tag: -K1 › -K2; Position`).

## Flujo de liberación

Con aprobadores definidos (*Datos de proyecto › Revisiones… › Flujo de
aprobación*: nombres de usuario y el número de aprobaciones necesarias) una revisión
nueva empieza como borrador, se envía a revisión, los aprobadores la aprueban o la
rechazan (queda registrado con usuario, fecha y nota) y se libera una vez aprobada. Las
páginas modificadas de cualquier modo después del envío bloquean la liberación hasta que
la revisión se envía de nuevo (las aprobaciones anteriores quedan sin efecto).

## Líneas base y comparación / fusión

Una línea base guarda el proyecto completo (páginas, dispositivos,
artículos, cables, diseño del armario, ajustes); al liberar una revisión se guarda una
automáticamente. *Datos de proyecto › Comparar y líneas base…* compara el proyecto con
una línea base, con las páginas de una revisión o con otro archivo de proyecto: páginas
añadidas / eliminadas / modificadas (con los elementos modificados), dispositivos,
artículos, potenciales, cables, mazos, envolventes, regleteros, conectores, tarjetas de
PLC y secciones de ajustes. Las diferencias seleccionadas se adoptan.

## Revisión: comentarios y marcado en rojo

Comentarios en una posición de la página con estado (abierto, en
curso, resuelto, rechazado), autor, fecha y respuestas; marcado redline (por modificar) y
greenline (modificado conforme a obra) como línea, rectángulo, nube de revisión o texto.
Se dibujan en una capa propia (se muestra en pantalla, no en las exportaciones), se
enumeran en la pestaña *Comentarios* del navegador y se pueden añadir a páginas liberadas
y a páginas que otro usuario tiene bloqueadas. En el servidor de proyectos se fusionan
registro por registro, sin bloqueos.
Los **comentarios PDF** que un revisor ha hecho en una exportación PDF (Acrobat u otro
visor: notas, cuadros de texto, resaltados, formas, respuestas) se importan en la capa de
comentarios (*Comentarios › Importar comentarios PDF…*, comando `import_pdf_comments`);
los rectángulos y las líneas se convierten además en marcado redline, y los comentarios
importados anteriormente se omiten.

## Solicitudes de cambio

Solicitudes procedentes de dentro o de fuera del proyecto (número, título,
descripción, solicitante, origen, prioridad, estado, páginas y dispositivos afectados,
revisión que la implementa) con comentarios vinculados (*Datos de proyecto › Solicitudes de cambio…*). Intercambio
con otros sistemas como CSV (`Number;Title;Description;Requester;Origin;Date;Priority;
Status;Pages;Devices;Revision`) o JSON; la importación actualiza las solicitudes con un número conocido.
