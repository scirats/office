# OFFICE — Native Linux Agent Runtime & Orchestrator

## 0. Objetivo

Quiero diseñar e implementar **Office**, una plataforma de agentes de IA construida directamente sobre Linux.

La máquina estará dedicada prácticamente en exclusiva a Office.

No quiero tratar Linux simplemente como el sistema operativo donde corre mi aplicación.

Quiero que:

> **Linux sea parte fundamental del runtime de Office.**

Office debe aprovechar directamente las primitivas modernas del kernel Linux para:

- identidad;
- procesos;
- aislamiento;
- permisos;
- filesystem;
- recursos;
- networking;
- seguridad;
- observabilidad;
- ejecución;
- lifecycle;
- recuperación.

Quiero evitar recrear en userspace mecanismos que Linux ya implementa correctamente.

La plataforma base será inicialmente:

```text
Artix Linux
    +
Dinit
    +
Linux Kernel
    +
Office
```

Office será implementado preferentemente en **Rust**.

No quiero Docker, Podman, Kubernetes ni systemd como dependencia arquitectónica.

Pueden estudiarse para aprender de sus soluciones, pero Office debe construirse directamente sobre primitivas Linux cuando tenga sentido.

---

# 1. Idea fundamental

La máquina representa conceptualmente una **oficina**.

Dentro de ella existen:

```text
Office
│
├── Company
│   │
│   ├── Company Resources
│   │
│   ├── Project
│   │   ├── repositories
│   │   ├── knowledge
│   │   ├── artifacts
│   │   └── agents
│   │
│   └── Project
│
├── Company
│
└── ...
```

Una instalación de Office puede manejar múltiples compañías.

Una compañía puede tener múltiples proyectos.

Un proyecto puede contener múltiples repositorios.

Un agente/especialista puede trabajar en múltiples proyectos.

Los agentes realizan trabajos concretos.

Una ejecución de un agente debería ser normalmente efímera.

---

# 2. Modelo organizacional

Conceptualmente:

```text
Office
  │
  ├── Company A
  │     │
  │     ├── shared resources
  │     │
  │     ├── Project Alpha
  │     │      ├── repository A
  │     │      ├── repository B
  │     │      ├── knowledge
  │     │      ├── artifacts
  │     │      └── agents
  │     │
  │     └── Project Beta
  │
  └── Company B
```

Los proyectos NO son repositorios.

Un proyecto puede representar un sistema completo compuesto por:

- backend;
- frontend;
- mobile;
- infraestructura;
- documentación;
- modelos;
- datasets;
- herramientas;
- múltiples repositorios;
- conocimiento;
- decisiones arquitectónicas;
- artefactos.

---

# 3. Agentes

Un agente representa un especialista.

Ejemplos:

```text
backend-engineer
frontend-engineer
security-engineer
devops-engineer
reviewer
architect
researcher
```

Pero debemos separar cuatro conceptos:

```text
Agent Definition
        │
        ▼
Agent Identity
        │
        ▼
Task Execution
        │
        ▼
Linux Process Tree
```

No deben considerarse equivalentes.

Un agente puede existir persistentemente como definición sin tener ningún proceso activo.

Ejemplo:

```text
backend-engineer
    │
    ├── execution #1001 → process tree → exit
    ├── execution #1034 → process tree → exit
    └── execution #1088 → process tree → exit
```

El agente persiste conceptualmente.

Las ejecuciones son efímeras.

---

# 4. Principio arquitectónico principal

Aplicar esta regla:

> **Si Linux ya tiene una primitiva robusta para representar o controlar algo, Office debe intentar orquestarla en lugar de reinventarla.**

Ejemplos:

```text
Necesidad                   Linux

identidad                   UID/GID
ownership                   UID/GID
permisos básicos            DAC
permisos extendidos         POSIX ACL
procesos                    Linux process model
process handles             pidfd
aislamiento                 namespaces
jerarquía recursos          cgroup v2
CPU limits                  cgroup v2
memory limits               cgroup v2
PID limits                  cgroup v2
I/O control                 cgroup v2
resource pressure           PSI
filesystem                  VFS
filesystem views            mount namespaces
filesystem temporal         OverlayFS
UID mapping                 user namespaces/idmapped mounts
restricción filesystem      Landlock
restricción de red          Landlock/network namespace/etc.
syscall filtering           seccomp
privilegios                 Linux capabilities
no privilege escalation     no_new_privs
procesos activos            /proc
event multiplexing          epoll
timers                      timerfd
signals                     signalfd/pidfd
event notification          eventfd
runtime ephemeral state     /run
host services               Dinit
```

Office debe ser una capa de **orquestación y política**, no una reimplementación de Linux.

---

# 5. Separación fundamental: Kernel / Dinit / Office

La arquitectura debe mantener esta separación:

```text
Hardware
   │
   ▼
Linux Kernel
   │
   ▼
Artix Linux
   │
   ▼
Dinit
   │
   ▼
Office
   │
   ▼
Agent Executions
```

Cada capa tiene responsabilidades diferentes.

---

# 6. Responsabilidad del kernel

Linux proporciona las primitivas fundamentales:

```text
processes
namespaces
cgroups
mounts
filesystems
permissions
networking
security mechanisms
resource accounting
IPC
events
```

Office no debe intentar sustituirlas.

---

# 7. Responsabilidad de Dinit

Dinit administra **servicios persistentes del host**.

Ejemplo:

```text
Dinit
│
├── networking
├── ssh/tailscale/etc.
├── host infrastructure
└── office
```

Dinit debe:

- arrancar Office;
- detener Office;
- supervisar Office;
- reiniciarlo según política;
- manejar dependencias del servicio;
- preparar/limpiar recursos relacionados con el servicio cuando corresponda;
- utilizar sus mecanismos actuales de lifecycle;
- utilizar integración con cgroups cuando sea útil.

Office NO debe convertirse en otro init system.

Office NO debe reimplementar:

```text
host service supervision
host boot ordering
generic host service dependencies
init
service manager
```

Regla:

> **Dinit supervisa el host. Office supervisa ejecuciones de agentes.**

---

# 8. No crear un servicio Dinit por tarea

Una ejecución de agente NO es un servicio del host.

No quiero:

```text
Dinit
├── office
├── task-0001
├── task-0002
├── task-0003
└── ...
```

Quiero:

```text
Dinit
   │
   └── Office
          │
          ├── execution A
          ├── execution B
          └── execution C
```

Office debe manejar directamente las ejecuciones efímeras.

---

# 9. Responsabilidades de Office

Office será el control plane.

Debe administrar:

```text
Office
│
├── companies
├── projects
├── repositories
├── agent definitions
├── agent identities
├── tasks
├── executions
├── runtimes
├── policies
├── scheduler
├── sandbox builder
├── resource allocation
├── artifact management
├── execution supervision
└── reconciliation
```

Office recibe una intención/tarea y determina:

