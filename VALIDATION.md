# Validation

Validated on Linux, in two formats. **The current one is 0.2.0, the flat SDK package**
(Horizon `0b14f8acfd97fa2c750e42795249dcc981178ca4`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`), measured 2026-11-18. Every command ran
through `ports/tools/build-slot.sh` with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`; evidence is in
`ports/_shared/evidence/silentgear/` (`sdk-*.log`; the retired logs are listed as historical in
`README-evidence-index.md`).

## 0.2.0 — the SDK ladder

| Check | Command | Result |
|---|---|---|
| runtime only, before any glue | `cargo test --offline` | **21 passed**, 0 failed (`sdk-test-runtime-only.log`) |
| glue red | the same, helpers on `todo!()` | **25 passed, 4 failed**, "not yet implemented" (`sdk-test-red-glue-todo.log`) |
| tests | `cargo test --offline` | **29 passed**, 0 failed — 21 `runtime::`, 8 `tests::` (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | exit 0 (`sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/silentgear.hmod` (`sdk-build.log`) |
| registry check | `cargo horizon-mod check silentgear.hmod` | exit 0, 50,986 B, SHA-256 `c1bc5a29…d1bb7` (`sdk-check.log`) |
| fits the pinned build | `cargo horizon-mod fits silentgear.hmod --build nightly` | exit 0, `0b14f8a  made for` (`sdk-fits.log`) |
| reproducible | a second `rm -f Cargo.lock && cargo horizon-mod build` | same bytes, `cmp` clean (`sdk-build-second.log`, `sdk-sha256.log`) |
| runtime untouched | `cmp` vs the retired file, and `unzip -p silentgear.hmod src/runtime.rs \| sha256sum` | `b65ff924…d9f1c` in all three (`sdk-sha256.log`) |
| **in game** | `tools/e2e.sh` on a launcher-built game | **not run** — no `horizon-launcher` here and no game at this pin; recorded as gap 0 |

## 0.1.0 — the retired `.hnmod` (historical)

Validated 2026-11-09/10 against the campaign pin `1d63e0a5` / mixins `5a64af93` (schema 6). These
results belong to the container launcher ≥ 0.2.70 can no longer load; they are kept as the record of
slice 1's behaviour, not as evidence for 0.2.0.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass (empty) |
| `cargo clippy --all-targets --locked --offline -p horizon-silentgear -- -D warnings` | Pass |
| `cargo test --offline --locked -p horizon-silentgear` | **21 passed**, 0 failed (red first: 0 passed, 22 failed) |
| `python3 tools/horizon.py stage` | `stage_changed: true`, 5 injections |
| `python3 tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** |
| `python3 tools/horizon.py test` | `test result: ok. 21 passed; 0 failed; 1240 filtered` |
| `python3 tools/horizon.py build` (fast) | `cargo_exit: 0`, 0 warnings |
| `tools/e2e.sh` (dev mod) | **PASS**, `EXIT:0` (Overworld arena at y=100) |
| `SELFTEST=1 tools/e2e.sh` | **EXIT:1** (`the pickaxe's damage is 1, expected 2`) |
| `EMBEDDED=1 tools/e2e.sh` (only the `.hnmod`, no `--dev-mod`) | **PASS**, `EXIT:0`; control leg shows the ids unknown |
| `SELFTEST=1 EMBEDDED=1 tools/e2e.sh` | **EXIT:1** |
| `check-e2e-logs.py silentgear --strict` | `logs=3 ok=3 missing-exit=0 bad-exit=0` |
| `horizon-mod check content/silentgear.hmod` | 24,569 B, `ec9806d9…`, content only |
| `horizon-mod check silentgear.hnmod --kind native-mod` | 34,940 B, `59e43dee…`, native-mod for 26.3 |
| `tools/hmod-load.sh` (previous slice) | **PASS** with `--dev-mod`; the no-`--dev-mod` control registers none |

The 0.1.0 e2e was fail-capable: the `SELFTEST` legs fail an assertion in the same scenario, the
stick phase is a same-run differential (no break, no drop), and the unassembled-blueprint and
wrong-part phases are refusals. `tools/e2e.sh` keeps the same scenario and the same `SELFTEST=1`
leg for 0.2.0; only the launch (a launcher-built `GAME=`) and the control (`CONTROL_GAME=` instead
of the `.horizon-cache` trick) changed.

This is a slice-1 verification: the other tools, materials as functional gear, traits, grading and
the custom recipe types are not implemented (see `notes.md`'s gap list). The per-stack tool-stat
limitation of the pin is documented in `notes.md` and `FINDINGS.md`; the slice builds only the
combination the item components can honour.
