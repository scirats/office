# 01 — Daemon V1

Estado: propuesto

Primer daemon de Office siguiendo `../ESTANDARES.md`. Implementa el protocolo
discutido (CloudEvents sobre WebSocket en socket Unix) con el mínimo vertebral:
comandos de entrada, eventos de salida y replay.

## Alcance

Incluido:

- transporte WebSocket sobre Unix socket, subprotocolo negociado `office.v1`;
- envelope CloudEvents 1.0 (structured JSON, un evento por frame);
- comandos: `subscription.add`, `subscription.remove`, `subscription.resume`,
  `task.submit`, `execution.cancel`;
- eventos: `command.accepted`, `command.rejected`, `task.accepted`,
  `execution.state_changed`, `execution.log`, `error`;
- historia de eventos ordenada con secuencia por `subject`;
- replay por cursor y detección de `resync_required`;
- lifecycle de ejecución simulado (state machine) como demostración.

Excluido (a propósito): persistencia, sandbox Linux real, autenticación,
métricas, artifacts.

## Engranajes

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `protocol` | Vocabulario de la wire: envelope CloudEvents y taxonomía v1 | ninguno |
| `execution` | Ciclo de vida y máquina de estados de una ejecución | `Clock`, `IdGenerator` |
| `broker` | Historia ordenada, secuencia, suscripciones y replay | ninguno |
| `officed` | Composición, adaptadores (reloj, ids, transporte) y puentes | — |

Los engranajes no se conocen entre sí: solo `officed` los une por puentes
(`bridges/encoder.rs` y el despacho en `daemon.rs`).

## Protocolo

- Subprotocolo: `office.v1` negociado en el handshake; sin él, el handshake se
  rechaza con `400`.
- Envelope: CloudEvents 1.0 structured JSON. Extensiones propias:
  `correlationid`, `causationid`, `sequence`.
- Dirección: el cliente envía comandos (intenciones); el daemon emite eventos
  (hechos).
- `sequence` es por `subject` (`task:<id>`, `execution:<id>`), asignado por el
  daemon; es la base del replay.
- `subscription.resume` reenvía eventos posteriores a `last_sequence`; si el
  cursor quedó fuera del buffer, responde `office.error.v1` con
  `code = resync_required`.

## Ejecución

```bash
cd daemon
cargo build
OFFICE_SOCKET=/tmp/office.sock cargo run -p officed
```

Por defecto escucha en `/run/office/office.sock`. Un cliente WebSocket debe
negociar el subprotocolo `office.v1` y hablar CloudEvents.

## Pipeline

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Decisiones y desviaciones

- **Infraestructura reutilizada:** se usa `tokio` + `tokio-tungstenite` para el
  transporte. `ESTANDARES.md` §6 prefiere no reinventar infraestructura
  general; el control preciso de Linux (epoll, seccomp, etc.) corresponderá al
  plano de ejecución, no a este daemon de control.
- **Tests de integración:** viven en `crates/officed/tests/` en lugar de
  `daemon/tests/`. Cargo no compila tests de un workspace virtual sin paquete
  raíz; el paquete que los posee es `officed`.
- **Replay en memoria:** el buffer es acotado y no sobrevive al reinicio del
  daemon. Persistencia (SQLite) queda para V1 real, según `PROMPT.md` §58–§68.
- **`allowed(result_large_err)`:** la firma de `negotiate_subprotocol` la fija el
  contrato `Callback` de tungstenite; no es reducible.

## Pendiente para la V1 real

1. Persistencia del estado de dominio (SQLite) y reconciliación.
2. Sandbox Linux por ejecución (namespaces, cgroup v2, pidfd, mounts RO/RW).
3. Autenticación por contexto local (`SO_PEERCRED`).
4. Separación control plane / privileged helper.
