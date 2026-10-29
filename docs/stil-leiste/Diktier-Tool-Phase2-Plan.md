# Stil-Leiste für Handy – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eigener Handy-Build, in dem `Alt+Leertaste` jedes Diktat mit einer Stil-Leiste (`0 Roh · 1 Standard · 2 E-Mail · 3 WhatsApp`) startet. Die Leiste lässt sich per Zifferntaste umschalten, und der gewählte Stil bestimmt die Mistral-Aufbereitung.

**Architecture:**
- Ein neues Rust-Modul `style_bar` hält den Stil des laufenden Diktats und meldet jede Änderung als Event an das Overlay.
- Ein Windows-Tastatur-Hook (`digit_capture`) fängt die Ziffern ab. Er ist nur aktiv, solange der Stil noch umschaltbar ist.
- `actions.rs` startet die Sitzung beim Aufnahmebeginn und friert den Stil direkt vor der Aufbereitung ein. Die gewählte Prompt-ID wird ausdrücklich durchgereicht, statt sie aus den Einstellungen zu lesen.
- Das React-Overlay zeichnet die Leiste und die umgebaute Pille in den 3-t-s-Farben.

**Tech Stack:**
- Tauri 2
- Rust (Crate `windows` 0.61 für `WH_KEYBOARD_LL`)
- React/TypeScript
- Bun
- Ausgangsstand: Handy v0.9.8

**Spec:** `AI\Setup\Diktier-Tool-Phase2-Design.md` (freigegeben 2026-10-28). Beim Ausführen beide Dateien lesen.

## Global Constraints

- **Repo:** GitHub-Fork `shomann83/handy-diktier-app` von `cjpais/Handy`, Arbeitszweig `stil-leiste`.
- **Lokaler Klon:** `C:\dev\handy-diktier-app` (nicht in OneDrive, nicht unter `C:\Program Files`).
- **Ziffern:** `0` = Roh (fest). `1`–`9` = `post_process_prompts` in Listenreihenfolge, angezeigt wird der Prompt-Name. Startwert ist immer `1`; gibt es keinen Prompt, ist es `0`.
- **Stil gilt nur für ein Diktat.** Das Umschalten verändert `post_process_selected_prompt_id` in den Einstellungen nie.
- **Umschaltbar** während Aufnahme und Transkription. Ab dem Moment, in dem die Aufbereitung startet (bei Roh: das Einfügen), ist der Stil fest und die Leiste wird abgeblendet.
- **Ziffern werden nur während einer umschaltbaren Sitzung abgefangen.** Das gilt für obere Reihe und Ziffernblock, mit und ohne gehaltenes Alt. Sie werden geschluckt und landen nie im Zielfenster. Ziffern ohne passenden Stil werden ebenfalls geschluckt und ignoriert.
- **Farben:**
  - Flächen: `#000000` (leicht deckend)
  - inaktive Stile: Text `#CED3DC`
  - aktiver Stil: Fläche `#35D6DD`, Text `#000000`, fett
  - Pegelanzeige und Kreisel: `#35D6DD`
  - Aufnahme-Punkt: `#FFFF00` (einziges Gelb)
  - Statustexte: `#FFFFFF`
  - Schrift: Lato
- **Pille (Variante 2):** genauso breit wie die Leiste. Links Punkt, Mitte Pegelanzeige über die volle Breite ohne Text, rechts Zeit + ✕.
- **Texte:**
  - Roh-Feld: „Roh“
  - KI-Phase: „KI bereitet auf ({{style}})…“
  - Transkription: wie bisher „Transkribiere...“
- **Updates:** Update-Prüfung standardmäßig aus, `createUpdaterArtifacts: false`, Installationspfad bleibt `%LOCALAPPDATA%\Handy`.
- **Sicherheit:** Kein API-Schlüssel in Repo oder Logs. Keine Datei ohne Sebastians Bestätigung löschen.
- **Dokumentation:** Jede Installation wird in `AI\Setup\Installierte-Programme.md` eingetragen.

## Review Focus

