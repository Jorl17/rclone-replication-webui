import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { ArrowLeft, Info } from 'lucide-react';
import { Trans, useTranslation } from 'react-i18next';
import { useCreateChannel, useNotificationPreview, useNotifications, useUpdateChannel } from '../hooks/useNotifications';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { LanguagePicker } from '../i18n/LanguagePicker';
import { resolveLanguage, type SupportedLanguage } from '../i18n';
import {
  emptyTemplates,
  normalizeTemplates,
  type ChannelTemplates,
  type NotificationEvent,
} from '../types/notification';

type FormValues = {
  name: string;
  apprise_url: string;
  enabled: boolean;
  language: SupportedLanguage;
  templates: ChannelTemplates;
};

const EVENTS: NotificationEvent[] = ['error', 'success', 'skipped', 'test'];

export function NotificationFormPage() {
  const { t, i18n } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const navigate = useNavigate();
  const { data: channels } = useNotifications();
  const existing = channels?.find(c => c.id === id);
  const createChannel = useCreateChannel();
  const updateChannel = useUpdateChannel();
  const uiLanguage = resolveLanguage(i18n.resolvedLanguage ?? i18n.language);
  const [event, setEvent] = useState<NotificationEvent>('error');

  const { register, handleSubmit, reset, setValue, watch, formState: { errors, isSubmitting, dirtyFields } } = useForm<FormValues>({
    defaultValues: {
      name: '',
      apprise_url: '',
      enabled: true,
      language: existing ? resolveLanguage(existing.language ?? 'fr') : uiLanguage,
      templates: emptyTemplates(),
    },
  });

  const language = watch('language');
  const templates = watch('templates');
  const preview = useNotificationPreview(language, templates, !isEdit || !!existing);
  const previewMessage = preview.data?.language === language ? preview.data[event] : undefined;

  useEffect(() => {
    if (!existing) return;
    reset({
      name: existing.name,
      apprise_url: existing.apprise_url,
      enabled: existing.enabled,
      language: resolveLanguage(existing.language ?? 'fr'),
      templates: normalizeTemplates(existing.templates),
    });
  }, [existing, reset]);

  useEffect(() => {
    if (isEdit || dirtyFields.language) return;
    setValue('language', uiLanguage);
  }, [isEdit, dirtyFields.language, uiLanguage, setValue]);

  const onSubmit = async (values: FormValues) => {
    try {
      if (isEdit && id) {
        await updateChannel.mutateAsync({ id, payload: values });
      } else {
        await createChannel.mutateAsync(values);
      }
      navigate('/notifications');
    } catch { /* handled */ }
  };

  const mutationError = createChannel.error || updateChannel.error;
  const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

  const eventLabel = (key: NotificationEvent) => {
    if (key === 'error') return t('tasks.notify.onError');
    if (key === 'success') return t('tasks.notify.onSuccess');
    if (key === 'skipped') return t('tasks.notify.onSkipped');
    return t('notifications.form.eventTest');
  };

  return (
    <div className="p-8 max-w-5xl animate-fade-in">
      <button onClick={() => navigate('/notifications')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 mb-4 transition-colors">
        <ArrowLeft size={14} /> {t('notifications.form.back')}
      </button>

      <h1 className="text-2xl font-bold text-surface-900 mb-1">
        {isEdit ? t('notifications.form.editTitle') : t('notifications.form.createTitle')}
      </h1>
      <p className="text-sm text-surface-500 mb-6">
        {isEdit ? t('notifications.form.editSubtitle') : t('notifications.form.createSubtitle')}
      </p>

      {mutationError && <div className="mb-4"><ErrorBanner message={(mutationError as Error).message} /></div>}

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
          <div className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('notifications.form.name')} <span className="text-red-400">*</span></label>
              <input {...register('name', { required: t('validation.required') })} className={inputCls} placeholder={t('notifications.form.namePlaceholder')} />
              <p className="text-xs text-surface-400 mt-1">{t('notifications.form.nameHelp')}</p>
              {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name.message}</p>}
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('notifications.form.url')} <span className="text-red-400">*</span></label>
              <input {...register('apprise_url', { required: t('validation.required') })} className={`${inputCls} font-mono`} placeholder="mmost://mattermost.example.com/webhook-token" />
              <div className="flex items-start gap-1.5 mt-1.5">
                <Info size={12} className="shrink-0 mt-0.5 text-surface-400" />
                <p className="text-xs text-surface-400">
                  <Trans
                    i18nKey="notifications.form.urlHelp"
                    components={{
                      br: <br />,
                      code: <code className="bg-surface-100 px-1 rounded" />,
                    }}
                  />
                </p>
              </div>
              {errors.apprise_url && <p className="text-red-500 text-xs mt-1">{errors.apprise_url.message}</p>}
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('language.label')}</label>
              <LanguagePicker
                value={language}
                onChange={(next) => setValue('language', next, { shouldDirty: true })}
                variant="select"
                ariaLabel={t('language.label')}
                className="w-48 border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow"
              />
              <p className="text-xs text-surface-400 mt-1">{t('notifications.form.languageHelp')}</p>
            </div>
            <div className="flex items-center gap-3 py-1">
              <input type="checkbox" {...register('enabled')} id="enabled" className="rounded border-surface-300 text-brand-600 focus:ring-brand-500" />
              <label htmlFor="enabled" className="text-sm text-surface-700 cursor-pointer">
                {t('notifications.form.channelEnabled')}
                <span className="block text-xs text-surface-400">{t('notifications.form.channelEnabledHelp')}</span>
              </label>
            </div>
          </div>
          <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
            <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">{t('notifications.form.messageLegend')}</legend>
            <p className="text-xs text-surface-400">{t('notifications.form.messageHelp')}</p>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('notifications.form.event')}</label>
              <select
                value={event}
                onChange={(e) => setEvent(e.target.value as NotificationEvent)}
                className={inputCls}
              >
                {EVENTS.map((key) => (
                  <option key={key} value={key}>{eventLabel(key)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('notifications.form.subject')}</label>
              <input
                {...register(`templates.${event}.subject`)}
                className={inputCls}
                placeholder={previewMessage?.subject}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-surface-700 mb-1">{t('notifications.form.body')}</label>
              <textarea
                {...register(`templates.${event}.body`)}
                rows={3}
                className={inputCls}
                placeholder={previewMessage?.body}
              />
              <p className="text-xs text-surface-400 mt-1">{t('notifications.form.placeholders')}</p>
            </div>
            <div>
              <p className="text-xs font-medium text-surface-500 mb-1">{t('notifications.form.preview')}</p>
              {preview.isError && <p className="text-xs text-red-600">{t('notifications.form.previewFailed')}</p>}
              {!preview.isError && previewMessage && (
                <div className="max-h-28 overflow-y-auto rounded-md bg-surface-50 px-2.5 py-2 text-xs text-surface-600">
                  <p className="font-medium text-surface-800">{previewMessage.subject}</p>
                  <pre className="mt-1 whitespace-pre-wrap font-sans leading-snug">{previewMessage.body}</pre>
                </div>
              )}
            </div>
          </fieldset>
        </div>
        <div className="flex gap-3 pt-4 border-t border-surface-200">
          <button type="submit" disabled={isSubmitting} className="px-5 py-2.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors shadow-sm">
            {isEdit ? t('common.save') : t('notifications.form.createSubmit')}
          </button>
          <button type="button" onClick={() => navigate('/notifications')} className="px-5 py-2.5 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
            {t('common.cancel')}
          </button>
        </div>
      </form>
    </div>
  );
}
