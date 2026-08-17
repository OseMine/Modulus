# Modulus — Task List

## Review-Runde 6 (2026-08-17) — neue Befunde (Gesamtprojekt-Review)

- [ ] N1 (mittel): `Waveform::Square` (waveform.rs:53-59) ist nicht
      phasen-normalisiert (`if phase < PI` auf rohem `phase`, keine
      `rem_euclid`) — gleiche Klasse wie U3. Bei `Oscillator::generate_at`
      (FM-Bridge, `phase + offset`) liefert Square bei Phase ≥ 2π dauerhaft
      −1 (Harness: `generate(7.0)` → −1, korrekt wäre +1, da 7.0−2π = 0.72).
      Fix: `phase.rem_euclid(TWO_PI)` vor dem Vergleich; Regressions-Test
      analog zu `phase_stays_bounded_in_generate_at` für Square.
- [ ] N2 (niedrig): `Voice::trigger` (voice.rs:111-119) resettet den Filter
      nicht (`filter.reset()` fehlt) — bei Retrigger/Steal bleiben die
      Filter-Zustände (y1..y4, oldx/oldy*) des vorherigen Anschlags erhalten
      → potenzielle Klicks. `Voice::reset` resettet den Filter korrekt.
- [ ] N3 (niedrig): `osc2_level`/`modulator_level` sind im AM-Modus tot —
      `voice.rs:144` und `am_bridge.rs:195` verwenden in AM-Mode weder
      `osc2_level` noch `modulator_level` (Harness: Level 0.0 vs. 1.0 →
      bitidentischer Peak). Der „OSC 2 Level“-Slider (params.rs `osc2_level`)
      hat im AM-Modus keinen Effekt; docs/MODULES.md listet `modulator_level`
      als am_bridge-Param ohne Hinweis.
- [ ] N4 (niedrig, Repo): kein LICENSE-File — Cargo.toml deklariert ISC,
      README.md:102 verweist auf „repository license files“, das File fehlt.
- [ ] N5 (niedrig, CI): `paths-ignore` unvollständig/inkonsistent mit G8 —
      build.yml ignoriert nur `docs/**`/`reports/**`/`*.md` (Änderungen an
      `.github/**`/`setups/**` triggern volle 3-OS-CI), lint.yml ignoriert
      `reports/**` nicht.

> Runde-5-Befunde U1–U6 weiterhin offen (U1/U2/U3 in dieser Runde erneut
> verifiziert: U1 Re-Enable-Peak 0.959; U2 AM-Peak 1.9997; U3 84 % flach bei −1).

## Review-Runde 5 (2026-08-12) — neue Befunde (UI-/DSP-/Doku-/Repo-Review)

- [ ] U1 (hoch): Synth-Chorus-Bypass friert die Delay-Linie ein —
      `SynthEngine::process` (crates/modulus-core/src/synth.rs:46-49)
      überspringt den Chorus bei `chorus_enabled == false`. Identischer
      Defekt wie #19, aber nur im FX-Pfad (`FxEngine`) behoben. Harness
      (gelöscht): Re-Enable-Peak 1.000 bei Skip vs. 0.000 bei
      `dry_wet = 0`-Keep-Alive. Fix wie FxEngine (Chorus immer mit
      `dry_wet = 0` aufrufen); Regressions-Test über `SynthEngine` analog
      zu `chorus_bypass_keeps_delay_line_running`.
- [ ] U2 (mittel): AM-Modus übersteigt ±1 — `voice.rs:144` und
      `am_bridge.rs:195` (`carrier * (1 + modulator * depth) * level`)
      erreichen Peak 1.53 bei depth=1/level=1 (untergräbt die
      Einheitsnorm-Garantie aus #20).
- [ ] U3 (niedrig): `vintage_saw` (waveform.rs:153-159) ist nicht
      phasen-normalisiert (rohes `phase`, keine `rem_euclid`), 84 % der
      Periode flach bei −1; bei FM/`generate_at` mit Offset falsch.
- [ ] U4 (niedrig, Doku): `docs/LUA.md:72-74` konstruiert `ModuleEvents`
      mit entfernten Feldern `note_on`/`note_off` (seit B9) — kompiliert
      nicht.
- [ ] U5 (niedrig, Doku): `docs/PARAMETERS.md` zählt 30/15 Parameter,
      real sind es 29/14 (Code + ARCHITECTURE.md sind korrekt).
- [ ] U6 (Repo): ungemergter Feature-Branch `origin/ui-setup-selector`
      (Setup-Selector im Synth-Editor, `crates/modulus-synth/src/setups.rs`,
      kein PR, veraltet ggü. main) — Feature als PR auf aktuellen `main`
      heben oder Branch löschen.

> Erledigte Aufgaben wurden nach `archived-todo.md` verschoben.
> Stand 2026-08-12: Review-Runde-3-Befunde #18–#21 behoben; #23 (Issue-Templates) erledigt; nur noch #15 (deferred) offen.

## Offene (nicht erledigte) Punkte
- [ ] Runtime host verification in a DAW — deferred: no DAW on this machine (covered nightly by `--test` plugin_host)

## Offene GitHub-Issues
- #15 Runtime-Host-Verifikation in einer DAW (deferred, offen) — erfordert eine echte DAW auf dem Testsystem

Bereits implementiert und archiviert: #9–#14, #17–#21, #23, G1–G8, B1–B9 sowie
alle FX-Review-Befunde — siehe `archived-todo.md`.
