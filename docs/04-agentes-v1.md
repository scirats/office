# 04 — Agentes V1 (contrato del engranaje)

Estado: aceptado

Implementa el modelo de `02-agente-instancia-sesion.md` y
`03-agente-identidad-recursos.md` en el daemon. Extiende el protocolo descrito en
`01-daemon-v1.md`. Sigue `../ESTANDARES.md`: un engranaje (`agent`), puertos,
contratos explícitos y pruebas.

## Alcance

Incluido:

- `Agent Definition` global, versionada (persistida);
- `Agent Instance` como enlace `agent ↔ proyecto` con unicidad;
- comandos y eventos v1 para definir, listar, enlazar, listar y desenlazar;
- persistencia en SQLite (misma base que el resto del dominio).

Excluido (a propósito): `Session` (efímera, no deja rastro, ver `docs/02`),
credenciales, identidad de ejecución Linux y materialización del sandbox.

## Engranaje

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `agent` | Definiciones e instancias de agente (dominio y reglas) | `AgentRepository`, `InstanceIdSource` |

`officed` provee los adaptadores (SQLite, UUIDv7) y los puentes. Ningún otro
engranaje conoce a `agent`.

## Dominio

- `AgentDefinition { id, name, role, prompt, tools, model, version }`.
  `id` se deriva del `name` por slug (misma regla que workspace). Re-definir el
  mismo `id` incrementa `version`.
- La V1 configura **prompt** (system prompt), **tools** y **model**. Las tools y
  el modelo se referencian por **id** de sus catálogos (`docs/05`, `docs/06`);
  el agente no conoce a los engranajes `tool`/`model`, la resolución la hace el
  puente. No lleva runtime, policy ni sandbox: la instancia corre como el usuario
  del workspace y trabaja dentro de su homedir (`docs/03`).
- `AgentInstance { id, agent, workspace, project }`.
- `AgentInstanceRecord` incluye `created_at` para listados.

Invariantes:

1. `id` de definición es un slug válido.
2. `link` exige una definición existente; si no, `UnknownAgent`.
3. `link` exige un proyecto existente (`docs/08`); si no, la composición rechaza
   con `command_failed`. El proyecto ya no es implícito en el enlace.
4. `UNIQUE(agent, workspace, project)`: un agente solo puede tener una instancia
   por proyecto (`AlreadyLinked`).
5. Un agente sin instancia no trabaja ("en la banca", `docs/03`).

## Puertos

```rust
pub trait AgentRepository {
    fn save_definition(&self, definition: &AgentDefinition) -> Result<(), StoreError>;
    fn find_definition(&self, id: &str) -> Result<Option<AgentDefinition>, StoreError>;
    fn list_definitions(&self) -> Result<Vec<AgentDefinition>, StoreError>;

    fn save_instance(&self, instance: &AgentInstance) -> Result<(), StoreError>;
    fn find_instance(&self, agent: &str, workspace: &str, project: &str)
        -> Result<Option<AgentInstance>, StoreError>;
    fn list_instances(&self, workspace: Option<&str>, project: Option<&str>)
        -> Result<Vec<AgentInstanceRecord>, StoreError>;
    fn remove_instance(&self, agent: &str, workspace: &str, project: &str)
        -> Result<bool, StoreError>;
}

pub trait InstanceIdSource {
    fn instance_id(&self) -> String;
}
```

## Protocolo

Comandos (cliente → daemon):

| Tipo | Datos | Efecto |
|------|-------|--------|
| `office.agent.definition.create.v1` | `name, role, prompt, tools[]` (ids del catálogo), `model` (id del catálogo) | crea/versiona una definición |
| `office.agent.definition.list.v1` | — | lista definiciones |
| `office.agent.definition.delete.v1` | `id` | borra la definición si no tiene instancias enlazadas |
| `office.agent.instance.create.v1` | `agent, workspace, project` | enlaza (crea instancia) |
| `office.agent.instance.list.v1` | `workspace?, project?` | lista instancias (filtro opcional) |
| `office.agent.instance.delete.v1` | `agent, workspace, project` | desenlaza (idempotente) |

Eventos (daemon → cliente):

| Tipo | Datos | `subject` |
|------|-------|-----------|
| `office.agent.definition.created.v1` | `AgentDefinitionView` | `agent:<id>` |
| `office.agent.definition.listed.v1` | `definitions[]` | — |
| `office.agent.definition.deleted.v1` | `id` | `agent:<id>` |
| `office.agent.instance.created.v1` | `AgentInstanceView` | `agent-instance:<id>` |
| `office.agent.instance.listed.v1` | `instances[]` | — |
| `office.agent.instance.deleted.v1` | `agent, workspace, project` | `agent-instance:<id>` |

Los eventos de creación llevan `correlationid` del comando. Todo comando emite
`office.command.accepted.v1`; un fallo emite `office.command.rejected.v1`.

### Errores del dominio

| Error | Cuándo | Código de rechazo |
|-------|--------|-------------------|
| `InvalidName` | `name` no produce slug válido | `command_failed` |
| `InvalidRole` | `role` vacío | `command_failed` |
| `InvalidScope` | `workspace`/`project` no son slug válido | `command_failed` |
| `UnknownAgent` | enlazar una definición inexistente | `command_failed` |
| `StillLinked` | borrar una definición con instancias enlazadas | `command_failed` |
| `AlreadyLinked` | ya existe instancia `(agent, workspace, project)` | `command_failed` |
| `InvalidModel` | `model` vacío en la definición | `command_failed` |
| `Store` | fallo interno de persistencia | `command_failed` |

## Persistencia

Migraciones `0003_agents.sql` y `0004_agent_definition_tools.sql` (rename
`skills` → `tools`, retira `runtime`/`policy`):

```sql
CREATE TABLE agent_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    prompt TEXT NOT NULL,
    tools TEXT NOT NULL,           -- JSON array
    model TEXT NOT NULL DEFAULT '',
    version INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE agent_instances (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    workspace TEXT NOT NULL,
    project TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (agent_id, workspace, project)
);
```

## Pendiente

1. Historial de versiones (hoy se conserva la última con `version` incrementado).
2. `Session` y ejecución real (efímeras) y `UNIQUE` con estado observado.
3. Credenciales por nivel (store de secretos, `docs/03`).
4. Desenlazar en cascada y reconciliación de instancias huérfanas.
