//! Per-dictation style selection shown in the recording overlay.
//!
//! A session starts with the recording and offers `0` = raw (no LLM) plus up
//! to nine post-processing prompts in list order, starting on the first
//! prompt. Digits switch the style until the session is locked right before
//! post-processing (or paste, for raw). The choice never touches the stored
//! `post_process_selected_prompt_id`.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use tauri::AppHandle;
use tauri_specta::Event;

use crate::settings::{get_settings, AppSettings, LLMPrompt};

/// Label of the raw slot; the overlay replaces it with its localized text.
const RAW_LABEL: &str = "Roh";
const MAX_PROMPTS: usize = 9;

#[derive(Clone, Debug, PartialEq)]
pub enum StyleChoice {
    Raw,
    /// Post-processing prompt id.
    Prompt(String),
}

/// Sent to the overlay whenever the style bar changes.
#[derive(Clone, Debug, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct StyleBarEvent {
    pub visible: bool,
    pub labels: Vec<String>,
    pub active: u8,
    pub locked: bool,
}

impl StyleBarEvent {
    fn hidden() -> Self {
        Self {
            visible: false,
            labels: Vec::new(),
            active: 0,
            locked: false,
        }
    }
}

/// Starts and stops the system-wide digit capture.
pub trait DigitCapture: Send + Sync {
    fn start(&self);
    fn stop(&self);
}

pub struct StyleSession {
    labels: Vec<String>,
    /// Prompt id per slot; `None` for the raw slot 0.
    prompt_ids: Vec<Option<String>>,
    active: u8,
    locked: bool,
}

impl StyleSession {
    pub fn new(prompts: &[LLMPrompt], raw_label: &str) -> Self {
        let mut labels = vec![raw_label.to_string()];
        let mut prompt_ids = vec![None];
        for prompt in prompts.iter().take(MAX_PROMPTS) {
            labels.push(prompt.name.clone());
            prompt_ids.push(Some(prompt.id.clone()));
        }
        let active = if prompt_ids.len() > 1 { 1 } else { 0 };
        Self {
            labels,
            prompt_ids,
            active,
            locked: false,
        }
    }

    /// Switch to `digit`; returns whether the active style changed.
    pub fn select(&mut self, digit: u8) -> bool {
        if self.locked || digit as usize >= self.labels.len() || digit == self.active {
            return false;
        }
        self.active = digit;
        true
    }

    pub fn lock(&mut self) -> StyleChoice {
        self.locked = true;
        match &self.prompt_ids[self.active as usize] {
            Some(id) => StyleChoice::Prompt(id.clone()),
            None => StyleChoice::Raw,
        }
    }

    pub fn event(&self) -> StyleBarEvent {
        StyleBarEvent {
            visible: true,
            labels: self.labels.clone(),
            active: self.active,
            locked: self.locked,
        }
    }
}

/// The current session plus the capture it controls, kept free of Tauri so
/// the lifecycle is testable.
pub struct SessionCell {
    session: Option<StyleSession>,
    capture: Arc<dyn DigitCapture>,
}

impl SessionCell {
    pub fn new(capture: Arc<dyn DigitCapture>) -> Self {
        Self {
            session: None,
            capture,
        }
    }

    pub fn begin(&mut self, session: StyleSession) -> StyleBarEvent {
        let event = session.event();
        self.session = Some(session);
        self.capture.start();
        event
    }

    /// Returns the new bar state only when the style actually changed.
    pub fn select(&mut self, digit: u8) -> Option<StyleBarEvent> {
        let session = self.session.as_mut()?;
        session.select(digit).then(|| session.event())
    }

    /// Freezes the style and releases the digits. The session stays (locked)
    /// until `end` so the overlay keeps showing it during post-processing.
    pub fn lock(&mut self) -> Option<StyleChoice> {
        let choice = self.session.as_mut()?.lock();
        self.capture.stop();
        Some(choice)
    }

    pub fn current_event(&self) -> Option<StyleBarEvent> {
        self.session.as_ref().map(StyleSession::event)
    }

    /// Closes the session and releases the digits. Returns the hidden-bar event
    /// only for a session that was still switchable; after the lock the bar
    /// keeps showing the style until the overlay itself hides.
    pub fn end(&mut self) -> Option<StyleBarEvent> {
        let session = self.session.take();
        self.capture.stop();
        session
            .filter(|s| !s.locked)
            .map(|_| StyleBarEvent::hidden())
    }
}

// ---- App level -------------------------------------------------------------

static APP: OnceLock<AppHandle> = OnceLock::new();

struct HookCapture;

impl DigitCapture for HookCapture {
    fn start(&self) {
        crate::digit_capture::start(Box::new(|digit| {
            // The hook procedure must return fast and `end`/`lock` join the
            // hook thread while holding CELL, so never touch CELL from here.
            tauri::async_runtime::spawn(async move {
                if let Some(app) = APP.get() {
                    select(app, digit);
                }
            });
        }));
    }

    fn stop(&self) {
        crate::digit_capture::stop();
    }
}

static CELL: LazyLock<Mutex<SessionCell>> =
    LazyLock::new(|| Mutex::new(SessionCell::new(Arc::new(HookCapture))));

fn cell() -> std::sync::MutexGuard<'static, SessionCell> {
    CELL.lock().unwrap_or_else(|e| e.into_inner())
}

fn emit(app: &AppHandle, event: StyleBarEvent) {
    if let Err(e) = event.emit_to(app, "recording_overlay") {
        log::warn!("Failed to emit style bar event: {e}");
    }
}

/// Prompts offered in the bar. With post-processing switched off only raw is
/// offered, so nothing is sent to the LLM provider.
fn session_prompts(settings: &AppSettings) -> &[LLMPrompt] {
    if settings.post_process_enabled {
        &settings.post_process_prompts
    } else {
        &[]
    }
}

