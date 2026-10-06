# Office — Documentación de diseño

Este directorio captura las decisiones que vamos tomando al revisar `../PROMPT.md`.

## Método de trabajo

1. Se propone un tema y se conversa.
2. Lo acordado se escribe aquí como decisión, con su justificación.
3. Cada decisión puede corregirse; ninguna es definitiva mientras diga `Estado: abierto`.
4. Las correcciones al `PROMPT.md` se reflejan primero aquí y luego (si aplica) en el prompt.

## Convenciones

- Cada documento trata un tema acotado.
- Toda decisión incluye: contexto, decisión, razones, alternativas descartadas, estado.
- Estados: `abierto`, `propuesto`, `aceptado`, `revisar`.

## Índice

| Doc | Tema | Estado |
|-----|------|--------|
| [00-revision.md](00-revision.md) | Revisión sección por sección del PROMPT.md | abierto |
| [01-daemon-v1.md](01-daemon-v1.md) | Daemon V1: protocolo CloudEvents/WebSocket y engranajes | propuesto |
| [02-agente-instancia-sesion.md](02-agente-instancia-sesion.md) | Modelo de agente: Definition, Instance y Session | aceptado |
| [03-agente-identidad-recursos.md](03-agente-identidad-recursos.md) | Identidad de ejecución, niveles de recursos y credenciales | aceptado |
| [04-agentes-v1.md](04-agentes-v1.md) | Contrato del engranaje `agent` (definiciones e instancias) | aceptado |
| [05-tools-v1.md](05-tools-v1.md) | Contrato del engranaje `tool` (catálogo definido en código) | aceptado |

Próximos documentos (a crear cuando los abordemos): lifecycle padre/hijo, UID/GID, threat model, build-vs-reuse.

## Correcciones principales pendientes

1. Separar el lifecycle en fase padre y fase hijo (§69/§123).
2. Unificar el orden de seguridad: `no_new_privs -> Landlock -> seccomp -> drop caps` (§31/§69/§123).
3. Fijar el modelo UID/GID mínimo para V1 (§15).
4. PID 1 y reaping de zombies; montar `/proc` tras crear el PID namespace (§42/§52).
5. Decisión build-vs-reuse del sandbox (§114).
6. Recortar la V1 (§116).
