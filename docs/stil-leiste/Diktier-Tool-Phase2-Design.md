# Diktier-Tool Phase 2: Stil-Leiste im Aufnahme-Fenster (Design)

Stand: 2026-10-28 · Grundlage: Gespräch mit Sebastian, Handy v0.9.8 · Anleitung Phase 1: [[Diktier-Tool-Handy]]

## Absicht

Sebastian diktiert systemweit mit Handy (Parakeet V3 lokal, Aufbereitung über Mistral). Heute nutzt das KI-Kürzel immer den einen Prompt, der gerade in den Einstellungen ausgewählt ist. Zum Wechseln muss er jedes Mal die Einstellungen öffnen. Das bremst genau in dem Moment, in dem er schnell etwas in eine Mail oder einen Chat sprechen will.

Ziel ist, den Stil **während des Diktats** zu sehen und mit einer Zifferntaste umzuschalten, ohne die Hand von der Tastatur zu nehmen und ohne Fensterwechsel.

## Was gebaut wird (von Sebastian entschieden)

1. **Ein Kürzel für alles:** Handys Kürzel `transcribe` (aktuell `Alt+Leertaste`) startet die Aufnahme **immer mit Stil-Leiste und KI-Aufbereitung**. Das zweite Kürzel (`transcribe_with_post_process`) wird nicht mehr gebraucht. **Die Tastenkombination bleibt in Handys Einstellungen frei wählbar** (Nachtrag 28.10.2026, nichts hart einbauen). Einschränkung: Während des Diktats sind 0–9 (ohne Modifier oder mit Alt/Win) für die Stil-Wahl reserviert. Strg+Ziffer und Shift+Ziffer gehen durch (Nachtrag 29.10.2026 nach Codex-Prüfung), Kürzel wie `Strg+1` funktionieren also.
2. **Stil-Leiste über der Aufnahme-Pille:** `0 Roh · 1 Standard · 2 E-Mail · 3 WhatsApp`. Gleich breite Felder, Aussehen nach Entwurf „Variante A“ aus der Abstimmung (siehe Skizze unten).
   **Farben nach 3-t-s-CI, Variante „Schwarz + Türkis“** (statt Handy-Pink; Quelle `AI\Context\corporate-identity-3ts.md`):
   - Flächen von Leiste und Pille: Schwarz `#000000`, leicht deckend.
   - Inaktive Stile: Text Light Grey `#CED3DC`.
   - Aktiver Stil: Fläche Türkis `#35D6DD`, Text Schwarz, fett.
   - Pegelanzeige und Lade-Kreisel: Türkis `#35D6DD`.
   - Aufnahme-Punkt: Gelb `#FFFF00`. Das ist der einzige Gelb-Akzent.
   - Zeitzähler und ✕: Light Grey auf dunklem Grau.
   - Statustexte: Weiß.
   - Schrift: Lato, mit Systemschrift als Rückfall.
   - Gilt einheitlich, unabhängig vom hellen oder dunklen Windows-Design.
3. **Startwert ist immer „Standard“.** Es gibt **keine** automatische Vorwahl nach aktiver App.
4. **Umschalten mit 0–9**, solange aufgenommen wird **und** solange Handy noch transkribiert. Die Leiste zeigt den Wechsel sofort an. Sobald der Text an Mistral geht, ist der Stil fest und die Leiste wird blasser.
5. **Der Wechsel gilt nur für dieses eine Diktat.** Beim nächsten Diktat steht wieder „Standard“.
6. **„0 Roh“:** keine KI. Der erkannte Text wird direkt eingefügt (heutiges Verhalten des Roh-Kürzels, einschließlich Wörterbuch-Korrektur).
7. **Aufteilung der Pille (Variante 2):** genauso breit wie die Stil-Leiste. Links ein pulsierender Punkt, in der Mitte eine Pegelanzeige über die ganze Breite (kein Text während der Aufnahme), rechts Zeitzähler und ✕ zum Abbrechen.
8. **Phasen in der Pille:** Aufnahme → „Transkribiere…“ → „KI bereitet auf (E-Mail)…“ → Fenster verschwindet, Text wird eingefügt. Bei „Roh“ entfällt die KI-Phase.
9. **Fork auf GitHub** im Konto `shomann83`, damit der Code gesichert ist und Handy-Updates später eingespielt werden können.

```
   ┌───────────┬────────────┬──────────┬─────────────┐
   │ 0 Roh     │▓1 Standard▓│ 2 E-Mail │ 3 WhatsApp  │   ← aktiver Stil türkis
   └───────────┴────────────┴──────────┴─────────────┘
   (●  ▁▃▅▇▅▃▁▂▄▆▄▂▁▃▅▇▅▃▁▂▄▆▄▂▁       0:07   ✕ )      ← Pille, gleiche Breite
```

## Ziele

