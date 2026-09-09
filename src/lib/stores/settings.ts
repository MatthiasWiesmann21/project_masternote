import { writable } from 'svelte/store';

export interface Settings {
  theme: 'light' | 'dark' | 'system';
  hideOnBlur: boolean;
  defaultCategory: number | null;
  graphSignedIn: boolean;
}

const defaultSettings: Settings = {
  theme: 'system',
  hideOnBlur: true,
  defaultCategory: null,
  graphSignedIn: false
};

export const settings = writable<Settings>(defaultSettings);

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
  });

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    settings.subscribe((s) => {
      if (s.theme === 'system') applyTheme('system');
    })();
  });
}
