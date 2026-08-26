# OmegaOS

Un plano de control en el terminal para ejecutar en paralelo una flota de agentes de programación con IA, donde todos obedecen el mismo reglamento tipado.

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | [中文](README.zh.md) | Español

> El [README en inglés](README.md) es la versión canónica y la más actualizada; esta traducción puede ir un poco por detrás.

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS no es una biblioteca que importas. Lo instalas en una máquina Linux y obtienes el comando `omega`, una TUI para vigilar y matar sesiones, una capa de orquestación que reparte el trabajo a los agentes y un puente de Telegram para el control desde el móvil. Las sesiones nuevas usan OpenAI Codex por defecto. Claude Code, Google Antigravity, Gemini CLI para Enterprise/API, OpenRouter/Pi, Hermes, GLM y Kimi siguen siendo opciones explícitas. Cada agente recibe un contexto de políticas compacto, tipado y acotado al rol, compilado a partir de la misma doctrina.

Versión actual: mira [CHANGELOG.md](CHANGELOG.md) (`omega -V` en una máquina ya instalada). Lo uso a diario; da por hecho que te encontrarás asperezas.

## Instalación

Un comando en una máquina Linux (en macOS funciona casi todo):

```
npx omega-os
```

Ese comando clona el repo y lanza el instalador detrás de una pantalla de progreso interactiva con lluvia de código de Matrix (escribe para inyectar glifos, `espacio` para lanzar un pulso; `npx omega-os --plain` para una barra normal). Si prefieres hacerlo a mano:

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

El instalador descarga binarios `rmux` + `omega` + `omega-gatewayd` precompilados para tu plataforma cuando hay una versión publicada (verificados por checksum), y si no, compila desde el código fuente — así un clon nuevo siempre reproduce el sistema, solo que más rápido cuando existe un binario. Fuerza la compilación desde el código fuente con `OMEGA_FROM_SOURCE=1 ./install.sh`.

## Actualización

```
omega update           # fetch + fast-forward + reinstalación
omega update --check   # ¿qué cambiaría? (no toca nada)
```

Actualiza la copia de trabajo que encuentra (`$OMEGA_SRC`, el directorio actual, luego
`~/Station/SideBusiness/OmegaOS`, `~/Station/OmegaOS`, `~/OmegaOS` — o pasa `--dir`),
recompila desde el código fuente y vuelve a lanzar el instalador. Tu estado de
`~/.omega` se conserva: los secretos, los proyectos, la configuración de Telegram y
`config.toml` nunca se sobrescriben.

Si tienes cambios locales o commits sin subir en la copia de trabajo, la actualización
**se detiene y te lo dice** en vez de tocar tu trabajo — haz commit, stash o push, y
vuelve a lanzarla.

También se mantiene al día: cada instalación busca novedades a diario a las 03:30 e instala
lo que encuentra, y se salta cualquier noche en la que tu copia de trabajo tenga trabajo local,
un agente esté a mitad de turno o el mismo commit ya haya fallado tres veces. Instalar de forma
automática significa confiar en el repo cada noche, así que cambiarlo es un solo comando:

```
omega config set auto_update check   # avísame en vez de instalar
omega config set auto_update off     # no hacer nada
```

## Los primeros 5 minutos

El stack se instala solo; lo único que queda es la parte personal. **`omega guide`
imprime el paso a paso completo** (también guardado en `~/.omega/GETTING-STARTED.md`,
y mostrado al final de la instalación). En resumen:

