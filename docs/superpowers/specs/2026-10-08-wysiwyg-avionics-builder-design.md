# WYSIWYG Avionics Builder: Design Spec

- **Date:** 2026-10-08
- **Status:** Approved for implementation
- **License:** MIT OR Apache-2.0
- **Repo:** github.com/Vihaan2012-cmyk/WYSIWYG-Avionics-Development-Framework-For-FS2020-24

## 1. Purpose

WYSIWYG Avionics Builder is a desktop app for building MSFS 2020/2024 glass-cockpit displays visually. It covers PFD, ND, ECAM/EWD/SD and EFB-style touch screens.

The aircraft developer declares displays in `avdev.json` at the aircraft repo root. The app then works as follows:

1. It scans the aircraft repo. It finds every variable (L:, A:, E:, H:, K:, EventBus topics, Coherent events) and how the aircraft names, types and accesses them. It also finds existing displays in `panel.cfg`.
2. The developer draws each display on a canvas, using nested layers, shapes, imported SVGs and reusable components.
3. The developer binds any property to variables. A binding can be written in three ways: preset actions, an expression language, or a node graph.
4. The developer defines states, state machines and pages that overlay or switch content (for example Direct Law or Approach).
5. Every edit is written immediately to a readable TSX source file (MSFS Avionics Framework). That file is the source of truth. Developers can also edit it by hand, in the built-in editor or in VS Code, and the canvas follows.
6. Compiling has three steps:
   - A strict, rustc-style checker runs. It reports error codes, exact source spans, help text and one-click fixes.
   - Mach bundles the result.
   - The app writes a complete, ready-to-open FlyByWire ACE project.
7. Displays can be previewed live inside the app. The preview is a superset of ACE: variable panel, scenario timeline, multi-display workspace and a live link to the running simulator.

**Non-goals for this release:**
- Writing the MSFS simulator package (html_ui/VCockpit + panel.cfg edits) automatically. The output is ACE-only. The generated bundle still boots in the sim if a developer wires it manually (§9.4).
- Real-time multi-user editing. Collaboration is through git.
- Visual diff of display versions. Diffs are text only.

## 2. Decisions record

| Topic | Decision |
|---|---|
| Relationship to ACE | Built-in ACE-style preview, plus export of a full FBW ACE project (flybywiresim/ace master) |
| Simulators | MSFS 2020 and 2024 |
| Platforms | Windows and Linux |
| Audience | Public open source, MIT OR Apache-2.0 |
| App shell | Tauri 2 desktop app. Rust core with a React 19 + TypeScript UI. Not a browser/localhost app |
| Generated code | MSFS Avionics Framework (`@microsoft/msfs-sdk`, FSComponent) TSX in the "AVD dialect" (§6) |
| Source of truth | The TSX file. Canvas edits are written to it instantly (debounced ~100 ms). Hand edits round-trip to the canvas. Code outside the dialect is preserved as opaque code blocks |
| Code editing | Built-in Monaco split view, plus external editors through file watching |
| `avdev.json` | One per aircraft, at the repo root |
| Display fields | id, name, width, height, interactive, power/brightness var, refresh rate, sim gauge mapping |
| Design files | One TSX file per display under `avdev/displays/` |
| `panel.cfg` | Scanned. Found displays are suggested for `avdev.json` |
| Scanner | All file types: TS/JS/TSX/JSX, XML behaviors, C/C++/Rust WASM sources, cfg/flt, and best-effort strings from compiled binaries. Infers naming, unit and access-pattern conventions. Metadata is inferred and editable. File watcher plus a rescan button |
| ARINC 429 | First-class type with SSM decoding |
| Inputs | L:, A:, E:, H:, K:, EventBus topics, Coherent events |
| Shapes | Basics, arcs, bezier paths, images, gradients, dashes, boolean ops, glow/shadow |
| Import | SVG import to editable shapes, plus a tracing reference image |
| Layers | Nested, with clip masks |
| Components | Project-wide library, shared across displays |
| Repeaters | Yes, with visible-window culling |
| Bindings | Presets, expressions and node graph, all in v1 |
| Text | Number formatting, conditional text, rolling drum digits, unit conversion |
| Timing | Smoothing and rate limiting, flashing, timed sequences, latches and delays |
| Writes | Displays can write variables and fire events. Developers can create new variables |
| States | Conditional overlays plus optional explicit state machines. Priority order, with a warning on overlap. States can be used as booleans |
| Pages | A separate concept from states |
| Interactive UI | v1 includes buttons, toggles, scroll lists, text input and a keyboard |
| Preview | Variable panel, scenario timeline, live sim link, multi-display workspace |
| Fonts | Project-wide. CSS is generated automatically. A missing font is an error |
| Colours | Named palette, plus alternate palettes |
| Checker | rustc-level: codes, spans, help, quick fixes, configurable lints, strict typing including units |
| Runtime perf | Change-driven updates, capped at each display's refresh rate |
| Export | A full ACE project folder |
| Toolchain | The user links an existing ACE install and Mach install (Node + Mach). The Windows installer is Inno Setup with a link page; Linux uses a first-run link screen |
| Git | Files live in the aircraft repo. Built-in git UI (status, diff, stage, commit, branch, log, push, pull) |
| Build strategy | One complete spec and plan, then one continuous build |
| UI look | Matches the ACE2 screenshot: dark navy, cyan accent, rounded cards, mono font for data, left icon rail, tab bar |
| Test aircraft | FBW A380X (flybywiresim/aircraft, `fbw-a380x` + `fbw-common`), plus a synthetic fixture aircraft |

## 3. Architecture

```
aircraft repo (e.g. fbw-a380x)                  WYSIWYG Avionics Builder (Tauri 2)
├─ avdev.json                         ┌───────────────── Rust core (crates/) ──────────────────┐
├─ avdev/                             │ avd-core     ids, spans, Diagnostic, units, var refs    │
│  ├─ displays/<Id>.tsx  ◄───────────►│ avd-project  avdev.json schema, layout, app config      │
│  ├─ components/<Name>.tsx           │ avd-scan     scanner, VarDb, conventions, panel.cfg     │
│  ├─ logic/<name>.ts                 │ avd-expr     expression + action language               │
│  ├─ theme.ts  vars.json             │ avd-syntax   oxc TSX ⇄ dialect model, span splicing     │
│  ├─ scenarios/  editor.json         │ avd-model    document model, semantic commands, undo    │
│  ├─ workspace.json  tsconfig.json   │ avd-check    resolver, type/unit checker, lints, fixes  │
│  ├─ fonts/  images/  reference/     │ avd-build    entries, mach config, node runner, ACE out │
│  ├─ .avd/ (gitignored cache/build)  │ avd-svg      SVG import, boolean ops                    │
│  └─ out/ace-project/ ◄──────────────│ avd-sim      SimBridge: SimConnect FFI, relay, mock     │
├─ src/**, *.xml, *.cfg ─────────────►│ avd-git      git2 + git CLI                             │
                                      │ avd-app      Tauri commands/events (src-tauri)          │
                                      └──────────────────────▲─────────────────────────────────┘
                                                  invoke / Channel / emit
                                      ┌──────────────────────▼─────────────────────────────────┐
                                      │ packages/ui  React 19 + Vite + Tailwind 4 + zustand     │
                                      │   canvas overlay · layers · inspector · Monaco · vars   │
                                      │   states/pages · node graph (@xyflow) · timeline · git  │
                                      │   preview iframes ← avd-preview:// scheme               │
                                      │ packages/shim       ACE-compatible host (superset)      │
                                      │ packages/runtime    @avd/runtime (ships in output)      │
                                      │ packages/mach-plugin esbuild resolve + NDJSON reporter  │
                                      └────────────────────────────────────────────────────────┘
```

### 3.1 Repository layout (this repo)

```
Cargo.toml                    # workspace
rust-toolchain.toml           # channel = "stable" (Windows devs: rustup override to MSVC host)
crates/avd-core  avd-project  avd-scan  avd-expr  avd-syntax  avd-model
       avd-check avd-build    avd-svg   avd-sim   avd-git
src-tauri/                    # avd-app (Tauri 2.12)
packages/runtime/             # @avd/runtime (TS, built with tsup → dist/)
packages/shim/                # @avd/shim (preview host shim)
packages/mach-plugin/         # @avd/mach-plugin (CommonJS esbuild plugin)
packages/ui/                  # React app (Vite 8)
spec/expr-conformance/*.json  # shared Rust/TS expression test corpus
fixtures/sample-aircraft/     # synthetic aircraft covering every scanner pattern
installer/windows/avd.iss     # Inno Setup 7.1 script
docs/diagnostics/AVxxxx.md    # one explanation page per diagnostic code (embedded in app)
package.json                  # npm workspaces: packages/*
```

### 3.2 Pinned key dependencies

**Rust:**
- tauri 2.12, tauri-build 2.7, tauri-plugin-dialog 2.8
- oxc_allocator, oxc_parser, oxc_ast, oxc_ast_visit, oxc_span, all pinned to `=0.153.0`
- notify 8.2 + notify-debouncer-full 0.7
- git2 0.21
- serde, serde_json, schemars (JSON Schema for avdev.json)
- usvg (SVG import), i_overlay (boolean ops)
- libloading (SimConnect FFI), tokio, thiserror
- insta (snapshot tests)

**TypeScript:**
- React 19, Vite 8, Tailwind 4 (`@tailwindcss/vite`), zustand 5, lucide-react
- @monaco-editor/react 4.7 + monaco-editor 0.57, loaded locally with `loader.config({ monaco })` and no CDN
- @xyflow/react 12
- vitest

**Runtime package:**
- `@microsoft/msfs-sdk`, pinned to the version the A380X uses (read from `fbw-a380x`/root `package.json` at implementation time and recorded in `packages/runtime/package.json`)

### 3.3 Edit data flow

1. A UI gesture (drag, inspector change, bind, add state) becomes a semantic `Command`, for example `SetProp { el, prop, value }`.
2. `avd-model` applies the command to the in-memory model. The command is id-anchored, so it can be inverted for undo.
3. `avd-syntax` turns the command into minimal text splices on the owning `.tsx` file. Original formatting and comments are kept everywhere else. The file is re-parsed to validate the result.
4. The file is written to disk, debounced at 100 ms. The watcher ignores self-writes by comparing content hashes.
5. The `mach watch` preview process rebuilds. The `@avd/mach-plugin` reporter emits an NDJSON result line, and the preview iframe hot-reloads. Simulator state lives in the host shim, so it survives the reload.
6. **Fast path:** for static property edits, the UI also calls `window.__avd.patch(elId, props)` in the preview iframe. The change shows within the same frame, before the rebuild lands.
7. **External edits** (VS Code, git checkout):
   - The watcher triggers a re-parse, then a model diff, then a UI update.
   - The external edit becomes an undo boundary.
   - Undo/redo commands re-locate their targets by element id. They do not rely on stale spans.

