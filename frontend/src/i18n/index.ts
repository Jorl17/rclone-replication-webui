import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';
import cronstrue from 'cronstrue/i18n';
import en from './locales/en.json';
import fr from './locales/fr.json';
import pt from './locales/pt.json';

export const SUPPORTED_LANGUAGES = ['en', 'fr', 'pt'] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export const LANGUAGE_STORAGE_KEY = 'rclone-ui-language';

export function resolveLanguage(lng: string): SupportedLanguage {
  if (lng.toLowerCase().startsWith('pt')) return 'pt';
  if (lng.toLowerCase().startsWith('fr')) return 'fr';
  return 'en';
}

function applyDocumentLanguage(lng: string) {
  const language = resolveLanguage(lng);
  document.documentElement.lang = language === 'pt' ? 'pt-PT' : language;
  document.title = i18n.t('app.documentTitle');
}

export const i18nReady = i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      fr: { translation: fr },
      pt: { translation: pt },
    },
    supportedLngs: [...SUPPORTED_LANGUAGES],
    // Forgotten keys stay French so upstream users never see English by accident.
    fallbackLng: 'fr',
    nonExplicitSupportedLngs: true,
    interpolation: { escapeValue: false },
    detection: {
      order: ['localStorage', 'navigator'],
      // Infer from the browser until the user picks a language. Do not write that inference.
      caches: [],
      lookupLocalStorage: LANGUAGE_STORAGE_KEY,
      convertDetectedLanguage: (lng) => resolveLanguage(lng),
    },
  })
  .then(() => {
    applyDocumentLanguage(i18n.resolvedLanguage ?? i18n.language);
  });

i18n.on('languageChanged', applyDocumentLanguage);

/** Persist only after an explicit UI choice. Browser inference must not write this key. */
export function persistLanguage(language: string) {
  const resolved = resolveLanguage(language);
  window.localStorage.setItem(LANGUAGE_STORAGE_KEY, resolved);
  return i18n.changeLanguage(resolved);
}

export default i18n;

export function cronstrueLocale(language: string): 'en' | 'fr' | 'pt_PT' {
  const resolved = resolveLanguage(language);
  if (resolved === 'pt') return 'pt_PT';
  if (resolved === 'fr') return 'fr';
  return 'en';
}

export function dateLocale(language: string): string {
  const resolved = resolveLanguage(language);
  if (resolved === 'pt') return 'pt-PT';
  if (resolved === 'fr') return 'fr';
  return 'en-GB';
}

export function formatCronHuman(expr: string, language: string): string {
  const locale = cronstrueLocale(language);
  // HEAD called `cronstrue.toString(expr, { locale: 'fr' })` with no extra flags.
  if (locale === 'fr') {
    return cronstrue.toString(expr, { locale: 'fr' });
  }
  return cronstrue.toString(expr, {
    locale,
    use24HourTimeFormat: true,
  });
}
