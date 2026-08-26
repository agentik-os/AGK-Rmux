# OmegaOS

一个终端控制平面，用于并行运行一支 AI 编码 agent 编队，编队里的每个 agent 都遵守同一套类型化规则手册。

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | 中文 | [Español](README.es-ES.md)

> [英文版 README](README.md) 是权威且最新的版本；本翻译可能略有滞后。

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS 不是一个供你 import 的库。你把它装在一台 Linux 机器上，得到的是 `omega` 命令、一个用来盯着会话并随手 kill 掉它们的 TUI，以及一层把活儿派给 agent 的编排逻辑。还附带一个 Telegram 桥接，方便你用手机来驱动它。新会话默认使用 OpenAI Codex。Claude Code、Google Antigravity、面向 Enterprise/API 的 Gemini CLI、OpenRouter/Pi、Hermes、GLM 和 Kimi 仍可显式选择。每个 agent 都会收到由同一 doctrine 编译出的紧凑、类型化、按角色裁剪的策略上下文。

当前版本见 [CHANGELOG.md](CHANGELOG.md)（在已安装的机器上运行 `omega -V`）。我每天都在用它，粗糙的地方在所难免。

## 安装

在一台 Linux 机器上，一条命令搞定（macOS 大体上也能用）：

```
npx omega-os
```

它会克隆仓库，并在一块交互式的《黑客帝国》数字雨进度屏后面跑安装器（打字可以往里注入字符，按 `space` 会激起一圈脉冲；想要一根朴素的进度条就用 `npx omega-os --plain`）。更想手动来：

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

只要已经发布了对应版本，安装器就会下载适配你所在平台的预编译 `rmux` + `omega` + `omega-gatewayd` 二进制（用 checksum 校验），否则就退回到从源码构建——所以一次全新的 clone 总能复现出这套系统，只是有二进制的时候会更快。想强制从源码构建就用 `OMEGA_FROM_SOURCE=1 ./install.sh`。

## 更新

```
omega update           # 拉取 + fast-forward + 重新安装
omega update --check   # 会有什么变化? (什么都不碰)
```

它会更新它找到的那份 checkout（`$OMEGA_SRC`、当前目录，然后是
`~/Station/SideBusiness/OmegaOS`、`~/Station/OmegaOS`、`~/OmegaOS`——或者用
`--dir` 指定），从源码重新构建，再重新跑一遍安装器。你的 `~/.omega`
状态会被保留：secrets、项目、Telegram 配置和 `config.toml` 永远不会被覆盖。

如果那份 checkout 里有本地改动或者还没 push 的 commit，更新会
**停下来并告诉你**，而不是去动你的活儿——先 commit、stash 或者 push，
然后再跑一遍。

它也会让自己保持最新：每次安装都会设好一个每天 03:30 的检查，找到什么就装什么；
碰上你的 checkout 里有本地改动、有 agent 正在一个回合当中、或者同一个 commit
已经失败过三次的夜晚，就跳过。自动安装意味着每晚都要信任这个仓库，
所以要改掉它只需一条命令：

```
omega config set auto_update check   # 只提醒我, 不要安装
omega config set auto_update off     # 什么都不做
```

## 最初 5 分钟

技术栈会自己装好，剩下的只有跟你个人相关的那几步。**`omega guide`
会打印完整的分步指引**（同时保存在 `~/.omega/GETTING-STARTED.md`，
安装结束时也会显示一遍）。简而言之：

