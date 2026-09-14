import Foundation
import Network

let parameters = NWParameters.tcp
parameters.includePeerToPeer = true
parameters.allowLocalEndpointReuse = true

do {
    let listener = try NWListener(using: parameters)

    listener.service = NWListener.Service(
        name: "MiCuadernoDigital",
        type: "_micuaderno._tcp"
    )

    listener.stateUpdateHandler = { state in
        switch state {
        case .setup:
            print("Sidecar Apple: preparando listener")

        case .waiting(let error):
            print("Sidecar Apple: esperando · \(error)")

        case .ready:
            if let port = listener.port {
                print("Sidecar Apple: listo")
                print("Puerto dinámico: \(port)")
                print("Bonjour: MiCuadernoDigital._micuaderno._tcp")
                print("Peer-to-peer: activado")
            }

        case .failed(let error):
            print("Sidecar Apple: ERROR · \(error)")
            exit(1)

        case .cancelled:
            print("Sidecar Apple: cancelado")

        @unknown default:
            print("Sidecar Apple: estado desconocido")
        }
    }

    listener.newConnectionHandler = { connection in
        print("Sidecar Apple: conexión entrante detectada")
        connection.cancel()
    }

    listener.start(queue: .main)

    print("Sidecar Apple 0.1 arrancando...")
    RunLoop.main.run()

} catch {
    print("No se pudo crear el listener: \(error)")
    exit(1)
}