- Stil während der Aufnahme sichtbar und mit einer Taste umschaltbar.
- Kein zusätzlicher Wartemoment: Wer nichts drückt, bekommt „Standard“ ohne Verzögerung.
- Alles andere an Handy verhält sich wie bisher (Einfügen, Zwischenablage, Abbrechen mit Esc/✕, Rückfall auf den Rohtext, wenn Mistral nicht erreichbar ist).

## Nicht-Ziele

- Automatische Stilwahl nach aktiver App (bewusst verworfen).
- Bedienung per Maus in der Stil-Leiste.
- Neue Prompt-Verwaltung: Die Stile **sind** Handys vorhandene Prompts (siehe unten).
- Andere Anbieter als Mistral, Cloud-Spracherkennung, macOS/Linux-Pflege.
- Upstream-Pull-Request an Handy (kann später kommen, ist nicht Teil dieses Umbaus).

## Anti-Ziele (wäre ein Fehlschlag, auch wenn es technisch läuft)

- **Ziffern landen im Zielfenster.** Eine während der Aufnahme gedrückte 2 darf weder als „2“ in Outlook erscheinen noch dort ein Alt-Kürzel auslösen (Alt+2 ist in Office z. B. die Schnellzugriffsleiste).
- **Ziffern sind außerhalb des Diktats blockiert.** Die Tasten dürfen nur während Aufnahme und Transkription abgefangen werden.
- **Das Fenster nimmt den Fokus.** Dann landet der Text im falschen Fenster.
- **Spürbar langsamer als heute**, wenn man nicht umschaltet.
- **Ein Handy-Update überschreibt den Fork** unbemerkt.

## Rahmenbedingungen

- Windows 11, Ryzen 7 5700U, keine dedizierte GPU. Handy v0.9.8 als Ausgangsstand (Tauri 2, Rust + React/TypeScript, Bun).
- Sebastian nutzt **Halten (Push-to-Talk)**: Er hält `Alt+Leertaste` gedrückt und drückt währenddessen mit einem anderen Finger die Ziffer. Die Ziffer muss also **auch bei gehaltenem Alt** erkannt werden, ebenso ohne Modifier (nach dem Loslassen, während der Transkription). Ziffernblock-Tasten zählen wie die Ziffern der oberen Reihe.
- API-Schlüssel nur in Handys Einstellungen, nie im Repo. Standardmäßig nur Mistral als externer Anbieter.
- Keine Dateien ohne Sebastians Bestätigung löschen.
- Jede neue Installation (Build-Werkzeuge, eigener Handy-Build) wird in `AI\Setup\Installierte-Programme.md` dokumentiert.

## Zuordnung Ziffer → Stil (meine Entscheidung)

- `0` = Roh (fest).
- `1`–`9` = Handys gespeicherte Prompts (`post_process_prompts`) **in ihrer Listenreihenfolge**. Angezeigt wird der Prompt-Name. Gibt es nur drei Prompts, zeigt die Leiste `0`–`3`.
- „Standard“ = der Prompt an erster Stelle. Handy kann Prompts nicht umsortieren. Deshalb hat Sebastian den englischen Handy-Prompt „Improve Transcriptions“ selbst in Handy gelöscht (erledigt 28.10.2026). Danach lautet die Reihenfolge Standard, E-Mail, WhatsApp. Eine Sortier-Funktion wird nicht gebaut.
- Gibt es keinen Prompt oder fehlt die Mistral-Konfiguration, verhält sich jede Wahl wie „Roh“ (heutiger Rückfall).

## Technischer Ansatz (meine Entscheidungen)

- **Fork:** GitHub-Fork `shomann83/handy-diktier-app` von `cjpais/Handy` (Name von Sebastian, 28.10.2026), Arbeitszweig `stil-leiste`. Lokaler Klon **außerhalb von OneDrive**, vorübergehend unter `C:\dev\handy-diktier-app`, weil Build-Ordner groß sind und OneDrive-Sync dabei stört. Nach Abschluss wird er an einen festen Ort verschoben oder (nach Rückfrage) gelöscht, weil alles auf GitHub liegt (Merker in `tasks.md`). Nicht unter `C:\Program Files`: Bauen dort bräuchte Admin-Rechte.
- **Tasten:** Handy registriert heute schon während der Aufnahme kurzzeitig ein globales Kürzel für Abbrechen (`register_cancel_shortcut` / `unregister_cancel_shortcut` in `src-tauri/src/shortcut/mod.rs`). Die Ziffern werden nach demselben Muster registriert und wieder freigegeben, inklusive Varianten mit gehaltenem Alt und Ziffernblock. Wie genau sich das mit Handys Tastatur-Backend (`handy_keys` bzw. Tauri-Global-Shortcut) umsetzen lässt, klärt der Builder (siehe unten).
- **Stil pro Diktat:** Der gewählte Stil wird als Wert der laufenden Aufnahme mitgeführt und an die Nachbearbeitung übergeben (`post_process_transcription` in `actions.rs` nimmt heute `post_process_selected_prompt_id` aus den Einstellungen). Die gespeicherte Auswahl in den Einstellungen wird dabei nicht verändert.
- **Fenster:** Das bestehende Overlay (`src/overlay/RecordingOverlay.tsx` + `.css`, Fenstergröße in `src-tauri/src/overlay.rs`) wird um die Leiste erweitert. Das Overlay bleibt nicht fokussierbar. Die Fenstergröße wächst um die Höhe der Leiste. Die Position (unten, 40 px Abstand) bleibt.
- **Beschriftungen:** Deutsche Texte über Handys i18n-Dateien (`src/i18n/locales/de/translation.json`); die englischen Einträge ergänzen.
- **Kürzel:** `transcribe` löst den neuen Ablauf aus. `transcribe_with_post_process` bleibt im Code, wird aber in Sebastians Einstellungen nicht belegt.
- **Updates:** Update-Prüfung im Fork standardmäßig aus (`update_checks_enabled` / `update_checks_forced_disabled`). Die winget-Installation wird mit `winget pin add cjpais.Handy` festgehalten, damit `winget upgrade --all` den eigenen Build nicht ersetzt.
- **Installation:** Eigener Build (NSIS-Installer aus `bun run tauri build`) wird über die vorhandene Installation gelegt. Einstellungen in `%APPDATA%\com.pais.handy` bleiben erhalten. Vorher eine Sicherung von `settings_store.json` anlegen.