1. **接上 Codex** *（默认运行时必需）*：跑 `omega codex-login`，然后用 `omega codex-login-status` 确认。Claude 仍然是可选的，使用 `claude auth login`。
2. **Telegram 远程控制** *（推荐）*——token 从 [@BotFather](https://t.me/BotFather) 拿，你的 id 从 [@userinfobot](https://t.me/userinfobot) 拿，然后 `OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <ID> --user-id <ID>`（用环境变量的写法能让 token 不出现在进程列表里）。想要一个项目一个话题：建群 + 打开 Topics + 把机器人设为管理员 → `/setupgroup` → `/sync`。
3. **服务密钥** *（可选）*——`~/.omega/provisioning/services.env`（Vercel / GitHub / Convex / Stripe / 给语音用的 OpenAI）驱动新 app 的自动 provisioning。
4. **加一个项目**——`omega` → **[N] New Project**，Telegram → *Import from GitHub*，或者干脆把一个仓库丢到 `~/Station/<Category>/` 下面。
5. **验证**——`omega doctor`：每一行都是 `[+]`。

下面是一次真实的 `omega doctor` 输出：

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

`[!]` 那几行是警告，每一行都自带修复命令；`omega doctor --fix` 会修好机械性的那些。

## 你能做什么

- **派发任务。**`omega dispatch <Project> "<mission>"` 把活儿交给该项目的 oracle，由它来规划、派生 worker，并为结果把关。`omega orchestrate` 用一条命令跑完分类 → 规划 → 派发 → 监控 → 门禁这条完整流水线。
- **跑类型化的计划。**`/omg-planner` 把一次构建拆解成一张类型化的 DAG（`.planner/tracker.json`）；`omega plan-run` 负责执行，并配上两道保障：结构性的不可跳过强制（Gate），以及独立的验证命令证明（Guardian）。
- **一键起一整个 app。**`/omg-new-project` 用你的密钥去 provision Vercel/Convex/GitHub/Clerk/Stripe，搭好技术栈的脚手架，然后跑 vision → PRD → 计划 → 构建。
- **安全地并行。**worker 用真正的建议锁（advisory lock，`fs2`）声明自己的文件，而 `omega spawn-worker --worktree` 会给每个并行 worker 一份属于它自己的 git worktree，并在最后干净地 merge。一个完成信号只产生一个候选结果。只有独立的验证者和任务验收门禁才能把它关闭。
- **审计一切。**一整套 Quality Arsenal，23 项法证级的 Gestalt-Popper 审计（`codeaudit`、`secaudit`、`perfaudit`、`a11yaudit` 等等），会按刚刚改了什么自动挑选，外加 `/omg-acceptance`——一道自主的浏览器验收门禁，扫遍每一条路由，并把发现的问题修掉。
- **召集一个 council。**`/omg-llm-council` 把同一个问题并行抛给四个不同的 Claude 模型，让它们匿名互评，再综合出一份保留了异议的裁决——不需要 API key，它就跑在你现有的会话里。
- **让 agent 去浏览网页。**`/omg-browser-use` 驱动一个云端浏览器，去做那些用脚本化的 Playwright 表达不出来的任务。
- **顺带把 go-to-market 也做了。**内置了一整套 marketing 套件（市场调研、定位、内容策略、社交、cold email、广告创意、发布策略），外加 Higgsfield 那对做视觉识别（visual identity）的 skill。
- **在手机上收报告。**每一项任务结束时，都会往项目的 Telegram 话题里发一份带品牌样式的 PDF 报告；任务跑着的时候，还有一张实时进度卡片就地刷新。一个 deposit 机器人给 agent 提供了一个私有收件箱，用来接收你从手机上发过去的文件。
- **运维它。**`omega doctor`（整套技术栈的健康检查）、`patrol`（会话看门狗）、`usage`（token 预算 + Telegram 告警）、`backup`（把无法复现的 `~/.omega` 状态打成单个 tgz）、`cleanup` / `kill-all`、`timeline`（回放一项任务）、`resurrect`（救活一个崩掉的 oracle）、`provision`（按客户分组的凭据）。
- **端到端解决 Linear ticket。**`/omg-linear` 负责修复、留取证据、审计到 100/100、发评论，并把 ticket 挪到 review——绝不挪到 Done；那一步由人来做。见 [Linear 集成](#linear-集成)。

三个入口：`ratatui` 写的 TUI（5 个标签页：Sessions、Menu、Agentic、Settings、Help）、`omega` CLI（40+ 命令），以及 Telegram 中枢。还有一个 RPC 模式（stdin/stdout 上的 JSONL），让别的工具也能驱动它。再往下，这一切都跑在 [rmux](https://github.com/agentik-os/rmux) 上——一个 Rust 写的终端复用器，不依赖 tmux。

## doctrine

有一个类型化的注册表，包含 7 条 Law 和 47 条具名的操作 Rule。`omega rules list` 会打印当前集合。编译器就在 `crates/omega-core/src/rules.rs` 里；它产出的上下文是确定性的、按 provider 适配的，并受一条 24 KB 的 OmegaOS 硬预算约束。

**Law 不可违背。**它们约束每一个 agent，并且凌驾于每一条 rule、每一个 task 之上。一共七条：

- **L0 — Ship the truth（交付真相）。**一处改动，只有在一次干净的重新构建能复现它、并且它已被 push 之后，才算完成。差一点的都只是草稿。
- **L1 — Runtime is the only truth（运行时是唯一的真相）。**代码和注释陈述的是意图，只有真正跑起来才会显出现实。两者不一致时，运行时说了算。
- **L2 — Researcher, not sycophant（做研究者，别做马屁精）。**遇到有缺陷的前提，先用推理去质疑它，再动手。不要装出来的自信。「这应该能行」而拿不出证据，就是撒谎。
- **L3 — Decide and proceed（决断并推进）。**被派发出去的 agent 是自主的。它绝不停下来问「我该继续吗？」它自己决断、自己执行、事后再汇报。
- **L4 — Done means 100%, verified（完成意味着 100%，且经过验证）。**92% 不叫完成。把任务逐条列出来，逐条做完，逐条对着运行时验证。
- **L5 — Quality over speed（质量高于速度）。**真协议不存在精简版、轻量版或快速版。403 或 401 是中止，不是通过。
- **L6 — Finish the mission（完成整项任务）。**把每一项被要求的交付物列出来、执行、验证并汇报。计划或部分阶段不是合法的停止点。

**Rule 是操作层面的。**它们是具名的（R-SCOPE、R-VERIFY、R-CITE……），归入 Universal、QualityGate、Orchestration、Reporting、Safety 几类。每条 Rule 都按它所约束的角色来划定范围：Master、Oracle、Worker。一个 worker 不会被压上一堆它根本无从下手的编排规则，一个 oracle 也不会背上 worker 那套文件加锁的纪律。同一个注册表，切出不同的片。

### 漏斗

机制就在这里。`rules::compile_rule_context_for_provider` 组合紧凑的 Law 核心、角色契约、任务相关 Rule、provider 机制和 skill 引用。超出上下文预算的输出会被拒绝，而不是静默截断。每一份编译出来的上下文都带一个确定性摘要，用于检测漂移。

一个在这棵树里往下三层的 worker，带着的七条 Law 和最顶上的 Master 一模一样。操作规程只有在角色、任务、风险和 provider 都要求时才会被加载。这样既让那些不变量保持普适，又不必把每一份操作手册都塞进每一个回合。

看完整内容：

```
omega rules list
```

![omega rules list —— OmegaOS 输出的 Law 与 Rule](assets/omega-rules.svg)

## 架构

四层，自上而下：

```
┌─────────────────────────────────────────────────────────────────┐
│  第 1 层 — 人机界面                                             │
│  TUI (5 个标签页) · CLI (40+ 命令) · Telegram 中枢              │
│                      ↓ 意图                                     │
├─────────────────────────────────────────────────────────────────┤
│  第 2 层 — Master (常驻大脑 — Atlas 话题)                       │
│  14 个 Matrix 命名的 agent 模板, 分类 → 路由                    │
│                      ↓ 派发                                     │
├─────────────────────────────────────────────────────────────────┤
│  第 3 层 — Oracle (每个项目 1 个)                               │
│  分类 → 规划 → 派发 worker → 质量门禁                           │
│                      ↓ 拆解                                     │
├─────────────────────────────────────────────────────────────────┤
│  第 4 层 — Worker (短命, 并行, 按文件锁圈定)                    │
│  执行 → 验证 → done.json → Oracle 确认 → 关闭                   │
└─────────────────────────────────────────────────────────────────┘
```

**第 2 层 —— Master。**一个常驻的 agent，保持运行，挂了会自动重启，并续上它自己的对话。它内置 14 个以《黑客帝国》角色命名的 agent 模板（Oracle、Morpheus、Seraph、Keymaker、Smith、Niobe、Architect、Merovingian、Neo、Zion、Link、Construct、Pythia、Council）。Master 是个派发器。它只做分类，把活儿路由给各个 oracle。

**第 3 层 —— Oracle。**每个项目一个。它给请求分类、做规划、派发 worker，并在最后跑质量门禁。一个 oracle 负责编排。它自己不动项目代码，所以打分的 agent 和写代码的 agent 永远不是同一个。

**第 4 层 —— Worker。**短命的。它们并行运行，每一个都被一份文件锁声明圈定到它自己的那些文件上（通过 `fs2` 提供的建议性文件锁）——可选地还能圈进属于它自己的 git worktree。一个 worker 通过写一个 `done.json`、把 status 标成 `done_clean`、`pending` 或 `failed` 来发出完成信号；没有这个 status，它就不算完成。

### 一个任务是怎么跑起来的

一个请求可以从 TUI、CLI 或者 Telegram 进来。无论从哪里出发，它最终都会落到 Master 手里。Master 读取它、分类它，再把它路由到负责相关项目的那个 oracle。oracle 规划这项任务、把它拆成若干子任务，然后给每个子任务派发一个 worker。worker 对照真实运行时核对自己的结果，写出各自的 `done.json`；oracle 读这份文件，跑一道门禁，再把结果向上逐级汇报。

worker 并不非得一个一个地啃自己的子任务。它可以在进程内跑一个 Workflow：派生一批并行的子 agent，核对它们的输出，再把它们汇成一个答案。代码评审就是这么干的，调研、审计和设计工作也一样。

验证是刻意做成对抗式的：一个 worker 报了「done」并不会就此结束核查；它的说法要交给若干独立的 agent，只有在多数（三取二）认可时它才能存活下来。Quality Arsenal 那套审计，正是接在这里——接在门禁这一环。

这一切都依赖前面讲的那个 doctrine 漏斗：每个 agent，在每一层级，一被派发，就会立刻被注入按角色裁剪过的那套 Laws 和 Rules。

这一节 README 本身就是个例子。它是一个 Workflow 产出的。一个 agent 写了初稿，几个独立的审阅者通读了一遍，专门揪那些像 AI 写出来的句子，另一个 agent 对着他们标出来的地方做了修订，再由母语者完成翻译。所以这段文字没有哪一部分是出自某一次没人复核的单遍生成。

## 技术栈

这是一个 Rust workspace，含三个 crate：

- `omega-core` —— 编排、规则注册表、doctor、timeline、cleanup、patrol、文件范围加锁。
- `omega-cli` —— `omega` 二进制，构建在 `clap` 之上。
- `omega-tui` —— 会话管理器，构建在 `ratatui` 之上。

再往下，它跑在 [rmux](https://github.com/agentik-os/rmux) 上——一个 Rust 写的终端复用器：一个守护进程、一套类型化 SDK，外加 PTY 处理。rmux 是一个类型化的 Rust 库，所以 OmegaOS 直接调用它，而不是另起一个 tmux 进程再去解析它吐出来的文本。整个项目任何地方都没有对 tmux 的依赖。

Bun 和 TypeScript 负责 PDF 报告的渲染（经由 Next.js 和 Playwright），以及那些 Telegram 机器人。Bash 只在一个地方露面：安装的引导脚本。

## 远程连接

rmux 守护进程掌管着每一个会话，所以你断开连接之后，你的 agent 还在继续跑。想回到它们那儿，就 **attach**——把你的终端重新连回一个已经在跑的会话：

```
rmux attach              # 重新连回上一个会话
rmux attach -t claude-1  # 连到指定的那一个
rmux list-sessions       # 看看有哪些是活的
```

再用 `Ctrl-b d` detach 出来——会话和它里面的 agent 会在没有你的情况下继续跑。

`omega` 把你真正会伸手去用的那几个入口包了起来：

```
omega                       # 打开 TUI 会话管理器 (浏览 / 启动 / 监控)
omega attach -t claude-1    # 直接钻进某个会话里干活
omega master                # 跳到 Master 会话
omega list                  # 列出每一个活着的会话
```

用菜单（`omega`）来管理和启动；当你想埋头在某一个会话里敲键盘时，就用直接 attach（`omega attach -t …`，或者 `rmux attach -t …`）——菜单里的预览是把窗格*镜像*出来，而直接 attach 是延迟最低的那条路。

从笔记本用 SSH 连过去时，普通 SSH 每敲一个键都要等一整个网络往返才会回显，所以在一台远处的机器上打字会觉得卡，agent 的输出也是一块一块地到——不管那台机器多快都一样，因为这是延迟，不是 CPU。`install.sh` 就是为此装上 [`mosh`](https://mosh.org) 的：它在本地回显你的按键，并通过 UDP 传屏幕差分，所以无论延迟多大，打字都是即时的，流式输出也很顺。直接连进某个会话：

```
mosh user@host -- omega attach -t claude-1
```

在 **Termius** 这类客户端里：填上主机 IP + 端口，把 **mosh** 开关打开，再加一段启动片段——`omega` 进菜单，或者 `omega attach -t <session>` 直接落到某个会话里。

（往回翻屏请用 rmux 的 `Alt+Up/Down`，不是 mosh 的 PageUp。）安装器还会把 `/etc/rmux.conf` 和一套 UTF-8 locale 配到系统层面，所以每一个账户——root 以及以后新建的用户——都能拿到同样一套加固过的会话（鼠标滚动、通过 SSH 拖选到本地剪贴板、按键跟手、truecolor），不需要任何按用户的额外配置。

## Linear 集成

如果你用 [Linear](https://linear.app) 来跟踪用户反馈，OmegaOS 可以端到端把这些 ticket 解决掉。两条命令。

`/omg-linear-setup` 是一次性的向导，在你自己的 app 里跑。它会装上一个应用内的反馈 widget（提交那一刻会抓取截图、页面 URL、被点击的元素，以及浏览器 console）、流水线所依赖的那几个 Linear 标签，还有那条把 widget 提交转成 Linear issue 的 API 路由。它会先探测你的技术栈、auth 提供方和 UI 库，所以写出来的代码是贴合这个项目的，而不是一份通用模板。

`/omg-linear` 干正事。它读取未关闭的 ticket，对每一个都去改代码、留下改动前后的证据，然后跑与这次改动相匹配的那些 Quality Arsenal 审计。只有当这些审计打到 100/100，一个 ticket 才会往前走。接着它会在 ticket 上贴一条修复验证评论，并把它挪到一个 review 状态——如果你的团队有 `In Review` 就用它，否则就用它自己创建的一个中性的 `Omega Review`。它绝不会把 ticket 标成 Done；那一步由人在核对之后来做。v2 引擎通过一个 Workflow 来跑这套流程：先给未关闭的 ticket 做分诊，再把每个 ticket 的修复与审计并行铺开，并在评论之前对每一份解决方案做对抗式验证。

它带触发守卫。只有当你点名叫它的时候（`/omg-linear`、`fix linear`、像 `KOM-7` 这样的 ticket id，或者一个 `linear.app` 链接），OmegaOS 才会去动 Linear。光是「feedback」这个词永远不会触发它；你不提 Linear，它也不会提。

```
omega_dir=~/.omega          # 协议会安装到 ~/.omega/skills/linear/
/omg-linear-setup           # 每个 app 一次 — 装上 widget + 标签 + 路由
/omg-linear                 # 解决未关闭的 ticket: 修复 -> 审计 -> 评论 -> In Review
```

## 局限

这些事，我宁愿你进来之前就知道。

- **Linux 优先。**在一台无头 VPS 上开发。没有 Windows。macOS 能拿到真正的修复（launchd 服务、Homebrew 路径），但被实战检验得少一些。
- TUI 假定终端支持 256 色。在 16 色终端上它会很丑。
- 默认的 agent 运行时是 OpenAI Codex，因此 `codex` CLI 必须已登录。Claude Code、Gemini、Pi、Hermes 和 GLM 是受支持的显式备选项。
- **单机。**rmux 守护进程是本地的。没有跨主机的编排。
- 这是 0.1.x。我每天都用，但你会撞上一些我还没撞过的粗糙地方。

## 接下来读 GUIDE.md

**[GUIDE.md](GUIDE.md)** 是操作员手册：术语（mission、oracle、worker、goal、plan、Atlas）、三个座舱、你的头几项任务、skill 目录，以及活儿是怎么被验证的。然后再往深处走：

- [docs/FEATURES.md](docs/FEATURES.md) —— **完整的功能目录**（每一个子系统 + 怎么用到它）。
- [docs/README.md](docs/README.md) —— 文档索引。
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) —— 全系统参考。
- [docs/MAP.md](docs/MAP.md) —— 每样东西在磁盘上的位置。
- [docs/THEMES.md](docs/THEMES.md) —— TUI 配色画廊。
- [docs/RESET-RECOVERY.md](docs/RESET-RECOVERY.md) —— 备份并重建一台机器。
- [CHANGELOG.md](CHANGELOG.md) —— 一个版本一个版本地看都发布了什么。

## 致谢

OmegaOS 建立在许多其他人的工作之上：

最大的一笔债欠给 [rmux](https://github.com/agentik-os/rmux)——这里的一切都跑在这个 Rust 终端复用器上。

Rust 技术栈的其余部分：

- [ratatui](https://github.com/ratatui/ratatui) 和 [crossterm](https://github.com/crossterm-rs/crossterm) —— TUI。
- [tokio](https://github.com/tokio-rs/tokio) —— 异步运行时。
- [clap](https://github.com/clap-rs/clap) 和 `clap_complete` —— CLI 与 shell 补全。
- [serde](https://github.com/serde-rs/serde)，配合 `serde_json`、`serde_yaml` 和 `toml` —— 配置与状态。
- [anyhow](https://github.com/dtolnay/anyhow) 和 [thiserror](https://github.com/dtolnay/thiserror) —— 错误处理。
- `chrono`（时间戳）、`dirs`（路径）、`fs2`（范围声明背后的那些建议性文件锁）、`regex`、`tempfile`、`tracing` 配合 `tracing-subscriber`（日志），以及 `reqwest`（Telegram 与 PDF 的 HTTP）。

Anthropic 出品的 [Claude Code](https://www.anthropic.com) 是 agent 运行时。

## 许可证

双许可，[MIT](LICENSE-MIT) 与 [Apache-2.0](LICENSE-APACHE) 任选其一，悉听尊便。标准的 Rust 惯例。挑你顺眼的那个就行。
