# Arquitectura

## Frontend
La v34 actual sigue siendo la interfaz. No se reescriben los flujos que ya funcionan.

## Persistencia
`save_state` valida el JSON, abre una transacción SQLite, sincroniza las tablas normalizadas y solo después actualiza el snapshot global. Si algo falla, la transacción se revierte.

Tablas: `students`, `observations`, `attendance_entries`, `units`, `sessions`, `criteria`, `products`, `product_criteria`, `evidence_columns`, `grades`, `agenda`, `meetings`, `meeting_participants`, `meeting_people`, `custom_days`, `issues`, `settings`, `app_state`, `state_backups`, `schema_meta`.

## Incidencias
Se guardan dentro del estado y en su propia tabla `issues`, con versión, tipo, sección, prioridad, descripción, expectativa y estado.

## Estrategia de migración
El snapshot permite abrir datos de versiones anteriores. Las tablas normalizadas permiten ir sustituyendo progresivamente lecturas globales por consultas específicas sin perder compatibilidad.