## Klärungen aus der Prüfung (meine Entscheidungen)

- **Maßstab für „nicht langsamer“:** Verglichen wird mit dem bisherigen KI-Kürzel (`Strg+Umschalt+Leertaste`) und demselben Prompt, nicht mit dem bisherigen Roh-Kürzel. Dass jedes Diktat ohne Ziffer jetzt durch Mistral läuft (≈ 1 s mehr als Roh), ist Sebastians bewusste Entscheidung („immer KI“). Der Umbau darf darüber hinaus keine Wartezeit hinzufügen. Spekulatives Vorab-Aufbereiten ist nicht nötig.
- **Ziffern bleiben bis zum Ende der Transkription registriert.** Das Original gibt `Esc` beim Aufnahme-Stopp frei; die Ziffern laufen länger und werden erst freigegeben, wenn der Text an Mistral geht (bzw. bei „Roh“ eingefügt wird) oder abgebrochen wird.
- **Alt gehalten:** Bei gehaltenem Alt darf Windows aus Ziffernblock-Eingaben keine Alt-Codes erzeugen. Die Ziffern werden vollständig abgefangen.
- **Aktivierungsmodus:** Sebastian nutzt `hold_or_toggle`. Ziffern müssen in beiden Fällen gehen (gehalten mit Alt, umgeschaltet ohne Alt).
- **Ziffer ohne passenden Prompt** (z. B. `5` bei drei Prompts): wird ignoriert, aber trotzdem abgefangen.
- **Build ohne Signaturschlüssel:** `createUpdaterArtifacts` im Fork abschalten. Installationspfad bleibt `%LOCALAPPDATA%\Handy`, damit `handy-glossar-sync.ps1` und der Anmelde-Task weiter passen.

## Dem Builder überlassen

- Wie die Ziffern technisch abgefangen werden (Low-Level-Hook vs. registrierte Kürzel), solange die Anti-Ziele eingehalten sind.
- Ob das Vulkan SDK für den Build gebraucht wird oder sich das Feature abschalten lässt.
- Details der Animation (Hervorhebung, Übergang beim Umschalten, Abblenden in der KI-Phase).
- Schnitt der Code-Änderungen und welche automatischen Tests sinnvoll sind (Handy hat Rust-Tests und Playwright).
- Einstellungs-Schalter, mit dem sich die Stil-Leiste abschalten lässt (zurück zum Original-Verhalten). Empfohlen, aber nicht Pflicht.

## Abnahme (was Sebastian prüft)

1. `Alt+Leertaste` halten → Leiste erscheint mit „1 Standard“ markiert, Pille zeigt Pegel, Zeit, ✕.
2. Während des Haltens `2` drücken → „E-Mail“ markiert, in Outlook erscheint keine „2“ und es öffnet sich nichts.
3. Loslassen, während „Transkribiere…“ `3` drücken → „WhatsApp“ markiert, Pille zeigt danach „KI bereitet auf (WhatsApp)…“.
4. `0` wählen → Text kommt ohne KI, sofort.
5. Nächstes Diktat → wieder „Standard“.
6. Außerhalb eines Diktats tippen 0–9 ganz normal.
7. Esc und ✕ brechen ab, nichts wird eingefügt.
8. Ohne Internet → Rohtext wird eingefügt.
9. Wartezeit ohne Umschalten gleich wie mit Original-Handy (Messung wie in der Phase-1-Testliste).
10. Getestet in Outlook, Browser, WhatsApp Desktop, Teams, Word.
