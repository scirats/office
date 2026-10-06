# 02 — Agente, Instancia y Sesión

Estado: aceptado

Modelo de agente que resuelve cómo un especialista atiende múltiples proyectos
sin mezclar contexto, coherente con `../ESTANDARES.md` (engranajes, puertos,
contratos) y con `../PROMPT.md` (§3, §69, §108–§109).

## Contexto y problema

`PROMPT.md` §3 separa cuatro conceptos:

```text
Agent Definition  →  Agent Identity  →  Task Execution  →  Linux Process Tree
```

pero no fija dónde vive el contexto del agente. El caso real es: **un solo
agente configurado atiende muchos proyectos** y no debe mezclar el contexto de
uno con el de otro.

## Decisión

Cuatro capas, donde el **enlace** (`project → agent`) es lo que materializa la
instancia:

```text
Agent Definition   configuración global: skills, prompt, runtime, policies
        │  enlazar a un proyecto  (edge project→agent)
        ▼
Agent Instance     referencia viva del agente en UN proyecto; siempre disponible;
                   sin contexto; solo config + scope del proyecto
        │  inicia una acción
        ▼
Session            acumula contexto mientras dura UNA tarea; al terminar se destruye
        │
        ▼
Execution(s)       process tree efímero (namespaces, cgroup v2, pidfd)
```

- **Agent Definition:** se configura una vez. Es global y compartida.
- **Agent Instance:** se crea al enlazar el agente a un proyecto. Es la
  referencia del agente dentro de ese proyecto; está siempre disponible, no
  guarda contexto y no consume recursos mientras está quieta.
- **Session:** se abre cuando la instancia inicia una acción. Acumula contexto
  durante su tarea y se descarta al terminar.
- **Execution:** el runtime Linux efímero que produce la sesión.

## Invariantes

1. **Unicidad por mundo.** `UNIQUE(agent_definition_id, project_id)`. Un
   proyecto puede tener N instancias (una por agente distinto), pero nunca dos
   del mismo agente. Analogía: mundos paralelos; en cada mundo hay una persona
   distinta y no se repite.
2. **La instancia no guarda contexto; la sesión sí, solo durante su tarea.**
3. **Una sesión activa por instancia.** Las demás tareas de esa instancia se
   encolan.
4. **La tarea no deja rastro.** Al terminar, la sesión se destruye sin persistir
   contexto ni estado, y la instancia no conserva nada de la tarea. Los efectos
   externos que el agente realice (p. ej. `git push`) ocurren fuera del modelo y
   no son estado de la sesión.
5. **Continuidad externa.** Lo único que sobrevive entre tareas es lo que vive
   fuera del agente: repos, knowledge, artifacts, base de datos. El contexto del
   agente es desechable y reconstruible.
6. **Correlación.** `session_id` une `task_id`, `project_id`, `company_id`,
   `definition_id` y sus `execution_id`s (cgroup, logs, auditoría). La instancia
   nunca guarda "proyecto actual": lo determina su enlace.
7. **Ciclo de vida de la instancia.** Vive mientras exista el enlace
   `project → agent`. Al desenlazar, la instancia desaparece y sus sesiones
   activas se cancelan y re-encolan.

## Lifecycle

```text
Instance   creada al enlazar ───────────────────────────► destruida al desenlazar
Session                  CREATED → BOOTSTRAPPING → RUNNING → CLOSED
                         fallos: FAILED / CANCELLED / CRASHED
Execution                máquina descrita en docs/01-daemon-v1.md
```

## Consecuencias

- **Aislamiento estructural:** una instancia pertenece a un solo proyecto, así
  que el contexto no puede cruzarse por construcción.
- **Sin degradación:** ninguna sesión arrastra memoria; cada tarea arranca en
  blanco.
- **Coste por tarea:** cada tarea paga el bootstrap de contexto. El *context
  builder* (armar el contexto mínimo por tarea, `PROMPT.md` §109) es pieza
  central, no un extra. Optimizaciones (índice cacheado, prompt caching, warm
  pool) son V2 y no constituyen contexto del agente.
- **Crash recovery idempotente:** si Office o la sesión mueren, la tarea se
  re-encola y se abre una sesión nueva sobre la misma instancia. La instancia
  sobrevive porque es un enlace, no un proceso.
- **Concurrencia:** muchas instancias del mismo agente en distintos proyectos
  corren en paralelo, cada una con su sandbox y cuotas; dentro de una instancia
  las tareas se serializan.
- **El graph ya lo enforça:** una arista `project → agent` con detección de
  duplicados es exactamente la unicidad 1:1.

## Alternativas descartadas

- **Instancia por tarea (efímera):** la instancia dejaría de ser una referencia
  estable del agente en el proyecto; se pierde el concepto de "agente enlazado".
  Se eligió instancia por proyecto y sesión por tarea.
- **Instancia con memoria entre tareas:** reutiliza contexto pero degrada y
  contamina. Se descarta explícitamente para evitar degradación.
- **Varias instancias del mismo agente por proyecto:** rompe la exclusividad de
  "una persona por mundo".

## Naming

`Instance` y `Session`. Evita la colisión con `Execution`, que en el resto del
sistema designa al process tree efímero.

## Pendiente

- Definición de la UI del agente (nodo → instancias → sesiones → TUI estilo
  opencode). Se aborda por separado.