```text
Task
 ↓
Company
 ↓
Project
 ↓
Agent
 ↓
Required resources
 ↓
Policy
 ↓
Runtime
 ↓
Sandbox
 ↓
Execution
 ↓
Artifacts/result
 ↓
Cleanup
```

---

# 10. Filesystem del host

No quiero crear arbitrariamente:

```text
/workspace
/company
/projects
/agents
```

en `/`.

Quiero respetar FHS y las convenciones normales de Linux cuando tenga sentido.

Propuesta inicial:

```text
/
├── etc/
│   └── office/
│
├── usr/
│   └── bin/
│       └── office
│
├── var/
│   ├── lib/
│   │   └── office/
│   ├── cache/
│   │   └── office/
│   └── log/
│       └── office/
│
├── run/
│   └── office/
│
├── opt/
│   └── office/
│       └── runtimes/
│
├── srv/
│   └── office/
│       └── companies/
│
└── home/
```

---

# 11. Responsabilidad de cada ruta

## `/etc/office`

Configuración global.

Ejemplo:

```text
/etc/office/
├── office.toml
├── policies/
├── sandbox-profiles/
└── runtime-policies/
```

No guardar estado mutable aquí.

---

## `/usr/bin/office`

Binario principal.

Idealmente:

```text
/usr/bin/office
```

Dinit ejecuta este binario.

---

## `/var/lib/office`

Estado persistente interno de Office.

Ejemplo:

```text
/var/lib/office/
├── state.sqlite
├── executions/
├── metadata/
└── internal/
```

---

## `/var/cache/office`

Datos reconstruibles:

```text
model cache
runtime downloads
indexes
repository indexes
temporary derived data
```

Nada crítico debería depender exclusivamente de este directorio.

---

## `/var/log/office`

Logs persistentes cuando decidamos almacenarlos como archivos.

Separar conceptualmente:

```text
host logs
office logs
execution logs
security logs
```

---

## `/run/office`

Estado efímero.

Ejemplos:

```text
/run/office/
├── office.sock
├── executions/
├── locks/
└── temporary-state/
```

Debe poder desaparecer completamente después de reboot.

---

## `/opt/office/runtimes`

Runtimes administrados por Office.

Ejemplo:

```text
/opt/office/runtimes/
├── node/
│   ├── 22/
│   └── 24/
├── python/
├── rust/
├── bun/
└── custom/
```

---

## `/srv/office`

Recursos de las organizaciones.

Ejemplo:

```text
/srv/office/
└── companies/
    ├── acme/
    ├── foo/
    └── bar/
```

---

# 12. Estructura de compañía

Ejemplo:

```text
/srv/office/companies/acme/
├── shared/
│   ├── knowledge/
│   ├── documentation/
│   ├── resources/
│   └── artifacts/
│
└── projects/
    ├── ecommerce/
    ├── mobile/
    └── infrastructure/
```

`shared` representa recursos disponibles para determinados proyectos/agentes de esa compañía.

No implica automáticamente permiso universal.

---

# 13. Estructura de proyecto

Ejemplo:

```text
/srv/office/companies/acme/projects/ecommerce/
├── repositories/
│   ├── backend/
│   ├── frontend/
│   ├── mobile/
│   └── infrastructure/
│
├── knowledge/
├── documentation/
├── artifacts/
└── resources/
```

Un proyecto puede contener cualquier cantidad de repositorios.

---

# 14. `/home`

No asumir automáticamente que cada agente necesita un home persistente.

Investigar tres modelos:

### Modelo A

```text
/home/agent-backend
/home/agent-security
```

Un home persistente por agente.

### Modelo B

Agentes conceptuales sin usuarios/home persistentes.

Las ejecuciones reciben identidades efímeras.

### Modelo C

Modelo híbrido.

Agentes que realmente necesitan estado persistente reciben identidad/home.

Otros utilizan identidades efímeras.

Determinar cuál es mejor.

---

# 15. UID/GID

No asumir que:

```text
1 agent = 1 permanent Linux user
```

es necesariamente la mejor solución.

Evaluar:

```text
persistent UID per agent
ephemeral UID pool
user namespaces
subuid/subgid
idmapped mounts
persistent identity + ephemeral execution identity
```

Quiero evitar llenar `/etc/passwd` con miles de identidades si no es necesario.

Investigar especialmente si un pool de execution identities sería mejor:

```text
office-exec-0001
office-exec-0002
...
```

o si user namespaces eliminan esa necesidad.

---

# 16. Sandbox por ejecución

Cada ejecución obtiene su propia vista del mundo.

Ejemplo conceptual:

```text
/
├── workspace/
│   ├── company/
│   ├── project/
│   ├── repositories/
│   └── artifacts/
│
├── runtime/
├── home/
├── tmp/
├── proc/
└── dev/
```

Pero esta raíz NO corresponde al host.

Es la vista del filesystem de esa ejecución.

---

# 17. Principle of non-existence

No quiero depender exclusivamente de:

> "el archivo existe pero permission denied."

Cuando sea posible quiero:

> **el recurso que el agente no necesita ni siquiera existe dentro de su mount namespace.**

Ejemplo:

El agente trabaja en:

```text
Company A
Project Backend
```

No debería poder descubrir:

```text
Company B
Project Finance
Project HR
```

porque esos árboles ni siquiera deberían estar montados.

Esto reduce:

- accidental access;
- information disclosure;
- policy complexity;
- attack surface.

---

# 18. Mount namespace

Cada ejecución debería tener su propio mount namespace cuando sea necesario.

Construir explícitamente su filesystem.

Ejemplo:

```text
host:

/srv/office/companies/acme/shared
/srv/office/companies/acme/projects/backend
/opt/office/runtimes/node/24
```

Vista del agente:

```text
/workspace/company
/workspace/project
/runtime
```

El agente no necesita conocer las rutas físicas del host.

---

# 19. Linux namespaces

Evaluar individualmente:

```text
CLONE_NEWUSER
CLONE_NEWNS
CLONE_NEWPID
CLONE_NEWNET
CLONE_NEWUTS
CLONE_NEWIPC
CLONE_NEWCGROUP
```

No utilizar namespaces simplemente porque existen.

Definir perfiles.

Ejemplo:

```text
profile = code-analysis

mount       yes
pid         yes
network     none
ipc         yes
uts         optional
user        evaluate
cgroup      yes
```

Otro:

```text
profile = web-agent

mount       yes
pid         yes
network     restricted
ipc         yes
user        yes
cgroup      yes
```

---

# 20. User namespaces

Investigar profundamente user namespaces.

Objetivo:

Permitir que dentro del sandbox:

```text
uid 0
```

NO signifique root real del host.

Ejemplo conceptual:

```text
sandbox uid 0
       ↓
host uid 100123
```

Evaluar:

- security implications;
- kernel attack surface;
- `/etc/subuid`;
- `/etc/subgid`;
- mapping;
- capabilities dentro del namespace;
- interacción con mounts;
- interacción con idmapped mounts.

