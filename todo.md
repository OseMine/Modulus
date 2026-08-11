# Modulus — Task List

> Erledigte Aufgaben wurden nach `archived-todo.md` verschoben (2026-08-11).
> Schritte 4 + 5 des Review-Auftrags.

## Review 2 / 2026-08-11 — GitHub Actions (Bericht: `reports/review-2026-08-11-gh-actions.md`)

### GH Actions: Bugs
- [ ] G1: Linux-Clippy bricht — `x11-xcb`/`x11` fehlt in `.github/actions/setup/action.yml:18-23`
      (kein Install in `lint`/`checks`; Fix 1d163c7 packte es nur in `checks` → Lint-Workflow rot,
      verifiziert Run 31492812946 „x11-xcb not found", exit 101)
- [ ] G2: `/oc`-Handler `opencode.yml:83-88` schlug fehl („Failed to parse JSON", leerer
      `prompt`/`use_github_token`) — **teilweise behoben**: Parallel-Lauf pushte
      `use_github_token: true` (d548e8e/d372f54); `prompt:`-Input fehlt weiterhin
- [ ] G3: `opencode-review.yml` ohne `concurrency`-Guard ⇒ parallele Dispatches (12:26–12:27
      UTC, 2026-08-11) kollidierten auf todo.md/reports → PR #7 blieb CONFLICTING
- [ ] G4: Windows-Installer wird in `build.yml` gebaut, aber nie ins Release gepackt
      (`release.yml:55-60` lädt nur `modulus-bundles-*`, nie `modulus-installer-*`)
- [ ] G5: Release-Notes-Fallback am 1. Tag leer — `release.yml:53` + `:88`
      (`git log … HEAD..HEAD` bei fehlendem PREV_TAG)
- [ ] G6: `opencode.yml:43` `needs: rust-check` ohne `if: always()` ⇒ `/oc` wird genau bei
      rotem Build übersprungen (verifiziert Run 31492376482, opencode-Job 0s)
- [ ] G7: Versionsdrift `actions/checkout@v6` (opencode-*) vs `@v4` (alle anderen) +
      Node-20-Deprecation in jedem Run
- [ ] G8: `build.yml` baut + bugdet bei jedem PR (auch Doku-/Review-PRs) ohne `paths-ignore`

### Übernommen aus PR #7 (Bridge-Engines-Review, bewahrt vor Branch-Löschung)
- [ ] B1: `filter`-Modul ignoriert `filter_type` — `VariableFilter::set_type()` wird nie
      aufgerufen (crates/modulus-core/src/modules/native/fx/filter.rs:93-102); alle Setups
      rendern als Moog (verifiziert, type 0–3 bitident)
- [ ] B2: `analog_saw` überschreitet ±1 (Range [−3.0, 1.0], Mean −0.84) —
      `shaped * 2.0 - 1.0` (crates/modulus-core/src/waveform.rs:89)
- [ ] B3: `pregain_db` wirkt als zweiter Master-Level statt Filter-Drive — erst nach Filter+Env
      multipliziert (crates/modulus-core/src/synth_setup.rs:523-525), widerspricht docs/SETUPS.md
- [ ] B4: Release-Tail beim Setup-/Modul-Pfad praktisch unhörbar — Sources gaten bei `note_off`
      sofort auf 0 (modules/native/soundgen/*, synth_setup.rs:477); nur ~15 nonzero Samples
- [ ] B5: Velocity wird in der Modul-/Setup-Engine ignoriert (synth_setup.rs:467; verifiziert:
      velocity 0.1 und 1.0 identischer Peak) — Plugin-Voice-Pfad (voice.rs:157) korrekt
- [ ] B6: `FilterType::Le13700` bitidentisch zu `Roland` (beide scale 1.0,
      crates/modulus-core/src/filter.rs:151-155)
- [ ] B7: `oscillator`-Modul clammt `level`/`pitch_semitones` nicht
      (modules/native/soundgen/oscillator.rs:90-102)
- [ ] B8: LFO→Filter-Modulation nur abwärts (mod_cv ∈ [1−depth,1]; SETUPS.md „around the base"
      irreführend) — modules/native/modulator/lfo.rs:115-117
- [ ] B9: `ModuleEvents.note_on/note_off` tote Objekte, nirgends konsumiert
      (crates/modulus-core/src/modules/mod.rs:71-81)

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
- #9 Review-Workflow von Koalitions-O-Mat auf Modulus umstellen (offen, 2026-08-11)
- #10 Filter-Envelope bipolar machen (Range -1.0..=1.0) (offen, 2026-08-11)
- #11 Default-Filter-Smoothing für Synth-Setups setzen (5–30 ms) (offen, 2026-08-11)
- #12 Editor-Stimmenanzeige in Echtzeit neu zeichnen (offen, 2026-08-11)
- #13 Chorus „Voices 0" als impliziten Bypass klären (offen, 2026-08-11)
- #14 UI-Helfer zwischen Synth- und FX-Editor teilen (offen, 2026-08-11)
- #15 Runtime-Host-Verifikation in einer DAW (deferred) (offen, 2026-08-11)