1. **Alt nach geschluckter Ziffer loslassen** (`Alt+Leertaste` halten, `2`, loslassen): In Outlook/Word darf weder die Menü-Tastenleiste aktiviert werden noch etwas anderes passieren. → manueller Test in Task 6, Schritt 4.
2. **Hook bleibt hängen**, z. B. weil die Transkription mit Fehler oder Panik endet: Danach müssen 0–9 überall wieder normal tippen. → `end_is_idempotent_and_stops_capture` (Task 3) und `FinishGuard` ruft `style_bar::end` (Task 4).
3. **Abbruch mit Esc oder ✕ während der Transkription:** Die Leiste verschwindet, die Ziffern sind freigegeben, nichts wird eingefügt. → `cancel_ends_session` (Task 3) und manueller Test Task 6.
4. **Ziffer nach dem Einfrieren** (Aufbereitung läuft schon): Sie ändert den Stil nicht mehr und tippt wieder normal. → `select_after_lock_is_ignored` (Task 3).
5. **Prompt-Liste geändert** zwischen Aufnahme-Start und Einfrieren (Prompt gelöscht): Es wird der Rohtext eingefügt (bisheriges Rückfallverhalten), kein Absturz. → `post_process_with_unknown_prompt_returns_none` (Task 4).

Hinweis zur Parallelität: Handys `TranscriptionCoordinator` merkt sich einen Tastendruck während `Stage::Processing` vor und startet die nächste Aufnahme erst nach `notify_processing_finished()`. Es läuft also nie mehr als eine Sitzung gleichzeitig. Deshalb muss `style_bar::end` in `FinishGuard::drop` **vor** `notify_processing_finished()` stehen.

---

### Task 1: Werkstatt, Fork und Basis-Build

