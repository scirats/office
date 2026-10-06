# 00 — Revisión del PROMPT.md

Estado: abierto

Revisión sección por sección del brief original (`../PROMPT.md`). Etiquetas:
`OK` (sin cambios), `AJUSTAR` (matiz/mejora), `DIFERIR` (correcto pero no en V1),
`FIX` (problema a corregir).

## Bloque 1 — Visión y modelo (§0–§9)

- §0 `OK` Tesis sólida. Fijar versiones exactas (Artix, dinit, kernel) al decidir.
- §1 `AJUSTAR` "Office" se usa para instalación, daemon y espacio conceptual. Glosario: Instance / Company / Project / Repository.
- §2 `OK` `shared` sin permiso universal. Acceso vía grants explícitos, no herencia.
- §3 `AJUSTAR` Falta versionar la Agent Definition; cada execution fija hash/versión.
- §4 `OK` La matriz de primitivas debe ser un artefacto vivo con columna "kernel mínimo".
- §5 `OK`
- §6 `OK`
- §7 `OK` Regla clave: "Dinit supervisa host / Office ejecuta agentes".
- §8 `OK`
- §9 `AJUSTAR` Separar el pipeline en control-plane y execution-plane (coherente con §57).

## Bloque 2 — Filesystem y UID/GID (§10–§15)

- §10 `AJUSTAR` Dueño/grupo `office`, `/run/office` 0750, socket 0660, dirs setgid.
- §11 `OK` SQLite WAL crea `-wal`/`-shm`.
- §12 `OK`
- §13 `OK`
- §14 `DIFERIR` Target Modelo C; V1 = Modelo B (HOME tmpfs efímero). Persistente opt-in.
- §15 `FIX` V1: un único UID no privilegiado. V2: userns + rango subuid amplio de un usuario de servicio.

## Bloque 3 — Sandbox, namespaces, cgroups, PSI (§16–§24)

- §16 `OK` El árbol es la vista del sandbox, no `/`.
- §17 `OK` "Principle of non-existence": mejor aporte conceptual del documento.
- §18 `AJUSTAR` Documentar el layout concreto que ve el agente.
- §19 `AJUSTAR` Definir esquema de profiles (TOML + validación) y ubicación.
- §20 `DIFERIR` userns correcto pero amplía superficie del kernel. V2.
- §21 `OK` `/office/executions/<id>` es suficiente.
- §22 `AJUSTAR` Añadir `memory.oom.group=1`.
- §23 `OK` `cgroup.kill` (5.14+). Añadir `cgroup.freeze`.
- §24 `AJUSTAR` Verificar `poll()` real sobre PSI en el kernel objetivo.

## Bloque 4 — Scheduler, OverlayFS, mounts, Landlock, red (§25–§34)

- §25 `OK`
- §26 `AJUSTAR` OverlayFS solo donde aporte; copy-up + idmapped + Landlock tienen aristas.
- §27 `AJUSTAR` Añadir whiteouts/opaque dirs y limpieza idempotente.
- §28 `OK`
- §29 `DIFERIR` idmapped mounts (V2); verificar interacción con OverlayFS.
- §30 `OK`
- §31 `FIX` Landlock sin CAP_SYS_ADMIN exige `no_new_privs` primero. Unificar orden.
- §32 `OK`
- §33 `OK`
- §34 `OK` Landlock net (TCP bind/connect) desde kernel 6.7; no reemplaza netns.

## Bloque 5 — Seguridad y proceso (§35–§44)

- §35 `OK`
- §36 `OK`
- §37 `OK`
- §38 `OK` Helper privilegiado con API mínima.
- §39 `OK`
- §40 `OK`
- §41 `OK`
- §42 `AJUSTAR` Requisito de reaping: en PID ns el hijo PID 1 debe cosechar zombies.
- §43 `OK` `cgroup.events` permite `poll`.
- §44 `OK` io_uring fuera de V1.

