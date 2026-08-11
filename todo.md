# Modulus — Task List

> Erledigte Aufgaben wurden nach `archived-todo.md` verschoben (2026-08-11).
> Schritte 4 + 5 des Review-Auftrags.

## Review-Befunde 2026-08-11 (Bericht: `reports/review-2026-08-11.md`)
- [ ] Review-Workflow von Koalitions-O-Mat auf Modulus umstellen
      (`.github/workflows/opencode-review.yml:44-71`, `opencode-todo-issues.yml:33-55`;
      Ziel-Dateien config.json/script.js/elections/ existieren hier nicht)
- [ ] Filter-Envelope bipolar: `filter_env_amount` Range `-1.0..=1.0`
      (`crates/modulus-synth/src/params.rs:179`, `crates/modulus-core/src/voice.rs:150`)
- [ ] Synth-Setups: Default-Filter-Smoothing setzen (~5–30 ms), sonst Zipper-Sweeps
      (`crates/modulus-core/src/synth_setup.rs:492-513`,
      `crates/modulus-core/src/modules/native/fx/filter.rs:56-63`)
- [ ] Editor-Stimmenanzeige echtzeit repainten; `editor_state.is_open()` aus der Sample-Schleife holen
      (`crates/modulus-synth/src/editor.rs:72-80`, `crates/modulus-synth/src/lib.rs:158`)
- [ ] Chorus „Voices 0" = impliziter Bypass klarstellen oder Range `1..=8`
      (`crates/modulus-core/src/fx.rs:91`, `crates/modulus-fx/src/params.rs:125`)
- [ ] UI-Helfer (`ACCENT/BG/PANEL`, `slider_row()`, `section()`) zwischen beiden Editoren teilen
      (`crates/modulus-synth/src/editor.rs:16-37`, `crates/modulus-fx/src/editor.rs:10-31`)

## Offene (nicht erledigte) Punkte aus der Konsolidierung
- [ ] Runtime host verification in a DAW — deferred: no DAW on this machine (covered nightly by `--test` plugin_host)

## Tracking offene GitHub-Issues
- (keine offenen Issues — #4 „Todo.md" wurde als erledigt geschlossen, 2026-08-11)