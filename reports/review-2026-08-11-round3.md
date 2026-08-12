# Modulus Code-Review — Runde 3 (2026-08-11)

Fokus: gesamtes Projekt (DSP, Modul-/Setup-Engine, CI). Verifiziert per
Build, `cargo test` (33 Tests), `clippy -D warnings`, `fmt --check` sowie
temporären Test-Harnesses (nach Verifikation gelöscht).

## Status der Vorgänger-Runden
- Runde 1: `reports/review-2026-08-11.md` (FX-Befunde)
- Runde 2: `reports/review-2026-08-11-gh-actions.md` (G1–G8, B1–B9)
- Issues #9–#14, #17 implementiert → geschlossen (Schritt 6)

## Neue Befunde

### F1 (mittel): FX-Filter koppelt linken und rechten Kanal über den Filter-State
`FxEngine::process` (crates/modulus-core/src/fx.rs:225-236) reicht beide
Kanäle des Stereo-Frames sequenziell durch **ein** `VariableFilter`-Objekt
(später Kanal x : `*sample = self.filter.process(*sample, sr)`). Dadurch
„erbt“ der rechte Kanal den Zustand des linken: filtert man links ein Signal,
ringt rechts prozessiertes Linkes mit, selbst bei stummem Rechtem.
Harness: Links 1 kHz-Sinus (0,5), Rechts DC 0.0 → Rechts-Peak 0.135
(muss ~0 sein). Mixing erwünscht; für Stereo-FX sollte pro Kanal ein
Filterzustand geführt oder der Filter auf die Summe angewendet werden.

### F2 (mittel): Plugin-Bypass friert die Chorus-Delay-Linie ein
In `FxEngine::process` (crates/modulus-core/src/fx.rs:238) wird der Chorus
bei `params.chorus_enabled == false` gar nicht erst aufgerufen
(`if params.chorus_enabled { self.chorus.process(...) }`). Wird der Chorus
während laufenden Audio-Streams deaktiviert, bleibt die Delay-Linie stehen;
beim Wiedereinschalten spielt er bis zu `MAX_DELAY_MS` (= 100 ms) veraltetes
Audio ab. Der in Runde 2 (#13) eingebaute „Delay-Linie weiter schreiben bei
voices==0/dry_wet==0“-Pfad greift nur *innerhalb* von `Chorus::process`, nicht
auf Ebene des Plugin-Bypasses. Harness: Peak 0.5 auf Re-Enable bestätigt.
Fix-Option: Chorus immer mit `ChorusParams { dry_wet: 0.0, … }` (Bypass)
zum „Laufen halten“ aufrufen, statt den Aufruf zu überspringen.

### F3 (niedrig): Bandlimitierte Oscillator-Wellenformen überschreiten ±1
`tests/waveform_peak_ranges` bestätigt:
- `va_saw`: Peak +1.083 / −0.987
- `va_square`: Peak +1.188 / −1.188
- `vintage_saw`: Peak +1.000 / −1.014

Die drei VA-/Vintage-Wellenformen überschreiten die Einheitsnorm leicht
(overshoot der Band-limitierenden Partialsumme). Bei hohen Filter-Leveln /
Chorus-/Gain-Stufen kann das früher clippen als die anderen Formen.

### F4 (niedrig): Phasen-Wrap des Oscillators bricht bei f > sample_rate
`Oscillator::generate` wrapt die Phase nur einmal:
`if phase >= TWO_PI { phase -= TWO_PI }` (crates/modulus-core/src/oscillator.rs).
Bei `freq >= sample_rate` ist `phase_increment >= 2π` und eine einzelne
Subtraktion reicht nicht mehr — die Phase driftet pro Sample weiter nach
oben. Praktisch erreichbar: Note 127 + `pitch_multiplier`/FM bis
24 Halbtöne → ~50 kHz bei 44,1 kHz. Harness: `square` togglet in 100 k
Samples nur 1× (degeneriert). Robust wäre `phase = phase.rem_euclid(TWO_PI)`
bzw. ein `% TWO_PI`-Wrap auch im `generate()`.

## Verifizierte Nicht-Befunde (OK)
- LFO-CV zentriert um 1; Envelope-Release exponentiell; VA Square harmonics 3
- ARP-4075-Rückkopplung mit Stage-Clamp ±1, Totbereich bei res=1.0 behoben
- `filt_smoothing`-Default 15 ms im Filter-Modul (…#11)
- Chorus `width`-Mapping (0 mono … 1 anti-phasig) korrekt

## Status
- Build, 33 Tests, clippy, fmt: **grün** (auch auf dieser Maschine)
- Issues #10–#14, #17 implementiert → geschlossen; #15 bleibt offen (deferred)
- Branch `opencode/dispatch-999690-20260811124200` (gemergt) bereinigt
- Berichts-Stub der Runde 3 wurde zur finalen Fassung gefüllt