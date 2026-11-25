# SHARKCODE 0.2.40 — Conservative quality analysis hotfix

SHARKCODE is a Rust/Bevy educational game with a Python execution service.

## Goal

End users must NOT install Docker Desktop, WSL2, Rust, pip or Python. A **Windows package maintainer** creates a portable distribution including the official CPython embeddable package:

1. On the developer machine: `package_windows.bat` (requires Rust/Cargo and internet once).
2. It creates `SHARKCODE_0_2_40_windows.zip` containing a single `SHARKCODE/` directory.
3. Unzip on a Windows x64 PC, then launch `SHARKCODE.exe`.

The package builder fetches CPython 3.13.17 from python.org, includes the standard library and a `SHA256_MANIFEST.json`, and does not add Python to the system PATH. The player has no installation steps beyond extracting the archive.

**Important status:** Packaging has not been run or tested on Windows by the maintainer of this patch; the official embedded CPython archive was inaccessible from this environment and is NOT included in the source ZIP. Run `package_windows.bat` to create an actual Windows distribution. `Cargo` is also unavailable here, so the Rust code has not been compiled.

## Security: NOT READY FOR PUBLIC UNTRUSTED CODE

A packaged interpreter is NOT a security sandbox. In this release the native worker uses a Job Object to limit memory/process count and enforces time and output limits. A Job Object cannot protect files, registry or network. Running Python with `-I` only isolates the *import configuration*, not host OS access. Executing hostile Python is **unsafe**.

A `cargo run` debug build supports existing gameplay for trusted development code and uses the bundled interpreter when present, otherwise an explicitly marked development fallback to system Python.

A **release** build refuses to execute submitted Python by default until real OS containment is implemented. Maintainers can temporarily opt into **unsafe execution for local testing only** using `SHARKCODE_ALLOW_UNSAFE_NATIVE=1`. **Never distribute this opt-in as an end-user instruction.** A real AppContainer/VM backend needs separate implementation and adversarial validation before the game may enable unrestricted `os`, `sys`, `subprocess`, or external mods.

The game UI can start in a release build, but interactive programming challenges cannot be completed without an isolated backend. Thus 0.2.40 is a source/packaging foundation, **not a safe public release**.

## Development

From the project root on Windows:

```powershell
cargo run
```

If no bundled Python is present, the debug build looks for Python in `PATH`. This is development convenience only. The production runtime never silently falls back to PATH.

## Portable layout

```text
SHARKCODE/
  SHARKCODE.exe
  runtime/
    inspector.py
    quality.py
    project_validator.py
    python/
      python.exe
      python313.dll
      python313.zip
      python313._pth
      ...
  content/
    nodes/
    validators/
    scrolls/
    test_lib/
  SHA256_MANIFEST.json
```

Game and developer saves are created on first run in `sharkcode_data/`. They are never packed from a maintainer's working tree.

## Region «4 4» — original SnakeCode port

- Exit from the River through the physical `river_to_for4` entry (one way).
- Original `forRun()` structure: entry, mandatory `for` tablet, four procedural encounters (20% optional advanced tablet, 80% random enemy), boss and terminal completion node.
- Original enemies: С приветом, Любитель каруселей, Человек из Кемерова, Счетовод (originally marked rare).
- Original boss «Страж бронзового змия»: all 400 products of 1..20, with repetitions.
- `for4.rs` uses independent RNG seeded from the run seed. River and Valley layout and saved indices remain unchanged.
- `Сковыль` is registered as Счетовод's item reward. Source archive did not specify its gameplay effect; none is invented.
- Content nodes are TOML files, validators Python modules; tablets are preserved from the original JavaScript templates.
- Rust/Bevy compilation and Windows visual testing not available in this environment.


## Версия 0.2.38
Добавлена Бесконечная башня и исправлены проверки и прокрутка Кодекса. Подробности — в RELEASE_NOTES.md.

## Версия 0.2.39

Новый оценщик (ранги S++, S, A, B, C), объективные PEP 8 и метрики
времени/памяти, три необязательных сундука-мимика с наградами, новые
скрижали и миграция сохранений. Детали — в `RELEASE_NOTES.md`.

## Версия 0.2.40

Переработан анализ качества: явные недостижимые ветки, бесполезные
выражения, недостижимые инструкции и постоянные условия снижают ранг.
Штрафы за произвольное число строк и подозрительные переменные отключены
без доказательства избыточности. Минимальный корректный код сохраняет S++.
Стартовый код игры защищён от новых AST-штрафов.
Сравнение по времени и памяти остаётся осторожной вспомогательной метрикой,
не является доказательством алгоритмической оптимальности.
Подробности — в `RELEASE_NOTES.md`.
