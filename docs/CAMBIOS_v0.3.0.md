# MiCuadernoDigital 0.3.0

## Diario

### Nueva sección · Actuaciones
Registro persistente y editable para coordinaciones, encargos y gestiones del centro.

Cada actuación guarda:
- fecha y hora;
- ámbito (TIC, Coordinación, Equipo directivo, Tutoría, Centro u Otro);
- quién la solicita;
- estado (Abierta / Finalizada);
- título;
- petición recibida;
- actuación realizada;
- resultado;
- seguimiento o pendientes.

Incluye filtros, búsqueda, edición, borrado y duplicado/reutilización.

### Editor enriquecido
Reuniones y Actuaciones incorporan un editor local sin dependencias externas con:
- negrita, cursiva y subrayado;
- listas con viñetas y numeradas;
- aumentar/disminuir sangría;
- títulos y párrafos;
- deshacer/rehacer.

Las reuniones antiguas escritas como texto plano siguen siendo compatibles.

## Google Calendar
- Un único calendario secundario llamado `MiCuadernoDigital`.
- Sincronización unidireccional MiCuadernoDigital → Google Calendar.
- Los tipos de Mi agenda se conservan mediante etiqueta + color.
- Sincronización automática opcional y botón manual.
- Eventos sin hora: día completo.
- Eventos con hora: duración configurable.
- Al completar una entrada se añade ✓.
- Al borrar una entrada local se elimina el evento sincronizado.
- Solo sale a Google el contenido de `Mi agenda`.

## Actualizaciones
Configuración incluye una zona de actualización:
- muestra la versión instalada;
- busca una release firmada;
- muestra versión y notas;
- descarga e instala desde la propia app.

La release debe estar publicada (no Draft) para que `releases/latest/download/latest.json` sea accesible.

## Datos
- Esquema SQLite 4.
- Nueva tabla normalizada `actuaciones`.
- Se conserva el snapshot completo como cinturón de seguridad.
- No cambia el identificador de aplicación `es.micuadernodigital.app`, por lo que se conserva la base existente al actualizar.
