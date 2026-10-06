# Estándares de desarrollo de Office

## 0. Alcance

Estas reglas definen **cómo se estructura, cómo se escribe y cómo se implementa**
el código de Office, y aplican a todo el sistema.

Son **agnósticas a librerías e infraestructura**: no fijan base de datos,
framework, transporte ni crate. La implementación es Rust, pero toda elección
concreta es un adaptador sustituible y no puede filtrarse al núcleo de un
engranaje.

Ninguna regla se relaja sin una decisión documentada.

## 1. Principios

- Todo el sistema son engranajes compuestos.
- Cada engranaje es un módulo con una especialidad y una única responsabilidad.
- Los engranajes no se conocen entre sí.
- La única forma de comunicación es puente → puerto → adaptador.
- Una sola fuente de verdad por regla de negocio.
- Los fallos se hacen visibles; nunca se ocultan con valores por defecto
  silenciosos.
- Toda capacidad tiene una prueba reproducible y documentación coherente.
- No se implementa una capacidad fuera del alcance acordado.

## 2. El sistema son engranajes

### 2.1 Engranaje

Un **engranaje** es la unidad mínima de capacidad: un módulo aislado con una
especialidad y una responsabilidad única. Contiene:

- su **dominio**: tipos, invariantes y reglas puras;
- sus **casos de uso**: las operaciones que ofrece;
- sus **puertos**: los contratos que consume y, si aplica, los que publica.

Un engranaje no conoce a ningún otro engranaje, ni sus tipos, ni sus adaptadores.

### 2.2 Especialidad y responsabilidad única

Cada engranaje declara en una frase qué resuelve y qué no. Una capacidad nueva es
un engranaje nuevo, no una rama más dentro de otro. Cuando un engranaje necesita
dos especialidades, son dos engranajes unidos por un puente.

### 2.3 Aislamiento

- No importa, referencia ni asume tipos internos de otro engranaje.
- No conoce adaptadores ajenos.
- Recibe todo lo externo a través de puertos.
- Solo expone lo que su contrato declara.

### 2.4 Comunicación: puente, puerto y adaptador

Los engranajes se conectan **solo** mediante estas tres piezas:

- **Puerto:** contrato que declara una capacidad necesaria. Lo define **quien lo
  consume**, no quien lo implementa. Es pequeño y enfocado.
- **Adaptador:** implementación concreta de un puerto (otro engranaje, una base
  de datos, un reloj, la red). Vive fuera del engranaje que define el puerto.
- **Puente:** el único punto donde dos piezas se encuentran. Une un puerto con un
  adaptador y concentra lo transversal al cruce: forma del mensaje, correlación,
  idempotencia, errores de frontera y traducción.

Reglas del cruce:

- Un engranaje habla por su puerto; nunca por el interior de otro engranaje.
- Ningún engranaje importa a otro: el puente los une en la composición.
- Lo que cruza un puente es un contrato explícito, no un objeto interno.
- El puente es el único lugar donde se permite conocer dos piezas a la vez.

### 2.5 Sistema

Un engranaje aislado no es el sistema. **El sistema es la composición de todos los
engranajes unidos por puentes.** La composición ocurre en un único punto de
entrada (arranque / composition root), que:

1. construye cada engranaje con sus adaptadores;
2. conecta cada puerto con su adaptador;
3. expone el input/output global del sistema.

La composición es explícita y visible: no hay resolución dinámica, buses ni
registros ocultos.

```text
puerto ──▶ [ puente ] ──▶ adaptador
   ▲                          │
 define el                 implementa
 contrato                  la capacidad

composición: engranaje A + puente + engranaje B + ... = sistema
```

## 3. Estructura de archivos

Office se implementa en Rust como varios proyectos independientes. Cada proyecto
es un workspace de Cargo que vive en su propia carpeta en la raíz; `daemon/` es
uno de ellos.

```text
office/
├── daemon/               # un proyecto Rust
│   ├── Cargo.toml        # workspace del proyecto
│   ├── crates/
│   │   ├── <gear>/       # un crate por engranaje
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── lib.rs    # superficie pública mínima
│   │   │       └── *.rs      # módulos internos, organizados con libertad
│   │   └── officed/      # crate binario: composición y arranque
│   │       ├── Cargo.toml
│   │       └── src/main.rs
│   ├── tests/            # pruebas de integración y contrato
│   ├── config/
│   └── migrations/
├── <project>/            # otro proyecto Rust y su propio workspace
└── adr/                  # decisiones
```

Reglas:

- Cada proyecto es un workspace de Cargo independiente en su propia carpeta.
- Dentro del daemon, un crate por engranaje y un crate binario de composición.
- La estructura interna de `src/` es libre: se agrupa por responsabilidad según
  lo que el engranaje realmente necesita. No se imponen taxonomías de carpetas;
  puerto, adaptador y puente son conceptos, no directorios obligatorios.
- `lib.rs` expone lo mínimo: el contrato de entrada y los puertos que el
  engranaje publica. Todo lo demás queda privado al crate.
- Los puertos viven en el crate que los define; no existe un crate central de
  contratos.
