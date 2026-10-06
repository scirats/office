# 03 — Agente: identidad, recursos y credenciales

Estado: aceptado

Complementa `02-agente-instancia-sesion.md`. Fija dónde vive la definición, quién
es la identidad de ejecución y cómo se accede a recursos y credenciales. Coherente
con `../PROMPT.md` §12–§15, §17, §49–§51 y §58–§62.

## Contexto

`docs/02` fijó `Definition → Instance → Session → Execution`. Faltaba:
Definition global o por workspace, identidad de ejecución, y el modelo de
permisos/credenciales por nivel.

## Decisión

### Definiciones globales

- **Agent Definition + versiones:** globales a la instalación →
  `/var/lib/office/state.sqlite` (metadatos + versiones con hash).
- No viven en ningún home ni workspace.

### Niveles de recursos (aditivos)

```text
N0  Office      config global + Agent Definitions
N1  Workspace   usuario + home + shared  → recursos generales + credenciales base
N2  Project     repos/knowledge/artifacts → recursos puntuales + credenciales del proyecto
```

- Acceso = `N1 ∪ N2`: el proyecto **suma**, nunca revoca.
- El **workspace es la capa base** donde se crean los proyectos (equivale a la
  "compañía" del prompt).
- **No se trabaja a nivel workspace.** Un agente solo trabaja en **proyectos**.

### "En la banca"

Un agente sin proyecto enlazado **no tiene instancia** y no puede trabajar; solo
existe su definición. Al enlazarlo a un proyecto nace la instancia. Analogía: al
entrar a la empresa se recibe el fotocheck y accesos base, pero sin proyecto no
hay responsabilidades ni trabajo.

### Instancias project-scoped

- `Agent Instance = (definition, project)`.
- `project_id NOT NULL`, `UNIQUE(agent_definition_id, project_id)`.
- No existen instancias workspace-scoped.

### Identidad de ejecución

- Todas las ejecuciones de un workspace corren como el **usuario del workspace**
  (UID/GID). No se crea un usuario Linux por agente (`PROMPT.md` §15).
- Los agentes de un mismo proyecto comparten el usuario del workspace y sus
  credenciales (base + proyecto).

### Credenciales

- Store de secretos de Office, con claves por nivel:

  ```text
  workspace/<ws>/…          # credenciales base (fotocheck)
  project/<ws>/<proj>/…     # credenciales del proyecto
  ```

- La ejecución recibe la **unión** de ambos niveles.
- Se inyectan por ejecución (FD/memfd, §50–§51). **Nunca** en el home ni en la DB
  en claro (§49).

### Aislamiento

Aunque compartan UID y credenciales, el aislamiento entre proyectos lo da el
**mount namespace**: se monta el árbol general del workspace **+** el proyecto
puntual; los demás proyectos no existen dentro del namespace (§17). DAC da acceso
al workspace; el mount tree recorta *qué* de ese workspace se ve.

## Consecuencias

- **En la banca:** agente sin proyecto = sin instancia = sin ejecuciones.
- **Toda `Session` es project-scoped.**
- **Concurrencia:** N instancias por proyecto (una por agente distinto),
  serializadas dentro de cada instancia (`docs/02`).
- **Permisos:** el usuario del workspace habilita el acceso general; el proyecto
  agrega recursos y credenciales; el namespace decide la visibilidad final.

## Alternativas descartadas

- **Instancias workspace-scoped:** un agente solo trabaja en proyectos.
- **Usuario/credenciales por agente:** evita llenar `/etc/passwd` y multiplicar
  secretos (§15).
- **Credenciales dentro del proyecto o del home:** riesgo de filtración (§49).

## Pendiente

1. Modelo del secret store: formato, cifrado, rotación, API.
2. Cómo se otorgan/revocan credenciales base y de proyecto.
3. UID/GID concreto y evaluación de user namespaces (V2, §20).
