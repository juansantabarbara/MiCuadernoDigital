# Google Calendar · configuración única

MiCuadernoDigital utiliza OAuth para aplicaciones de escritorio y crea un calendario secundario llamado **MiCuadernoDigital**.

## Configuración en Google Cloud
1. Crear/seleccionar un proyecto en Google Cloud Console.
2. Habilitar **Google Calendar API**.
3. Configurar la pantalla de consentimiento OAuth.
4. Crear credenciales → **OAuth client ID** → tipo **Desktop app**.
5. Copiar el Client ID que termina en `.apps.googleusercontent.com`.
6. En MiCuadernoDigital → Configuración → Google Calendar, pegar el Client ID y pulsar **Conectar con Google**.
7. Autorizar en el navegador.

## Privacidad
La integración utiliza el scope `calendar.app.created`: MiCuadernoDigital crea y gestiona su calendario de aplicación. La sincronización envía únicamente elementos de **Mi agenda**. Alumnado, calificaciones, observaciones, reuniones, actuaciones e inteligencia pedagógica permanecen locales.

## Dirección de sincronización
Por diseño, la 0.3.0 usa una fuente única de verdad:

**MiCuadernoDigital → Google Calendar**

Los cambios hechos directamente en Google pueden ser reemplazados en la siguiente sincronización.
