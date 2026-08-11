# Code-Review Modulus — GitHub Actions — 2026-08-11 (2. Review-Runde)

- **Repo:** `OseMine/Modulus` (Rust-Workspace, Synthesizer/FX-Plugins, nih-plug + egui)
- **Fokusbereich laut Dispatch:** `Gh Actions`
- **Untersucht:** alle 5 Workflows (`.github/workflows/`) + alle 5 Composite-Actions
  (`.github/actions/`), CI-Verhalten anhand realer Ausführungen, Issues/PRs/Branches
- **Methodik:** Lesen aller Workflow-/Action-Dateien + Verifikation gegen reale
  GitHub-Runs (Logs via `gh run view`), Shell-Logik simuliert (Release-Packaging),
  Action-Versionen gegen `api.github.com` geprüft.

---

## 0. WICHTIGER SYSTEMFUND (fortbestehend): Koalitions-O-Mat-Prompt im Review-Workflow

- **Datei:** `.github/workflows/opencode-review.yml:44–72`

Das automatisierte Review-Prompt referenziert weiterhin die **„Koalitions-O-Mat"**-
Web-App (`config.json`, `script.js`, `index.html`, `einfache-sprache.json`,
`elections.json`, `elections/`). Diese Dateien **existieren in diesem Repo nicht**
(Rust-Workspace). Jeder Cron-/Dispatch-Lauf (montags 06:00 UTC) startet damit mit
einem auf nicht vorhandene Dateien zeigenden Prompt. Der Hinweis aus dem UI-Review
(2026-08-11, Punkt 0) wurde **nicht** in den Workflow eingearbeitet.

