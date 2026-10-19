# 09 — Ejecución de modelo V1 (contrato del engranaje `inference`)

Estado: aceptado

Cierra el hueco detectado al probar el Playground: `task.submit` resolvía una
ejecución **simulada**. Ahora el daemon resuelve el agente real y llama al
proveedor. Sigue `../ESTANDARES.md`: un engranaje nuevo (`inference`), un puerto,
contratos explícitos y pruebas.

## Alcance

Incluido:

- chat completions **OpenAI-compatible** con streaming SSE;
- resolución agente → modelo → proveedor → credencial en la composición;
- estado real de la ejecución y logs por delta en `execution.log`;
- rechazo tipado cuando falta agente, modelo, proveedor o credencial.

Excluido (a propósito): la API nativa de Anthropic (protocolo distinto),
tool-calling, context builder, timeouts y cancelación real del stream.

## Engranaje

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `inference` | Chat completions OpenAI-compatible y decodificación SSE | `HttpClient` |

`model` sigue siendo solo **catálogo** (qué modelos y proveedores existen);
`inference` es quien **ejecuta**. No se llama `provider` para no colisionar con
`model::Provider`.

`officed` provee el adaptador `ReqwestHttpClient` y los puentes.

## Dominio

- `ChatRequest { endpoint, api_key, model, system, input }`.
- `ChatError`: `Transport` (fallo de red/HTTP) y `Decode` (respuesta inválida).

## Puerto

```rust
pub type ByteStream = BoxStream<'static, Result<Vec<u8>, HttpError>>;

pub trait HttpClient: Send + Sync {
    fn post_stream(
        &self,
        url: String,
        headers: Vec<(String, String)>,
        body: serde_json::Value,
    ) -> BoxFuture<'static, Result<ByteStream, HttpError>>;
}
```

El puerto sólo mueve bytes; el cuerpo, las cabeceras y el parseo de SSE son del
engranaje.

## Servicio

`ChatService::stream(ChatRequest) -> Future<Result<ChatStream, ChatError>>`:

1. arma `POST {endpoint}/chat/completions` con `model`, `stream: true` y los
   mensajes `system` + `user`;
2. inyecta `Authorization: Bearer <api_key>`;
3. decodifica SSE: cada línea `data:` con `choices[0].delta.content` se emite
   como delta; `[DONE]` termina; los `delta` sin `content` (p. ej. `role`) se
   ignoran; las líneas partidas entre chunks se bufferizan.

## Adaptador

`ReqwestHttpClient` (en `officed`, con `reqwest` + `rustls`) implementa el
puerto. Un HTTP no-2xx se convierte en `HttpError` con estado y cuerpo.

## Flujo de ejecución

En `officed` (composition root), `submit_task`:

```text
resolve agent definition (slug)     → si falta, command.rejected
resolve model (definition.model)    → si falta, command.rejected
resolve provider (model.provider)   → si falta, command.rejected
resolve credential (credential_ref)  → si falta, command.rejected
accept_task                          → task.accepted
CREATED → PREPARING
spawn run_execution:
    PREPARING → READY → RUNNING
    stream deltas → execution.log (stdout)
    FINISHING → SUCCEEDED
    (error de stream → execution.log stderr + RUNNING → FAILED)
```

La credencial viaja solo en la cabecera; `Secret` tiene `Debug` redactado y
nunca se registra. El engine de `execution` no cambia: sigue siendo la máquina de
estados; lo que cambia es que ahora la alimentan eventos reales.

## Protocolo

Sin cambios. `office.task.submit.v1` sigue igual; lo que antes producía líneas
fijas ahora produce texto del modelo.

## Pendiente

1. Anthropic nativo (endpoint y formato distintos).
2. Tool-calling: bucle de herramientas y ejecución de tools.
3. Context builder por tarea (`PROMPT.md` §108–109).
4. `execution.cancel` que aborte el stream en curso.
5. Timeouts, reintentos y presupuesto de tokens.
6. Persistencia de la conversación (hoy la sesión es efímera).
