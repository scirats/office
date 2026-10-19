# 07 — Secretos V1 (contrato del engranaje)

Estado: aceptado

Complementa `03-agente-identidad-recursos.md` (modelo de credenciales) y
`06-modelos-v1.md` (los proveedores referencian credenciales). Introduce el
store de secretos como engranaje propio: es una capacidad distinta a la de
modelos o agentes (`../ESTANDARES.md` §2.2).

## Especialidad

> Guardar y resolver credenciales de Office. Resuelve una `credential_ref` a su
> valor. **No** sabe de modelos, agentes ni ejecuciones, y **nunca** expone
> valores fuera de sus consumidores.

## Alcance

Incluido:

- `SecretRef` (clave) y `Secret` (valor, con `Debug` redactado);
- puerto `SecretStore` (`resolve`, `references`, `set`);
- catálogo de secretos en `/etc/office/secrets.toml`, **modo 0600 obligatorio**;
- `office.secret.list.v1` → `office.secret.listed.v1` (solo nombres);
- `office.secret.set.v1` → `office.secret.updated.v1`: el daemon escribe el
  valor en el archivo 0600 y **nunca** lo devuelve.

Excluido (a propósito): inyección por ejecución (FD/`memfd`), cifrado en disco,
rotación, y el cumplimiento de scopes por nivel (`workspace/…`, `project/…`).
El cliente que usa la key (execution plane) tampoco existe aún.

## Engranaje

| Crate | Especialidad | Puertos que consume/publica |
|-------|--------------|------------------------------|
| `secret` | Store de secretos (dominio + puerto) | `SecretStore` |
| `officed` (adaptador) | `FileSecretStore`: lee y valida el TOML 0600 | — (implementa `SecretStore`) |

El engranaje `secret` no contiene adaptadores; `officed` los hospeda y los
inyecta, igual que con `model` y `tool`.

## Dominio

- `SecretRef` es una clave (`provider/anthropic`) y puede aparecer en
  configuración y listados.
- `Secret` es el valor. Su `Debug` es `Secret(***)` para que no se filtre por
  logs; el valor solo sale por `expose()`.

## Puerto

```rust
pub trait SecretStore {
    fn resolve(&self, reference: &str) -> Option<Secret>;
    fn references(&self) -> Vec<String>; // solo nombres, ordenados
    fn set(&self, reference: &str, value: &str) -> Result<(), SecretError>;
}
```

## Configuración

Archivo TOML global, por defecto `/etc/office/secrets.toml`, sobreescribible con
`OFFICE_SECRETS`. Las claves coinciden con el `credential_ref` de cada proveedor
en `models.toml`.

```toml
[secrets]
"provider/anthropic" = "sk-ant-..."
"provider/openai" = "sk-..."
```

`FileSecretStore` **rechaza** el arranque si el archivo no está en modo `0600`
(fallo visible), si no se puede leer o si el TOML es inválido. Si el archivo no
existe, el daemon arranca con un store vacío y avisa por stderr. Un ejemplo vive
en `../daemon/config/secrets.example.toml`.

## Protocolo

| Tipo | Datos | Efecto |
|------|-------|--------|
| `office.secret.list.v1` | — | lista referencias presentes (nombres) |
| `office.secret.set.v1` | `model, value` | escribe la credencial del proveedor de ese modelo |

| Tipo | Datos |
|------|-------|
| `office.secret.listed.v1` | `references[]` (solo nombres) |
| `office.secret.updated.v1` | `model` |

El cliente identifica la credencial por **modelo**; el daemon resuelve
`model → provider → credential_ref`. Los valores solo viajan en el sentido
`secret.set` (cliente→daemon); **nunca** se devuelven, ni van a la DB o logs.

En el cliente web, el editor de **Definition** muestra, debajo de Model, un
input de API key que se guarda solo (al salir del campo o con Enter). No hay
botón Save; el valor se envía una vez y el daemon lo escribe en el archivo 0600.
Si el proveedor ya tiene credencial, el input muestra el placeholder
`••••••••` (nunca el valor), derivado de `model.listed`.

## Sincronía con el archivo

`SecretStore::reload` vuelve a leer el archivo. El daemon lo invoca al
reconciliar y antes de responder `secret.list` / `model.list`, de modo que
editar `secrets.toml` por fuera se refleje sin reiniciar. La escritura es
atómica (temporal + `rename`), así que una lectura concurrente ve el estado
anterior o el nuevo, nunca uno a medias.

Aviso de seguridad: hoy el valor viaja por el WebSocket sobre loopback, que
**no tiene autenticación**. La mitigación (token/`SO_PEERCRED`) queda pendiente.

## Relación con `model`

`Provider.credential_ref` en `models.toml` es la clave que este store resuelve.
El acoplamiento ocurre en el execution plane (pendiente): al preparar una
ejecución, se resuelve `credential_ref → Secret` y se inyecta (FD/`memfd`).

## Pendiente

1. Inyección por ejecución (`FD`/`memfd`) y su ciclo de vida corto.
2. Scopes por nivel (`workspace/<ws>/…`, `project/<ws>/<proj>/…`, `docs/03`).
3. Cifrado en disco y rotación.
4. Cliente del API por `ProviderKind` que consuma la credencial resuelta.
