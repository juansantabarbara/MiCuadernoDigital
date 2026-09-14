use mdns_sd::{ServiceDaemon, ServiceInfo};

pub fn start_discovery() -> Result<ServiceDaemon, String> {
    let mdns = ServiceDaemon::new()
        .map_err(|e| format!("No se pudo iniciar mDNS de Sidecar: {e}"))?;

    let service_type = "_micuaderno._tcp.local.";
    let instance_name = "MiCuadernoDigital";
    let host_name = "micuadernodigital.local.";
    let port = 38491;

    let service = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        "",
        port,
        &[("protocol", "1")][..],
    )
    .map_err(|e| format!("No se pudo crear el anuncio Sidecar: {e}"))?
    .enable_addr_auto();

    mdns.register(service)
        .map_err(|e| format!("No se pudo publicar Sidecar por mDNS: {e}"))?;

    Ok(mdns)
}
