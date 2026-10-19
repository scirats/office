# 08 — Proyectos V1 (contrato del engranaje)

Estado: aceptado

Implementa el modelo organizacional de `../PROMPT.md` §1–§2 y §13: una compañía
tiene proyectos y un proyecto tiene agentes. Corrige una desviación de la V1
previa: el proyecto **no** se derivaba de `agent_instances`, por lo que no podía
existir sin agente y no permitía verlo como unidad propia. Sigue
`../ESTANDARES.md`: un engranaje (`project`), puertos, contratos explícitos y
pruebas.

## Alcance

Incluido:

- `Project` como entidad de primera clase, global y persistida;
- identidad `(workspace, id)`;
- comandos y eventos v1 para crear, listar y borrar;
- persistencia en SQLite (misma base que el resto del dominio);
- validación en la composición: el proyecto pertenece a un workspace existente.

Excluido (a propósito): repositorios, knowledge, artifacts, policies y
`Session`/ejecución (ver `docs/02`, `docs/03`).

## Engranaje

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `project` | Proyectos de un workspace (dominio y reglas) | `ProjectRepository` |

`officed` provee el adaptador SQLite y los puentes. Ningún otro engranaje conoce
a `project`; `agent` sigue sin conocerlo y su `link` no cambia.

## Dominio

- `Project { id, workspace, name }`. `id` se deriva del `name` por slug (misma
  regla que workspace y agente).
- `ProjectRecord` incluye `created_at` para listados.

Invariantes:

1. `id` es un slug válido.
2. `workspace` es un slug válido.
3. `UNIQUE(workspace, id)`: el mismo nombre de proyecto no se repite dentro de
   un workspace, pero sí puede repetirse en workspaces distintos
   (`AlreadyExists`).
4. **Crear un proyecto no exige agentes.** El enlace `project → agent` es un paso
   posterior y opcional.
5. Un proyecto puede tener **N instancias** (una por agente distinto), según
   `docs/02`; nunca dos del mismo agente.
6. Borrar un proyecto exige que no tenga instancias enlazadas.
7. Borrar un workspace exige que no tenga proyectos.

## Puertos

```rust
pub trait ProjectRepository {
    fn save(&self, project: &Project) -> Result<(), StoreError>;
    fn find(&self, workspace: &str, id: &str) -> Result<Option<ProjectRecord>, StoreError>;
    fn list(&self, workspace: Option<&str>) -> Result<Vec<ProjectRecord>, StoreError>;
    fn remove(&self, workspace: &str, id: &str) -> Result<bool, StoreError>;
}
```

La existencia del workspace la valida la composición (`officed`), no el
engranaje: `project` no conoce a `workspace`.

## Protocolo

Comandos (cliente → daemon):

| Tipo | Datos | Efecto |
|------|-------|--------|
| `office.project.create.v1` | `workspace, name` | crea un proyecto |
| `office.project.list.v1` | `workspace?` | lista proyectos (filtro opcional) |
| `office.project.delete.v1` | `workspace, id` | borra un proyecto sin instancias |

Eventos (daemon → cliente):

| Tipo | Datos | `subject` |
|------|-------|-----------|
| `office.project.created.v1` | `id, workspace, name` | `project:<workspace>/<id>` |
| `office.project.listed.v1` | `projects[]` | — |
| `office.project.deleted.v1` | `workspace, id` | `project:<workspace>/<id>` |

Los eventos de creación/borrado llevan `correlationid` del comando. Todo comando
emite `office.command.accepted.v1`; un fallo emite `office.command.rejected.v1`.

### Cambio de contrato en `agent`

`office.agent.instance.create.v1` ahora exige que el proyecto exista
(`project ... not found`). Antes el proyecto era implícito en el enlace; ese
comportamiento se retira. La unicidad `(agent, workspace, project)` no cambia.

### Errores del dominio

| Error | Cuándo | Código de rechazo |
|-------|--------|-------------------|
| `InvalidName` | `name` no produce slug válido | `command_failed` |
| `InvalidWorkspace` | `workspace` no es slug válido | `command_failed` |
| `AlreadyExists` | ya existe `(workspace, id)` | `command_failed` |
| `UnknownProject` | borrar un proyecto inexistente | `command_failed` |
| `Store` | fallo interno de persistencia | `command_failed` |

## Persistencia

Migración `0007_projects.sql`:

```sql
CREATE TABLE projects (
    id TEXT NOT NULL,
    workspace TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (workspace, id)
);
```

## UI

- Crear un proyecto pide **workspace** y **nombre**, nada más. No pide agente.
- Los agentes se enlazan **después**, arrastrando una arista desde el nodo
  agente hasta el nodo proyecto (o al revés). Se pueden enlazar varios; el
  duplicado se ignora y el daemon lo rechaza.
- El grafo dibuja los nodos proyecto desde la lista de proyectos, no desde las
  instancias: un proyecto aparece aunque no tenga agentes.

## Pendiente

1. Repositorios, knowledge y artifacts por proyecto (`PROMPT.md` §13).
2. Borrado en cascada y reconciliación de proyectos huérfanos si se permite
   borrar workspaces con contenido.
3. Renombrar un proyecto (hoy el `id` es estable y el `name` es inmutable).
