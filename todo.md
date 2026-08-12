# Modulus — Task List

> Erledigte Aufgaben wurden nach `archived-todo.md` verschoben (2026-08-11).
> Abschluss Review-Runde 2 (GH Actions, Bridges, FX); nur noch Deferred-Punkte offen.

## Review-Runde 3 (2026-08-11) — Neue Befunde (#18–#21)
- [ ] #18 FX-Filter koppelt linken und rechten Kanal über den Filter-State
      (`FxEngine::process`, crates/modulus-core/src/fx.rs:233-235)
- [ ] #19 Plugin-Bypass friert die Chorus-Delay-Linie ein
      (crates/modulus-core/src/fx.rs:238, Re-Enable spielt ≤100 ms alte Delay-Inhalte)
- [ ] #20 VA-/Vintage-Wellenformen überschreiten ±1 (va_saw 1.083, va_square 1.188)
- [ ] #21 Oscillator-Phasen-Wrap bricht bei freq >= sample_rate (~50 kHz bei +24 st)

## Offene (nicht erledigte) Punkte
- [ ] Runtime host verification in a DAW — deferred: no DAW on this machine (covered nightly by `--test` plugin_host)

## Offene GitHub-Issues
- #15 Runtime-Host-Verifikation in einer DAW (deferred) (offen, 2026-08-11)
- #18–#21 Review-Runde-3-Befunde (offen, 2026-08-11)

Bereits implementiert und archiviert: #9, #10, #11, #12, #13, #14, #17 sowie
G1–G8, B1–B9 und alle FX-Review-Befunde — siehe `archived-todo.md`.