import { writable } from 'svelte/store';
import { Store } from '@tauri-apps/plugin-store';
import { locale, type Locale } from '$lib/i18n';

export interface Settings {
  theme: 'light' | 'dark' | 'system';
  hideOnBlur: boolean;
  defaultCategory: number | null;
  graphSignedIn: boolean;
  autosaveInterval: number;
  language: Locale;
}

const defaultSettings: Settings = {
  theme: 'system',
  hideOnBlur: true,
  defaultCategory: null,
  graphSignedIn: false,
  autosaveInterval: 800,
  language: 'en'
};

export const settings = writable<Settings>(defaultSettings);

let tauriStore: Store | null = null;

/** Persist language to the Tauri store so the Rust backend can read it. */
async function persistLanguage(lang: string) {
  try {
    tauriStore ??= await Store.load('settings.json');
    await tauriStore.set('language', lang);
    await tauriStore.save();
  } catch {
    // best-effort — backend falls back to 'en'
  }
}

export function applyTheme(theme: 'light' | 'dark' | 'system') {
  const isDark =
    theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.classList.toggle('dark', isDark);
}

export function initSettings() {
  const stored = localStorage.getItem('masternote-settings');
  if (stored) {
    try {
      const parsed = { ...defaultSettings, ...JSON.parse(stored) };
      settings.set(parsed);
      applyTheme(parsed.theme);
    } catch {
      settings.set(defaultSettings);
      applyTheme('system');
    }
  } else {
    settings.set(defaultSettings);
    applyTheme('system');
  }

  settings.subscribe((s) => {
    localStorage.setItem('masternote-settings', JSON.stringify(s));
    applyTheme(s.theme);
    locale.set(s.language);
    persistLanguage(s.language);
  });

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    settings.subscribe((s) => {
      if (s.theme === 'system') applyTheme('system');
    })();
  });
}
