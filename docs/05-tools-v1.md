# 05 — Tools V1 (contrato del engranaje)

Estado: aceptado

Complementa `04-agentes-v1.md`. Introduce el catálogo de tools como engranaje
propio, porque es una capacidad con ciclo de vida distinto al de un agente
(`../ESTANDARES.md` §2.2). Las tools **no se crean por API**: se implementan en
código; el API solo las lista y los agentes las referencian.

## Especialidad

> El catálogo de tools del sistema: qué capacidades existen y su metadata.
> No sabe de agentes ni de proyectos.

## Alcance

Incluido:

- `Tool { id, name, description }` definido en código (catálogo estático);
- `office.tool.list.v1` → `office.tool.listed.v1`;
- resolución `tool_id → Tool` para armar las definiciones de agente.

Excluido (a propósito): crear/editar tools por API (son código), implementación
del ejecutor de cada tool (execution plane) y permisos por tool.

## Engranaje

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `tool` | Catálogo de tools (consulta y resolución): dominio + puerto | `ToolCatalog` |
| `toolbox` | Adaptador: catálogo **definido en código** (`BuiltinToolCatalog`) | — (implementa `ToolCatalog`) |

El engranaje `tool` no contiene adaptadores; `toolbox` los hospeda y `officed`
solo lo construye e inyecta. Un engranaje nuevo (`register`) no existe a
propósito: las tools se definen en código, no por API.

## Dominio

Una tool es una **función que el modelo puede invocar** (tool calling), con la
forma del API de function calling: `type: "function"` + `function.name`,
`function.description`, `function.parameters` (JSON Schema de los argumentos).

- `ToolId` (slug, = `function.name`) y
  `Tool { id, name, description, parameters }`, donde `parameters` es el JSON
  Schema del objeto de argumentos.
- Invariante: el catálogo es la **única fuente de verdad** de la definición de
  una tool. El envoltorio `type: "function"` es un detalle del wire hacia el
  modelo, no del dominio.

## Puertos

```rust
pub trait ToolCatalog {
    fn list(&self) -> Vec<Tool>;
    fn get(&self, id: &str) -> Option<Tool>;
}
```

## Relación con `agent` (aislamiento, §2.3)

- `agent` **no conoce** a `tool` ni lo declara en su `Cargo.toml`.
- La definición de agente guarda **ids** de tools (`Vec<String>`), no
  `{name, description}` embebidos.
- El **puente** en `officed` (el único que conoce a ambos) resuelve
  `id → Tool` y valida al definir un agente: un id desconocido se rechaza.

## Protocolo

Comandos:

| Tipo | Datos | Efecto |
|------|-------|--------|
| `office.tool.list.v1` | — | lista el catálogo |

Eventos:

| Tipo | Datos |
|------|-------|
| `office.tool.listed.v1` | `tools[] ({ id, name, description, parameters })` |

`parameters` es el JSON Schema de los argumentos, listo para el campo
`function.parameters` de la llamada al modelo.

La definición de agente (`04-agentes-v1.md`) cambia:

- comando `office.agent.definition.create.v1` recibe `tools[]` como **ids**;
- evento `office.agent.definition.created/listed.v1` devuelve `tools[]`
  **resueltas** (`{ id, name, description }`).

### Errores del dominio

| Error | Cuándo | Código de rechazo |
|-------|--------|-------------------|
| `UnknownTool(id)` | se referencia un id que no está en el catálogo | `command_failed` |
| `InvalidTool` | id vacío o duplicado en la definición | `command_failed` |

## Persistencia

La tool no se persiste (es código). La referencia del agente sí: la columna
`agent_definitions.tools` pasa a ser un JSON de **ids**. Migración
`0005_agent_tool_ids.sql` convierte los objetos previos `{name, description}` a
sus `name` (ids).

## Pendiente

1. Implementación del ejecutor de cada tool y su binding al sandbox
   (`docs/10`: sólo `shell` en V1).
2. Permisos/capacidades por tool (qué concede cada una).
3. Versionado del catálogo y compatibilidad de definiciones ante cambios.
4. Flag `strict` de function calling (validación estricta del JSON Schema).
