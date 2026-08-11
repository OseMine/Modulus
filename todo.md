# Modulus — Task List

## 2026-08-11 — Code-Review Bridge Engines (reports/review-2026-08-11.md)

Neue Befunde aus dem automatiserten Review (Fokus: Bridge Engines /
`am_bridge`/`fm_bridge`/`bridge_lead.json` + Modul-/Setup-Engine). Details
inkl. Verifikation im Review. Abgehakte Todos aus älteren Phasen sind in
`archived-todo.md`.

### Bugs (Modul-/Setup-Pfad)
- [ ] B1: `filter`-Modul ignoriert `filter_type` — `VariableFilter::set_type()`
      wird nie aufgerufen (crates/modulus-core/src/modules/native/fx/filter.rs,
      `set_param`/`process`). Alle Setups (bridge_lead, juno_106, dx7) rendern
      als Moog statt Roland (filter_type 1). Verifiziert: type 0–3 bitident.
- [ ] B2: `analog_saw` überschreitet ±1 (Range [−3.0, 1.0], Mean −0.84) —
      `shaped * 2.0 - 1.0` (crates/modulus-core/src/waveform.rs:89).
- [ ] B3: `pregain_db` wirkt als zweiter Master-Level statt als Filter-Drive —
      wird in `SynthGraph::process_frame` erst nach Filter+Env multipliziert
      (crates/modulus-core/src/synth_setup.rs:523–525), widerspricht
      docs/SETUPS.md.
- [ ] B4: Release-Tail beim Setup-/Modul-Pfad praktisch unhörbar — Sources
      (oscillator/am_bridge/fm_bridge) gaten bei `note_off` sofort auf 0;
      Envelope-Release hat nichts zu formen (modules/native/soundgen/*,
      synth_setup.rs:477). Verifiziert: nach note_off nur ~15 nonzero Samples.

### Fehlende Features
- [ ] B5: Velocity wird in der Modul-/Setup-Engine ignoriert (alle
      `AudioModule::note_on` verwerfen velocity; synth_setup.rs:467). Verifiziert:
      velocity 0.1 und 1.0 identischer Peak. Plugin-Voice-Pfad (voice.rs:157)
      korrekt.

### Verbesserungsvorschläge
- [ ] B6: `FilterType::Le13700` ist bitidentisch zu `Roland` (beide scale 1.0,
      crates/modulus-core/src/filter.rs:153–154) — differenzieren oder entfernen.
- [ ] B7: `oscillator`-Modul clammt `level`/`pitch_semitones` nicht
      (modules/native/soundgen/oscillator.rs:97–98) — Pegel-Konvention.
- [ ] B8: LFO→Filter-Modulation nur abwärts möglich (mod_cv ∈ [1−depth,1];
      SETUPS.md „around the base" irreführend) —
      crates/modulus-core/src/modules/native/modulator/lfo.rs:115–117.
- [ ] B9: `ModuleEvents.note_on`/`note_off` tote Objekte — nirgends konsumiert
      (crates/modulus-core/src/modules/mod.rs:71–81).

### Tracking offene GitHub-Issues
- [ ] Issue #4 „Todo.md — Implement the Test of todo.md's changes and check
      them off in there" (2026-08-05) — Abhaken wird in dieser Runde
      durchgeführt; Issue nach Abschluss schließen.

### Repository-Aufräumen
- [ ] Branch `origin/new` löschen (restloses Duplikat; PR #5 CLOSED ohne Merge,
      74 Dateien / −6034 Zeilen Differenz zu main, Inhalt via e46ed85 in main).
      Nicht gelöscht, da kein Push in dieser Runde erlaubt.

## Offen (verbleibend)
- [ ] Runtime host verification in a DAW — deferred: no DAW on this machine (covered nightly by `--test` plugin_host)