## Bloque 6 — Runtimes, secrets, FS, IPC, API (§45–§56)

- §45 `OK`
- §46 `OK`
- §47 `OK`
- §48 `OK`
- §49 `OK`
- §50 `AJUSTAR` V1: FD pasado o tmpfs; env vars con cuidado por `/proc`.
- §51 `OK` memfd + seals: mejor opción V2.
- §52 `AJUSTAR` Montar `/proc` en el PID ns y dentro del hijo; ocultar `/proc/sys`.
- §53 `OK`
- §54 `OK` Añadir `/dev/pts`, `/dev/tty` si hay PTY.
- §55 `OK`
- §56 `OK` Unix socket + HTTP/JSON para V1.

## Bloque 7 — Planos, persistencia, reconciliación, estado (§57–§68)

- §57 `OK`
- §58 `OK`
- §59 `OK`
- §60 `OK`
- §61 `OK` SQLite para estado de dominio.
- §62 `OK`
- §63 `OK`
- §64 `AJUSTAR` Single-flight/locking: no admitir ejecuciones durante reconciliación (o por-ejecución).
- §65 `OK`
- §66 `OK` Añadir estados terminales explícitos.
- §67 `OK`
- §68 `OK`

## Bloque 8 — Lifecycle, artifacts, concurrencia, APIs (§69–§87)

- §69 `FIX` Mezcla fase padre y fase hijo. Reescribir en dos fases.
- §70 `OK`
- §71 `OK`
- §72 `OK`
- §73 `OK`
- §74 `OK`
- §75 `OK`
- §76 `AJUSTAR` Artifacts explícitos.
- §77 `OK` Worktree superior a OverlayFS para repos Git.
- §78 `OK`
- §79 `AJUSTAR` Worktree por ejecución resuelve gran parte de la concurrencia.
- §80 `OK`
- §81 `OK`
- §82 `OK`
- §83 `OK`
- §84 `OK`
- §85 `OK`
- §86 `OK`
- §87 `OK`

## Bloque 9 — FDs, clone3, OOM, fairness, Artix/Dinit (§88–§100)

- §88 `AJUSTAR` `close_range` debe retener FDs explícitos de secretos.
- §89 `OK`
- §90 `OK` clone3 preferido.
- §91 `OK` Añadir `memory.oom.group` y clasificar causas.
- §92 `OK`
- §93 `OK` CPU weight + quotas por compañía.
- §94 `OK` Threat model obligatorio.
- §95 `OK`
- §96 `OK`
- §97 `OK` Verificar doc actual de dinit.
- §98 `OK`
- §99 `OK` Verificar sintaxis real.
- §100 `OK` V1 root para aprender; target helper separado.

## Bloque 10 — Appliance, kernel, FS, Git, knowledge (§101–§115)

- §101 `OK`
- §102 `OK`
- §103 `AJUSTAR` Añadir CONFIG_PSI, cgroup2 completo, Landlock, userns, overlay, idmapped, pidfd, seccomp, netfilter.
- §104 `OK`
- §105 `OK` ext4/XFS por simplicidad; Btrfs si aporta de verdad.
- §106 `OK`
- §107 `OK`
- §108 `OK`
- §109 `OK`
- §110 `OK`
- §111 `OK`
- §112 `OK`
- §113 `OK`
- §114 `FIX` Decisión build-vs-reuse del sandbox sin tomar.
- §115 `OK`

## Bloque 11 — Entregables, V1–V3, plan (§116–§126)

- §116 `AJUSTAR` Recortar V1; Landlock/seccomp fuera del camino crítico inicial.
- §117 `OK`
- §118 `OK`
- §119 `AJUSTAR` Producir incremental: B (Threat Model) -> A (ADR) -> resto.
- §120 `OK`
- §121 `OK`
- §122 `OK`
- §123 `FIX` Falta proc mount, reaping, retención de FDs, oom.group; unificar con §31/§69.
- §124 `OK`
- §125 `OK`
- §126 `OK`