While dragging, the canvas overlay moves a ghost of the selection at 60 fps. The command is committed on pointer-up.

## 4. Files in the aircraft repo

### 4.1 `avdev.json` (written by the aircraft developer; JSON Schema published by the app)

```json
{
  "$schema": "https://raw.githubusercontent.com/Vihaan2012-cmyk/WYSIWYG-Avionics-Development-Framework-For-FS2020-24/main/schema/avdev.schema.json",
  "version": 1,
  "aircraft": { "name": "A380X", "title": "FlyByWire A380-842" },
  "paths": {
    "root": "avdev",
    "scan": ["."],
    "scanExclude": ["**/node_modules/**", "**/out/**", "**/.git/**", "avdev/**"],
    "output": "avdev/out/ace-project"
  },
  "sdk": "bundled",
  "displays": [
    {
      "id": "PFD_1",
      "name": "Captain PFD",
      "width": 768,
      "height": 1024,
      "interactive": false,
      "refreshHz": 30,
      "power": { "on": "L:A380X_PFD_1_POWERED@bool", "brightness": "L:A380X_PFD_1_BRIGHTNESS@number" },
      "simGauge": { "panelCfg": "src/base/.../panel/panel.cfg", "section": "VCockpit03", "gauge": "htmlgauge00" }
    }
  ],
  "lints": { "AV5003": "deny", "perf": "warn" },
  "conventions": { "lvarPrefix": "A380X_" }
}
```

Field rules:

**`displays[].id`**
- Matches `^[A-Za-z][A-Za-z0-9_]*$` and is unique.
- It is used as the TSX file name, the ACE instrument folder name, the ACE `config.name` and the ACE canvas title. ACE requires `config.name` to equal the folder name.

**`width` and `height`**
- Integers, 16 to 8192.

**`interactive`**
- Defaults to `false`.
- When `true`, the display may contain widgets.
- When `false`, a widget raises AV3101.

**`refreshHz`**
- Optional, defaults to 30, range 1 to 120.
- Caps how often the runtime evaluates the display.

**`power`**
- Optional.
- `on` is a bool expression. When it is false the runtime blanks the display: the root is hidden and evaluation is skipped.
- `brightness` is a 0..1 number that becomes the root opacity.
- Values are expression-language strings (§7).

**`simGauge`**
- Optional, informational.
- Used for `panel.cfg` cross-checks (AV4201 when the size disagrees) and for the "Suggested displays" UI.

**`sdk`**
- `"bundled"` (default) or `"aircraft"`. Selects which `@microsoft/msfs-sdk` copy is bundled (§9.2).

**`lints`**
- Maps a code (`AV5003`) or a group (`perf`, `style`, `states`, `units`, `assets`) to `allow`, `warn`, `deny` or `forbid` (§8.4).

**`conventions`**
- Optional overrides for inferred conventions (§5.4).

Unknown keys raise AV0901 (warning).

### 4.2 `avdev/` (written by the app; everything except `.avd/` and `out/` is committed)

| Path | Content |
|---|---|
| `displays/<Id>.tsx` | Display source in the AVD dialect (§6) |
| `components/<Name>.tsx` | Shared components (§6.6) |
| `logic/<name>.ts` | Code blocks: exported, type-annotated functions callable from expressions (§7.6) |
| `theme.ts` | Fonts, palette, alternate palettes (§6.9) |
| `vars.json` | Variable metadata overrides and app-created variables (§5.5) |
| `editor.json` | Editor-only state per display: guides, grid, tracing image path/opacity/offset, locked/hidden-in-editor ids |
| `workspace.json` | Preview workspace layout: placed displays, positions, zoom |
| `scenarios/<name>.json` | Preview scenarios (§10.3) |
| `fonts/`, `images/`, `reference/` | Assets (developer-provided fonts, imported raster images, tracing images) |
| `tsconfig.json` | App-owned: `jsx: "react"`, `jsxFactory: "FSComponent.buildComponent"`, `jsxFragmentFactory: "FSComponent.Fragment"`, `paths` for `@avd/runtime`, `strict: true` |
| `.gitignore` | `.avd/` and `out/` |
| `.avd/` | Scan index cache, build dir (generated entries, mach config, bundles), logs |
| `out/ace-project/` | Compiled ACE project (§9.3) |

## 5. Scanner (`avd-scan`)

### 5.1 Pipeline

1. **File walk.** Walk the `paths.scan` roots with `ignore`-crate semantics, honouring `.gitignore` and `scanExclude`.
2. **Classify each file by extension:**
   - `ts, tsx, js, jsx, mjs, cjs`: script
   - `xml`: behavior
   - `cpp, cc, c, h, hpp`: C/C++
   - `rs`: Rust
   - `cfg`: cfg
   - `flt`: flight file
   - `wasm`, and `.js` larger than 1 MB or minified: binary
3. **Extract occurrences.** Run the extractor for the file's kind (§5.2) to produce `Occurrence { var: VarKey, unit: Option<String>, access: Read|Write|Fire|Subscribe|Publish, pattern: AccessPatternId, file, span, confidence }`.
4. **Aggregate** the occurrences into the `VarDb` (§5.3).
5. **Infer conventions** (§5.4).
6. **Parse `panel.cfg`** files into `DisplaySuggestion`s (§5.6).
7. **Persist** the index to `avdev/.avd/scan-index.json`. The index is keyed by file path, mtime and hash, so a rescan only re-reads changed files.
8. **Watch.** notify-debouncer-full watches the scan roots with a 300 ms debounce. Changes are re-extracted incrementally and a `scan:updated` event is emitted.

### 5.2 Extraction patterns

The patterns below were verified against the FBW A380X source (flybywiresim/aircraft `master`, 2026-10).

**Pre-processing:**
- XML is entity-unescaped first (`&gt;`, `&lt;`, `&amp;`, `&quot;`).
- In JS/TS, string literals and template literals are tokenised with a lightweight lexer, so that comments are skipped.

**Templates:** names containing `#index#`, `#ID#`, `#PARAM#`, `${...}` or Rust `format!` `{}` are templates.
- They are expanded when the parameter set is finite and statically visible:
  - SimVarPublisher `indexed: true` expands to `_1`, `_2`, `_3`, from the `` `${T}_${1 | 2 | 3}` `` key type when present, otherwise `1..4`;
  - XML `<UseTemplate>` parameters;
  - Rust `for`/`map` over literal arrays.
- Otherwise they are stored as `TemplateVar { pattern, sites }` and matched by glob in autocomplete (e.g. `A380X_RMP_*_VHF_TX_*`).
- Fully dynamic names, such as a variable passed as the name argument, go to an "unresolved" bucket with their site, for display.