---

# 21. cgroup v2

Usar cgroup v2 para contener recursos de cada ejecución.

Una posible estructura:

```text
/office
└── executions
    ├── exec-001
    ├── exec-002
    └── exec-003
```

Evitar replicar innecesariamente toda la jerarquía de dominio:

```text
company/project/agent/task/...
```

dentro de cgroups si no aporta nada.

Los cgroups representan principalmente **resource/control domains**, no necesariamente el modelo completo de negocio.

Investigar cuál es la estructura mínima adecuada.

---

# 22. Límites por ejecución

Evaluar:

```text
cpu.max
cpu.weight

memory.max
memory.high
memory.low

pids.max

io.max
io.weight
```

Una ejecución debe contener todos sus descendientes.

Si el agente lanza:

```text
node
├── npm
├── compiler
├── test worker
└── child process
```

todos deben permanecer dentro del mismo resource domain.

---

# 23. Terminar una ejecución

No matar solamente el PID principal.

La ejecución completa debe poder terminarse.

Investigar mecanismos modernos de cgroup v2 como:

```text
cgroup.kill
cgroup.events
```

cuando estén disponibles.

Objetivo:

```text
terminate execution
        ↓
terminate complete process tree
        ↓
confirm empty cgroup
        ↓
cleanup resources
```

---

# 24. PSI

Usar Pressure Stall Information.

No quiero un scheduler que únicamente mire:

```text
CPU = 90%
RAM = 75%
```

porque utilization != contention.

Analizar:

```text
/proc/pressure/cpu
/proc/pressure/memory
/proc/pressure/io
```

y PSI por cgroup cuando corresponda.

El scheduler podría considerar:

```text
CPU pressure
memory pressure
I/O pressure
```

para decidir cuándo lanzar nuevas ejecuciones.

---

# 25. Scheduler adaptativo

Ejemplo conceptual:

```text
incoming task
      │
      ▼
priority
      │
      ▼
runtime requirements
      │
      ▼
available resources
      │
      ▼
PSI
      │
      ▼
admit / queue / reject
```

No quiero polling agresivo.

Investigar PSI triggers/event mechanisms.

---

# 26. OverlayFS

Quiero evaluar OverlayFS para tareas que modifican repositorios.

Ejemplo:

```text
original repository
       │
       ▼
    lowerdir
       +
execution upperdir
       +
workdir
       │
       ▼
     merged
       │
       ▼
     agent
```

El agente modifica `merged`.

El repositorio original permanece intacto.

---

# 27. Lifecycle del overlay

Ejemplo:

```text
create execution
      ↓
create upper/work
      ↓
mount overlay
      ↓
agent modifies files
      ↓
agent finishes
      ↓
inspect diff
      ↓
accept/reject/export
      ↓
unmount
      ↓
delete execution filesystem
```

Analizar:

- whiteouts;
- copy-up;
- rename semantics;
- inode behavior;
- Git behavior;
- build tools;
- nested filesystems;
- performance;
- security.

---

# 28. No usar OverlayFS para todo

OverlayFS debe resolver un problema concreto.

Por ejemplo, para análisis read-only:

```text
bind mount RO
```

puede ser suficiente.

Para generación de artifacts:

```text
dedicated RW directory
```

puede ser suficiente.

Elegir la primitiva mínima.

---

# 29. Idmapped mounts

Investigar idmapped mounts para evitar:

```bash
chown -R
```

sobre repositorios grandes.

Conceptualmente:

```text
host:

UID 1000 owns repository

        ↓ idmapped mount

sandbox:

UID 0/1000/etc sees appropriate ownership
```

sin modificar los metadatos reales.

Investigar interacción con:

- user namespaces;
- OverlayFS;
- filesystem elegido;
- Git;
- build tools;
- Landlock.

---

# 30. Landlock

Utilizar Landlock como capa de restricción adicional cuando sea apropiado.

Modelo:

```text
mount namespace
       +
DAC/ACL
       +
Landlock
       =
effective accessible filesystem
```

Landlock no sustituye automáticamente DAC.

Landlock debe **reducir** lo permitido.

---

# 31. Landlock + OverlayFS

Analizar cuidadosamente esta combinación.

No asumir que:

```text
protect lowerdir
```

equivale automáticamente a proteger:

```text
merged overlay tree
```

Diseñar la política sobre la vista real que utilizará el agente.

Orden esperado:

```text
create namespace
      ↓
construct mounts
      ↓
construct overlay
      ↓
final filesystem view
      ↓
apply Landlock
      ↓
drop privileges
      ↓
execute agent
```

Verificar este orden técnicamente.

---

# 32. Network restrictions

No todos los agentes necesitan Internet.

Perfiles:

```text
offline
restricted-egress
company-services-only
internet
```

Un agente que solo analiza código:

```text
network = none
```

Un investigador:

```text
network = internet
```

Un agente CI:

```text
network = package registries + internal services
```

---

# 33. Network namespaces

Evaluar:

```text
network namespace
veth
routing
nftables
DNS
```

pero no construir una red virtual completa cuando no sea necesaria.

Para tareas offline:

```text
empty network namespace
```

puede ser suficiente.

---

# 34. Landlock network support

Investigar las capacidades actuales de Landlock para restricciones de red.

No asumir que reemplaza completamente:

```text
network namespace
nftables
routing
```

Determinar exactamente qué restricciones puede aplicar el kernel actual.

---

# 35. Seccomp

Utilizar seccomp para reducir superficie de syscalls.

Pero:

> **seccomp no es por sí mismo un sandbox completo.**

Modelo:

```text
namespace isolation
        +
filesystem isolation
        +
Landlock
        +
capability dropping
        +
no_new_privs
        +
seccomp
```

---

# 36. Perfiles seccomp

No crear inicialmente una gigantesca syscall allowlist extremadamente frágil.

Investigar perfiles por runtime:

```text
node
python
rust compiler
git
shell
research-agent
```

Comenzar con restricciones de alto valor y bajo riesgo.

Medir antes de endurecer.

---

# 37. Linux capabilities

Office probablemente necesitará operaciones privilegiadas.

Pero no quiero:

```text
office runs forever with unrestricted root
```

sin analizar alternativas.

Investigar privilege separation.

---

# 38. Privilege-separated architecture

Evaluar:

```text
              Dinit
                │
                ▼
             Office
                │
        ┌───────┴────────┐
        │                │
        ▼                ▼
 control plane       privileged helper
 unprivileged             │
        │                  ├── namespaces
        │                  ├── mounts
        │                  ├── cgroups
        │                  └── identity setup
        │
        ├── API
        ├── scheduler
        ├── metadata
        └── task planning
```

El privileged helper debería:

- tener API mínima;
- validar estrictamente argumentos;
- realizar pocas operaciones;
- no interpretar lógica compleja de negocio.

---

# 39. Capabilities