**Files:**
- Modify: `src-tauri/tauri.conf.json` (`"createUpdaterArtifacts": true` → `false`)
- Modify: `src-tauri/src/settings.rs` (`default_update_checks_enabled()` → `false`)
- Create: `docs/stil-leiste/Diktier-Tool-Phase2-Design.md`, `docs/stil-leiste/Diktier-Tool-Phase2-Plan.md` (Kopien aus `AI\Setup\`)
- Modify (OneDrive): `AI\Setup\Installierte-Programme.md`

**Interfaces:** Produces: baubares Repo unter `C:\dev\handy-diktier-app`, Zweig `stil-leiste`.

- [ ] **Step 1: Werkzeuge installieren**
  ```powershell
  winget install -e --id Rustlang.Rustup; winget install -e --id Oven-sh.Bun; winget install -e --id Kitware.CMake
  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
  ```
  Danach `rustup default stable-msvc`.
  - Das Vulkan SDK (`KhronosGroup.VulkanSDK`) nur installieren, wenn Schritt 4 ohne es scheitert.
  - Alles Installierte sofort in `Installierte-Programme.md` eintragen: Abschnitt „Entwicklung Handy-Fork“ mit winget-ID und Zweck.
- [ ] **Step 2: Fork und Klon**
  ```powershell
  gh repo fork cjpais/Handy --fork-name handy-diktier-app --clone=false
  git clone https://github.com/shomann83/handy-diktier-app C:\dev\handy-diktier-app
  cd C:\dev\handy-diktier-app; git switch -c stil-leiste
  ```
- [ ] **Step 3: Basis prüfen.**
  - `bun install`
  - `cd src-tauri; cargo test` → alle Tests PASS (Ausgangszustand festhalten; vorhandene Fehlschläge notieren, nicht beheben)
  - `bun run test:keyboard` → PASS
- [ ] **Step 4: Basis-Build.**
  - `bun run tauri build` → Installer unter `src-tauri\target\release\bundle\nsis\`
  - Erst danach `createUpdaterArtifacts` und den Update-Default ändern, neu bauen → wieder erfolgreich. Ohne Signaturschlüssel muss der Build trotzdem durchlaufen.
- [ ] **Step 5: Commit + Push**
  ```bash
  git add -A && git commit -m "chore: Fork-Basis – Updater aus, Design und Plan abgelegt" && git push -u origin stil-leiste
  ```

---

### Task 2: Ziffern-Hook (`digit_capture`)

**Files:**
- Create: `src-tauri/src/digit_capture.rs`
- Modify: `src-tauri/src/lib.rs` (`mod digit_capture;`)
- Modify: `src-tauri/Cargo.toml`: `windows`-Features ergänzen, falls für `SetWindowsHookExW`, `GetMessageW`, `PostThreadMessageW`, `GetCurrentThreadId` nötig (`Win32_UI_WindowsAndMessaging` ist vorhanden; ggf. `Win32_System_Threading`)

**Interfaces:**
- Produces:
  - `pub fn digit_from_vk(vk: u32) -> Option<u8>`
  - `pub fn start(on_digit: Box<dyn Fn(u8) + Send + Sync + 'static>)`
  - `pub fn stop()`
  - Beide Funktionen sind idempotent. Auf Nicht-Windows-Systemen sind sie No-ops.

- [ ] **Step 1: Failing tests** (in `digit_capture.rs`, `#[cfg(test)]`):
  ```rust
  #[test] fn top_row_digits_map() { for d in 0..=9u8 { assert_eq!(digit_from_vk(0x30 + d as u32), Some(d)); } }
  #[test] fn numpad_digits_map() { for d in 0..=9u8 { assert_eq!(digit_from_vk(0x60 + d as u32), Some(d)); } }
  #[test] fn other_keys_do_not_map() { for vk in [0x20u32, 0x41, 0x12, 0x1B, 0x2F, 0x3A, 0x5F, 0x6A] { assert_eq!(digit_from_vk(vk), None); } }
  ```
- [ ] **Step 2:** `cargo test digit_capture` → FAIL (Funktion fehlt)
- [ ] **Step 3: Implementieren.** Ablauf:
  - `start` öffnet einen eigenen Thread, installiert dort `SetWindowsHookExW(WH_KEYBOARD_LL, …)` und betreibt eine `GetMessageW`-Schleife.
  - Die Hook-Prozedur gibt für jede Nachricht (`WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, `WM_SYSKEYUP`) mit `digit_from_vk(..).is_some()` den Wert `1` zurück, die Taste wird also geschluckt. Bei Key-Down ruft sie zusätzlich `on_digit` auf.
  - Alle anderen Tasten gehen per `CallNextHookEx` weiter.
  - Eingefügte Ereignisse (`LLKHF_INJECTED`) werden **nicht** geschluckt, damit Handys eigenes Einfügen nie blockiert wird.
  - `stop` schickt `WM_QUIT` per `PostThreadMessageW`. Der Thread entfernt den Hook und endet.
  - Zustand in einem `static Mutex<Option<(thread_id, JoinHandle)>>`. Den Callback hält ein `static`, weil die Hook-Prozedur keinen Kontext bekommt.
- [ ] **Step 4:** `cargo test digit_capture` → PASS; `cargo build` → ok.
- [ ] **Step 5: Commit** `feat: Ziffern-Hook für die Stil-Leiste`

---

### Task 3: Stil-Sitzung (`style_bar`)

**Files:**
- Create: `src-tauri/src/style_bar.rs`
- Modify: `src-tauri/src/lib.rs` (`mod style_bar;`)

**Interfaces:**
- Consumes: `digit_capture::{start, stop}` (Task 2), `settings::LLMPrompt`
- Produces:
  ```rust
  #[derive(Clone, Debug, PartialEq)] pub enum StyleChoice { Raw, Prompt(String) }      // Prompt-ID
  #[derive(Clone, Debug, Serialize, Type)] pub struct StyleBarEvent { pub visible: bool, pub labels: Vec<String>, pub active: u8, pub locked: bool }
  pub trait DigitCapture: Send + Sync { fn start(&self); fn stop(&self); }
  pub struct StyleSession { /* labels, prompt_ids, active, locked */ }
  impl StyleSession {
      pub fn new(prompts: &[LLMPrompt], raw_label: &str) -> Self;   // labels[0]=raw_label, dann bis zu 9 Prompt-Namen
      pub fn select(&mut self, digit: u8) -> bool;                    // true = geändert
      pub fn lock(&mut self) -> StyleChoice;
      pub fn event(&self) -> StyleBarEvent;                           // visible = true
  }
  // App-Ebene (globaler Zustand, emittiert "style-bar-update" an das Fenster "recording_overlay"):
  pub fn begin(app: &AppHandle);
  pub fn lock(app: &AppHandle) -> Option<StyleChoice>;   // None = keine Sitzung aktiv
  pub fn end(app: &AppHandle);
  ```
  Die App-Funktionen nutzen intern eine Hilfsschicht, damit die Tests ohne Tauri laufen:
  ```rust
  pub struct SessionCell { session: Option<StyleSession>, capture: Arc<dyn DigitCapture> }
  impl SessionCell {
      pub fn new(capture: Arc<dyn DigitCapture>) -> Self;
      pub fn begin(&mut self, session: StyleSession) -> StyleBarEvent;    // startet capture
      pub fn select(&mut self, digit: u8) -> Option<StyleBarEvent>;      // Some nur bei Änderung
      pub fn lock(&mut self) -> Option<StyleChoice>;                     // stoppt capture, Sitzung bleibt (locked) bis end
      pub fn end(&mut self) -> StyleBarEvent;                            // stoppt capture, session = None, visible=false
  }
  ```
  Test-Helfer: `three()` → `StyleSession::new(&[p("a","Standard"),p("b","E-Mail"),p("c","WhatsApp")], "Roh")`; `cell_with_fake()` → `(SessionCell, Arc<FakeCapture>)`, Zellen in Tests als `let (mut cell, cap)`. Die Produktiv-Implementierung von `DigitCapture` ruft `digit_capture::start` mit einem Callback auf, der `select` und ein Event auslöst. Das Roh-Label kommt aus einem festen Text „Roh“; die UI übersetzt ohnehin, siehe Task 5.

- [ ] **Step 1: Failing tests** (Prompt-Hilfe: `fn p(id:&str,name:&str)->LLMPrompt`; `FakeCapture` mit `AtomicUsize`-Zählern für start/stop):
  ```rust
  #[test] fn starts_on_first_prompt() { let s = StyleSession::new(&[p("a","Standard"),p("b","E-Mail"),p("c","WhatsApp")], "Roh");
      assert_eq!(s.event().labels, vec!["Roh","Standard","E-Mail","WhatsApp"]); assert_eq!(s.event().active, 1); assert!(!s.event().locked); }
  #[test] fn no_prompts_starts_raw() { assert_eq!(StyleSession::new(&[], "Roh").event().active, 0); }
  #[test] fn at_most_nine_prompts() { let ps: Vec<_> = (0..12).map(|i| p(&i.to_string(), "x")).collect();
      assert_eq!(StyleSession::new(&ps, "Roh").event().labels.len(), 10); }
  #[test] fn select_valid_digit() { let mut s = three(); assert!(s.select(2)); assert_eq!(s.event().active, 2); }
  #[test] fn select_out_of_range_ignored() { let mut s = three(); assert!(!s.select(7)); assert_eq!(s.event().active, 1); }
  #[test] fn lock_returns_prompt_id() { let mut s = three(); s.select(3); assert_eq!(s.lock(), StyleChoice::Prompt("c".into())); assert!(s.event().locked); }
  #[test] fn lock_raw() { let mut s = three(); s.select(0); assert_eq!(s.lock(), StyleChoice::Raw); }
  #[test] fn select_after_lock_is_ignored() { let mut s = three(); s.lock(); assert!(!s.select(2)); assert_eq!(s.event().active, 1); }
  #[test] fn begin_starts_capture_lock_stops_it() { let (mut cell, cap) = cell_with_fake(); cell.begin(three()); assert_eq!(cap.starts(), 1);
      assert_eq!(cell.lock(), Some(StyleChoice::Prompt("a".into()))); assert_eq!(cap.stops(), 1); }
  #[test] fn end_is_idempotent_and_stops_capture() { let (mut cell, cap) = cell_with_fake(); cell.begin(three()); cell.end(); cell.end();
      assert!(cap.stops() >= 1); assert_eq!(cell.lock(), None); }
  #[test] fn cancel_ends_session() { let (mut cell, cap) = cell_with_fake(); cell.begin(three()); cell.select(2); cell.end();
      assert_eq!(cell.lock(), None); assert!(cap.stops() >= 1); }
  ```
- [ ] **Step 2:** `cargo test style_bar` → FAIL
- [ ] **Step 3: Implementieren.**
  - `StyleSession`, `SessionCell` (mit `begin/select/lock/end`, die das jeweilige `StyleBarEvent` zurückgeben oder auslösen) und die App-Funktionen.
  - Globaler Zustand: `static CELL: Lazy<Mutex<SessionCell>>`.
  - `end` sendet `StyleBarEvent { visible: false, labels: vec![], active: 0, locked: false }`.
  - Events gehen per `app.emit_to("recording_overlay", "style-bar-update", ev)`. Den exakten Fensternamen aus `overlay.rs` übernehmen.
  - `StyleBarEvent` für specta registrieren wie die anderen Events, damit `src/bindings.ts` den Typ erzeugt.
- [ ] **Step 4:** `cargo test style_bar` → PASS
- [ ] **Step 5: Commit** `feat: Stil-Sitzung mit Ziffern-Auswahl`

---

### Task 4: Ablauf in `actions.rs` anbinden

**Files:**
- Modify: `src-tauri/src/actions.rs`:
  - `post_process_transcription` (Zeile 178)
  - `process_transcription_output` (Zeile 410)
  - `FinishGuard::drop` (Zeile 93)
  - `TranscribeAction` (Zeilen 114, 445–620, 758–766)
  - `ACTION_MAP` (Zeile 965)
- Modify: `src-tauri/src/managers/transcription_jobs.rs:310` (Aufrufer)
- Modify: `src-tauri/src/utils.rs` (`cancel_current_operation` → zusätzlich `style_bar::end`)

**Interfaces:**
- Consumes: `style_bar::{begin, lock, end, StyleChoice}` (Task 3)
- Produces:
  ```rust
  async fn post_process_transcription(settings: &AppSettings, transcription: &str, prompt_id: &str) -> Option<String>;
  pub(crate) async fn process_transcription_output(app: &AppHandle, transcription: &str, prompt_id: Option<&str>) -> ProcessedTranscription;
  // prompt_id None = keine Aufbereitung (ersetzt den bool `post_process`)
  struct TranscribeAction { post_process: bool, style_bar: bool }
  ```

- [ ] **Step 1: Failing test** (in `actions.rs` tests, `#[tokio::test]`):
  ```rust
  #[tokio::test] async fn post_process_with_unknown_prompt_returns_none() {
      let mut s = crate::settings::get_default_settings();
      s.post_process_models.insert(s.post_process_provider_id.clone(), "m".into());
      assert_eq!(post_process_transcription(&s, "hallo", "gibt-es-nicht").await, None); }
  ```
