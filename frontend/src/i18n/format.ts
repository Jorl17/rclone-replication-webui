import type { TFunction } from 'i18next';
import { dateLocale, resolveLanguage } from './index';

/**
 * French is the upstream default: keep the exact `toLocaleString('fr')` call
 * the maintainer already ships. Other languages use their locale, 24-hour only.
 */
export function formatDateTime(date: Date, language: string, options?: Intl.DateTimeFormatOptions): string {
  if (resolveLanguage(language) === 'fr') {
    return options ? date.toLocaleString('fr', options) : date.toLocaleString('fr');
  }
  return date.toLocaleString(dateLocale(language), {
    ...options,
    hour12: false,
  });
}

function byteUnitLabels(units: unknown): string[] {
  if (Array.isArray(units)) return units.map(String);
  if (typeof units === 'string' && units.length > 0) {
    return units.split(',').map((part) => part.trim()).filter(Boolean);
  }
  return ['B', 'KB', 'MB', 'GB', 'TB'];
}

export function formatBytes(bytes: number, t: TFunction): string {
  if (bytes === 0) return t('units.bytes.zero');
  const labels = byteUnitLabels(t('units.bytes.units', { returnObjects: true }));
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), labels.length - 1);
  const value = bytes / Math.pow(1024, i);
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${labels[i]}`;
}

export function formatDuration(ms: number | null): string {
  if (ms == null) return '';
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}
