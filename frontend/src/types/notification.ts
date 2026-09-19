import type { SupportedLanguage } from '../i18n';

export interface TemplateOverride {
  subject: string;
  body: string;
}

export interface ChannelTemplates {
  error: TemplateOverride;
  success: TemplateOverride;
  skipped: TemplateOverride;
  test: TemplateOverride;
}

export type NotificationEvent = keyof ChannelTemplates;

export interface NotificationChannel {
  id: string;
  name: string;
  apprise_url: string;
  enabled: boolean;
  language: SupportedLanguage;
  templates: ChannelTemplates;
  created_at: string;
  updated_at: string;
  task_count?: number;
}

export interface CreateChannelPayload {
  name: string;
  apprise_url: string;
  enabled: boolean;
  language: SupportedLanguage;
  templates: ChannelTemplates;
}

export interface NotificationPreview {
  language: string;
  error: { subject: string; body: string };
  success: { subject: string; body: string };
  skipped: { subject: string; body: string };
  test: { subject: string; body: string };
}

export function emptyTemplates(): ChannelTemplates {
  return {
    error: { subject: '', body: '' },
    success: { subject: '', body: '' },
    skipped: { subject: '', body: '' },
    test: { subject: '', body: '' },
  };
}

export function normalizeTemplates(value?: Partial<ChannelTemplates> | null): ChannelTemplates {
  const empty = emptyTemplates();
  if (!value) return empty;
  return {
    error: { subject: value.error?.subject ?? '', body: value.error?.body ?? '' },
    success: { subject: value.success?.subject ?? '', body: value.success?.body ?? '' },
    skipped: { subject: value.skipped?.subject ?? '', body: value.skipped?.body ?? '' },
    test: { subject: value.test?.subject ?? '', body: value.test?.body ?? '' },
  };
}