- [ ] **Step 2:** `cargo test post_process_with_unknown_prompt` → FAIL (Signatur passt nicht)
- [ ] **Step 3: Implementieren.**
  - `post_process_transcription` sucht den Prompt über das neue `prompt_id`, nicht mehr über `post_process_selected_prompt_id`.
  - `process_transcription_output` nimmt `prompt_id: Option<&str>` und füllt `post_process_prompt` aus genau diesem Prompt.
  - `transcription_jobs.rs` übergibt `settings.post_process_selected_prompt_id.as_deref()`, wenn es bisher `true` übergab, sonst `None`.
  - `ACTION_MAP`:
    - `"transcribe"` → `TranscribeAction { post_process: false, style_bar: true }`
    - `"transcribe_with_post_process"` → `{ post_process: true, style_bar: false }`
  - `start`: Wenn `self.style_bar`, direkt nach erfolgreichem `try_start_recording` (bei `register_cancel_shortcut`) `style_bar::begin(app)` aufrufen; im Fehlerzweig `style_bar::end(app)`.
  - `stop` (async-Teil, vor Zeile 758):
    - `let choice = if style_bar { style_bar::lock(&ah) } else { None };`
    - `prompt_id` ergibt sich so:
      - `Some(StyleChoice::Prompt(id))` → `Some(id)`
      - `Some(StyleChoice::Raw)` → `None`
      - `None` → bei `post_process` `settings.post_process_selected_prompt_id`, sonst `None`
    - `show_processing_overlay` nur, wenn `prompt_id.is_some()`.
    - Den Verlauf mit `post_process = prompt_id.is_some()` speichern.
  - `FinishGuard::drop`: `crate::style_bar::end(&self.0);` als **erste** Zeile.
  - `utils::cancel_current_operation`: zusätzlich `crate::style_bar::end(app)`.