- Un engranaje no depende de otro engranaje en su `Cargo.toml`. Solo el crate de
  composición conoce a todos.
- `officed` es el único binario del daemon: construye engranajes, une puertos con
  adaptadores y arranca.

Nombres:

- crates, módulos y tipos en inglés técnico;
- un concepto, un nombre en todo el sistema;
- el nombre de un engranaje dice su especialidad.

## 4. Diseño de código

- **Responsabilidad única:** cada módulo, tipo y función tiene una razón
  principal para cambiar. Una función no valida, transforma, persiste y responde
  a la vez.
- **Dependencias hacia dentro:** la lógica interna de un engranaje no conoce nada
  externo; depende solo de sus puertos; los adaptadores viven fuera y los
  implementan; la composición conoce a todos.
- **Composición sobre herencia.**
- **Open/closed** solo cuando existe una variación real; sin plugins
  hipotéticos.
- **Liskov:** toda implementación de un puerto respeta su contrato completo:
  errores, invariantes, efectos y límites.
- **Interface segregation:** capacidades pequeñas y enfocadas en lugar de una
  interfaz gigante de la que todos dependan.
- **Dependency inversion** solo en fronteras reales que deban sustituirse
  (persistencia, transporte, reloj, identificadores, proveedores externos). Las
  estructuras internas simples se usan directamente, sin envolverlas en traits.
- No se crean traits, factories ni archivos solo por patrón. Una abstracción se
  justifica únicamente por una frontera que deba protegerse o sustituirse.
- Funciones pequeñas, con responsabilidad clara y efectos visibles.

## 5. Contratos

- Antes de implementar una capacidad se documentan sus entradas, salidas,
  errores, permisos y eventos.
- Resultados y errores tipados; una cadena nunca es el contrato de error.
- Los errores distinguen fallos esperables del dominio, conflictos recuperables
  y fallos internos.
- Los eventos describen hechos aceptados, no instrucciones futuras.
- Todo mensaje que cruza un puente lleva actor, correlación, causa e
  idempotencia cuando aplique.
- Los contratos son explícitos y versionados; una ruptura se declara, nunca es
  silenciosa.

## 6. Implementación

- Código simple, explícito y verificable; sin abstracciones prematuras.
- Fallos visibles; sin valores por defecto silenciosos.
- No reinventar infraestructura general: si existe una solución madura, se
  reutiliza; el código propio se reserva para las reglas específicas de Office.
- Toda dependencia se justifica por necesidad, madurez, licencia, mantenimiento,
  historial de seguridad, estabilidad de API y coste de actualización; no puede
  introducir más riesgo o complejidad que el problema que elimina.
- Evitar los mecanismos de escape del lenguaje (unsafe, punteros crudos,
  reflexión) salvo decisión explícita y revisión especializada.
- Evitar clones y asignaciones innecesarias en rutas críticas, priorizando la
  claridad antes que la microoptimización.
- Sin código muerto, hacks ni workarounds sin fecha de retiro.
- Comentarios: ninguno que repita el código; un comentario explica *por qué*. Los
  contratos públicos tienen documentación.
- Nombres de módulos, tipos, funciones, variables, enums y comandos en inglés
  técnico.

## 7. Pruebas

Cada feature cubre, según corresponda:

- **Unitarias:** reglas puras, invariantes, transiciones, permisos.
- **Integración:** infraestructura real en entornos reproducibles y aislados
  (contenedores), nunca servicios instalados directamente en el host.
- **Contrato:** mensajes, errores, eventos y serialización.
- **Regresión:** cada bug corregido obtiene una prueba que falla antes del fix.
- **Propiedades o casos límite:** ciclos, duplicados, reintentos, concurrencia,
  cierres bloqueados y datos ausentes.

Pipeline mínimo, mapeado al toolchain elegido:

```text
formato
compilación / type check
análisis estático (linters)
pruebas
auditoría de dependencias
```

Una feature no está terminada si solo compila; debe probar sus reglas y sus
fallos esperables.

## 8. Cambios y revisión

Todo cambio indica qué contrato o engranaje afecta. La revisión comprueba:

- alcance correcto;
- reglas de dominio y permisos;
- migraciones y compatibilidad;
- pruebas de éxito y de rechazo;
- eventos y auditoría;
- ausencia de secretos y de dependencias innecesarias;
- documentación actualizada.

Las migraciones aplicadas en un entorno compartido son inmutables; un cambio de
esquema siempre se entrega como una nueva migración reproducible.

Las features publicadas conservan compatibilidad hacia atrás o declaran
explícitamente una ruptura de contrato con su migración.

Los controles automáticos se ejecutan desde la primera implementación: formato,
compilación, linters, pruebas y revisión de dependencias.

Si una decisión cambia una invariante general, primero se actualiza la
documentación y luego el código.

## 9. Definición de terminado

Una tarea está terminada cuando:

1. el comportamiento está documentado;
2. el código respeta engranajes, puertos y contratos;
3. formato, compilación, linters y pruebas pasan;
4. las migraciones son reproducibles;
5. errores, eventos y permisos están cubiertos;
6. no se declara implementada una capacidad fuera del alcance actual.
