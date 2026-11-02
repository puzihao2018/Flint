# Maintaining translations

[简体中文](../CN/LOCALIZATION.md)

Under **Settings & storage → Language**, Flint supports Follow system, Simplified Chinese, and English. Follow system is the default. Chinese system locales, including zh-Hant, use Simplified Chinese. Other unsupported locales fall back to English.

## Implementation

- src/locales/zh-CN.json and en.json share keys. The Chinese source messages serve as stable lookup keys, while translations can change independently. React and the Rust native menus read the same files.
- src/i18n.ts resolves the locale, looks up messages, and interpolates values locally.
- src-tauri/src/preferences.rs writes preferences.json in the data directory using a temporary file and replacement. A failed write leaves the in-memory preference unchanged. Preferences are independent of service configuration.
- The main window and menu bar synchronize through the preferences-changed event. The WebView supplies the system language on startup. An explicit preference takes precedence.
- src/locales/backend-templates.json maps known engine diagnostics to translated display strings. Unknown errors and original logs retain their text. Prefer error codes and structured arguments for new complex diagnostics.

## Editing copy

Use t() with a complete message key and named parameters. Keep keys and placeholder sets identical in both resources. Treat command examples such as `${port}` as literal text. Translate Flint's empty-log placeholder but preserve user-provided names, paths, commands, versions, and logs. Native file pickers follow the operating system.

Add translations for static engine messages and full templates for dynamic diagnostics. Preserve ports, PIDs, paths, and other diagnostic values. Apply templates only to the diagnostic display.

## Adding a language

1. Add a locale JSON with the existing keys.
2. Extend Locale, LanguagePreference, the resource map, system-locale matching, and the settings selector.
3. Extend Rust Language, resolve_locale, and native-menu resource selection.
4. Test fallback, persistence, native menus, long strings, and accessible names in the actual UI.

Run npm run test:i18n for key and placeholder parity, static UI coverage, catalog messages, fallback, special-character interpolation, and nested errors. npm test also exercises Rust preference persistence, failure behavior, and service regressions. Check the native macOS UI separately.
