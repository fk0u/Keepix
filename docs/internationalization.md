# Internationalization (i18n) Architecture

Keepix features fully localized bilingual support for **English** and **Indonesian**. To maintain minimal overhead and zero boot latency, Keepix implements a customized translation system instead of standard node-module alternatives.

---

## 🛠️ Translation Core

The localization layer resides in [index.ts](file:///d:/Project/Keepix/src/lib/i18n/index.ts) and is built using Svelte store primitives.

```typescript
import { writable, derived } from 'svelte/store';

// 1. Current active language state
export const locale = writable<Language>('en');

// 2. Translation lookup function
export const t = derived(locale, ($locale) => {
  return (key: string, vars: Record<string, string | number> = {}) => {
    let text = translations[$locale][key] || translations['en'][key] || key;
    
    // Interpolate dynamic variables
    Object.entries(vars).forEach(([k, v]) => {
      text = text.replace(new RegExp(`{${k}}`, 'g'), String(v));
    });
    
    return text;
  };
});
```

### Key Highlights
- **Derived Reactivity**: The `$t` store function updates all Svelte text nodes automatically when the user swaps languages in Settings.
- **Variable Interpolation**: Supports dynamic string placeholders (e.g., `$t('dialog.confirm_delete', { name: project.name })`).
- **Graceful Fallback**: If a key is missing in the Indonesian dictionary, it falls back to the English variant, preventing blank text in the UI.

---

## 🗄️ Database Syncing & Persistence

The active language preference is synchronized with the SQLite database to survive app restarts:
1. When Keepix boots, it invokes `commands::get_setting("language")` from the Tauri Rust layer.
2. If a setting is found, Svelte's `locale` store is set to the saved language (`'en'` or `'id'`). If not found, it defaults to the host machine's system locale or `'en'`.
3. When the user toggles the language dropdown inside the Preferences Modal:
   - Svelte updates the `locale` store.
   - Svelte invokes `commands::set_setting("language", selectedLanguage)`, which writes the new value into the SQLite settings registry.

---

## 📝 Localization Dictionary Structure

Translations are organized in modular sections inside the `translations` object:
- **`app.*`**: Application credentials and titles.
- **`home.*`**: Project select screens, project search indicators, and onboarding cards.
- **`edit.*`**: Professional image sliders, presets, transforms, and pixel actions.
- **`metadata.*`**: Camera models, lenses, ISO metrics, and aperture descriptions.
- **`shortcuts.*`**: Guided keys inside the Help overlay modal.
- **`settings.*`**: Interface scales, custom folders, default exports, and UI language selectors.
- **`toast.*`**: Temporary notifications (e.g. "Adjustments Copied", "Project Deleted").

---

## ➕ Adding New Localized Keys

To add a new translation string:

1. Open [src/lib/i18n/index.ts](file:///d:/Project/Keepix/src/lib/i18n/index.ts).
2. Find the `translations` constant.
3. Add your key-value pair under the `en` object:
   ```typescript
   'workspace.import_label': 'Drag & drop media folder here',
   ```
4. Add the corresponding key-value pair under the `id` object:
   ```typescript
   'workspace.import_label': 'Seret & lepas folder media di sini',
   ```
5. Reference the key in Svelte components:
   ```svelte
   <span class="label">{$t('workspace.import_label')}</span>
   ```