Investigar:

```text
capability bounding set
permitted set
effective set
inheritable set
ambient capabilities
securebits
```

Office debe conservar únicamente las capacidades realmente necesarias.

---

# 40. `no_new_privs`

Antes de ejecutar código potencialmente no confiable:

```text
PR_SET_NO_NEW_PRIVS
```

cuando sea apropiado.

La ejecución no debería poder recuperar privilegios posteriormente mediante `execve`.

---

# 41. pidfd

Usar mecanismos modernos de process handling.

No quiero depender únicamente de:

```text
PID = 1234
```

porque los PID se reutilizan.

Investigar:

```text
pidfd_open()
pidfd_send_signal()
CLONE_PIDFD
clone3()
```

---

# 42. Process supervision

Modelo deseado:

```text
spawn
  ↓
obtain pidfd
  ↓
register with event loop
  ↓
monitor
  ↓
process exits
  ↓
collect status
  ↓
cleanup
```

Un pidfd debe representar una referencia estable al proceso.

---

# 43. Event-driven Office

Office debería minimizar polling.

Investigar una arquitectura basada en:

```text
epoll
pidfd
timerfd
eventfd
signalfd
Unix sockets
cgroup events
PSI events
inotify/fanotify where justified
```

Conceptualmente:

```text
                epoll
                  │
      ┌───────────┼───────────┐
      │           │           │
    pidfd      sockets     timerfd
      │
 cgroup/PSI events
```

---

# 44. io_uring

Evaluar `io_uring`, pero NO adoptarlo automáticamente.

Preguntar:

> ¿Qué problema concreto de Office resuelve mejor que epoll + async filesystem operations?

Si no existe una respuesta clara, mantenerlo fuera de V1.

---

# 45. Runtime management

Los runtimes son infraestructura de Office.

No pertenecen a las compañías.

Ejemplo:

```text
/opt/office/runtimes/
├── node/
│   ├── 22.20.0/
│   └── 24.x/
├── python/
├── rust/
├── go/
└── custom/
```

---

# 46. Runtime installation

Office podría obtener runtimes mediante APIs/repositorios.

Pipeline:

```text
download
   ↓
verify hash/signature
   ↓
unpack staging
   ↓
validate
   ↓
atomic install
   ↓
mark available
```

Nunca ejecutar directamente un runtime parcialmente descargado.

---

# 47. Runtime immutability

Los runtimes deberían montarse normalmente:

```text
read-only
```

dentro del sandbox.

Evaluar:

```text
fs-verity
```

para contenido que requiera verificación/inmutabilidad.

Analizar si aporta valor suficiente.

---

# 48. Runtime cache

Evitar descargar repetidamente lo mismo.

Mantener:

```text
content-addressed cache
```

cuando sea conveniente.

Ejemplo conceptual:

```text
sha256:<digest>
```

y referencias:

```text
node:24 → digest XYZ
```

---

# 49. Secrets

Los secretos de una compañía NO deberían vivir indiscriminadamente junto al proyecto.

Evitar:

```text
/srv/office/companies/acme/secrets.txt
```

como arquitectura principal.

Un agente debe recibir solamente los secretos necesarios para su ejecución.

---

# 50. Secret injection

Investigar:

```text
pipes
file descriptors
memfd_create
sealed memfd
tmpfs
environment variables
Unix sockets
```

Comparar:

```text
security
lifetime
visibility
/proc exposure
child inheritance
cleanup
```

Preferir mecanismos de corta duración.

---

# 51. `memfd`

Investigar si secretos/configuración efímera pueden entregarse mediante:

```text
memfd_create()
```

y seals.

Ejemplo:

```text
Office
   ↓
create memfd
   ↓
write secret
   ↓
seal
   ↓
pass FD
   ↓
agent
```

Sin necesidad de crear un archivo persistente.

---

# 52. `/proc`

No exponer automáticamente `/proc` del host.

Si usamos PID namespace:

```text
mount proc inside namespace
```

para que el agente vea únicamente su mundo de procesos cuando sea posible.

Analizar seguridad y mount ordering.

---

# 53. `/sys`

No montar todo `/sys` RW.

Preferir:

```text
absent
```

o:

```text
minimal RO view
```

cuando una ejecución realmente lo necesite.

---

# 54. `/dev`

No bind-mount completo `/dev` del host sin analizarlo.

Crear/exponer solamente dispositivos necesarios.

Ejemplo:

```text
/dev/null
/dev/zero
/dev/random
/dev/urandom
```

y pseudo-terminales cuando correspondan.

No exponer dispositivos físicos innecesarios.

---

# 55. IPC

Analizar:

```text
Unix domain sockets
pipes
shared memory
eventfd
```

para comunicación entre Office y ejecuciones.

Preferir Unix sockets locales para APIs internas cuando resulte apropiado.

---

# 56. API de Office

Office puede exponer inicialmente:

```text
/run/office/office.sock
```

como Unix domain socket.

Sobre él puede existir:

```text
HTTP
custom protocol
gRPC
```

pero justificar la elección.

Para clientes remotos puede existir posteriormente un gateway.

No exponer el daemon root directamente a Internet.

---

# 57. Control plane vs execution plane

Separar conceptualmente:

```text
Control Plane

companies
projects
tasks
scheduler
metadata
policies

        │
        ▼

Execution Plane

namespaces
cgroups
mounts
processes
runtimes
pidfd
resource monitoring
```

Esto permitirá posteriormente separar procesos sin rediseñar el dominio.

---

# 58. Persistencia

No quiero una base de datos simplemente porque "todas las aplicaciones necesitan una".

Clasificar estado.

---

# 59. Estado cuyo owner es Linux

No duplicar innecesariamente:

```text
running processes
PID state
mount state
cgroup membership
CPU consumption
memory consumption
filesystem ownership
kernel resource state
```

Linux es la autoridad mientras el sistema está vivo.

---

# 60. Estado cuyo owner es Office

Office sí necesita representar:

```text
companies
projects
repositories
agent definitions
tasks
executions
relationships
policies
timestamps
results
artifact metadata
runtime metadata
audit references
```

Este es estado de dominio.

---

# 61. SQLite

Para una máquina única, evaluar:

```text
/var/lib/office/state.sqlite
```

No introducir PostgreSQL prematuramente.

SQLite puede proporcionar:

```text
transactions
indexes
constraints
WAL
crash recovery
queries
```

con muy poca infraestructura.

Pero analizar si realmente es necesaria para cada tipo de metadata.

---

# 62. No usar filesystem como una SQL improvisada

No quiero terminar haciendo:

```text
find
grep
directory names
JSON everywhere
```

para representar relaciones complejas solo por evitar una base de datos.

Regla:

> **Linux administra recursos; Office administra dominio.**

No confundir ambas responsabilidades.

---

# 63. Desired state vs observed state

Office debería distinguir:

```text
desired state
```

de:

```text
observed Linux state
```

Ejemplo:

