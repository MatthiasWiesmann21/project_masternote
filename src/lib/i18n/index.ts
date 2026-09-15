import { writable, derived } from 'svelte/store';
import { en } from './en';
import { de } from './de';
import { fr } from './fr';
import { es } from './es';

export type Locale = 'en' | 'de' | 'fr' | 'es';

export const localeLabels: Record<Locale, string> = {
  en: 'English',
  de: 'Deutsch',
  fr: 'Français',
  es: 'Español',
};

const dicts: Record<Locale, Record<string, string>> = { en, de, fr, es };

export const locale = writable<Locale>('en');

/**
 * Translation function store. Usage in markup: {$t('settings.theme')}
 * In code: get(t)('key', { name: 'x' })
 */
export const t = derived(locale, ($locale) => {
  return (key: string, vars?: Record<string, string | number>): string => {
    let s = dicts[$locale]?.[key] ?? dicts.en[key] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        s = s.replaceAll(`{${k}}`, String(v));
      }
    }
    return s;
  };
});