- [ ] **Step 4:**
  - `cargo test` → alles PASS (inkl. Task 2/3)
  - `cargo clippy -- -D warnings` → ohne neue Warnungen
- [ ] **Step 5: Smoke-Test.**
  - `bun run tauri dev`
  - `Alt+Leertaste` halten, „Test eins zwei“ sprechen, dabei `2` drücken, loslassen.
  - Im Debug-Log (Strg+Umschalt+D) erscheint die Aufbereitung mit dem E-Mail-Prompt.
  - Im Editor erscheint **keine** „2“.
- [ ] **Step 6: Commit** `feat: Stil pro Diktat statt globaler Prompt-Auswahl`

---

### Task 5: Overlay – Stil-Leiste, Pille, Farben

**Files:**
- Create: `src/overlay/styleBar.ts`, `src/overlay/styleBar.test.ts`, `src/overlay/StyleBar.tsx`
- Create: `src/overlay/fonts/Lato-Regular.ttf`, `src/overlay/fonts/Lato-Heavy.ttf` (Kopie aus `AI\Context\assets\3-t-s\fonts\`, OFL-Lizenz)
- Modify: `src/overlay/RecordingOverlay.tsx` (Listener + Einbau oberhalb `.scard`, Minimal-Zweig ab Zeile ~343), `src/overlay/RecordingOverlay.css`
- Modify: `src-tauri/src/overlay.rs:49-50` (`OVERLAY_WIDTH` 256 → 320, `OVERLAY_HEIGHT` 50 → 86)
- Modify: `src/i18n/locales/de/translation.json`, `src/i18n/locales/en/translation.json` (`overlay.styleRaw`, `overlay.processingWithStyle`)

**Interfaces:**
- Consumes: Event `style-bar-update` mit `StyleBarEvent` (Task 3)
- Produces (`styleBar.ts`):
  ```ts
  export type StyleBarState = { visible: boolean; labels: string[]; active: number; locked: boolean };
  export const HIDDEN: StyleBarState;
  export function chipLabels(state: StyleBarState, rawLabel: string): string[];        // labels[0] → rawLabel
  export function processingLabel(state: StyleBarState, fallback: string, withStyle: (style: string) => string): string;
  ```

- [ ] **Step 1: Failing tests** (`bun test src/overlay/styleBar.test.ts`):
  ```ts
  test("raw chip is localized", () => expect(chipLabels({visible:true,labels:["Roh","Standard"],active:1,locked:false}, "Raw")).toEqual(["Raw","Standard"]));
  test("processing label names active style", () => expect(processingLabel({visible:true,labels:["Roh","Standard","E-Mail"],active:2,locked:true}, "Verarbeite...", s => `KI bereitet auf (${s})…`)).toBe("KI bereitet auf (E-Mail)…"));
  test("processing label falls back when bar hidden", () => expect(processingLabel(HIDDEN, "Verarbeite...", s => s)).toBe("Verarbeite..."));
  ```
- [ ] **Step 2:** `bun test src/overlay/styleBar.test.ts` → FAIL
- [ ] **Step 3: Implementieren.**
  - `styleBar.ts`.
  - `StyleBar.tsx`: gleich breite Felder `<kbd>{i}</kbd>{label}`. Der aktive Stil bekommt die Klasse `on`, abgeblendet wird mit `locked` (inaktive Felder `opacity: .45`).
  - `RecordingOverlay.tsx`:
    - Listener `style-bar-update` → State, beim Ausblenden auf `HIDDEN` zurücksetzen.
    - Leiste über der Pille rendern, wenn `visible`.
    - In der KI-Phase `processingLabel(...)` statt `t("overlay.processing")`.
  - CSS:
    - `@font-face` für Lato 400/800.
    - Farben laut Global Constraints auf `.scard`, `.chip`, Welle, Punkt, Kreisel, Zeit, ✕. Die Farben gelten fest, unabhängig vom Theme.
    - `--ov-pill-w` / `--ov-work-w` = Leistenbreite (300px).
    - Minimal-Zeile (Aufnahme) neu anordnen: Punkt | Welle mit `flex:1` | Zeit + ✕.
  - Übersetzungen:
    - `de`: `"styleRaw": "Roh"`, `"processingWithStyle": "KI bereitet auf ({{style}})…"`
    - `en`: `"Raw"`, `"Polishing ({{style}})…"`
    - Danach `bun run check:translations`. Falls das Skript fehlende Schlüssel in anderen Sprachen bemängelt, dort die englischen Texte eintragen.
- [ ] **Step 4:**
  - `bun test src/overlay/styleBar.test.ts` → PASS
  - `bun run lint` → ok
  - `bun run build` → ok
- [ ] **Step 5: Sichtprüfung.**
  - In `bun run tauri dev` diktieren, Screenshot vom Overlay in den Phasen Aufnahme, Umschalten, Transkribiere und KI-Phase.
  - Abgleich mit Design-Skizze und Farben, in heller und dunkler Windows-Darstellung.
- [ ] **Step 6: Commit** `feat: Stil-Leiste und Pille im 3-t-s-Design`

---

### Task 6: Installieren, Abnahme, Doku

**Files:**
- Modify (OneDrive): `AI\Setup\Diktier-Tool-Handy.md` (neuer Abschnitt „Phase 2: Stil-Leiste“ mit Bedienung + Update-Weg), `AI\Setup\Installierte-Programme.md` (Abschnitt 4f: eigener Build, winget-Pin)

- [ ] **Step 1: Vorbereiten.**
  - `winget pin add cjpais.Handy`
  - Sicherung: `Copy-Item $env:APPDATA\com.pais.handy\settings_store.json $env:APPDATA\com.pais.handy\settings_store.json.bak-phase2`
- [ ] **Step 2: Bauen und installieren.**
  - `bun run tauri build`
  - Den NSIS-Installer aus `src-tauri\target\release\bundle\nsis\` ausführen. Installationsziel bleibt `%LOCALAPPDATA%\Handy`.
  - Handy starten. Prompts, Wörterbuch und Schlüssel sind noch da.
  - In Handy prüfen: Die Update-Prüfung ist aus.
- [ ] **Step 3: Zweites Kürzel lösen.** In Sebastians Handy die Belegung `Strg+Umschalt+Leertaste` entfernen bzw. ändern, damit nur `Alt+Leertaste` aktiv ist.
- [ ] **Step 4: Abnahme mit Sebastian.** Alle zehn Punkte aus „Abnahme“ im Design durchgehen, dazu Review Focus 1–5. Ergebnis je Punkt notieren. Fehlschläge zuerst beheben und bauen, dann erneut prüfen.
- [ ] **Step 5: Push und Doku.**
  - `git push`. Pull Request `stil-leiste` → `main` **im eigenen Fork** öffnen und mergen (nicht an `cjpais/Handy`).
  - Doku-Dateien wie oben aktualisieren, mit Update-Weg: `git fetch upstream; git merge upstream/main`, Konflikte lösen, Build, Installation.
  - Den Merker zum Verschieben des Entwicklungsordners in `BrainBox\tasks.md` bleibt offen.