```text
SQLite:

execution 123 = running

Linux:

no process
no cgroup
```

Office debe detectar la inconsistencia.

---

# 64. Reconciliation loop

Después de arrancar:

```text
Office starts
    ↓
load persistent metadata
    ↓
inspect Linux state
    ↓
compare
    ↓
reconcile
```

Inspeccionar:

```text
cgroups
processes
mounts
execution directories
runtime state
artifacts
locks
```

---

# 65. Crash recovery

Diseñar todas las operaciones importantes para ser:

```text
idempotent
```

Ejemplo:

```text
cleanupExecution(id)
```

debe funcionar si:

- el proceso ya murió;
- el cgroup ya está vacío;
- el mount ya fue desmontado;
- el directorio ya desapareció parcialmente.

---

# 66. Execution state machine

Definir explícitamente estados.

Ejemplo inicial:

```text
CREATED
   ↓
PREPARING
   ↓
READY
   ↓
RUNNING
   ↓
FINISHING
   ↓
SUCCEEDED

o

FAILED
CANCELLED
ORPHANED
```

Pero analizar una máquina de estados robusta.

No depender de booleans ambiguos como:

```text
is_running
is_finished
has_failed
```

---

# 67. Execution ID

Cada ejecución debe tener identificador único.

Ejemplo:

```text
exec_01...
```

Este ID conecta:

```text
database
cgroup
logs
artifacts
runtime dirs
metrics
audit events
```

No depender exclusivamente del PID.

---

# 68. Execution filesystem

Ejemplo:

```text
/run/office/executions/<id>/
```

para estado completamente efímero.

Y cuando se necesite persistencia temporal:

```text
/var/lib/office/executions/<id>/
```

Separar claramente ambos.

---

# 69. Lifecycle completo

Diseñar este flujo:

```text
receive task
    ↓
validate
    ↓
resolve company
    ↓
resolve project
    ↓
select agent
    ↓
select runtime
    ↓
resolve policy
    ↓
admission control
    ↓
allocate execution ID
    ↓
create cgroup
    ↓
prepare execution directories
    ↓
create namespaces
    ↓
construct filesystem
    ↓
create overlays if needed
    ↓
apply id mappings
    ↓
configure network
    ↓
inject required secrets
    ↓
set resource limits
    ↓
drop capabilities
    ↓
apply no_new_privs
    ↓
apply Landlock
    ↓
apply seccomp
    ↓
spawn
    ↓
obtain pidfd
    ↓
monitor
    ↓
collect stdout/stderr
    ↓
collect metrics
    ↓
process exits
    ↓
collect artifacts
    ↓
validate result
    ↓
kill remaining descendants
    ↓
confirm cgroup empty
    ↓
unmount
    ↓
destroy namespace resources
    ↓
destroy cgroup
    ↓
remove secrets
    ↓
persist final state
    ↓
cleanup
```

Validar técnicamente el orden.

Especialmente:

```text
namespace creation
mount creation
Landlock
capability dropping
seccomp
exec
```

El orden correcto importa.

---

# 70. Logs

Distinguir:

```text
Office logs
Execution logs
Agent stdout
Agent stderr
Security events
Audit events
Metrics
```

No mezclarlos indiscriminadamente.

---

# 71. stdout/stderr

Office debería capturar stdout/stderr de ejecuciones mediante pipes u otro mecanismo apropiado.

Asociarlos siempre con:

```text
execution_id
timestamp
stream
```

---

# 72. Observabilidad

V1 debe utilizar mecanismos simples.

Ejemplo:

```text
/proc
cgroup files
PSI
pidfd
structured application events
```

No comenzar obligatoriamente con eBPF.

---

# 73. eBPF

Considerarlo posteriormente para:

```text
tracing
security observability
network visibility
performance analysis
audit
```

Pero únicamente cuando resuelva problemas reales.

---

# 74. BPF LSM

Investigar BPF LSM como mecanismo avanzado.

No hacerlo dependencia fundamental de V1.

Modelo:

```text
V1
DAC
namespaces
cgroups
Landlock
capabilities
seccomp

V2/V3
optional BPF/eBPF policy/observability
```

---

# 75. fanotify / inotify

No observar todo el filesystem indiscriminadamente.

Evaluar:

```text
inotify
fanotify
```

solo cuando exista una necesidad concreta.

Para detectar modificaciones de una tarea, OverlayFS puede proporcionar una representación más natural que observar millones de eventos.

---

# 76. Artifacts

Una ejecución puede producir:

```text
patch
source files
build
report
JSON
binary
logs
screenshots
etc.
```

Artifacts deben ser explícitos.

No considerar automáticamente todo `upperdir` como resultado válido.

---

# 77. Commit model

Diseñar cómo una ejecución modifica un proyecto.

Opciones:

```text
A. direct write
B. overlay → diff
C. overlay → patch
D. overlay → atomic promotion
E. Git worktree
F. dedicated clone
```

Comparar especialmente:

```text
OverlayFS
vs
Git worktree
```

Para repositorios Git, un worktree puede ser más simple y semánticamente correcto que OverlayFS en determinados casos.

No imponer OverlayFS si Git ya resuelve mejor el problema.

---

# 78. Atomic promotion

Los cambios producidos por un agente no deberían convertirse automáticamente en estado oficial.

Modelo:

```text
execution
   ↓
candidate changes
   ↓
validation
   ↓
review/policy
   ↓
promotion
```

Especialmente importante cuando varios agentes trabajan simultáneamente.

---

# 79. Concurrencia

Diseñar para:

```text
agent A → repo X
agent B → repo X
agent C → repo Y
```

simultáneamente.

Evitar locks globales.

Determinar cuándo utilizar:

```text
filesystem locks
SQLite transactions
Git branches/worktrees
optimistic concurrency
```

---

# 80. File locking

No asumir que `flock` resuelve todas las condiciones de carrera.

Analizar correctamente:

```text
flock
fcntl locks
O_EXCL
rename
link
atomic filesystem operations
```

Elegir según la operación.

---

# 81. Atomic filesystem operations

Aprovechar operaciones atómicas Linux cuando sea posible.

Ejemplo conceptual:

```text
write temp
fsync
rename
fsync directory
```

para configuraciones/metadata críticas almacenadas como archivos.

---

# 82. `openat2()`

Investigar `openat2()` seriamente.

Especialmente:

```text
RESOLVE_BENEATH
RESOLVE_IN_ROOT
RESOLVE_NO_SYMLINKS
RESOLVE_NO_MAGICLINKS
RESOLVE_NO_XDEV
```

Office manejará paths controlados potencialmente por agentes.

No utilizar concatenación ingenua:

```text
base + userPath
```

para operaciones privilegiadas.

---

# 83. Path traversal

El privileged component debe defenderse contra:

```text
../
symlink attacks
mount traversal
magic links
/proc tricks
TOCTOU
```

Preferir APIs basadas en file descriptors y `*at()`.

