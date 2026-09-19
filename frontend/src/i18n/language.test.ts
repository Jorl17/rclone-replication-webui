import { describe, expect, it } from 'vitest';
import { cronstrueLocale, dateLocale, resolveLanguage } from './index';

describe('resolveLanguage', () => {
  it('maps Portuguese variants to pt', () => {
    expect(resolveLanguage('pt')).toBe('pt');
    expect(resolveLanguage('pt-PT')).toBe('pt');
    expect(resolveLanguage('pt-BR')).toBe('pt');
  });

  it('maps French variants to fr', () => {
    expect(resolveLanguage('fr')).toBe('fr');
    expect(resolveLanguage('fr-FR')).toBe('fr');
  });

  it('maps everything else to en', () => {
    expect(resolveLanguage('en')).toBe('en');
    expect(resolveLanguage('en-GB')).toBe('en');
    expect(resolveLanguage('de')).toBe('en');
  });
});

describe('dateLocale', () => {
  it('uses European locales and English as en-GB', () => {
    expect(dateLocale('pt-PT')).toBe('pt-PT');
    expect(dateLocale('fr')).toBe('fr');
    expect(dateLocale('en-US')).toBe('en-GB');
  });
});

describe('cronstrueLocale', () => {
  it('uses cronstrue European Portuguese', () => {
    expect(cronstrueLocale('pt')).toBe('pt_PT');
    expect(cronstrueLocale('fr-FR')).toBe('fr');
    expect(cronstrueLocale('de')).toBe('en');
  });
});
