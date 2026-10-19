# 06 — Modelos V1 (contrato del engranaje)

Estado: aceptado

Complementa `04-agentes-v1.md` y `05-tools-v1.md`. Introduce el catálogo de
modelos como engranaje propio, porque es una capacidad con ciclo de vida
distinto al de un agente (`../ESTANDARES.md` §2.2). Los modelos **no se crean por
API**: se declaran en configuración; el API solo los lista y los agentes los
referencian.

## Especialidad

> El catálogo de modelos de IA y sus proveedores: qué modelos existen, qué
> proveedor los sirve y qué capacidades tienen. No sabe de agentes, proyectos ni
> ejecuciones.

## Alcance

Incluido:

- `Provider { id, kind, endpoint, credential_ref }` y
  `Model { id, provider, api_name, display_name, context_window, max_output_tokens, capabilities }`;
- catálogo declarado en `/etc/office/models.toml` (adaptador de archivo);
- `office.model.list.v1` → `office.model.listed.v1`;
- resolución `model_id → Model` al definir un agente.

Excluido (a propósito): el cliente que llama al API del modelo, el store de
secretos, cuotas/límites por modelo y selección por tarea. La V1 fija **un**
modelo por `Agent Definition`.

## Engranaje

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `model` | Catálogo de modelos y proveedores (dominio + puerto) | `ModelCatalog` |
| `officed` (adaptador) | `FileModelCatalog`: lee y valida el TOML | — (implementa `ModelCatalog`) |

El engranaje `model` no contiene adaptadores; `officed` los hospeda y los
inyecta. A diferencia de `toolbox` (catálogo en código), aquí el adaptador es de
archivo, pero el dominio no cambia: sustituirlo por SQLite/API no toca `model`.

## Dominio

- `ProviderId`, `ModelId`.
- `ProviderKind`: `Anthropic | OpenAi | AzureOpenAi | Local | Other(String)`.
- `Provider` incluye `endpoint` y `credential_ref`.
- `Capabilities { tool_calling, vision, reasoning }`.

Invariantes:

1. `Model.id` y `Provider.id` son únicos en el catálogo.
2. Todo `Model.provider` referencia un `Provider` existente.
3. `credential_ref` es una **clave**, nunca el secreto. El secreto lo entrega
   el store de secretos por ejecución (`docs/03` §49–§51, `docs/07`).

## Puertos

```rust
pub trait ModelCatalog {
    fn list(&self) -> Vec<Model>;
    fn get(&self, id: &str) -> Option<Model>;
    fn provider(&self, id: &str) -> Option<Provider>;
}
```

## Configuración

Archivo TOML global (PROMPT §11), por defecto `/etc/office/models.toml`,
sobreescribible con `OFFICE_MODELS`. Si falta, el daemon arranca con catálogo
vacío y avisa por stderr; si existe pero es inválido, el arranque falla de forma
visible.

```toml
[[provider]]
id = "anthropic"
kind = "anthropic"
endpoint = "https://api.anthropic.com"
credential_ref = "provider/anthropic"

[[model]]
id = "claude-sonnet"
provider = "anthropic"
api_name = "claude-sonnet-4-20250514"
display_name = "Claude Sonnet"
context_window = 200000
max_output_tokens = 8192
capabilities = { tool_calling = true, vision = true, reasoning = false }
```

Un ejemplo completo vive en `../daemon/config/models.toml`.

## Relación con `agent` (aislamiento, §2.3)

- `agent` **no conoce** a `model` ni lo declara en su `Cargo.toml`.
- La definición de agente guarda el **id** del modelo (`String`), no el objeto.
- El **puente** en `officed` resuelve `id → Model` y valida al definir un
  agente: un id desconocido o vacío se rechaza.
- En el cliente web, el catálogo alimenta el selector del draft **New agent** (al
  crear) y el editor de **Definition** del agente; el modelo elegido al crear se
  conserva en la definición.

## Protocolo

Comandos:

| Tipo | Datos | Efecto |
|------|-------|--------|
| `office.model.list.v1` | — | lista el catálogo |

Eventos:

| Tipo | Datos |
|------|-------|
| `office.model.listed.v1` | `models[] ({ id, provider, provider_kind, endpoint, api_name, display_name, context_window, max_output_tokens, capabilities, credential_configured })` |

`credential_configured` indica si el `credential_ref` del proveedor ya está en el
secret store (`docs/07`). La UI lo usa para enmascarar el input de API key
(`••••••••`) sin exponer el valor.

`credential_ref` **no** cruza el wire.

La definición de agente (`04-agentes-v1.md`) cambia:

- comando `office.agent.definition.create.v1` recibe `model` (id);
- evento `office.agent.definition.created/listed.v1` devuelve `model`.

### Errores del dominio

| Error | Cuándo | Código de rechazo |
|-------|--------|-------------------|
| `InvalidModel` | `model` vacío en la definición | `command_failed` |
| `unknown model` | el id no está en el catálogo (validado por el puente) | `command_failed` |
| `ModelConfigError` | TOML ilegible, inválido, duplicado o con provider colgante | fallo de arranque |

## Persistencia

El modelo no se persiste (es configuración). La referencia del agente sí: se
agrega la columna `agent_definitions.model` con la migración
`0006_agent_definition_model.sql`.

## Pendiente

1. Inyección de la credencial por ejecución (`credential_ref → Secret` vía
   FD/`memfd`, `docs/07`).
2. Cliente del API por `ProviderKind` y binding al execution plane.
3. Límites/cuotas por modelo y proveedor.
4. Modelo por tarea con override sobre el de la definición.