---

# 84. File-descriptor-oriented design

Cuando sea posible:

```text
resolve path once
     ↓
obtain FD
     ↓
operate through FD
```

en lugar de resolver repetidamente strings de paths.

Investigar:

```text
openat2
open_tree
move_mount
mount_setattr
```

y APIs modernas de mount.

---

# 85. New mount API

Investigar las APIs modernas de mounts de Linux:

```text
fsopen()
fsconfig()
fsmount()
open_tree()
move_mount()
mount_setattr()
```

Determinar si son mejores para Office que depender únicamente de `mount(2)`.

Especialmente para:

- detached mount trees;
- atomic construction;
- idmapped mounts;
- mount attributes;
- safer sandbox setup.

---

# 86. `mount_setattr()`

Investigar:

```text
MOUNT_ATTR_RDONLY
MOUNT_ATTR_NOSUID
MOUNT_ATTR_NODEV
MOUNT_ATTR_NOEXEC
MOUNT_ATTR_IDMAP
```

Esto puede permitir construir vistas de filesystem mucho más precisas.

Ejemplo:

```text
company knowledge
    RO + NOSUID + NODEV

runtime
    RO + NOSUID + NODEV

artifacts
    RW + NOSUID + NODEV
```

`NOEXEC` debe decidirse según el propósito.

---

# 87. Mount hardening

Aplicar cuando corresponda:

```text
nosuid
nodev
noexec
readonly
```

por mount.

No depender exclusivamente de permisos Unix.

---

# 88. `close_range()`

Investigar `close_range()` antes de ejecutar agentes.

El proceso hijo no debería heredar accidentalmente file descriptors sensibles del daemon.

Pipeline:

```text
fork/clone
    ↓
retain explicit FDs
    ↓
close_range
    ↓
security setup
    ↓
exec
```

---

# 89. FD inheritance

Todos los FDs internos deberían usar:

```text
O_CLOEXEC
SOCK_CLOEXEC
pipe2(O_CLOEXEC)
```

por defecto.

Solo pasar explícitamente los FDs que el agente necesita.

---

# 90. Process creation

Investigar cuál debería ser la primitiva central:

```text
clone3()
```

vs combinaciones tradicionales de:

```text
fork
unshare
setns
clone
```

Office necesita control preciso de:

```text
namespaces
pidfd
UID mappings
process lifecycle
```

Elegir la solución menos propensa a races.

---

# 91. OOM

Definir qué ocurre cuando una ejecución excede memoria.

No permitir que un agente comprometa todo el host.

Utilizar cgroup memory controls.

Registrar claramente:

```text
normal exit
timeout
cancelled
OOM killed
security violation
runtime failure
```

como causas distintas.

---

# 92. Timeouts

Una ejecución debe poder tener:

```text
wall-clock timeout
CPU budget
idle timeout
```

si corresponde.

No implementar todos necesariamente en V1.

---

# 93. Scheduler fairness

Si existen múltiples compañías:

```text
Company A → 100 tasks
Company B → 2 tasks
```

A no debería necesariamente monopolizar toda la máquina.

Investigar:

```text
weighted scheduling
per-company quotas
priority
resource reservations
```

sin convertir Office en Kubernetes.

---

# 94. Security boundaries

Definir explícitamente contra qué amenazas protegemos.

Ejemplos:

```text
buggy agent
malicious task input
malicious repository
malicious dependency
compromised runtime
agent escaping filesystem
resource exhaustion
secret theft
cross-company access
```

No afirmar "sandbox seguro" sin threat model.

---

# 95. Trust levels

Podría haber distintos perfiles:

```text
trusted
semi-trusted
untrusted
```

Ejemplo:

```text
trusted internal compiler
    ↓
menos aislamiento

unknown repository agent
    ↓
máximo aislamiento
```

Evitar pagar el máximo coste de aislamiento cuando no aporta valor.

---

# 96. Artix Linux

La máquina base debe mantenerse pequeña.

Objetivo:

```text
Artix minimal
+
Dinit
+
Linux kernel
+
Office
+
networking mínimo
+
administration essentials
```

Nada más por defecto.

---

# 97. Dinit

Utilizar la versión actual de Dinit disponible/adecuada para Artix.

Antes de implementar integración específica:

- revisar documentación actual;
- revisar changelog/releases;
- revisar soporte actual de cgroups;
- revisar service dependencies;
- revisar `prepared-by`;
- revisar service includes/templates;
- revisar `dinitctl`;
- revisar `dinit-check`;
- revisar shutdown/restart semantics.

No basarse en documentación histórica.

---

# 98. Dinit boundary

Mantener el acoplamiento pequeño.

Office debería poder conceptualmente correr bajo otro supervisor sin reescribir su arquitectura.

Dinit proporciona:

```text
host lifecycle
```

No el modelo de dominio.

---

# 99. Dinit service conceptual

Algo parecido a:

```text
type = process
command = /usr/bin/office
restart = true
```

más los parámetros reales apropiados.

NO copiar este ejemplo literalmente sin verificar la sintaxis y recomendaciones actuales de Dinit.

Generar posteriormente la configuración correcta usando documentación actual.

---

# 100. ¿Root daemon?

No aceptar automáticamente:

```text
Office = root forever
```

como arquitectura final.

V1 puede comenzar privilegiada para aprender.

Pero investigar desde el principio:

```text
privileged launcher/helper
+
unprivileged control plane
```

como diseño objetivo.

---

# 101. Office como appliance

NO crear todavía una distribución propia.

Primera etapa:

```text
Artix
+
Office package/configuration
```

Segunda etapa:

```text
reproducible Artix image
```

Solo después considerar:

```text
Office OS
```

si existen razones concretas.

---

# 102. ¿Cuándo justificar una distro propia?

Solo si necesitamos controlar consistentemente:

```text
kernel config
boot
packages
filesystem layout
updates
security policy
rollback
machine provisioning
```

y Artix estándar comienza a ser una limitación.

No hacerlo por branding.

---

# 103. Kernel

Investigar qué kernel/configuración necesita Office.

Necesitamos verificar soporte para:

```text
cgroup v2
namespaces
user namespaces
seccomp
Landlock
OverlayFS
idmapped mounts
PSI
pidfd
new mount API
fs-verity if used
BPF if later used
```

No asumir que todo está habilitado en cualquier kernel.

---

# 104. Kernel hardening

Investigar opciones razonables de hardening sin destruir compatibilidad.

Pero separar:

```text
host hardening
```

de:

```text
agent sandboxing
```

Son problemas relacionados pero diferentes.

---

# 105. Filesystem del host

Evaluar:

```text
ext4
Btrfs
XFS
```

para Office.

Criterios:

```text
OverlayFS behavior
reflinks
snapshots
idmapped mounts
fs-verity
performance
recovery
operational simplicity
```

No elegir Btrfs únicamente porque tenga snapshots.

---

# 106. Btrfs opcional

