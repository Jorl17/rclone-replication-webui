import cronstrue from 'cronstrue/i18n';
import { describe, expect, it } from 'vitest';
import i18n, { formatCronHuman } from './index';
import { formatBytes, formatDateTime, formatDuration } from './format';

describe('formatDateTime', () => {
  const date = new Date(2026, 8, 19, 15, 4);

  it('keeps the upstream French toLocaleString call', () => {
    expect(formatDateTime(date, 'fr')).toBe(date.toLocaleString('fr'));
    expect(formatDateTime(date, 'fr', { dateStyle: 'medium', timeStyle: 'short' })).toBe(
      date.toLocaleString('fr', { dateStyle: 'medium', timeStyle: 'short' }),
    );
  });

  it('never uses a 12-hour clock for English or Portuguese', () => {
    for (const language of ['en', 'pt']) {
      const formatted = formatDateTime(date, language);
      expect(formatted).not.toMatch(/AM|PM|am|pm/);
      expect(formatted).toMatch(/15/);
    }
  });
});

describe('formatBytes', () => {
  it('uses French units after switching language', async () => {
    const previous = i18n.language;
    await i18n.changeLanguage('fr');
    expect(formatBytes(1536, i18n.t.bind(i18n))).toBe('1.5 Ko');
    await i18n.changeLanguage(previous);
  });
});

describe('formatDuration', () => {
  it('returns an empty string for a missing duration', () => {
    expect(formatDuration(null)).toBe('');
  });

  it('keeps sub-second values in milliseconds', () => {
    expect(formatDuration(250)).toBe('250ms');
  });
});

describe('formatCronHuman', () => {
  it('matches the upstream French cronstrue call', () => {
    expect(formatCronHuman('0 0 * * *', 'fr')).toBe(
      cronstrue.toString('0 0 * * *', { locale: 'fr' }),
    );
  });
});