1. **Conecta Codex** *(obligatorio para el runtime por defecto)*: ejecuta `omega codex-login` y luego comprueba con `omega codex-login-status`. Claude sigue siendo opcional mediante `claude auth login`.
2. **Control remoto por Telegram** *(recomendado)* — el token de [@BotFather](https://t.me/BotFather), tu id de [@userinfobot](https://t.me/userinfobot) y, después, `OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>` (la forma con variable de entorno mantiene el token fuera de la lista de procesos). Para un tema por proyecto: grupo + temas (Topics) activados + bot administrador → `/setupgroup` → `/sync`.
3. **Claves de servicio** *(opcional)* — `~/.omega/provisioning/services.env` (Vercel / GitHub / Convex / Stripe / OpenAI para voz) alimenta el aprovisionamiento automático de aplicaciones nuevas.
4. **Añade un proyecto** — `omega` → **[N] New Project**, Telegram → *Import from GitHub*, o basta con dejar un repo en `~/Station/<Category>/`.
5. **Verifica** — `omega doctor`: todas las líneas en `[+]`.

Esta es una ejecución real de `omega doctor`:

```
OmegaOS doctor

  [+] binary           omega 0.1.6
  [+] rmux daemon      connected, 6 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         7 Laws + 47 Rules
  [+] agent CLI        codex available
  [+] state dir        /home/vibe/.omega/state
  [+] telegram service omega-tg-bot active
  [+] hooks            track + verify present, registered in settings.json
  [+] secrets dir      /home/vibe/.omega present
  [+] memory           249088MB available
  [+] usage cache      usage cache 1 min old
  [+] codex auth       Codex login valid
  [+] telegram poller  1 poller
  [+] provisioning     provisioning: VERCEL_TOKEN, CONVEX_TEAM_TOKEN, STRIPE_SECRET_KEY
```

Las líneas `[!]` son avisos que llevan el comando de reparación en la propia línea; `omega doctor --fix` arregla los que son mecánicos.

## Lo que puedes hacer

- **Despacha misiones.** `omega dispatch <Project> "<mission>"` entrega el trabajo al oracle de ese proyecto, que planifica, lanza workers y somete el resultado al control de calidad. `omega orchestrate` ejecuta en un solo comando el pipeline completo de clasificar → planificar → despachar → monitorizar → controlar la calidad.
- **Ejecuta planes tipados.** `/omg-planner` descompone un desarrollo en un DAG tipado (`.planner/tracker.json`); `omega plan-run` lo ejecuta con una barrera estructural que impide saltarse pasos (Gate) y con la prueba de un comando de verificación independiente (Guardian).
- **Arranca aplicaciones enteras.** `/omg-new-project` aprovisiona Vercel/Convex/GitHub/Clerk/Stripe a partir de tus claves, monta el esqueleto del stack y luego recorre visión → PRD → plan → construcción.
- **Paraleliza con seguridad.** Los workers reservan sus archivos con bloqueos consultivos reales (`fs2`), y `omega spawn-worker --worktree` da a cada worker paralelo su propio worktree de git con un merge limpio al final. Una señal de finalización crea un resultado candidato. Solo un verificador independiente y el control de aceptación de la misión pueden cerrarlo.
- **Audítalo todo.** Un Quality Arsenal de 23 auditorías forenses Gestalt-Popper (`codeaudit`, `secaudit`, `perfaudit`, `a11yaudit`, …) autoseleccionadas según lo que haya cambiado, más `/omg-acceptance` — un control autónomo de aceptación en navegador que recorre todas las rutas y arregla lo que encuentra.
- **Convoca un consejo.** `/omg-llm-council` plantea la misma pregunta a cuatro modelos Claude distintos en paralelo, hace que se revisen entre sí de forma anónima y sintetiza un veredicto sin borrar las discrepancias — sin claves de API, se ejecuta dentro de tu sesión actual.
- **Navega de forma agéntica.** `/omg-browser-use` maneja un navegador en la nube para tareas que un script de Playwright no puede expresar.
- **Haz también el go-to-market.** Un pack de marketing incluido (investigación de mercado, posicionamiento, estrategia de contenidos, social, cold email, creatividades publicitarias, estrategia de lanzamiento) más el dúo de identidad visual de Higgsfield.
- **Recibe informes en el móvil.** Cada misión termina con un informe PDF con la identidad de marca, publicado en el tema de Telegram del proyecto, y una tarjeta de progreso se va actualizando sobre el mismo mensaje mientras la misión avanza. Un bot de depósito da a los agentes un buzón privado para los archivos que envías desde el móvil.
- **Gestiónalo.** `omega doctor` (salud de todo el stack), `patrol` (vigilante de sesiones), `usage` (presupuesto de tokens + avisos por Telegram), `backup` (copia de seguridad del estado irreproducible de `~/.omega` → un solo tgz), `cleanup` / `kill-all`, `timeline` (reproducir una misión), `resurrect` (revivir un oracle caído), `provision` (grupos de credenciales por cliente).
- **Resuelve tickets de Linear de principio a fin.** `/omg-linear` arregla, captura pruebas, audita hasta 100/100, comenta y mueve el ticket a revisión — nunca a Done; eso lo hace una persona. Mira [Integración con Linear](#integración-con-linear).

Tres formas de entrar: la TUI de `ratatui` (5 pestañas: Sessions, Menu, Agentic, Settings, Help), la CLI de `omega` (más de 40 comandos) y el hub de Telegram. Un modo RPC (JSONL por stdin/stdout) lo maneja desde otras herramientas. Por debajo, todo se ejecuta sobre [rmux](https://github.com/agentik-os/rmux), un multiplexor de terminal en Rust — sin dependencia de tmux.

## La doctrina

Hay un registro tipado de 7 Leyes y 47 Reglas operativas con nombre. `omega rules list` imprime el conjunto actual. El compilador vive en `crates/omega-core/src/rules.rs`; emite un contexto determinista y adaptado a cada proveedor, con un tope estricto de 24 KB para el contexto de OmegaOS.

**Las Leyes son inviolables.** Vinculan a todos los agentes y prevalecen sobre cualquier regla y cualquier tarea. Son siete:

- **L0 — Entrega la verdad.** Un cambio no está hecho hasta que una recompilación limpia lo reproduzca y se haya hecho push. Menos que eso es un borrador.
- **L1 — El runtime es la única verdad.** El código y los comentarios declaran la intención. Solo ejecutarlo revela la realidad. Cuando discrepan, gana el runtime.
- **L2 — Investigador, no adulador.** Rebate una premisa defectuosa con argumentos antes de actuar. Nada de confianza fingida. «Esto debería funcionar» sin pruebas es una mentira.
- **L3 — Decide y sigue.** Un agente despachado es autónomo. Nunca se para a preguntar «¿sigo?». Decide, ejecuta e informa después.
- **L4 — Hecho significa 100 %, verificado.** El 92 % no está hecho. Enumera las tareas, termina cada una y verifica cada una contra la ejecución real.
- **L5 — Calidad por encima de la velocidad.** Nada de variantes simplificadas, ligeras ni rápidas de un protocolo real. Un 403 o un 401 obliga a abortar la misión; nunca cuenta como aprobado.
- **L6 — Termina la misión.** Enumera, ejecuta, verifica e informa de cada entregable pedido. Un plan o una fase parcial no es un punto de parada legítimo.

**Las Reglas son operativas.** Tienen nombre (R-SCOPE, R-VERIFY, R-CITE, …) y se ordenan en Universal, QualityGate, Orchestration, Reporting y Safety. Cada Regla está acotada a los roles a los que vincula: Master, Oracle, Worker. A un worker no se le cargan reglas de orquestación que no puede aplicar, y un oracle no arrastra la disciplina de bloqueo de archivos del worker. Mismo registro, distintos recortes.

### El embudo

Este es el mecanismo. `rules::compile_rule_context_for_provider` combina el núcleo compacto de Leyes, el contrato de rol, las reglas relevantes para la misión, la mecánica del proveedor y las referencias a skills. Rechaza la salida que supere el presupuesto de contexto en vez de truncarla en silencio. Cada contexto compilado lleva un digest determinista para detectar la deriva.

Un worker tres niveles más abajo en el árbol lleva las mismas siete Leyes que el Master. Los procedimientos operativos se cargan solo cuando el rol, la misión, el riesgo y el proveedor los exigen. Así los invariantes siguen siendo universales sin inyectar todos los manuales de operación en cada turno.

Míralo entero:

```
omega rules list
```

![omega rules list — las Leyes y las Reglas, impresas por OmegaOS](assets/omega-rules.svg)

## Arquitectura

Cuatro niveles, de arriba abajo:

```
┌─────────────────────────────────────────────────────────────────┐
│  Nivel 1 — Interfaz humana                                      │
│  TUI (5 pestañas) · CLI (40+ cmds) · hub de Telegram            │
│                      ↓ intención                                │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 2 — Master (cerebro persistente — el tema Atlas)         │
│  14 plantillas con nombres de Matrix, clasificar → enrutar      │
│                      ↓ despacho                                 │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 3 — Oracle (1 por proyecto)                              │
│  Clasificar → Planificar → Lanzar workers → Control de calidad  │
│                      ↓ descomponer                              │
├─────────────────────────────────────────────────────────────────┤
│  Nivel 4 — Workers (efímeros, paralelos, por bloqueo de archivo)│
│  Ejecutar → Verificar → done.json → Oracle confirma → cierre    │
└─────────────────────────────────────────────────────────────────┘
```

**Nivel 2 — el Master.** Un agente persistente que se mantiene en marcha, se reinicia solo si se cae y retoma su propia conversación. Trae 14 plantillas de agente con nombres de personajes de Matrix (Oracle, Morpheus, Seraph, Keymaker, Smith, Niobe, Architect, Merovingian, Neo, Zion, Link, Construct, Pythia, Council). El Master es un despachador. Solo clasifica y enruta el trabajo hacia los oracles.

**Nivel 3 — Oracle.** Uno por proyecto. Clasifica la petición, planifica, despacha workers y ejecuta el control de calidad al final. Un oracle orquesta. No edita él mismo el código del proyecto, así que quien lo evalúa y quien lo escribe nunca son el mismo agente.

**Nivel 4 — Workers.** Efímeros. Se ejecutan en paralelo, cada uno acotado a sus propios archivos por una reserva con bloqueo de archivo (bloqueos consultivos vía `fs2`) — y opcionalmente a su propio worktree de git. Un worker señala que ha terminado escribiendo un `done.json` con estado `done_clean`, `pending` o `failed`; sin ese estado no está hecho.

### Cómo se ejecuta una misión

Una petición entra por la TUI, la CLI o Telegram. Empiece donde empiece, aterriza en el Master, que la lee, la clasifica y la enruta al oracle dueño del proyecto en cuestión. El oracle planifica la misión, la divide en tareas y despacha un worker por tarea. Los workers verifican sus propios resultados contra la ejecución real y escriben su `done.json`; el oracle lo lee, ejecuta el control de calidad e informa hacia arriba.

Un worker no tiene por qué ir resolviendo sus subtareas de una en una. Puede ejecutar un workflow en el mismo proceso: lanzar subagentes en paralelo, comprobar sus salidas y combinarlas en una sola respuesta. La revisión de código funciona así, y también la investigación, las auditorías y el trabajo de diseño.

La verificación es deliberadamente antagónica: que un worker diga «hecho» no cierra la comprobación; su afirmación pasa a agentes independientes y solo sobrevive si una mayoría (dos de tres) está de acuerdo. Las auditorías del Quality Arsenal encajan justo aquí, en el control de calidad.

Esto depende del embudo de la doctrina de más arriba: a todos los agentes, en todos los niveles, se les inyectan sus Leyes y Reglas acotadas al rol en el momento en que son despachados.

Esta sección del README es en sí misma un ejemplo. La produjo un workflow. Un agente escribió el borrador, lectores independientes lo recorrieron a la caza de prosa generada por IA, otro agente lo revisó a partir de lo que señalaron y hablantes nativos se encargaron de la traducción. Así que ninguna parte de este texto salió de una sola pasada sin revisar.

## Stack

Es un workspace de Rust con tres crates:

- `omega-core` — orquestación, el registro de reglas, doctor, timeline, cleanup, patrol y el bloqueo por alcance de archivos.
- `omega-cli` — el binario `omega`, construido sobre `clap`.
- `omega-tui` — el gestor de sesiones, construido sobre `ratatui`.

Por debajo funciona sobre [rmux](https://github.com/agentik-os/rmux), un multiplexor de terminal en Rust: un daemon, un SDK tipado y gestión de PTY. rmux es una biblioteca de Rust tipada, así que OmegaOS la llama directamente en vez de lanzar tmux como subproceso y analizar su salida de texto. No hay dependencia de tmux por ninguna parte.

Bun y TypeScript se encargan de renderizar los informes en PDF (a través de Next.js y Playwright) y de los bots de Telegram. Bash aparece exactamente en un sitio: el arranque de la instalación.

## Conexión en remoto

El daemon rmux es el dueño de todas las sesiones, así que tus agentes siguen en marcha después de que te desconectes. Para volver a ellos, haz **attach** — vuelve a conectar tu terminal a una sesión que ya está en marcha:

```
rmux attach              # volver a la última sesión
rmux attach -t claude-1  # conectar a una concreta
rmux list-sessions       # ver qué hay vivo
```

Vuelve a hacer detach con `Ctrl-b d` — la sesión y sus agentes siguen en marcha sin ti.

`omega` agrupa los puntos de entrada a los que de verdad recurres:

```
omega                       # abrir el gestor de sesiones de la TUI (explorar / lanzar / monitorizar)
omega attach -t claude-1    # entrar directamente en una sesión para trabajar en ella
omega master                # saltar a la sesión Master
omega list                  # listar todas las sesiones vivas
```

Usa el menú (`omega`) para gestionar y lanzar; usa un attach directo (`omega attach -t …`, o `rmux attach -t …`) cuando quieras teclear concentrado en una sola sesión — la vista previa del menú *refleja* el panel, mientras que un attach directo es el camino de menor latencia.

Por SSH desde un portátil, el SSH normal espera una ida y vuelta completa por la red antes de hacer eco de cada pulsación, así que en una máquina lejana teclear se nota lento y la salida de los agentes llega a trompicones — da igual lo rápida que sea la máquina, porque es latencia, no CPU. `install.sh` instala [`mosh`](https://mosh.org) para esto: hace eco de tus pulsaciones en local y envía las diferencias de pantalla por UDP, así que teclear es instantáneo y el streaming va fluido con cualquier latencia. Conéctate directamente a una sesión con:

```
mosh user@host -- omega attach -t claude-1
```

En un cliente como **Termius**: pon la IP y el puerto del host, activa el interruptor **mosh** y añade un fragmento de arranque — `omega` para el menú, o `omega attach -t <session>` para aterrizar directamente en una sesión.

(Usa el `Alt+Up/Down` de rmux para el scrollback, no el PageUp de mosh.) El instalador también deja configurados `/etc/rmux.conf` y una configuración regional UTF-8 para todo el sistema, así que cada cuenta —root y los usuarios futuros— recibe la misma sesión reforzada (rueda del ratón, selección arrastrando al portapapeles local por SSH, teclado ágil, truecolor) sin ninguna configuración por usuario.

## Integración con Linear

Si llevas el feedback de tus usuarios en [Linear](https://linear.app), OmegaOS resuelve los tickets de principio a fin. Dos comandos.

`/omg-linear-setup` es un asistente que se ejecuta una sola vez, dentro de tu propia app. Instala un widget de feedback integrado (recoge una captura de pantalla, la URL de la página, el elemento pulsado y la consola del navegador en el momento de enviar el informe), las etiquetas de Linear en las que se apoya el pipeline y la ruta de API que convierte un informe del widget en una issue de Linear. Primero detecta tu stack, tu proveedor de autenticación y tu biblioteca de UI, así que escribe código que encaje con el proyecto en vez de una plantilla genérica.

`/omg-linear` hace el trabajo. Lee los tickets abiertos y, para cada uno, arregla el código, captura pruebas de antes y después, y luego ejecuta las auditorías del Quality Arsenal que encajen con el cambio. Un ticket solo avanza si esas auditorías sacan 100/100. Después publica un comentario de verificación del arreglo en el ticket y lo mueve a un estado de revisión — `In Review` si tu equipo tiene uno, o si no un `Omega Review` neutro que crea él mismo. Nunca marca un ticket como Done; eso lo hace una persona después de comprobarlo. El motor v2 lo ejecuta a través de un Workflow: hace triaje de los tickets abiertos, reparte en paralelo el arreglo y la auditoría de cada ticket, y somete cada resolución a una verificación antagónica antes de comentar.

Solo se activa si lo llamas por su nombre. OmegaOS únicamente toca Linear cuando se lo pides de forma explícita (`/omg-linear`, `fix linear`, un id de ticket como `KOM-7` o un enlace de `linear.app`). La palabra suelta «feedback» no lo dispara nunca, y no mencionará Linear a menos que lo hagas tú.

```
omega_dir=~/.omega          # el protocolo se instala en ~/.omega/skills/linear/
/omg-linear-setup           # una vez por app — instala el widget, las etiquetas y la ruta
/omg-linear                 # resuelve los tickets abiertos: arreglo -> auditoría -> comentario -> In Review
```

## Límites

Prefiero que lo sepas antes de entrar.

- **Pensado para Linux.** Desarrollado en un VPS sin entorno gráfico. Sin Windows. macOS recibe correcciones de verdad (servicios launchd, ruta de Homebrew), pero está menos rodado.
- La TUI da por hecho un terminal de 256 colores. En uno de 16 colores se verá feo.
- El runtime de agente por defecto es OpenAI Codex, así que la CLI `codex` tiene que tener la sesión iniciada. Claude Code, Antigravity, Gemini CLI para Enterprise/API, OpenRouter/Pi, Hermes, GLM y Kimi son alternativas admitidas.
- **Una sola máquina.** El daemon rmux es local. No hay orquestación multi-host.
- Es 0.1.x. Lo uso a diario, pero encontrarás asperezas con las que yo todavía no me he topado.

## Siguiente paso: GUIDE.md

**[GUIDE.md](GUIDE.md)** es el manual del operador: el vocabulario (misión, oracle, worker, goal, plan, Atlas), los tres puestos de mando, tus primeras misiones, el catálogo de skills y cómo se verifica el trabajo. Luego, profundiza:

- [docs/FEATURES.md](docs/FEATURES.md) — **el catálogo completo de funcionalidades** (cada subsistema + cómo llegar a él).
- [docs/README.md](docs/README.md) — el índice de la documentación.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — la referencia del sistema completo.
- [docs/MAP.md](docs/MAP.md) — dónde vive cada cosa en el disco.
- [docs/THEMES.md](docs/THEMES.md) — la galería de paletas de la TUI.
- [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md) — copia de seguridad y reconstrucción de una máquina.
- [CHANGELOG.md](CHANGELOG.md) — qué ha salido en cada versión.

## Créditos

OmegaOS se apoya en el trabajo de mucha otra gente:

La mayor deuda es [rmux](https://github.com/agentik-os/rmux), el multiplexor de terminal en Rust sobre el que se ejecuta todo esto.

El resto del stack de Rust:

- [ratatui](https://github.com/ratatui/ratatui) y [crossterm](https://github.com/crossterm-rs/crossterm) — la TUI.
- [tokio](https://github.com/tokio-rs/tokio) — el runtime asíncrono.
- [clap](https://github.com/clap-rs/clap) y `clap_complete` — la CLI y el autocompletado del shell.
- [serde](https://github.com/serde-rs/serde) con `serde_json`, `serde_yaml` y `toml` — configuración y estado.
- [anyhow](https://github.com/dtolnay/anyhow) y [thiserror](https://github.com/dtolnay/thiserror) — el manejo de errores.
- `chrono` (marcas de tiempo), `dirs` (rutas), `fs2` (los bloqueos consultivos de archivo que hay detrás de las reservas de alcance), `regex`, `tempfile`, `tracing` con `tracing-subscriber` (logs) y `reqwest` (HTTP de Telegram y PDF).

[Claude Code](https://www.anthropic.com), de Anthropic, es el runtime de agente.

## Licencia

Doble licencia, [MIT](LICENSE-MIT) o [Apache-2.0](LICENSE-APACHE), a tu elección. La convención habitual en Rust. Elige la que prefieras.
