import { describe, expect, it } from 'vitest';
import { emptyTemplates, normalizeTemplates } from './notification';

describe('emptyTemplates', () => {
  it('starts every event with blank subject and body', () => {
    const templates = emptyTemplates();
    for (const event of ['error', 'success', 'skipped', 'test'] as const) {
      expect(templates[event]).toEqual({ subject: '', body: '' });
    }
  });
});

describe('normalizeTemplates', () => {
  it('returns empty templates when nothing was stored', () => {
    expect(normalizeTemplates(null)).toEqual(emptyTemplates());
    expect(normalizeTemplates(undefined)).toEqual(emptyTemplates());
  });

  it('fills missing events so the form always has four slots', () => {
    const templates = normalizeTemplates({
      error: { subject: 'Fail', body: 'code={exit_code}' },
    });
    expect(templates.error).toEqual({ subject: 'Fail', body: 'code={exit_code}' });
    expect(templates.success).toEqual({ subject: '', body: '' });
    expect(templates.skipped).toEqual({ subject: '', body: '' });
    expect(templates.test).toEqual({ subject: '', body: '' });
  });
});