Si Btrfs ofrece ventajas reales, considerar:

```text
snapshots
subvolumes
reflinks
quotas
rollback
```

para:

```text
runtime storage
company data
project snapshots
```

Pero no mezclar automáticamente Btrfs snapshots con el lifecycle de cada ejecución.

---

# 107. Git-aware execution

Office trabajará frecuentemente con software.

Por eso debe ser consciente de que Git ya ofrece:

```text
branches
worktrees
diffs
commits
merges
```

Para repositorios Git, evaluar:

```text
git worktree per execution
```

como alternativa de primera clase.

Ejemplo:

```text
project repository
       │
       ├── main worktree
       ├── execution A worktree
       └── execution B worktree
```

Esto puede simplificar muchísimo concurrencia y promoción de cambios.

---

# 108. Knowledge != repository

El contexto de un proyecto no debe depender de que el agente lea todos los repositorios completos cada vez.

Office debería posteriormente incorporar un sistema de conocimiento/indexado.

Pero mantenerlo separado del sandbox/runtime.

Conceptualmente:

```text
Project
├── raw resources
├── repositories
└── derived knowledge/index
```

El índice es reconstruible.

Los repositorios son fuente primaria.

---

# 109. No cargar todo el proyecto

Un agente debe recibir únicamente contexto relevante.

Office podría resolver:

```text
task
 ↓
project
 ↓
relevant resources
 ↓
minimal execution context
```

Esto reduce:

```text
tokens
I/O
latency
model confusion
cost
```

El sandbox y el context retrieval son problemas diferentes.

---

# 110. Arquitectura global objetivo

```text
┌─────────────────────────────────────────────┐
│                 HARDWARE                    │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                LINUX KERNEL                 │
│                                             │
│ processes       namespaces       pidfd      │
│ cgroup v2       VFS              PSI        │
│ Landlock        seccomp          network    │
│ capabilities    mount API        epoll      │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                ARTIX LINUX                  │
│                                             │
│ Minimal host                                │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                    DINIT                    │
│                                             │
│ Host service lifecycle                      │
│                                             │
│ networking                                  │
│ remote administration                       │
│ Office supervision                          │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│                   OFFICE                    │
│                                             │
│ Control Plane                               │
│ ├── companies                               │
│ ├── projects                                │
│ ├── agents                                  │
│ ├── tasks                                   │
│ ├── scheduler                               │
│ ├── policy engine                           │
│ └── metadata                                │
│                                             │
│ Execution Plane                             │
│ ├── sandbox builder                         │
│ ├── runtime manager                         │
│ ├── cgroup manager                          │
│ ├── namespace manager                       │
│ ├── mount manager                           │
│ ├── security policy                         │
│ └── process supervisor                      │
└──────────────────────┬──────────────────────┘
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
      Execution     Execution    Execution
          A             B            C
          │             │            │
       Agent          Agent        Agent
```

---

# 111. Mental model

La mejor analogía es:

```text
Linux machine
    =
Office building

Dinit
    =
building infrastructure manager

Office daemon
    =
company/workforce operating system

Company
    =
tenant/organization

Project
    =
work environment

Agent
    =
specialist/worker

Task
    =
assignment

Execution
    =
worker entering a temporary secure room

Sandbox
    =
that room

Runtime
    =
tools available inside the room

cgroup
    =
resource budget

namespace
    =
what the worker can see

Landlock
    =
what the worker may touch

seccomp
    =
which kernel operations are available

pidfd
    =
stable handle to the worker process
```

Pero la analogía nunca debe sustituir la semántica técnica real.

---

# 112. Principios no negociables

1. **Linux is the runtime.**

2. **Office is policy + orchestration.**

3. **Dinit manages persistent host services.**

4. **Office manages ephemeral agent executions.**

5. **Agent identity != execution != process.**

6. **Company != Linux user.**

7. **Project != repository.**

8. **No duplicar estado que Linux ya representa.**

9. **No usar filesystem como sustituto artificial de una base de datos relacional.**

10. **Least privilege by default.**

11. **Un recurso innecesario idealmente ni siquiera debe aparecer dentro del sandbox.**

12. **El kernel debe aplicar las restricciones críticas.**

13. **Preferir event-driven sobre polling.**

14. **Cada ejecución debe ser destruible como unidad.**

15. **Crash recovery debe ser idempotente.**

16. **Paths controlados por agentes son hostiles.**

17. **Preferir APIs basadas en FDs cuando mejoren seguridad.**

18. **No adoptar eBPF/io_uring/etc. solo porque sean modernos.**

19. **No crear nuestra propia distro antes de necesitarla.**

20. **No recrear Docker accidentalmente sin reconocer el coste.**

21. **No recrear systemd dentro de Office.**

22. **No recrear Git cuando Git ya resuelve el problema mejor.**

23. **Cada abstracción debe justificar su existencia.**

---

# 113. Investigación obligatoria antes de implementar

Quiero que investigues usando documentación ACTUAL.

Prioridad:

1. documentación oficial del kernel Linux;
2. documentación oficial de Artix;
3. documentación oficial de Dinit;
4. man-pages actuales;
5. código fuente upstream cuando sea necesario;
6. proyectos maduros que utilicen estas primitivas.

No asumir comportamiento por memoria.

Especialmente verificar versiones actuales y cambios recientes en:

```text
Linux namespaces
cgroup v2
Landlock
seccomp
pidfd
clone3
openat2
idmapped mounts
new mount API
OverlayFS
PSI
fs-verity
Dinit
Artix
```

---

# 114. Investigar qué estamos reinventando

Comparar Office conceptualmente con proyectos existentes como:

```text
runc
crun
bubblewrap
nsjail
systemd-nspawn
firejail
LXC
containerd
Docker
Podman
Kubernetes
```

NO para adoptarlos necesariamente.

Quiero descubrir:

- qué problemas difíciles ya encontraron;
- qué races tuvieron que resolver;
- qué ataques tuvieron que prevenir;
- qué partes parecen simples pero no lo son;
- qué primitivas utilizan;
- qué podemos reutilizar como conocimiento.

Office debe seguir siendo mucho más pequeño si sus requisitos son menores.

---

# 115. Pregunta crítica

Antes de implementar cualquier subsystem preguntar:

> ¿Esto pertenece realmente a Office, o Linux/Dinit/Git/SQLite ya lo resuelve?

Si la respuesta es que ya existe una solución apropiada:

**no reinventarla.**

---

# 116. V1 deseada

Diseñar una V1 extremadamente pequeña.

Objetivo aproximado:

```text
Artix + Dinit

Office
├── Unix socket API
├── SQLite metadata
├── execution IDs
├── basic task model
├── cgroup v2 per execution
├── mount namespace
├── PID namespace where useful
├── minimal filesystem
├── RO/RW bind mounts
├── resource limits
├── capability dropping
├── no_new_privs
├── basic Landlock
├── conservative seccomp
├── pidfd supervision
├── stdout/stderr capture
└── deterministic cleanup
```

