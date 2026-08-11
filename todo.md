# Modulus — Task List

> Erledigte Aufgaben wurden nach `archived-todo.md` verschoben (2026-08-11).
> Schritte 4 + 5 des Review-Auftrags.

## Review-Befunde FX 2026-08-11 (Bericht: `reports/review-2026-08-11.md`)
- [ ] ARP-4075-Resonanz hat Totbereich: `res=1.0` ⇒ Ausgang dauerhaft 0.0 (Stille).
      Formel friert die Stufe ein (`output - stage + stage - output == 0`)
      (crates/modulus-core/src/filter.rs:189-200 `process_arp`)
- [ ] Roland und LE13700-Filter sind bitidentisch (beide scale 1.0) —
      wirklich vier Modelle oder LE13700 nicht duplizieren/entfernen
      (crates/modulus-core/src/filter.rs:151-155)
- [ ] Chorus-`width` invertiert: width 0 → 180° (antiphase), width 1 → 360° (mono).
      Startphase L=0, R=PI + PI*width ⇒ Breite/Doku korrigieren
      (crates/modulus-core/src/fx.rs:100,117,134)
- [ ] `filt_smoothing` („ms") wirkt um Faktor 2π zu schnell: 50 ms ⇒ τ≈7.96 ms;
      `2π` aus der Koeffizienten-Formel entfernen oder als Hz umlabeln
      (crates/modulus-fx/src/lib.rs:80-85; modules/native/fx/filter.rs:56-63)
- [ ] FX: Filter-Cutoff/Resonance/Type werden nur pro Block statt pro Sample
      gelesen → Block-Automation-Aliasing; `smoothed.next()` nutzen
      (crates/modulus-fx/src/lib.rs:79-111)
- [ ] Chorus-Bypass bei `dry_wet==0` friert die Delay-Line ein → stale artefacts
      beim Wieder-Einblenden; bei dry_wet=0 weiterhin in die Leitung schreiben
      (crates/modulus-core/src/fx.rs:91)

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