/// Open a session for the dictation that is starting.
pub fn begin(app: &AppHandle) {
    let _ = APP.set(app.clone());
    let settings = get_settings(app);
    let session = StyleSession::new(session_prompts(&settings), RAW_LABEL);
    let event = cell().begin(session);
    emit(app, event);
}

fn select(app: &AppHandle, digit: u8) {
    let event = cell().select(digit);
    if let Some(event) = event {
        emit(app, event);
    }
}

/// Freeze the style; `None` when no session is open.
pub fn lock(app: &AppHandle) -> Option<StyleChoice> {
    let (choice, event) = {
        let mut cell = cell();
        let choice = cell.lock();
        (choice, cell.current_event())
    };
    if let Some(event) = event {
        emit(app, event);
    }
    choice
}

/// Close the session and release the digits. Safe to call repeatedly.
pub fn end(app: &AppHandle) {
    let event = cell().end();
    if let Some(event) = event {
        emit(app, event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::LLMPrompt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn p(id: &str, name: &str) -> LLMPrompt {
        LLMPrompt {
            id: id.to_string(),
            name: name.to_string(),
            prompt: "${output}".to_string(),
        }
    }

    fn three() -> StyleSession {
        StyleSession::new(
            &[p("a", "Standard"), p("b", "E-Mail"), p("c", "WhatsApp")],
            "Roh",
        )
    }

    #[derive(Default)]
    struct FakeCapture {
        starts: AtomicUsize,
        stops: AtomicUsize,
    }

    impl FakeCapture {
        fn starts(&self) -> usize {
            self.starts.load(Ordering::SeqCst)
        }
        fn stops(&self) -> usize {
            self.stops.load(Ordering::SeqCst)
        }
    }

    impl DigitCapture for FakeCapture {
        fn start(&self) {
            self.starts.fetch_add(1, Ordering::SeqCst);
        }
        fn stop(&self) {
            self.stops.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn cell_with_fake() -> (SessionCell, Arc<FakeCapture>) {
        let cap = Arc::new(FakeCapture::default());
        (SessionCell::new(cap.clone()), cap)
    }

    #[test]
    fn starts_on_first_prompt() {
        let s = three();
        assert_eq!(
            s.event().labels,
            vec!["Roh", "Standard", "E-Mail", "WhatsApp"]
        );
        assert_eq!(s.event().active, 1);
        assert!(!s.event().locked);
        assert!(s.event().visible);
    }

    #[test]
    fn no_prompts_starts_raw() {
        assert_eq!(StyleSession::new(&[], "Roh").event().active, 0);
    }

    #[test]
    fn at_most_nine_prompts() {
        let ps: Vec<_> = (0..12).map(|i| p(&i.to_string(), "x")).collect();
        assert_eq!(StyleSession::new(&ps, "Roh").event().labels.len(), 10);
    }

    #[test]
    fn select_valid_digit() {
        let mut s = three();
        assert!(s.select(2));
        assert_eq!(s.event().active, 2);
    }

    #[test]
    fn select_out_of_range_ignored() {
        let mut s = three();
        assert!(!s.select(7));
        assert_eq!(s.event().active, 1);
    }

    #[test]
    fn lock_returns_prompt_id() {
        let mut s = three();
        s.select(3);
        assert_eq!(s.lock(), StyleChoice::Prompt("c".into()));
        assert!(s.event().locked);
    }

    #[test]
    fn lock_raw() {
        let mut s = three();
        s.select(0);
        assert_eq!(s.lock(), StyleChoice::Raw);
    }

    #[test]
    fn select_after_lock_is_ignored() {
        let mut s = three();
        s.lock();
        assert!(!s.select(2));
        assert_eq!(s.event().active, 1);
    }

    #[test]
    fn begin_starts_capture_lock_stops_it() {
        let (mut cell, cap) = cell_with_fake();
        cell.begin(three());
        assert_eq!(cap.starts(), 1);
        assert_eq!(cell.lock(), Some(StyleChoice::Prompt("a".into())));
        assert_eq!(cap.stops(), 1);
    }

    #[test]
    fn end_is_idempotent_and_stops_capture() {
        let (mut cell, cap) = cell_with_fake();
        cell.begin(three());
        cell.end();
        cell.end();
        assert!(cap.stops() >= 1);
        assert_eq!(cell.lock(), None);
    }

    #[test]
    fn cancel_ends_session() {
        let (mut cell, cap) = cell_with_fake();
        cell.begin(three());
        cell.select(2);
        let hidden = cell.end().expect("unlocked session hides the bar");
        assert!(!hidden.visible);
        assert_eq!(cell.lock(), None);
        assert!(cap.stops() >= 1);
    }

    #[test]
    fn end_after_lock_leaves_hiding_to_the_overlay() {
        // After the lock the bar must stay (with the style name) until the
        // overlay itself hides, so `end` sends nothing.
        let (mut cell, cap) = cell_with_fake();
        cell.begin(three());
        cell.lock();
        assert!(cell.end().is_none());
        assert_eq!(cell.lock(), None);
        assert!(cap.stops() >= 1);
    }

    #[test]
    fn end_without_session_sends_nothing() {
        let (mut cell, _cap) = cell_with_fake();
        assert!(cell.end().is_none());
    }

    #[test]
    fn disabled_post_processing_offers_only_raw() {
        let mut s = crate::settings::get_default_settings();
        s.post_process_prompts = vec![p("a", "Standard"), p("b", "E-Mail")];
        s.post_process_enabled = true;
        assert_eq!(session_prompts(&s).len(), 2);
        s.post_process_enabled = false;
        assert!(session_prompts(&s).is_empty());
    }
}
