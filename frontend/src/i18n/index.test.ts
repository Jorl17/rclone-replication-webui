import { beforeEach, describe, expect, it } from 'vitest';
import i18n, { LANGUAGE_STORAGE_KEY, persistLanguage } from './index';

describe('i18n setup', () => {
  beforeEach(() => {
    localStorage.removeItem(LANGUAGE_STORAGE_KEY);
  });

  it('falls back to French for missing keys', async () => {
    expect(i18n.options.fallbackLng).toEqual(['fr']);
    i18n.addResource('fr', 'translation', '__test_missing_key_fallback', 'Texte français');
    await i18n.changeLanguage('en');
    expect(i18n.t('__test_missing_key_fallback')).toBe('Texte français');
  });

  it('does not cache inferred browser language', () => {
    const detection = i18n.options.detection as { caches?: string[] } | undefined;
    expect(detection?.caches ?? []).toEqual([]);
  });

  it('writes localStorage only after persistLanguage', async () => {
    expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBeNull();
    await persistLanguage('pt');
    expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe('pt');
    expect(i18n.resolvedLanguage).toBe('pt');
  });
});