Evitar inicialmente:

```text
eBPF
BPF LSM
complex virtual networking
distributed scheduling
multi-node clustering
custom filesystem
custom database
custom init
custom distro
complex container image format
```

---

# 117. V2 posible

Después de demostrar V1:

```text
user namespaces
idmapped mounts
OverlayFS/worktree strategies
advanced networking policies
secret injection
runtime manager
PSI-aware scheduler
privilege-separated helper
advanced recovery
content-addressed cache
fs-verity
```

---

# 118. V3 posible

Solo cuando exista necesidad real:

```text
eBPF observability
BPF LSM
multi-host Office
remote execution
distributed scheduler
advanced artifact store
custom Artix appliance image
strong reproducibility
hardware isolation
```

---

# 119. Lo que debes producir

Después de investigar todo lo anterior, quiero una propuesta técnica concreta.

NO quiero una respuesta genérica.

Produce:

## A. Architecture Decision

Explica la arquitectura recomendada.

## B. Threat Model

Define qué amenazas cubrimos.

## C. Linux Primitive Matrix

Para cada necesidad:

```text
problem
Linux primitive
reason
complexity
security implications
V1/V2/V3
```

## D. Filesystem Layout

Proporciona el árbol exacto recomendado.

## E. Process Architecture

Especifica qué procesos persistentes existen.

## F. Privilege Architecture

Explica qué corre como root y por qué.

## G. Execution Lifecycle

Detalla cada syscall/primitiva importante utilizada.

## H. Namespace Design

Qué namespaces se utilizan y cuándo.

## I. Cgroup Design

Jerarquía y límites.

## J. Mount Design

Cómo construir el filesystem del agente.

## K. UID/GID Design

Persistente vs efímero vs user namespaces.

## L. Security Stack

Orden exacto de:

```text
mounts
namespaces
UID mapping
capabilities
no_new_privs
Landlock
seccomp
exec
```

## M. Networking

Offline/restricted/full.

## N. Secrets

Cómo entregar credenciales.

## O. Runtime Management

Instalación, cache, verificación y mounting.

## P. Process Supervision

pidfd + epoll + cgroups.

## Q. Crash Recovery

Reconciliation y cleanup.

## R. Persistence

Qué vive en SQLite y qué vive en Linux.

## S. Git Integration

Worktree vs OverlayFS.

## T. Artix Configuration

Paquetes mínimos necesarios.

## U. Dinit Configuration

Configuración actual y correcta para Office.

## V. Kernel Requirements

Opciones/config necesarias.

## W. V1 Implementation Plan

Orden exacto de implementación.

---

# 120. Implementación

La implementación principal debe ser en:

```text
Rust
```

Preferir acceso directo y explícito a las APIs Linux.

Evaluar crates únicamente después de entender qué syscall abstraen.

No ocultar mecanismos críticos detrás de abstracciones gigantes sin necesidad.

Cuando corresponda, considerar:

```text
libc
rustix
nix
```

pero investigar sus versiones y capacidades actuales antes de elegir.

---

# 121. Filosofía de implementación

Quiero poder mirar el código de Office y entender:

```text
this creates the namespace
this creates the cgroup
this mounts the workspace
this restricts filesystem access
this drops capabilities
this applies seccomp
this spawns the process
this pidfd represents it
this epoll loop observes it
this destroys the execution
```

No quiero una pila de diez frameworks que esconda Linux.

---

# 122. Métrica de éxito

Office NO será exitoso por tener muchas features.

Será exitoso si una tarea puede pasar de:

```text
Task
```

a:

```text
isolated Linux execution
```

y regresar:

```text
Result + Artifacts
```

de manera:

```text
simple
secure
observable
recoverable
resource-bounded
reproducible
understandable
```

---

# 123. Flujo objetivo final

El camino ideal debe poder explicarse así:

```text
Artix boots
    ↓
Dinit starts
    ↓
Dinit starts Office
    ↓
Office reconciles Linux state
    ↓
Office opens /run/office/office.sock
    ↓
task arrives
    ↓
resolve company
    ↓
resolve project
    ↓
resolve agent
    ↓
resolve context
    ↓
resolve runtime
    ↓
resolve sandbox profile
    ↓
admission control
    ↓
allocate execution ID
    ↓
create cgroup
    ↓
create execution filesystem
    ↓
create namespaces
    ↓
construct mount tree
    ↓
bind required resources
    ↓
create worktree/overlay if required
    ↓
apply mount attributes
    ↓
configure UID/GID mapping
    ↓
configure networking
    ↓
inject secrets
    ↓
configure resource limits
    ↓
spawn execution
    ↓
obtain pidfd
    ↓
close unnecessary FDs
    ↓
drop capabilities
    ↓
no_new_privs
    ↓
Landlock
    ↓
seccomp
    ↓
exec runtime
    ↓
monitor via event loop
    ↓
collect output
    ↓
collect resource information
    ↓
process exits
    ↓
collect candidate artifacts
    ↓
terminate remaining descendants
    ↓
verify cgroup empty
    ↓
promote approved artifacts
    ↓
unmount execution filesystem
    ↓
destroy temporary resources
    ↓
remove cgroup
    ↓
persist final execution state
    ↓
return result
```

Debes verificar y corregir técnicamente este orden.

No asumir que este orden es perfecto simplemente porque aparece en este documento.

---

# 124. Resultado conceptual

Quiero terminar con algo que pueda describirse como:

> **Office is a native Linux execution and orchestration environment for AI workers.**

No es:

```text
Docker wrapper
Kubernetes clone
systemd replacement
VM manager
LLM framework
```

Office utiliza Linux como su execution substrate.

Los modelos de IA son solamente una de las cosas que un agente puede ejecutar.

---

# 125. Pregunta final de investigación

Después de estudiar todo:

> Si tuvieras una máquina Artix Linux dedicada exclusivamente a ejecutar Office, ¿cuál sería la arquitectura más pequeña, segura y elegante que podrías construir aprovechando al máximo el Linux moderno disponible actualmente?

No conserves una decisión de este documento únicamente porque yo la propuse.

Si existe una solución mejor:

**cámbiala y explica por qué.**

Si alguna de estas primitivas es innecesaria:

**elimínala.**

Si Linux moderno ofrece una primitiva mejor:

**úsala.**

Si Office está intentando resolver un problema que Dinit, Git, SQLite o el kernel ya resuelven:

**simplifica Office.**

El objetivo no es implementar todas las ideas de este documento.

El objetivo es descubrir la arquitectura mínima correcta.

---

# 126. Regla final

Priorizar siempre:

```text
correctness
    ↓
security
    ↓
simplicity
    ↓
observability
    ↓
performance
    ↓
features
```

Y mantener como principio central:

> **Don't build a virtual operating system inside Office. Let Linux be Linux. Office should orchestrate it.**