**Zusätzlich geprüft:** `.github/workflows/opencode-todo-issues.yml:33–55` enthält
**keinen** Koalitions-Prompt mehr (nur generisches „Erzeuge GitHub-Issues aus
todo.md") — der dortige Verweis aus dem früheren Befund ist damit **veraltet**.

**Empfehlung:** Prompt in `opencode-review.yml` auf Modulus umschreiben (Fokus
„GitHub Actions" ⇒ `.github/workflows/*.yml`, `.github/actions/*`); oder
Koalitions-Reste endgültig entfernen.

---

## 1. Befunde

### G1 (HOCH) — Linux-Clippy bricht: `x11-xcb` wird von `lint`/`checks` nie installiert

- **Dateien:** `.github/actions/setup/action.yml:18–23` (installiert nur
  `libgl1-mesa-dev … libwayland-dev`, **ohne** `libx11-xcb-dev`/`libx11-dev`),
  `.github/actions/lint/action.yml` (kein Dep-Install), `.github/actions/checks/action.yml`
- **Verifikation (real):** Run `31492812946` (Lint, Push „action.yml aktualisieren",
  12:46 UTC) schlägt fehl: `Package 'x11-xcb' … not found` → `exit 101`.
  Ebenso Run `31492376482` (rust-check, 12:40 UTC): `x11-xcb required by 'virtual:world' not found`.
  Commit `1d163c7` hat die Dependencies **nur** in die `checks`-Action gepackt,
  **nicht** in die `setup`- oder `lint`-Action → der `Lint`-Workflow ist weiterhin rot.
- **Ursache:** `lint.yml` nutzt `./.github/actions/lint` + `./.github/actions/setup`;
  `setup` installiert die GUI-Libs nicht vollständig (egui/x11-Crates von
  `modulus-synth`) und `lint` installiert gar nichts.
- **Empfehlung:** `libx11-xcb-dev libx11-dev` in die geteilte `setup`-Action
  aufnehmen (ein Ort für alle Jobs); aus `checks` herauslösen.

### G2 (HOCH) — `/oc`-Handler (`opencode.yml`) schlägt immer fehl: „Failed to parse JSON"

- **Datei:** `.github/workflows/opencode.yml:83–88` (`anomalyco/opencode/github@latest`,
  nur `model` als Input)
- **Verifikation (real):** Runs `31492867553` und `31492376482` — Schritt *Run
  opencode* endet mit `Failed to parse JSON` / `Unexpected error` (exit 1). Der
  Action führt `opencode github run` mit **leerem `PROMPT`**-Input aus
  (`action.yml` des Providers: `prompt` optional, Standard), d. h. der synthetische
  Default-/Mentions-Prompt kommt nicht an. Der Issue-Trigger (aus Issue #2/#4-Zeit)
  funktionierte; nach dem Workflow-Rebuild ist `/oc` funktionslos.
- **Empfehlung:** `with: prompt:` (mindestens Platzhalter) oder `use_github_token:
  true` ergänzen und gegen neueste Action-Changelog prüfen; bis dahin
  `agent: reviewer` bzw. einen expliziten Prompt setzen.

### G3 (HOCH) — Kein Concurrency-Guard ⇒ parallele Review-Dispatches kollidieren

- **Datei:** `.github/workflows/opencode-review.yml` (kein `concurrency:`-Block;
  `build/lint/release/opencode` haben einen)
- **Verifikation (real):** Am 2026-08-11 liefen **vier parallele Dispätches**
  (12:26:40, 12:26:58, 12:27:11, 12:27:26 UTC) nahezu gleichzeitig; Ergebnis:
  PR #6 (UI-Review, gemerged) und PR #7 (Bridge-Review) kollidierten auf
  `reports/review-2026-08-11.md`, `todo.md`, `archived-todo.md` → PR #7 ist
  dauerhaft **CONFLICTING**. Ein `cancel-in-progress`-Guard (wie in `build.yml`)
  hätte die zweite Runde abgebrochen.
- **Empfehlung:** `concurrency: { group: opencode-review-${GITHUB_REF},
  cancel-in-progress: true }` (bzw. ein globaler `opencode-review`-Gruppenkey).

### G4 (MITTEL) — Windows-Installer wird gebaut, aber nie veröffentlicht

- **Dateien:** `.github/actions/installer/action.yml` (baut `.exe`, lädt Artifact
  `modulus-installer-Windows`), `.github/workflows/build.yml:38–40` (nur hier
  aufgerufen), `.github/workflows/release.yml` (lädt **nur** `modulus-bundles-*`,
  kein `modulus-installer-*`)
- **Verifikation:** Im Release-Workflow („Download bundles", Z. 56–60) fehlt der
  Installer-Pattern; Releases enthalten damit nur VST3/CLAP-Bundles, nie den
  Inno-Setup-Installer. Installer-Produktion ist auf CI (build.yml) beschränkt.
- **Empfehlung:** In `release.yml` den `installer`-Job (Windows) ergänzen und
  `*.exe` an die Release-Assets anhängen, oder Installer bewusst als
  „CI-only"-Feature dokumentieren.

### G5 (MITTEL) — Anfang 1. Release: Release-Notes-Fallback erzeugt leere Notes-Datei

- **Datei:** `.github/workflows/release.yml:53` (`git describe --tags --abbrev=0
  ${GITHUB_REF_NAME}~1 … || echo ''`) und `:88` (`git log --oneline
  "${PREV_TAG:-HEAD}..HEAD"`)
- **Verifikation (Logik):** Ohne Vorgänger-Tag ist `PREV_TAG=''` ⇒ Fallback wird
  `git log --oneline HEAD..HEAD` = **leer** ⇒ `RELEASE_NOTES.md` leer ⇒
  `gh release create` mit einzeiler-Notes. Beim allerersten Tag fehlen die Notizen.
- **Empfehlung:** `[ -s RELEASE_NOTES.md ] || git log --oneline --first-parent HEAD`
  als Fallback; oder `PREV_TAG` auf `HEAD` setzen statt `HEAD..HEAD`.

### G6 (NIEDRIG) — `opencode.yml`: rust-check-Gate blockiert `/oc` bei fehlgeschlagenem Build

- **Datei:** `.github/workflows/opencode.yml:43` (`needs: rust-check`), ohne
  `if: always()`.
- **Verifikation (real):** Run `31492376482`: `rust-check` exit 101 ⇒ `opencode`-Job
  **übersprungen (0s)**. Genau das Gegen-Szenario (fehlgeschlagener Build, den
  `/oc` fixen soll) wird durch das Gate blockiert.
- **Empfehlung:** `if: always()` am `opencode`-Job (Prüflog bleibt sichtbar) oder
  Gate weglassen, da der Agent ohnehin nur mit write-Permission startet.

### G7 (NIEDRIG) — Versionsdrift & Node-20-Deprecation

- **Dateien:** `opencode-review.yml`/`opencode-todo-issues.yml` nutzen
  `actions/checkout@v6`, alle übrigen `@v4`; `actions/github-script@v7` vs.
  `@v4`-Checkout-Kombis. Die GH-Runner warnen in jedem Run „Node.js 20 is being
  deprecated" (checkout@v4, cache@v4, github-script@v7).
- **Empfehlung:** einheitlich auf `checkout@v5` (Node 24) bzw. aktuelle Version
  anheben; Versionen zentral/pinned.

### G8 (NIEDRIG) — `build.yml` bugdet auf **jedem** PR/durchlauf komplett neu

- **Datei:** `.github/workflows/build.yml:3–7` (`push: branches [main]` +
  `pull_request:`), Bundle-Job ohne `paths`-Filter.
- **Beobachtung:** Review-/Doku-PRs (nur `.md`/`reports/`/`todo.md`) triggern den
  vollen 3-OS-Build inkl. Release-Bundle und Windows-Installer (choco) — unnötige
  Minuten/Artifacts. `paths-ignore` für `reports/**`, `todo.md`, `docs/**` spart
  Zeit; Artifact-Upload durch einen `if: github.ref == refs/heads/main`-Guard.
- **Rest:** `complete`-Job nach `needs: rust` ist rein dekorativ (kein Bug).

### G9 (INFO) — Release-Packaging-Logik ist korrekt (kein Verdacht)

- **Prüfung:** `release.yml` „Package bundles": Artifacts heißen
  `modulus-bundles-<runner.os>` (`Linux`/`Windows`/`macOS`); `${dir##*-}` ergibt
  gerade diese Werte, `case`-Zweige matchen (Simulation ausgeführt, siehe unten).
  Kein Dateiname/Suffix-Bug. `zip`/`tar` in der Shell verfügbar.

---

## 2. Verifikation (ausgeführt)

| Prüfung | Ergebnis |
| --- | --- |
| `.github/actions/setup/action.yml` GUI-Deps vs. x11-Crates | ❌ `libx11-xcb-dev`/`libx11-dev` fehlen |
| Lint-Run 31492812946 (12:46) | ❌ exit 101 „x11-xcb … not found" |
| rust-check-Run 31492376482 (12:40) | ❌ exit 101 x11-xcb; `opencode` übersprungen wegen `needs:` |
| opencode-Run 31492867553 / 31492376482 Schritt „Run opencode" | ❌ „Failed to parse JSON" (leerer Prompt) |
| Parallele Dispatch-Kollision (PR #6 merged, PR #7 CONFLICTING) | ✅ bestätigt — kein `concurrency`-Guard in `opencode-review.yml` |
| Release „Package bundles" Case-Matching (Simulation bash) | ✅ Linux/Windows/macOS matchen; `.exe`/Installer fehlt im Release |
| `actions/checkout@v6` existiert? | ✅ v6.1.0 vorhanden (`api.github.com`) |
| `opencode-todo-issues.yml` enthält noch Koalitions-Prompt? | ❌ nicht mehr — Befund 0 teilweise veraltet |

---

## 3. Befunds-Herkunft für diese Runde (GitHub-Hygiene §)

- **PR #7** (Branch `opencode/dispatch-6c46cd-20260811122745`, „Review: B1–B9 gefunden,
  Report+todo.md aktualisiert.") war bei Review-Start **open** und **CONFLICTING**.
  Sein Inhalt (Bridge-Engines-Review B1–B9) lag nur in diesem Branch vor. Ein
  paralleler Lauf hat den PR inzwischen **geschlossen**; die B1–B9-Findings wurden
  in die `todo.md` dieser Runde übernommen (bewahrt), Branch
  `opencode/dispatch-6c46cd-20260811122745` ist damit Löschkandidat.
- **PR #8** (Fokus FX, Branch `opencode/dispatch-1e4fb9-20260811122714`,
  „Review fertig: FX-Befunde verifiziert") — geprüft, **MERGEABLE**, inhaltlich
  fundiert (A1–A7 verifiziert, konsolidierter Bericht). **In dieser Runde gemerged**
  (Merge-Commit `c9ca9b6`, 2026-08-11).
- **PR #6** (Fokus UI) — bereits **MERGED** (`afaf609`), nicht angefasst.
- **Issues:** #2/#4 geschlossen. Paralleler Lauf (`opencode-todo-issues`, Run
  31493448470) hat am 2026-08-11 **Issues #9–15** aus den offenen todo.md-Punkten
  erzeugt (keine Duplikate). Diese werden im Todo-Tracking geführt.
- **Branches:** `main` + `opencode/dispatch-999690-…` (dieser Lauf). Branch
  `opencode/dispatch-6c46cd-20260811122745` (geschlossener PR #7) und Branch
  `opencode/dispatch-1e4fb9-20260811122714` (gemergter PR #8) wurden **gelöscht**
  (2026-08-11); Inhalt von PR #7 via todo.md bewahrt.
- **Kein Anwendungscode verändert** — nur Berichte + todo.md. Die in dieser Runde
  beobachteten `opencode.yml`-Änderungen (use_github_token, Fix G2) und
  `checks/action.yml`-Dep-Fix stammen von parallelen Läufen (Commits d372f54,
  d548e8e, 1d163c7), nicht von dieser Runde.

## 4. Empfohlene nächste Schritte (für Issues)

1. Fix G1: `libx11-xcb-dev libx11-dev` in `.github/actions/setup`.
2. Fix G2: `prompt:`-Input im `opencode`-Workflow ergänzen (use_github_token ist
   bereits durch Parallel-Lauf ergänzt).
3. Fix G3: `concurrency`-Guard in `opencode-review.yml`.
4. Fix G4: Installeur im `release.yml` veröffentlichen.
5. Fix G5: Notes-Fallback für Erstveröffentlichung.
6. Fix G6–G8: `if: always()`, Versionen/Deps, `paths-ignore`.

---

*Review via OpenCode (deepseek-v4-flash-free), 2026-08-11, Fokus GitHub Actions.*