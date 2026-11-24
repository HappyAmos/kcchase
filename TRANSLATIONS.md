# Translating kcchase

`kcchase` maintains user-facing strings in single-file YAML locale files under [`locales/`](locales/).

## Canonical Base Locale

The canonical source of truth for all user-facing strings is:

- [`locales/en_us.yml`](locales/en_us.yml)

All other locale files must mirror the exact key structure of `locales/en_us.yml`.

## Adding a New Translation

1. Copy [`locales/en_us.yml`](locales/en_us.yml) to `locales/<lang_region>.yml` using lowercase POSIX locale naming (for example: `locales/es_es.yml`, `locales/fr_fr.yml`, `locales/de_de.yml`, `locales/ja_jp.yml`).
2. Translate all string values while keeping every YAML key unchanged.
3. Preserve placeholder tokens (such as `{version}`, `{mode}`, `{value}`, `{active_mode}`) and CLI flag names (`--help`, `--version`, `--color`, etc.) verbatim.
4. Keep HUD strings concise so they fit cleanly in compact terminal layouts.
5. Run `make check` to ensure all checks pass.
6. Open a Pull Request targeting `main`.

## Requesting a Translation

If you would like to request a new language or report a translation issue without opening a PR, please use the **Translation Request** issue template:

- https://github.com/HappyAmos/kcchase/issues/new/choose