| Id | Language | Pattern (regex, after pre-processing) | Yields |
|---|---|---|---|
| `ts.simvar` | TS/JS | `SimVar\.(Get\|Set)(SimVar\|GlobalVar\|GameVar)Value\(\s*([`'"])(.+?)\3\s*,\s*([`'"][^`'"]+[`'"]\|SimVarValueType\.\w+\|'[^']+'\s+as\s+SimVarValueType)` | name, unit, read/write; prefix `L:/A:/E:/H:/K:`; no prefix means `A:` |
| `ts.registered` | TS/JS | `RegisteredSimVar\.create\w*(<[^>]*>)?\(\s*([`'"])([^`'"]+)\2(\s*,\s*([^)]+))?` | name, unit, read |
| `ts.publisher` | TS/JS | Map entries `\[\s*'(\w+)'\s*,\s*\{\s*name:\s*([^,]+?),\s*type:\s*([^,}]+)(.*?indexed:\s*true)?` inside `new Map<...>(...)`. `name` may be a string literal or `EnumName.member`, resolved via `export enum \w+ \{ member = 'L:...' \}` in the same or an imported file | topic → var + unit (+ indexed); sets `bus_alias` |
| `ts.bus.on` | TS/JS | `\.on\(\s*'(\w+)'\s*\)`, with `getSubscriber<(\w+)>` / `getArincSubscriber<(\w+)>` in the same file | topic subscribe; ARINC hint when `getArincSubscriber` or `Arinc429*ConsumerSubject.create(` wraps it |
| `ts.bus.pub` | TS/JS | `\.pub\(\s*'(\w+)'` | topic publish (derived topics with no var) |
| `ts.hooks` | TS/JS | `use(Split\|Interaction)?SimVar\(\s*(['"`])(.+?)\2\s*,\s*(['"`])(.+?)\4`; `useGlobalVar\(` same shape; `useInteractionSimVar` 3rd argument is an H: event | name, unit, read/write |
| `ts.arinc` | TS/JS | `useArinc429Var\(\s*(['"`])(.+?)\1`, `Arinc429Word\.fromSimVarValue\(\s*(['"`])(.+?)\1`, `\.(setFromSimVar\|writeToSimVar)\(\s*(['"`])(.+?)\2`, `Arinc429LocalVarOutputWord\(\s*(['"`])([^'"`]+)\1`, `Arinc429Word\.toSimVarValue\(\s*(['"`])(.+?)\1` | ARINC var (type ⇒ `arinc429`) |
| `ts.coherent` | TS/JS | `Coherent\.(call\|trigger\|on)\(\s*['"`]([^'"`]+)` | `C:` event, call/trigger/subscribe |
| `ts.literal` | TS/JS | fallback: `['"`]([LAEKHO]):([A-Za-z0-9_ .:#${}]+)['"`]` not already matched | name, confidence 0.5 |
| `xml.rpn` | XML | `\((>?)([LAEKHOFBPRSC]):([^,)\s][^,)]*?)(?:\s*,\s*([^)]+))?\)` | `>` means write/fire; `O:`, `F:`, `P:`, `R:`, `S:`, `C:` (component-local, function and other RPN namespaces) are ignored except `B:` (input events, recorded as kind B) |
| `xml.tagvar` | XML | `<(\w*VAR\w*)>\s*([LA]):([^<]+)</\1>` and template parameter defaults | name |
| `cpp.raw` | C/C++ | `register_named_variable\(\s*"([^"]+)"`; `make_unique<LocalVariable>\(\s*"([^"]+)"`; `(get\|set)_named_variable_value` (direction, via id-variable link best-effort) | full L: name |
| `cpp.framework` | C/C++ | `make_named_var\(\s*"([^"]+)"\s*(?:,\s*UNITS\.(\w+))?`; prefix from `MsfsHandler\s+\w+\(\s*"[^"]*"\s*,\s*"([^"]+)"\)` in the same module tree (e.g. `A32NX_`) | prefixed L: name, unit |
| `cpp.events` | C/C++ | `make_(sim_event\|client_event)\(\s*"([^"]+)"`; `MapClientEventToSimEvent\([^,]+,\s*[^,]+,\s*"([^"]+)"` | K: events |
| `cpp.calc` | C/C++ | `execute_calculator_code\(\s*"([^"]+)"`, with the string re-scanned with `xml.rpn` | vars/events |
| `rs.identifier` | Rust | `get_(unprefixed_)?identifier\(\s*(?:&?format!\()?\s*"([^"]+)"`; prefix from `key_prefix\s*=\s*"([^"]+)"` in the matching `*_wasm` crate (`get_unprefixed` skips the prefix) | L: name |
| `rs.variable` | Rust | `Variable::named\(\s*(?:&?format!\()?\s*"([^"]+)"` (prefixed); `Variable::aircraft\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*([^)]+)\)` (A: var + unit + index); `Variable::aspect` ignored (internal) | name, unit |
| `rs.arinc` | Rust | `write_arinc429\(\s*&?(self\.)?(\w+)` linked back to the field's `get_identifier` | ARINC var |
| `cfg.panel` | cfg | §5.6 | displays |
| `cfg.generic` | cfg / flt | `\b([LA]):([A-Za-z0-9_ :]+?)(?:,\s*([\w ]+))?(?=[)\s,;]\|$)` in values | name, unit (confidence 0.7) |
| `bin.strings` | wasm / large js | printable runs ≥ 6 bytes matching `^(L:)?[A-Z][A-Z0-9]*_[A-Z0-9_]{3,}$`, kept only when the prefix matches a known convention prefix | name (confidence 0.3, read/write unknown) |

**Normalisation:**
- Names of kinds L, A, E, H and K are compared case-insensitively, by upper-casing. B and C are case-sensitive.
- A trailing `:N` on A:/E: names is the SimVar index.
- Unit strings are mapped through the units table (§7.2): `SimVarValueType.Bool` becomes `bool`, `'Degree'` becomes `degrees`, `NM` becomes `nautical miles`, `Knots` becomes `knots`, and so on.

H: events arrive on the msfs-sdk bus as `hEvent` strings without the `H:` prefix. Strings compared against `hEvent` (`eventName === 'A32NX_...'`, `.startsWith('A32NX_KCCU_L')`) are recorded as H: subscriptions.

### 5.3 VarDb

```rust
struct VarRecord {
  key: VarKey,                 // { kind: L|A|E|H|K|B(bus topic)|C(coherent), name, index: Option<u32> }
  ty: VarType,                 // Number | Bool | Enum{values} | Arinc429{inner: Number|Bool|Discrete} | String
  units: Vec<(String, u32)>,   // unit → occurrence count; first = canonical
  access: AccessSet,           // read / write / fire / subscribe / publish seen
  sites: Vec<Site>,            // file + span + pattern id (capped at 200 per var, count kept)
  bus_alias: Option<String>,   // EventBus topic ↔ underlying SimVar mapping from SimVarPublisher defs
  origin: Scanned | AppCreated,
  meta: VarMeta,               // range{min,max}, default, description, enum labels, display unit, tags
}
```

How `ty` is inferred, in priority order:
1. An explicit override in `vars.json`.
2. ARINC usage, such as wrapper classes or `.ssm` access (§5.2).
3. Units: `bool`/`boolean` gives Bool, `enum` gives Enum, `string` gives String.
4. Numeric otherwise.

Range is inferred when the code clamps or compares against literals. Without that, the range comes from the unit's defaults (degrees 0..360, percent 0..100, bool 0..1). If none applies, the range is left unset.

### 5.4 Conventions

Inferred from the VarDb and stored in `scan-index.json`. Visible and overridable in Settings → Conventions and in `avdev.json` `conventions`.

| Convention | How it is inferred | How it is used |
|---|---|---|
| `lvarPrefix` | Most frequent leading token of L: names, with secondary prefixes ranked. Full names come from TS/XML/raw C++ only; prefixed sources (C++ framework, Rust) contribute after their runtime prefix is applied. For the A380X this infers primary `A32NX_` (~1660 TS uses) and secondary `A380X_` (newer A380-only hardware) | Default prefix for app-created vars (developer can choose primary or secondary per var); name suggestions |
| `case` | SCREAMING_SNAKE / snake / Pascal / mixed, by majority | Validating new var names (AV1301 lint) |
| `indexStyle` | `_1`/`_2` suffix vs `:1` SimVar index | Name suggestions, side-duplicated components |
| Unit per var | Majority unit across occurrences | Default `@unit` when an expression omits it |
| Access pattern (TS) | Majority among: SimVar API, msfs-sdk SimVarPublisher+EventBus, FBW React hooks | Reported in UI; generated code always uses the runtime's batched reader (§9.1), which is compatible with all of them |
| ARINC encoding | FBW f64-packed word (§5.7) | Runtime decode |

### 5.5 `vars.json`

```json
{
  "version": 1,
  "vars": {
    "L:A380X_FCS_DIRECT_LAW": { "type": "bool", "description": "Direct law active" },
    "L:A380X_SD_PAGE": { "type": "enum", "values": { "0": "ENG", "1": "BLEED", "2": "PRESS" } },
    "L:A380X_ADIRS_ADR_1_COMPUTED_AIRSPEED": { "type": "arinc429", "unit": "knots", "range": [30, 520] }
  },
  "created": {
    "L:A380X_AVD_PFD_TEST": { "type": "number", "unit": "number", "default": 0, "description": "Created in WYSIWYG Avionics Builder" }
  }
}
```

A var in `created` that later appears in the scan is merged, and the UI offers to drop the `created` entry. A var that disappears from the scan while still bound raises AV1102 (error), with a fix that recreates it under `created`.

### 5.6 `panel.cfg` discovery

The parser handles:
- `[VCockpitNN]` sections with `size_mm`, `pixel_size`, `texture`;
- `htmlgaugeNN=<path>,x,y,w,h` lines, with `;` comments.

Each `htmlgauge` line gives `DisplaySuggestion { name: last path segment of the gauge folder, width: w, height: h, section, gauge }`. Gauges already mapped by a display's `simGauge` are hidden. The UI presents suggestions with checkboxes. Accepting one inserts a display entry into `avdev.json` using a span-preserving JSON edit, so formatting is kept.

### 5.7 ARINC 429 encoding

This is FBW's encoding, implemented identically in `fbw-common` TS `arinc429.ts`, C++ `Arinc429.cpp` and Rust `shared/arinc429.rs`.

**Packing:**
- One LVAR (an f64) holds an unsigned integer `raw = SSM * 2^32 + float32_bits(value)`.
- Bits 0–31 hold the IEEE-754 float32 bit pattern of the value. Bits 32–33 hold the SSM.
- The packing is exact because `raw < 2^34`.

**SSM values:**

| SSM | Code |
|---|---|
| `FailureWarning` (FW) | `0b00` |
| `NoComputedData` (NCD) | `0b01` |
| `FunctionalTest` (FT) | `0b10` |
| `NormalOperation` (NO) | `0b11` |

**Decode:**
```
u32 = raw mod 2^32
ssm = trunc(raw / 2^32) & 3
value = f32_from_bits(u32)
```

**Validity:**
- `isNormal` ⇔ SSM is NO.
- `valueOr(d)` returns `value` when SSM is NO or FT, and `d` otherwise. This matches FBW's `valueOr`.
- `isFailure` ⇔ SSM is FW. `isNcd` ⇔ SSM is NCD. `isFt` ⇔ SSM is FT.

**Discrete words:** `bit(n)` is **1-indexed**: `((trunc(value) >> (n-1)) & 1) != 0`, the same as FBW's `bitValue`.

**Encode** (used by the preview shim and by `set()` on an ARINC var):
- `raw = float32_bits(value) + ssm * 2^32`.
- Written with `SimVar.SetSimVarValue(name, 'string', raw.toString())`, exactly as FBW does, to avoid float rounding through the unit system.

**Detection:** names alone do not reveal ARINC vars. A var is ARINC if any of these hold:
- it has a `ts.arinc` or `rs.arinc` occurrence;
- its bus topic is consumed through `getArincSubscriber`, `Arinc429ConsumerSubject` or `Arinc429LocalVarConsumerSubject`;
- its publisher topic ends in `Raw`;
- it is overridden in `vars.json`.

## 6. The AVD dialect (display source format)

### 6.1 Principles

- A display file is valid TSX that compiles with esbuild, using `jsxFactory = FSComponent.buildComponent`.
- **Dialect elements.** Every JSX element whose tag is imported from `@avd/runtime`, or is a component from `avdev/components`, is a *dialect element*. It maps 1:1 to an editor object.
- **Opaque code blocks.** Any other JSX element, any non-dialect attribute value, and any statement outside the `export default` display tree is preserved byte-for-byte.
  - The canvas shows the element at runtime, as rendered in preview, with a "code block" badge.
  - It can be selected, moved and deleted as a unit through its wrapping `<Code id>` element, which the app inserts around foreign JSX when it is first seen. Its inside is not visually editable.
- **Element ids.** Every dialect element has a string-literal `id` that is unique within the file (AV0103 on a duplicate).
  - The app generates ids as `<kind>_<4 hex>`; developers can rename them.
  - The id anchors commands, diagnostics, state overrides and `data-avd-id`.
- **Allowed attribute value shapes.** Anything else is an opaque value (AV9001 note), shown read-only in the inspector:
  - number literal, string literal, boolean, array literal of literals, object literal of literals;
  - `ex` tagged template (expression, §7);
  - `act` tagged template (action, §7.5);
  - a reference to an imported logic function.
- **Canonical formatting** of app-inserted code is 2-space indent, single quotes and attributes on one line until 100 columns, after which there is one attribute per line. Existing code is never reformatted.

### 6.2 File shape

```tsx
/** @jsx FSComponent.buildComponent */
/** @jsxFrag FSComponent.Fragment */
import { FSComponent } from '@microsoft/msfs-sdk';
import { Display, Page, Layer, Group, Rect, Text, Line, Arc, Path, Repeat, State, Override, Hide } from '@avd/runtime';
import { ex, act } from '@avd/runtime';
import { SpeedTape } from '../components/SpeedTape';

export default (
  <Display id="PFD_1" width={768} height={1024} background="$black" font="Ecam">
    <Page id="main" default>
      <Layer id="horizon" name="Horizon">
        <Group id="attitude" rotate={ex`-A:'PLANE BANK DEGREES'@degrees`} origin={[384, 400]}>
          <Rect id="sky" x={-616} y={-600} width={2000} height={1000} fill="$sky"
                translateY={ex`A:'PLANE PITCH DEGREES'@degrees * 8`} />
        </Group>
      </Layer>
      <Layer id="spdLayer" name="Speed tape" clip={{ x: 20, y: 180, width: 120, height: 440 }}>
        <SpeedTape id="spdTape" x={20} y={180} speed={ex`arinc(L:A380X_ADIRS_ADR_1_COMPUTED_AIRSPEED@knots)`} />
      </Layer>
    </Page>
    <State id="directLaw" name="Direct Law" priority={10} when={ex`L:A380X_FCS_DIRECT_LAW@bool`}>
      <Override target="spdTape" stroke="$amber" />
      <Text id="manTrim" x={384} y={90} anchor="middle" fill="$amber" size={26}>USE MAN PITCH TRIM</Text>
    </State>
  </Display>
);
```

The `@jsx` pragmas are always present (inserted on creation). The pragma, together with `avdev/tsconfig.json`, guarantees FSComponent JSX regardless of the aircraft repo's own tsconfig.

### 6.3 Element catalogue

**Common props.** Every visual element accepts these, and all are bindable with `ex` unless noted:

| Prop | Type | Notes |
|---|---|---|
| `id` | string | Required, not bindable |
| `name` | string | Label shown in the layers panel, not bindable |
| `x`, `y` | number | Position |
| `translateX`, `translateY` | number | Applied after position |
| `rotate` | number | Degrees |
| `origin` | `[x, y]` | Rotation/scale origin, relative to the element |
| `scaleX`, `scaleY` | number | |
| `opacity` | number | 0..1 |
| `visible` | bool | |
| `fill`, `stroke` | color | |
| `strokeWidth` | number | |
| `dash` | number[] | |
| `linecap`, `linejoin` | enum | |
| `glow` | `{radius, color}` | |
| `shadow` | `{dx, dy, blur, color}` | |
| `allow` | string[] | Lint codes allowed for this element and its subtree |

**Transform order** (SVG `transform`): `translate(x+translateX, y+translateY) rotate(rotate, origin) scale(scaleX, scaleY, about origin)`.

**Colors.** A color value is one of:
- `#rgb`, `#rrggbb` or `#rrggbbaa`;
- `$paletteName`, for example `$amber`;
- `gradient:<id>`;
- `none`.

**Shapes and containers:**

| Element | Specific props |
|---|---|
| `Display` | `id`, `width`, `height`, `background`, `font` (default font family), `palette` (bindable: name of the active alternate palette); children: `Page`+, `State`, `StateMachine`, `Defs`, `Graph`, `Var` |
| `Page` | `id`, `default` (bool), `when` (bool ex; first page whose `when` is true wins, else `default`); children: visual elements |
| `Layer` | `name`, `clip` (`{x,y,width,height,rx?}` or `{path}`), `locked` is editor-only (in `editor.json`) |
| `Group` | `clip` optional |
| `Rect` | `width`, `height`, `rx`, `ry` |
| `Ellipse` | `rx`, `ry` (a circle when equal) |
| `Line` | `x2`, `y2` (relative to x,y) |
| `Polyline`, `Polygon` | `points` (`[[x,y],...]`, relative) |
| `Arc` | `r`, `start`, `end` (degrees, 0 = up, clockwise), `close` (`none`/`chord`/`pie`); `start`/`end` bindable |
| `Path` | `d` (SVG path data; bindable string) |
| `Text` | `font`, `size`, `weight`, `anchor` (`start`/`middle`/`end`), `baseline` (`top`/`middle`/`bottom`/`alphabetic`), `letterSpacing`; children: literal text and/or `{ex`...`}` |
| `Image` | `src` (path under `avdev/images`), `width`, `height` |
| `RollingDigits` | `value` (number ex), `digits`, `step` (value per digit roll, e.g. 20 for the altimeter's last two digits), `font`, `size`, `windowHeight`, `pad`; renders a drum counter |
| `Repeat` | `as` (loop variable name), `from`, `to`, `step`, `cull` (`{axis: 'x'\|'y'\|'angle', center: ex, extent: number, spacing: number}`); children are instantiated per index and may reference the loop variable in `ex`. With `cull`, only indices whose position falls within `center ± extent` are materialised, using a recycled pool |
| `Defs` | Children: `LinearGradient`, `RadialGradient` (`id`, `stops: [[offset, color], ...]`, coordinates) |
| `Code` | Wrapper for opaque JSX: `id` plus the original children, unchanged |
| `Var` | Declares a display-local variable: `id` (name of `D:` var), `type`, `initial` |

**Widgets.** Allowed only when `interactive: true`:

| Element | Props |
|---|---|
| `Button` | Shape children for the visual; `onPress` (act), `onRelease` (act), `enabled` (bool ex), `pressedFill` |
| `Toggle` | `value` (bool ex), `onChange` (act, `$value` bound to the new value) |
| `ScrollList` | `width`, `height`, `items` (number ex for the count), `itemHeight`, `as` (index var); child template rendered per visible item; drag/wheel scroll with inertia |
| `TextInput` | `value` (string ex), `onCommit` (act, `$value`), `maxLength`, `charset` (`alnum`/`numeric`/`any`), `font`, `size`; focus opens the `Keyboard` if present, otherwise uses physical key events |
| `Keyboard` | `layout` (`qwerty`/`numeric`/`mcdu`), styled by props; sends key presses to the focused `TextInput` |
| `HitArea` | Invisible rect: `onPress`, `onRelease`, `onDrag` (act with `$dx`, `$dy`) |

### 6.4 States

```tsx
<State id="approach" name="Approach" priority={20} when={ex`S:locCaptured && S:gsArmed`}>
  <Override target="fmaCol2" fill="$green" />
  <Hide target="vsiNumber" />
  <Show target="lsScale" />
  <Layer id="apprExtras" name="Approach extras"> ... visual children shown only while active ... </Layer>
</State>
```

- **`when`** is a bool expression. A state is active while it is true.
- **`priority`** is an integer; higher wins.
- **Overrides:**
  - `<Override target prop=value ...>` sets any bindable prop of the target element while the state is active. Values may be `ex`.
  - `<Hide target>` and `<Show target>` override visibility.
  - Visual children of a `State` exist only while it is active. They are rendered above the base content, in source order.
- **Conflict resolution:** for each (element, prop), the highest-priority active state that sets it wins, and the base value applies when none does.
  - Two states with equal priority overriding the same prop raise AV3002 (error). The fix is to change one priority.
  - Two states with different priorities overriding the same prop raise AV3001 (warning, lint `states`, `allow`-able), naming both states.
- **States as booleans:** each state is readable in expressions as `S:<id>`. A dependency cycle among `S:` references raises AV3003 (error) and shows the cycle path.

### 6.5 State machines

```tsx
<StateMachine id="apprMode" initial="off" priority={15}>
  <Mode id="off" />
  <Mode id="armed"><Override target="gsDiamond" stroke="$cyan" /></Mode>
  <Mode id="captured"><Override target="gsDiamond" stroke="$green" /></Mode>
  <Transition from="off" to="armed" when={ex`L:A380X_APPR_ARMED@bool`} />
  <Transition from="armed" to="captured" when={ex`L:A380X_GS_CAPTURED@bool`} after={0.5} />
  <Transition from="*" to="off" when={ex`!L:A380X_APPR_ARMED@bool`} />
</StateMachine>
```

**Semantics:**
- Exactly one mode is active per machine.
- Each tick, the transitions out of the current mode, plus `*` transitions, are evaluated in source order, and the first one whose `when` holds is taken.
- `after={s}` requires `when` to hold continuously for `s` seconds.
- At most one transition is taken per tick.
- Modes are readable as `S:apprMode.captured`.
- Modes carry the same overrides and children as `State`, at the machine's `priority`.

**Checks:**
- AV3010: mode unreachable from `initial` (warning).
- AV3011: transition references an unknown mode (error).
- AV3012: mode has no outgoing transitions and is not terminal-annotated (`terminal`) (note).

### 6.6 Components

```tsx
// avdev/components/Tape.tsx
/** @jsx FSComponent.buildComponent */
import { FSComponent } from '@microsoft/msfs-sdk';
import { defineComponent, Group, Rect, Text, Line, Repeat, ex } from '@avd/runtime';

export const Tape = defineComponent({
  name: 'Tape', width: 120, height: 440,
  props: { value: 'number', pxPerUnit: 'number', min: 'number', max: 'number', labelEvery: 'number' },
}, (
  <Group id="root">
    <Rect id="bg" width={120} height={440} fill="$tapeGrey" />
    <Group id="scale" translateY={ex`@value * @pxPerUnit + 220`}>
      <Repeat id="ticks" as="v" from={ex`@min`} to={ex`@max`} step={10}
              cull={{ axis: 'y', center: ex`@value`, extent: 30, spacing: 10 }}>
        <Line id="tick" x={100} y={ex`-v * @pxPerUnit`} x2={20} y2={0} stroke="$white" strokeWidth={2} />
        <Text id="lbl" x={90} y={ex`-v * @pxPerUnit`} anchor="end" baseline="middle"
              visible={ex`v % @labelEvery == 0`}>{ex`fmt(v, '000')`}</Text>
      </Repeat>
    </Group>
  </Group>
));
```

- Instances are used as JSX: `<Tape id="spdTape" x={20} y={180} value={ex`...`} pxPerUnit={4.2} min={30} max={520} labelEvery={20} />`.
- Props are typed: `number`, `bool`, `string`, `color`, `arinc429`, or `number@<unit>`. Inside the component they are referenced as `@name`.
- Element ids inside a component are scoped. From outside they are addressed as `spdTape/bg`, for example `<Override target="spdTape/bg" fill="$amber" />`.
- Editing a component opens its file. All instances in all displays update on the next rebuild.
- "Make component" on a selection extracts it into a new component file, replaces the selection with an instance, and lifts the bound expressions to props, which the developer chooses in a dialog.

### 6.7 Pages

- A `Display` has one or more `Page`s. Exactly one page is visible: the first whose `when` is true, otherwise the `default` page.
- The display-local `D:page` var, written by the `goto(P:id)` action, forces a page. A page whose `when` is omitted is selectable only by `goto` or as the default.
- `P:<id>` is a bool readable in expressions (true while visible).
- States apply across pages. They can target elements on any page, and an override on a hidden page has no effect.
- The Pages panel lists pages with thumbnails. The canvas edits one page at a time. A page switcher in the canvas toolbar shows the page's `when` condition.

### 6.8 Node graphs

```tsx
<Graph id="fmaLogic" outputs={{ col1Text: 'string', col1Color: 'color' }}>
  <Node id="n1" kind="var" ref="L:A380X_FMA_LATERAL_MODE@enum" x={40} y={60} />
  <Node id="n2" kind="switch" cases={{ '10': 'NAV', '11': 'HDG', '20': 'LOC' }} default="" x={260} y={60} />
  <Node id="n3" kind="output" port="col1Text" x={480} y={60} />
  <Edge from="n1.out" to="n2.in" />
  <Edge from="n2.out" to="n3.in" />
</Graph>
```

- A graph is declared under `Display` or inside a component. Its outputs are readable as `G:fmaLogic.col1Text`.
- **Node kinds** mirror the expression language one-to-one, so every graph can be lowered to a set of expressions. That lowering is the compile and runtime path:
  - `var`, `const`, `state`, `prop`, `loopVar`
  - math: `add`, `sub`, `mul`, `div`, `mod`, `neg`, `abs`, `min`, `max`, `clamp`, `map`, `interp`, `round`, `floor`, `ceil`, `wrap`
  - trig: `sin`, `cos`, `atan2`
  - comparison: `gt`, `ge`, `lt`, `le`, `eq`, `ne`
  - logic: `and`, `or`, `not`, `select`, `switch`
  - text: `fmt`, `concat`
  - ARINC: `arinc`, `ssm`, `isNormal`, `isFailure`, `isNcd`, `isFt`
  - timing: `smooth`, `rateLimit`, `flash`, `pulse`, `delayOn`, `delayOff`, `latch`, `rising`, `falling`, `timeSince`
  - `convert`, `logic` (calls a code-block function), `output`
- **Expression ↔ graph conversion.** "Convert expression to graph" auto-lays out a graph. "Convert graph to expression" works when the graph has a single output.
- Graph cycles are an error (AV3020). Timing nodes do not break cycles.

### 6.9 Theme (`avdev/theme.ts`)

```ts
import { defineTheme } from '@avd/runtime';
export default defineTheme({
  fonts: {
    Ecam: { src: 'fonts/ECAMFontRegular.ttf', weight: 400 },
    EcamBold: { src: 'fonts/ECAMFontBold.otf', weight: 700 },
  },
  palette: { black: '#000000', white: '#ffffff', green: '#00ff00', amber: '#ffa000', cyan: '#00ffff',
             magenta: '#ff94ff', red: '#ff0000', sky: '#0b8ee4', ground: '#8c4b18', tapeGrey: '#3a3a3a' },
  palettes: { dim: { white: '#bbbbbb', green: '#00bb00' } },
});
```

- **Fonts:** TTF, OTF, WOFF and WOFF2 are accepted.
  - A missing font file raises AV4001 (error).
  - An unsupported extension raises AV4002 (error).
  - A font referenced by an element but not declared in the theme raises AV1201 (error), with a fix to add it if a file with that name exists in `avdev/fonts`.
  - The runtime resolves palette colors to literal values. No CSS variables are used, for Coherent GT safety.
- **Alternate palettes:** `Display palette={ex`L:X@bool ? 'dim' : ''`}` switches between them. A palette is merged over the base.

## 7. Expression and action language (`avd-expr`; mirrored in `@avd/runtime`)

### 7.1 Grammar (EBNF)

```
expr      := ternary
ternary   := or ('?' expr ':' expr)?
or        := and ('||' and)*
and       := cmp ('&&' cmp)*
cmp       := add (('=='|'!='|'<'|'<='|'>'|'>=') add)?
add       := mul (('+'|'-') mul)*
mul       := unary (('*'|'/'|'%') unary)*
unary     := ('!'|'-') unary | postfix
postfix   := primary ('.' ident | '[' expr ']')*
primary   := number | string | 'true' | 'false' | varref | stateRef | pageRef | graphRef
           | localRef | propRef | loopVar | call | '(' expr ')' | array
varref    := kind ':' (ident (':' digits)? | quoted) ('@' (ident | quoted))?
kind      := 'L' | 'A' | 'E' | 'B' | 'C'          (H:/K: only valid in actions)
stateRef  := 'S:' ident ('.' ident)?
pageRef   := 'P:' ident
graphRef  := 'G:' ident '.' ident
localRef  := 'D:' ident
propRef   := '@' ident
call      := ident '(' (expr (',' expr)*)? ')'
array     := '[' (expr (',' expr)*)? ']'
number    := digits ('.' digits)? (('e'|'E') ('+'|'-')? digits)?
string    := '"' ... '"' | "'" ... "'"     (quoted identifiers in varref use single quotes)
```

Inside a template literal, `${...}` substitutions are not allowed (AV0201), because expressions must be statically analysable. Whitespace and `//` comments are allowed.

### 7.2 Types and units

**Types:** `number<unit?>`, `bool`, `string`, `color`, `enum<VarKey>`, `arinc429<inner>`, `array<T>`.

**Units:**
- MSFS unit names and their aliases are grouped into dimensions:
  - length: feet, meters, nautical miles
  - speed: knots, feet per minute, meters per second, mach
  - angle: degrees, radians
  - pressure: millibars/hectopascals, inHg
  - temperature: celsius, fahrenheit, kelvin, rankine
  - mass: pounds, kilograms
  - time: seconds, minutes, hours
  - ratio: percent, percent over 100
  - unitless: number, position, bool, enum
- A var reference without `@unit` takes the var's canonical unit (§5.3).

**Rules:**
- `+`, `-`, comparisons, `min`, `max`, `clamp` and the `map` input range need operands of equal unit. A unitless literal adopts the other operand's unit. A different unit of the same dimension raises AV2003 (error), with a fix to wrap it in `convert(x, 'unit')`. A different dimension raises AV2004 (error).
- `*` and `/` with a unitless operand keep the unit. Unit × unit gives a unitless number.
- `bool` is never implicitly a number. `L:X@bool * 2` raises AV2001, with a fix of `(L:X@bool ? 1 : 0) * 2`.
- `arinc429` used where a number is expected raises AV2010, with the fix `.value`. Members: `.value`, `.ssm` (enum FW/NCD/FT/NO), `.isNormal`, `.isFailure`, `.isNcd`, `.isFt`, `.valueOr(x)`, `.bit(n)` for discrete words.
- `enum` compares to its declared labels as strings (`L:A380X_SD_PAGE == 'BLEED'`) or to numbers. An unknown label raises AV2020, with a "did you mean" fix.
- Every prop has a declared type (§6.3). A mismatch raises AV2002.

### 7.3 Built-in functions

| Category | Functions |
|---|---|
| Math | `min max clamp(x,lo,hi) abs round(x,step?) floor ceil sign sqrt pow mod(x,m) wrap(x,lo,hi) lerp(a,b,t) map(x,i0,i1,o0,o1,clamp?) interp(x, [[x0,y0],...])` |
| Trig | `sin cos tan atan2 deg rad` (degrees in, degrees out by default) |
| Text | `fmt(x, pattern)`, `concat(...)`, `pad(s, n, ch)`, `upper`, `str(x)` |
| Units | `convert(x, unit)`, plus shorthands: `hpaToInhg`, `inhgToHpa`, `kgToLb`, `lbToKg`, `cToF`, `fToC`, `ftToM`, `mToFt` |
| ARINC | `arinc(varref)` (decode), `ssm(x)` |
| Timing (stateful) | `smooth(x, tauSec)`, `rateLimit(x, unitsPerSec)`, `flash(cond, hz=1, duty=0.5)`, `pulse(cond, sec)` (true for `sec` after a rising edge), `flashFor(cond, sec, hz=1)` (flashes for `sec` after a rising edge, then steady true while cond holds), `delayOn(cond, sec)`, `delayOff(cond, sec)`, `latch(set, reset)`, `rising(cond)`, `falling(cond)`, `timeSince(cond)` |
| Color | `color(name)`, `rgba(r,g,b,a)`, `mix(c1, c2, t)` |
| Select | `select(cond, a, b)`, `switch(x, k1, v1, ..., default)` |

`fmt` patterns:
- `0` is a zero-padded digit and `#` an optional digit.
- `.` is the decimal point.
- `+` forces a sign.
- `,` is the thousands separator.
- A `round` suffix like `|10` rounds to a step.
- Examples: `fmt(x,'000')`, `fmt(x,'+0.0')`, `fmt(x,'#####|10')`.
- Conditional text uses the ternary: `L:ALT@feet > L:TRANS_ALT@feet ? concat('FL', fmt(L:ALT@feet/100,'000')) : fmt(L:ALT@feet,'#####')`.

Timing functions are evaluated every tick. Their output is cached per call site and keyed by the source span, so instances in a `Repeat` get separate state per index. A timing function inside a `cull`ed repeat raises AV5010 (warning), because the state resets when an index is recycled.

### 7.4 Presets

Presets are UI forms that read and write canonical expression shapes:

| Preset | Canonical expression |
|---|---|
| Move linearly | `map(<src>, i0, i1, o0, o1, true)` |
| Rotate | `map(<src>, i0, i1, deg0, deg1)` |
| Scale | `map(...)` |
| Show when | `<bool expr>` |
| Color when | `<cond> ? <c1> : <c2>`, or chained ternaries |
| Text value | `fmt(<src>, '<pattern>')` |
| Flash when | `flash(<cond>, hz)` on `visible` |

The inspector recognises these shapes, in the parsed AST, and shows the preset form. Any other expression shows the expression editor, which has syntax highlighting, var autocomplete from the VarDb, inline type and unit hints, and live value preview while in Preview mode. The binding switcher has three modes: Preset, Expression, Graph.

### 7.5 Actions (`act` template)

```
action    := stmt (';' stmt)*
stmt      := 'set' '(' settable ',' expr ')' | 'toggle' '(' settable ')'
           | 'fire' '(' event (',' expr)? ')' | 'emit' '(' busref ',' expr ')'
           | 'call' '(' string (',' expr)* ')' | 'goto' '(' pageRef ')'
           | 'if' '(' expr ')' '{' action '}' ('else' '{' action '}')?
settable  := varref(L|A) | localRef
event     := 'H:' ident | 'K:' ident
```

`$value`, `$dx` and `$dy` are available inside widget handlers. Writing an A: var that MSFS marks read-only raises AV2101 (warning; the table of settable A: vars is maintained in `avd-expr/data/settable_avars.json`).

### 7.6 Code blocks (`avdev/logic/*.ts`)

```ts
export function fmaLateral(mode: number, armed: boolean): string { ... }
```

- Functions are exported with parameter and return type annotations limited to `number`, `boolean` and `string`. Missing or other annotations raise AV1401.
- Expressions call them by name: `fmaLateral(L:A380X_FMA_LAT@enum, S:navArmed)`.
- The display imports them. The app adds `import { fmaLateral } from '../logic/fma'` and registers them in `<Display logic={{ fmaLateral }}>`.
- Calls are treated as pure. The function is re-evaluated when its inputs change.

## 8. Checker (`avd-check`)

### 8.1 Pipeline

1. **Parse.** oxc parses all display, component, logic and theme files (AV0xxx).
2. **Lower to model.** Build the model with spans.
3. **Resolve names.** Covers vars, states, modes, pages, graphs, props, palette, fonts, images, components, logic functions and element ids (AV1xxx).
4. **Parse and type-check expressions** (AV2xxx).
5. **Analyse states, pages and graphs** (AV3xxx).
6. **Check assets** (AV4xxx).
7. **Check layout and perf** (AV5xxx).
8. **Apply lint levels.**
9. **Emit diagnostics.**

The checker runs incrementally on every edit (per file, 150 ms debounce) and fully before every compile.

### 8.2 Diagnostic format

```rust
#[derive(Serialize)]
struct Diagnostic {
  code: &'static str,              // "AV2003"
  severity: Severity,              // Error | Warning | Note | Help
  message: String,                 // "mismatched units: expected `knots`, found `meters per second`"
  primary: Label,                  // file, utf8 span, utf16 line/col (for Monaco), text
  secondary: Vec<Label>,
  notes: Vec<String>,              // "note: `L:X` is declared as `meters per second` in vars.json"
  help: Option<String>,            // "help: convert explicitly: `convert(L:X, 'knots')`"
  fixes: Vec<Fix>,                 // { title, applicability: MachineApplicable|MaybeIncorrect, edits: [{file, start, end, text}] }
  element: Option<ElementRef>,     // display + element id → canvas highlight
  lint: Option<&'static str>,      // group name if this is a lint
}
```

How diagnostics are shown:
- **Problems panel:** grouped by file, with a red/yellow badge in the left rail.
- **Monaco:** markers, with quick-fix code actions that apply `fixes`.
- **Canvas:** red/amber outline on `element`.
- **Terminal pane:** rustc-style text rendering, also written to `.avd/build/last-check.txt`.
- **`avd explain AV2003`:** opens `docs/diagnostics/AV2003.md`, which every code has (enforced by a test).

### 8.3 Code ranges

| Range | Area | Examples |
|---|---|---|
| AV00xx | Syntax, dialect, avdev.json | 0001 parse error, 0101 unknown dialect element, 0102 missing id, 0103 duplicate id, 0104 invalid prop value shape, 0105 unknown prop (did-you-mean), 0106 missing required prop, 0201 `${}` in ex, 0901 unknown avdev.json key, 0902 invalid avdev.json value, 0903 duplicate display id |
| AV1xxx | Name resolution | 1001 unknown var (help: closest VarDb names by Levenshtein + same prefix), 1102 bound var disappeared from scan, 1201 unknown font, 1202 unknown palette color, 1203 unknown image, 1204 unknown gradient, 1301 new var name violates convention, 1401 bad logic signature, 1402 unknown logic function, 1501 unknown override target, 1502 unknown state/page/graph, 1503 unknown component, 1504 unknown `@prop`, 1505 unknown loop var, 1601 unknown unit |
| AV2xxx | Types and units | 2001 bool used as number, 2002 prop type mismatch, 2003 unit mismatch same dimension, 2004 dimension mismatch, 2005 unitless where unit expected (lint `units`), 2010 ARINC used as number, 2020 unknown enum label, 2030 wrong arity, 2031 unknown function, 2101 write to read-only A: var, 2102 H:/K: event used in an expression, 2103 set()/toggle() target not settable |
| AV3xxx | States, pages, graphs, widgets | 3001 overlapping overrides (warning), 3002 equal-priority conflict, 3003 state cycle, 3010–3012 FSM reachability, 3020 graph cycle, 3101 widget on non-interactive display, 3201 no default page |
| AV4xxx | Assets | 4001 missing font file, 4002 unsupported font, 4003 missing image, 4101 image > 4 MB (warning), 4201 display size disagrees with panel.cfg gauge (warning) |
| AV5xxx | Layout and perf | 5001 element fully outside display bounds (warning), 5002 more than 2,000 materialised elements (warning), 5003 repeater > 50 instances without `cull` (warning), 5010 timing fn in culled repeat, 5020 `refreshHz` > 60 (note) |
| AV6xxx | Build and toolchain | 6001 Mach not linked, 6002 Node < 22 for Mach ≥ 1.2, 6003 Mach version unsupported, 6010 esbuild error (mapped to source span), 6011 esbuild warning, 6020 ACE output folder not writable |
| AV9xxx | Opaque code info | 9001 opaque attribute value (note), 9002 foreign JSX wrapped as Code (note) |

### 8.4 Lint levels

- **Levels:** `allow`, `warn`, `deny` (makes the lint an error) and `forbid` (deny that cannot be re-allowed by an element `allow`).
- **Groups:** `perf` (AV5xxx), `states` (AV3001, AV3010, AV3012), `units` (AV2005; AV2003 and AV2004 are hard errors), `style` (AV1301, AV9xxx), `assets` (AV4101, AV4201).
- **Errors** (non-lint diagnostics) are always errors and block compile. Warnings never block.
- **Element-level suppression:** `allow={['AV5003']}`.

## 9. Runtime and build

### 9.1 `@avd/runtime`

**Public API:**
- The elements of §6.3 to §6.8, as FSComponent `DisplayComponent`s.
- `ex`, `act`, `defineComponent`, `defineTheme`.
- `bootDisplay(config)`.

**Render model:**
- One root `<svg width height viewBox>` per display, with nested `<g>` elements.
- Text is SVG `<text>`. Clip paths are `<clipPath>` in `<defs>`, and gradients live in `<defs>`.
- FSComponent renders the initial tree and collects refs. After that, updates mutate DOM attributes directly. There are no per-attribute Subjects.

**Expression engine:**
- A compact TypeScript implementation of the §7.1 grammar compiles each expression once at boot into a closure tree. It does not use `eval` or `Function`.
- Each closure has a dependency set: vars, states, pages, props, loop vars, and a "time" flag for timing functions.
- The conformance corpus `spec/expr-conformance/*.json` ({expr, env, ticks, expected}) is run by both the Rust evaluator in `avd-expr` and the TS runtime, in CI.

**Scheduler** (per display):
- A tick fires on the host update.
- If less than `1/refreshHz` has elapsed since the last evaluation, the tick returns.
- Otherwise the runtime:
  1. reads every referenced var once, through one `SimVar.GetSimVarValue` per (var, unit) pair;
  2. marks the changed ones;
  3. re-evaluates expressions whose dependencies changed or that have the time flag;
  4. evaluates state, mode and page logic;
  5. resolves overrides by priority;
  6. writes a DOM attribute only when its value changed.
- Power gating (`power.on`) skips all of this while the display is off.

**Var access:**
- Reading and writing go through `SimVar.GetSimVarValue`/`SetSimVarValue`.
- `B:` topics use an msfs-sdk `EventBus` created by the runtime. When the scanner found a SimVarPublisher definition for a topic, the topic is fed from the underlying SimVar, so it works standalone. Otherwise it is subscribed through the bus, which relies on cross-instrument bus sync.
- `C:` subscribes with `Coherent.on`.
- `E:` vars use `SimVar.GetSimVarValue('E:...')`.

**ARINC:** `arinc()` decodes per §5.7.

**Events:**
- **Sim:** `fire(H:X)` uses `SimVar.SetSimVarValue('H:X', 'number', 0)`, falling back to `Coherent.call('TRIGGER_HTML_EVENT'...)` per msfs-sdk practice. `fire(K:X, v)` uses `SimVar.SetSimVarValue('K:X', 'number', v)`.
- **ACE:** `H:` and `K:` setting throws in ACE (§9.4), so the runtime catches the error and routes the event to `Coherent.trigger('AVD_EVENT', name, value)`, where ACE's Coherent panel shows it.

**Dev mode** (preview builds only, `process.env.AVD_DEV`):
- Every element gets `data-avd-id`.
- `window.__avd` exposes `patch(id, props)`, `inspect(id)` (current evaluated props) and `stats()` (eval counts, last tick ms).

### 9.2 Generated build directory (`avdev/.avd/build/`)

- **`entries/<Id>.tsx`:**
  ```tsx
  import display from '../../../displays/<Id>';
  import theme from '../../../theme';
  import { bootDisplay } from '@avd/runtime';
  import './<Id>.css';
  bootDisplay({ id, templateId: 'AVD_<Id>', mountId: '<Id>_CONTENT', display, theme, refreshHz, power, interactive });
  ```
- **`entries/<Id>.css`:** `@font-face` rules for every theme font, using `url('/Fonts/AVD/<file>')`. Mach marks `/Fonts/*` external, so the URL is left as is. The file also contains a `body{margin:0;background:#000}` reset. Writing this file guarantees `bundle.css` exists, which ACE requires.
- **`mach.config.cjs`:** generated for the linked Mach version.
  - **1.0.x:** top-level `plugins`.
  - **≥ 1.2:** `esbuild.plugins`.
  - **Common:**
    - `packageName: 'AVD'`, `packageDir: '.'`;
    - one instrument per display (`name: <Id>`, `index: 'entries/<Id>.tsx'`, no `simulatorPackage`);
    - `esbuild: { jsxFactory, jsxFragment, keepNames: true, define: {'process.env.AVD_DEV': ...}, banner: { js: <ace-polyfill> } }`;
    - plugins: `@avd/mach-plugin`'s `avdResolve(paths)` and `avdReporter()`. These are required by absolute path from the app's resources.
- **`avdResolve`:**
  - Pins `@avd/runtime` to the app-bundled build.
  - Pins `@microsoft/msfs-sdk` to exactly one copy: the bundled copy when `sdk: "bundled"`, or the copy resolved from the aircraft repo root when `sdk: "aircraft"`. This guarantees a single SDK instance in the bundle.
- **`avdReporter`:**
  - Registers an `onEnd` hook on every build and rebuild.
  - Prints one line to stdout: `@@AVD@@{"instrument":..,"ok":..,"errors":[esbuild Message],"warnings":[..],"ms":..}`.
  - The Rust runner parses only `@@AVD@@` lines and keeps all other output for the Build log pane.
- **ACE polyfill banner:**
  - Defines guarded stubs **only when absent**: `BaseInstrument`, `registerInstrument`, `Simplane`, `Include`, `GameState`, `LaunchFlowEvent`, `SimVar.GetSimVarArrayValues`, `Avionics`, `Utils`.
  - This lets msfs-sdk modules that reference these globals at import time evaluate inside ACE.
  - The exact list is determined at implementation time by bundling a test display and loading it in an ACE-equivalent DOM with only ACE's globals (§12.3). Every ReferenceError found is added to the list.
- Mach runs with `-d`, so its cwd is the build dir. It does not run with `-m`, because minification breaks the FSComponent Fragment name check. Bundles go to `./bundles/<Id>/`.

### 9.3 ACE project output (`avdev/out/ace-project/`)

```
.ace/project.json      { "name": "<aircraft.name>-avd", "paths": { "instrumentSrc": "instruments", "bundlesSrc": "bundles", "htmlUiSrc": "html_ui" } }   (tab-indented, relative paths)
.ace/.gitignore        data/*
.ace/canvas.json       every display placed left→right/top→bottom (wrap at 3000 px), __kind 'instrument', dataKind 'bundled', unique __uuid, title = Id
.ace/simvars.json      a control per bound A:/L: var (bool→Checkbox(3), number with range→Range(2) min/max/step, enum→Number(1), other→Number(1)); title = description or name; varUnit = canonical unit
.ace/data/simvar-values.json   initial values from vars.json defaults (unit written as 'number', matching ACE)
instruments/<Id>/config.json   { "index": "./index.tsx", "isInteractive": <interactive>, "name": "<Id>", "dimensions": { "width": W, "height": H } }
bundles/<Id>/bundle.js, bundle.css   (copied from build; empty bundle.css written if missing)
html_ui/Fonts/AVD/*    theme fonts
html_ui/Images/AVD/*   images referenced by displays
package.json           { "name": "<aircraft>-avd-ace", "private": true }  (satisfies ACE's project dialog checks)
README.md              how to open in ACE; generated-by notice
```

- `.ace/canvas.json` and `.ace/simvars.json` are regenerated on each compile **only if** they still match the last generated content (hash stored in `.avd/build/ace-hashes.json`). Otherwise the developer's ACE-side layout changes are kept, and a note is shown.
- **"Open in ACE":**
  - Launches the linked ACE. A packaged ACE executable is run directly. A repo clone gets `npm start` in the ACE folder, run with the linked Node.
  - Copies the project path to the clipboard and shows "In ACE: Open Project → paste".

### 9.4 Host compatibility of the generated bundle

`bootDisplay` detects the host:

- **MSFS:** `BaseInstrument` is a real function and `registerInstrument` exists.
  - Defines `class AVD_<Id> extends BaseInstrument` with `templateID` = `AVD_<Id>`.
  - `connectedCallback` renders into `#<Id>_CONTENT`.
  - `Update` drives the scheduler.
  - Calls `registerInstrument('avd-<id>', ...)`.
  - A developer can wire this bundle into the sim by hand with a Mach `baseInstrument` package entry. This path is not tested in CI and is documented as best-effort.
- **ACE / AVD preview:** `BaseInstrument` is absent or is the polyfill stub, and `#MSFS_REACT_MOUNT` exists.
  - Renders into `#MSFS_REACT_MOUNT`.
  - Drives the scheduler from the `update` CustomEvent on `#ROOT_ELEMENT` (ACE fires it every 50 ms).
  - Also runs a `requestAnimationFrame` loop capped at `refreshHz`, so updates are smoother than ACE's 20 Hz.
  - Pointer input is native DOM; ACE requires its "interaction mode" toggle (Enter).

### 9.5 Build commands and processes (`avd-build`)

**Compile** (button, Ctrl+B):
1. Full check. Stop if there are errors.
2. Generate the build dir.
3. `<node> <machDir>/dist/index.js build -c mach.config.cjs -d -s` (`-s` only where supported; capabilities are detected at link time from `mach --help`).
4. Parse the reporter lines and map esbuild messages to AV6010/AV6011 with source spans.
5. Write the ACE project.
6. Show a summary toast with per-display sizes and times.

**Preview:**
- A long-lived `mach watch` process on the same config, with `AVD_DEV=true` and inline sourcemaps.
- It is restarted when the config changes (displays added, theme fonts changed, Mach relinked) or when it exits because the initial build failed. In that case it restarts on the next save.
- Processes are spawned with `tokio::process` and `CREATE_NO_WINDOW` on Windows, and killed on project close and app exit (job object on Windows, process group on Linux).

**Toolchain linking** (`%APPDATA%\WYSIWYG Avionics Builder\config.json` on Windows, `~/.config/wysiwyg-avionics-builder/config.json` on Linux):
```json
{ "version": 1, "node": "C:\\Program Files\\nodejs\\node.exe", "mach": "C:\\Users\\me\\AppData\\Roaming\\npm\\node_modules\\@synaptic-simulations\\mach", "ace": "D:\\dev\\ace", "simconnectDll": null, "relay": null }
```
- **Validation on startup and in Settings:**
  - `node --version` must be ≥ 18, and ≥ 22 if Mach ≥ 1.2.
  - `<mach>/package.json` must have name `@synaptic-simulations/mach`; its version is recorded.
  - ACE: `<ace>/package.json` must have `"name": "ace"` and `electron`, or the folder must contain an ACE executable.
- **Per-project override:** if `<aircraft>/node_modules/@synaptic-simulations/mach` exists, Settings offers "Use the aircraft's Mach (vX)". This matches the aircraft's own builds.

## 10. Preview (ACE superset)

### 10.1 Host shim (`@avd/shim`)

`@avd/shim` is installed into preview iframes. The iframes are served via the `avd-preview://` custom scheme from `.avd/build/bundles` and the ACE `html_ui` layout. The DOM contract is identical to ACE: `<vcockpit-panel><ace-instrument id="ROOT_ELEMENT"><div id="MSFS_REACT_MOUNT">`, base href to html_ui, 50 ms `update` event. The shim provides:

| Global | Behaviour |
|---|---|
| `SimVar.GetSimVarValue/SetSimVarValue/GetGameVarValue` | Store keyed by `prefix:name`, with unit conversion between declared units. Supports L, A, E. H/K go to the event log and to subscribed scenario triggers |
| `Coherent.on/trigger/call` | `call` resolves via handlers registered by scenarios (default: resolves `undefined` after one tick, logged) |
| `RegisterViewListener`, `GetStoredData/SetStoredData` | Stored data persists to `.avd/preview-storage.json` |
| `Facilities.getMagVar`, `LatLongAlt` | ACE-equivalent |

The store lives in the UI process, so it survives iframe reloads, and it is shared across all preview iframes, like ACE.

### 10.2 Variable panel ("(x)" rail icon)

- Auto-lists every variable referenced by open displays, grouped by display. Pinned vars are added from VarDb search.
- **Controls follow the var type:**
  - bool: toggle;
  - number with range: slider + numeric field;
  - number: numeric field;
  - enum: dropdown of labels;
  - ARINC: value field + SSM dropdown (FW/NCD/FT/NO);
  - string: text.
- Shows the live value and the "last written by" display.
- Also lists states, modes and pages with their live status. Clicking one shows *why* it is active or inactive, with sub-expression values.

### 10.3 Scenarios (timeline, bulb icon)

```json
{ "version": 1, "name": "Takeoff roll", "duration": 60, "loop": false,
  "tracks": [
    { "var": "A:'AIRSPEED INDICATED'@knots", "keys": [[0, 0], [30, 160, "linear"], [45, 180, "smooth"]] },
    { "var": "L:A380X_FCS_DIRECT_LAW@bool", "keys": [[20, 0], [21, 1, "step"]] },
    { "var": "L:A380X_ADIRS_ADR_1_COMPUTED_AIRSPEED@knots", "arinc": true, "keys": [[0, 0, "step", "NCD"], [5, 30, "linear", "NO"]] }
  ],
  "events": [[25, "H:A380X_PFD_LS_PUSH"]],
  "coherentReplies": { "FLIGHTPLAN_GET": { "result": {} } } }
```

- **Editing:** a track lane per var with draggable keyframes and interpolation per segment (`step`, `linear`, `smooth`).
- **Playback:** play, pause, scrub, loop and speed (0.25× to 4×).
- **Recording:** from the variable panel or the live sim link, capturing the selected vars at 10 Hz into a new scenario.

### 10.4 Workspace

A canvas of multiple running displays, with each display's size from `avdev.json`. Displays can be dragged, zoomed and panned. Interaction mode is toggled with Enter. Saved to `avdev/workspace.json`.

### 10.5 Live sim link (`avd-sim`)

- **`SimBridge` trait:** `connect()`, `subscribe(vars: &[VarSpec])`, `set(var, value)`, `fire(event, value)`, `exec_calculator(code)` (optional capability), `disconnect()`. Values stream to the UI over a Tauri Channel at up to 30 Hz.
- **Windows: `SimConnectBridge`.**
  - A minimal FFI over `SimConnect.dll` loaded at runtime with `libloading`.
  - The DLL is found from the configured `simconnectDll`, then next to the exe, then the MSFS 2024/2020 SDK default paths. It is not shipped by us.
  - Calls used: `SimConnect_Open`, `Close`, `AddToDataDefinition` (one definition per var, `FLOAT64`), `RequestDataOnSimObject` (period `SIM_FRAME` with `CHANGED` flag), `GetNextDispatch` polling at 60 Hz, `SetDataOnSimObject` for L:/A: writes, and `MapClientEventToSimEvent` + `TransmitClientEvent` for K: events.
  - **H: events and calculator code** go through the MobiFlight WASM module's client-data channel when it is installed (detected via the `MobiFlight.Command` client data area). Otherwise those features are disabled with a UI notice.
- **`RelayBridge`** (Linux, or a remote Windows sim): a WebSocket client to `avd-relay`. `avd-relay` is a tiny Windows console binary built from the `avd-sim` crate, with the `relay` feature, that wraps `SimConnectBridge`.
- **`MockBridge`:** used in tests.
- **Live mode:** while connected, the preview store is fed from the sim and is read-only for subscribed vars. Writes from preview widgets are forwarded to the sim.

## 11. Editor UI (`packages/ui`)

### 11.1 Look

Visual tokens are taken from the ACE2 screenshot:

| Token | Value |
|---|---|
| Background | `#0b0f17` |
| Panel | `#111827` |
| Card | `#161e2d` |
| Border | `#253046` |
| Text | `#e6edf7` |
| Muted | `#8b97ad` |
| Accent | `#00c2d1` (cyan) |
| Accent hover | `#19d6e4` |
| Danger | `#ff5c5c` |
| Warning | `#ffb020` |
| Success | `#2fd27a` |
| Radius | 8–10 px |
| UI font | Inter, bundled |
| Data and code font | JetBrains Mono, bundled |

Fonts are bundled with the app, not loaded from a CDN. The canvas background is a grid pattern like ACE's.

**App frame:**
- **Title bar:** a custom-drawn bar with tabs: `Home` plus one tab per open project.
- **Left icon rail** (lucide icons), top to bottom:
  - Displays
  - Layers
  - Components
  - Variables (x)
  - States & Pages
  - Graphs
  - Scenarios (bulb)
  - Problems (bell, with a badge)
  - Git
  - spacer
  - Settings (gear)
- **Mode switch** in the toolbar: Design / Preview / Workspace. **View switch:** Canvas / Split / Code.

### 11.2 Home

- Recent projects (from app config) and a **New project** wizard:
  1. Pick the aircraft folder.
  2. If `avdev.json` is missing, offer to create one from a template.
  3. Run the scan, with a progress stream.
  4. Show `panel.cfg` display suggestions to accept.
  5. Create `avdev/` (theme, tsconfig, `.gitignore`, one starter TSX per display).
- **Open project** opens any folder containing `avdev.json`.

### 11.3 Design mode

- **Canvas:**
  - Display artboard at native resolution, rendered by the real runtime in an iframe, with an overlay layer for selection.
  - Zoom from 10% to 3200% with a pixel grid at 800% and above, pan, and rulers with draggable guides.
  - Snapping to guides, element edges and centres, the grid and the display centre.
  - Multi-select, marquee, align and distribute, z-order, group and ungroup, lock and hide in the editor, copy and paste (as TSX snippets via the clipboard).
  - Nudge with arrow keys (Shift ×10).
  - The tracing image underlay has opacity, offset and scale controls and is never emitted.
- **Tools:** select, direct-select (path points), rect, ellipse, line, polyline, polygon, arc, pen (bezier), text, image, repeater, and the widget tools (enabled only when the display is interactive).
- **Boolean ops** (union, subtract, intersect, exclude) on selected shapes. They are baked to a `Path` by `avd-svg` (i_overlay on flattened geometry, tolerance 0.1 px).
- **SVG import:** drop or pick a file. `usvg` normalises it, and it is converted to dialect elements (rect, ellipse, line, polyline, polygon, path, text, image, groups, gradients, transforms flattened into x/y/rotate/scale where exact; otherwise a `Path`).
- **Layers panel:** a tree with drag reordering and reparenting, eye and lock toggles (editor-only), clip-mask editing, rename, and search. State-only content appears under its State node.
- **Inspector:**
  - Geometry, style and text sections.
  - Each bindable prop has a binding button that opens the Preset / Expression / Graph editor.
  - A "States" section lists the overrides affecting the selected element, with their priority.
  - Opaque values are shown read-only, with a "Edit in code" link.
- **Split / Code view:**
  - Monaco shows the display TSX, with dialect `.d.ts` files loaded via `addExtraLib`, diagnostics as markers and quick fixes.
  - Selection syncs both ways: cursor to element, element to range.

### 11.4 States & Pages panel

- States are listed by priority, with drag reordering that rewrites the `priority` values.
- Each entry has an add/edit condition field, an "edit overrides" mode (canvas edits are recorded as `Override`s of the active state, shown by an orange frame), and a "preview this state" toggle that forces it active in Design mode.
- The state machine editor is a node view of modes and transitions.
- The pages list has thumbnails, conditions and a default marker.

### 11.5 Variables panel (Design mode)

- VarDb browser: search, filters (kind, type, prefix, read/write, bound, created) and per-var detail (sites with a jump to file, units, inferred type, editable metadata).
- **"Create variable"** applies the naming convention, with live AV1301 feedback.
- Dragging a var onto an element prop creates a preset binding.

### 11.6 Git panel

- Branch picker (create, switch).
- Changed files grouped as Displays, Components, Theme/vars, Other, with stage/unstage per file or all, and a Monaco diff view.
- Commit box with message and amend toggle.
- Log with graph lines, the last 200 commits.
- Push, pull and fetch buttons.
- **Backends:** `git2` for status, diff, stage, commit, branch, checkout and log. The system `git` CLI handles push, pull and fetch, so it uses the developer's credential helpers and SSH config. Output streams into the panel.

### 11.7 Settings

- Toolchain links: Node, Mach, ACE, SimConnect DLL and relay address, each with validation status.
- Conventions view and overrides.
- Lint levels editor (writes `avdev.json`).
- Editor preferences: autosave debounce, snapping, theme density.
- Keyboard shortcuts.

### 11.8 Undo/redo

- An app-level command stack per project. Each command is semantic and id-anchored.
- Inverse commands are recomputed against the current model at undo time.
- History survives external edits: commands whose targets no longer exist are skipped, with a toast.
- Shortcuts: Ctrl+Z / Ctrl+Shift+Z. Monaco keeps its own text undo while focused. Text edits made in Monaco become a single `TextEdit` command when focus leaves.

## 12. Testing

1. **Rust unit tests** in every crate.
   - **`avd-syntax`:**
     - Parse → model → print is byte-identical for untouched files.
     - Each command produces minimal diffs (insta snapshots).
     - Foreign code is preserved.
     - UTF-8 ↔ UTF-16 mapping is correct.
   - **`avd-expr`:** parser, typer and evaluator over `spec/expr-conformance`.
   - **`avd-check`:** at least one failing fixture per diagnostic code, with insta snapshots of the rendered diagnostics. A test asserts every code has a `docs/diagnostics` page.
2. **Scanner fixtures.** `fixtures/sample-aircraft` contains one file per access pattern of §5.2, a `panel.cfg` and ARINC usages. Tests assert the exact VarDb and convention output.
3. **TS runtime (vitest + jsdom):**
   - Rendering of every element.
   - The scheduler's change-driven behaviour, verified by counting DOM writes.
   - Timing functions with a fake clock.
   - ARINC decode, state priority, FSM transitions, page selection and widgets.
   - The same conformance corpus.
4. **Build integration** (CI job with Node 22):
   - Installs Mach 1.0.3 and the 1.3.x `latest` tag.
   - Builds `fixtures/sample-aircraft`.
   - Asserts the ACE project structure matches §9.3 exactly.
   - Loads each `bundle.js` in jsdom with ACE's DOM contract and **only** ACE's globals (§9.4), ticks the `update` event, and asserts the DOM output. This test determines the polyfill list.
5. **A380X smoke** (CI, nightly; local on demand):
   - Sparse shallow clone of `flybywiresim/aircraft` (`fbw-a380x`, `fbw-common`).
   - Run the scan and assert: more than 1,000 L: vars, known vars present, the `A380X_` prefix inferred, ARINC vars detected, PFD/ND/EWD/SD suggestions from `panel.cfg`.
   - Rebuild a reduced A380X PFD (attitude, speed tape, altitude tape, FMA text, Direct Law state) as `fixtures/a380x-pfd/` and compile it.
6. **UI:** React Testing Library for panels and inspector logic. A manual end-to-end checklist in `docs/testing/e2e-checklist.md` covers create project → scan → draw → bind → state → compile → open in ACE.
7. **CI:** GitHub Actions on windows-latest and ubuntu-latest: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `npm test`, build integration; the A380X smoke job runs nightly.

## 13. Packaging

- **Windows:**
  - `npm run tauri build -- --no-bundle`, then `ISCC installer/windows/avd.iss` (Inno Setup 7.1).
  - The installer ships the exe and `resources/`: `@avd/runtime` dist, `@avd/mach-plugin`, the pinned `@microsoft/msfs-sdk` and the dialect `.d.ts` files.
  - It installs WebView2 from the embedded bootstrapper if the registry check fails.
  - A custom **"Link runtimes"** wizard page asks for:
    - the Node executable, auto-filled from PATH;
    - the Mach folder, auto-filled from `%APPDATA%\npm\node_modules\@synaptic-simulations\mach` if present;
    - the ACE folder.
  - Each link is validated as in §9.5, with blocking error messages.
  - The page writes `config.json` as UTF-8 and is per-user (`PrivilegesRequired=lowest`).
  - The uninstaller keeps the config unless the user ticks "remove settings".
- **Linux:** the Tauri bundler produces `.deb` and `.AppImage`. On first run, if `config.json` is missing or invalid, the app shows the same "Link runtimes" screen.
- **Version and release:** Semver. GitHub Releases attach the Windows installer, the `.deb`, the AppImage and `avd-relay.exe`.

## 14. Risks and mitigations

| Risk | Mitigation |
|---|---|
| msfs-sdk touching missing globals in ACE | Polyfill banner, with its list derived from the jsdom ACE test (§12.4) |
| Mach version drift | Version-aware config generation, capability detection, CI against two versions |
| oxc AST API churn | Exact version pin; all AST access isolated in `avd-syntax/src/oxc_adapter.rs` |
| Rebuild latency in design mode | Fast-path DOM patching (§3.3 step 6); drag ghosting; esbuild incremental builds via `mach watch` |
| Coherent GT vs WebView2/WebKitGTK rendering differences | Runtime uses conservative SVG features only (no CSS variables, no foreignObject, filters only for glow/shadow) |
| SimConnect H: events | MobiFlight WASM channel when present; documented limitation otherwise |
| Large repos (A380X) scan time | Parallel extraction (rayon), cached index, incremental rescan |
