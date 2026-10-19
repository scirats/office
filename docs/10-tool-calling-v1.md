# 10 — Tool calling y sandbox V1

Estado: aceptado

Cierra dos huecos: las tools eran sólo catálogo (`docs/05`) y el agente no podía
invocarlas; y la ejecución corría en el host. Ahora el modelo recibe las tools,
puede pedir llamadas, y cada tool se ejecuta según el **ámbito**.

Sigue `../ESTANDARES.md`: dos engranajes nuevos (`runner`, `sandbox`), el
engranaje `inference` extendido, puertos explícitos y pruebas.

## Engranajes

| Crate | Especialidad | Puertos que consume |
|-------|--------------|---------------------|
| `inference` | Chat OpenAI-compatible: mensajes, tools y `tool_calls` | `HttpClient` |
| `runner` | Bucle agente↔tools con historial y presupuesto de turnos | `Chat`, `ToolRunner` |
| `tool` / `toolbox` | Catálogo y `tool → Action` (comando + validación) | `ToolCatalog` |
| `sandbox` | Mecanismo de aislamiento (bwrap), genérico | `Sandbox` |

`inference` no conoce `runner`; `runner` no conoce `inference` ni `sandbox`.
Los adaptadores viven en `officed`: `InferenceChat` (puente a `inference`),
`ToolExecutor` (plano de ejecución), `BwrapSandbox` (mecanismo) y `host::run`.

Separación de responsabilidades:

| Pieza | Responsabilidad |
|-------|-----------------|
| `inference` | Protocolo del modelo (`tool_calls`) |
| `runner` | Orquestar el bucle |
| `toolbox::action` | tool → `Action` (programa + args, validación) |
| `ToolExecutor` | Colocar la ejecución (sandbox vs host) + política |
| `BwrapSandbox` | Mecanismo de aislamiento, siempre aísla |
| `host::run` | Correr un proceso en el host |

## Ámbitos de ejecución

```text
Agent  (Playground)   → sandbox efímero (bwrap); sin workspace/project
Instance (proyecto)   → host, usuario del workspace, cwd en el project
```

El ámbito viaja en `runner::RunContext { execution_id, scope }`. El Playground
manda `project` vacío → `RunScope::Agent`. Un task con `project` exige una
instancia (`agent + project`); si no existe, se rechaza.

## Flujo del bucle

```text
messages = [system, user]
loop hasta max_steps:
    turno del modelo (tools incluidas)
      → Text  → execution.log (stdout)
      → ToolCall → ToolExecutor (sandbox/host) → execution.log (stream "tool")
                   → mensaje role:"tool" con el resultado
    si no hay ToolCall → fin
```

## Sandbox

`SandboxSpec { program, args, workdir, env, mounts, network, timeout }` con
`Mount { source, target, read_only }`. El spec es **genérico**: no conoce
workspace ni project.

- `BwrapSandbox` siempre aísla (`--unshare-all`: mount/pid/net/ipc/uts/user),
  aplica los `mounts` del spec y añade `/proc`, `/dev` y `/tmp`.
- `host::run` ejecuta un proceso en el host (instancias). No se llama "sandbox".

Para el ámbito agente, `ToolExecutor` traduce a mounts: los roots del sistema
RO (`/usr`, `/bin`, …, sólo si existen) y un `/workspace` efímero RW en
`/run/office/executions/<execution_id>/workspace`. Sin red por defecto.

`bwrap` es una dependencia **documentada de V1**, detrás del puerto `Sandbox`;
se puede reemplazar por primitivas crudas (unshare/mount/pivot_root/Landlock)
sin tocar `runner` ni `ToolExecutor`.

### Política y lifecycle

Los tunables viven en `ExecutionPolicy` (env con defaults):

```text
OFFICE_MAX_STEPS          (default 8)
OFFICE_TOOL_TIMEOUT_SECS  (default 30)
OFFICE_ALLOW_NETWORK      (default false)
OFFICE_EXECUTIONS_DIR     (default /run/office/executions)
OFFICE_CGROUP_ROOT        (default /sys/fs/cgroup; "none" desactiva)
OFFICE_MEMORY_MAX_MB      (opcional)
OFFICE_PIDS_MAX           (opcional)
OFFICE_LANDLOCK           (default true)
```

- El runner pone al hijo en su **propio process group**; al vencer el timeout
  mata el **árbol** (`killpg`) y, si hay cgroup, con `cgroup.kill`.
- **cgroup v2** (best-effort): crea `office/executions/<id>`, aplica `memory.max`
  y `pids.max`, mete el proceso y lo borra al terminar. Si el host no lo permite,
  la ejecución sigue sin cgroup.
- El ámbito `Instance` corre en host como el **usuario del workspace**
  (`uid`/`gid` desde `/etc/passwd`) con `cwd` en el project, y con **Landlock**:
  lectura/ejecución en todo el sistema y escritura sólo en project, home, `/tmp`
  y el dir de ejecuciones. Se aplica en el hijo vía `pre_exec` (sólo syscalls,
  sin asignaciones).
- Al terminar (éxito o fallo) se **limpia** `/run/office/executions/<id>`
  (idempotente). La reconciliación detecta proyectos/instancias huérfanos.

## Tools ejecutables

V1 implementa `shell` (`{ command }` → `/bin/sh -c …`). Las demás tools del
catálogo responden "tool is not implemented yet". La resolución
`definition.tools → ChatTool` valida que el id exista en el catálogo.

## Protocolo

Sin tipos nuevos: los eventos de tool viajan como `execution.log` con
`stream = "tool"` (llamada) y `stream = "tool-result"` (resultado). El Playground
muestra el texto del modelo como contenido principal, las tools compactas y las
transiciones de estado como un indicador de estado, no como líneas del
transcript.

## Pendiente

1. `editor` y el resto de tools (hoy sólo `shell`).
2. `pidfd` para supervisión estable; hoy se usa `wait` + process group.
3. Aprobar/denegar tools por política (capacidades por tool, `docs/05`).
4. `Sandbox` para instancias (mount namespace con el project RW, runtime RO);
   hoy la instancia usa host + Landlock.
5. Red en el ámbito `Instance` (Landlock network no cubre todo; evaluar netns).
