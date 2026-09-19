import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, Pencil, Trash2, Send } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useNotifications, useDeleteChannel, useTestChannel } from '../hooks/useNotifications';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { resolveLanguage } from '../i18n';
import { ErrorBanner } from '../components/ui/ErrorBanner';
import { Tooltip } from '../components/ui/Tooltip';

export function NotificationsPage() {
  const { t } = useTranslation();
  const { data: channels, isLoading, error } = useNotifications();
  const deleteChannel = useDeleteChannel();
  const testChannel = useTestChannel();
  const navigate = useNavigate();
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({});

  const handleTest = async (id: string) => {
    const result = await testChannel.mutateAsync(id);
    setTestResults(prev => ({
      ...prev,
      [id]: {
        success: result.success,
        message: result.success ? t('notifications.list.sent') : t('notifications.list.error'),
      },
    }));
  };

  if (isLoading) return <div className="p-8 text-surface-400">{t('common.loading')}</div>;

  return (
    <div className="p-8 max-w-5xl animate-fade-in">
      <div className="flex items-end justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">{t('notifications.list.title')}</h1>
          <p className="text-sm text-surface-500 mt-1">
            {t('notifications.list.subtitle')}
          </p>
        </div>
        <Tooltip content={t('notifications.list.addTooltip')}>
          <button
            onClick={() => navigate('/notifications/new')}
            className="flex items-center gap-2 px-4 py-2.5 bg-brand-600 text-white rounded-lg hover:bg-brand-700 text-sm font-medium transition-colors shadow-sm"
          >
            <Plus size={16} /> {t('common.add')}
          </button>
        </Tooltip>
      </div>

      {error && <div className="mb-4"><ErrorBanner message={(error as Error).message} /></div>}

      <div className="bg-white rounded-xl border border-surface-200 shadow-sm overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-surface-50 border-b border-surface-200">
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('common.name')}</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content={t('notifications.list.colUrlTooltip')} position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">{t('notifications.list.colUrl')}</span>
                </Tooltip>
              </th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('language.label')}</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('notifications.list.colEnabled')}</th>
              <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                <Tooltip content={t('notifications.list.colUsedByTooltip')} position="bottom">
                  <span className="cursor-help border-b border-dashed border-surface-400">{t('notifications.list.colUsedBy')}</span>
                </Tooltip>
              </th>
              <th className="text-right px-5 py-3"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-100">
            {channels?.map(ch => (
              <tr key={ch.id} className="hover:bg-surface-50/50 transition-colors">
                <td className="px-5 py-3.5 font-medium text-surface-900">{ch.name}</td>
                <td className="px-5 py-3.5 text-surface-500 font-mono text-xs truncate max-w-xs">{ch.apprise_url}</td>
                <td className="px-5 py-3.5 text-xs font-medium text-surface-600">{t(`language.native.${resolveLanguage(ch.language ?? 'fr')}`)}</td>
                <td className="px-5 py-3.5">
                  <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${ch.enabled ? 'bg-emerald-50 text-emerald-700' : 'bg-surface-100 text-surface-500'}`}>
                    {ch.enabled ? t('notifications.list.active') : t('notifications.list.inactive')}
                  </span>
                </td>
                <td className="px-5 py-3.5 text-surface-500 text-xs">
                  {ch.task_count
                    ? <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-brand-50 text-brand-700 font-medium">{t('notifications.list.usedByTasks', { count: ch.task_count })}</span>
                    : <span className="text-surface-300">{t('notifications.list.usedByNone')}</span>}
                </td>
                <td className="px-5 py-3.5">
                  <div className="flex items-center justify-end gap-1">
                    {testResults[ch.id] && (
                      <span className={`text-xs mr-2 ${testResults[ch.id].success ? 'text-emerald-600' : 'text-red-600'}`}>
                        {testResults[ch.id].message}
                      </span>
                    )}
                    <Tooltip content={t('notifications.list.testTooltip')}>
                      <button onClick={() => handleTest(ch.id)} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                        <Send size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content={t('notifications.list.editTooltip')}>
                      <button onClick={() => navigate(`/notifications/${ch.id}/edit`)} className="p-2 text-surface-400 hover:text-brand-600 hover:bg-brand-50 rounded-lg transition-colors">
                        <Pencil size={15} />
                      </button>
                    </Tooltip>
                    <Tooltip content={ch.task_count ? t('notifications.list.deleteBlocked') : t('notifications.list.deleteTooltip')}>
                      <button
                        onClick={() => setConfirmId(ch.id)}
                        disabled={!!ch.task_count}
                        className={`p-2 rounded-lg transition-colors ${ch.task_count ? 'text-surface-200 cursor-not-allowed' : 'text-surface-400 hover:text-red-600 hover:bg-red-50'}`}
                      >
                        <Trash2 size={15} />
                      </button>
                    </Tooltip>
                  </div>
                </td>
              </tr>
            ))}
            {!channels?.length && (
              <tr>
                <td colSpan={6} className="px-5 py-12 text-center text-surface-400">
                  <p className="text-base font-medium mb-1">{t('notifications.list.emptyTitle')}</p>
                  <p className="text-xs">{t('notifications.list.emptyBody')}</p>
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <ConfirmDialog
        open={!!confirmId}
        title={t('notifications.list.deleteTitle')}
        message={t('notifications.list.deleteMessage')}
        onConfirm={() => { if (confirmId) deleteChannel.mutate(confirmId); setConfirmId(null); }}
        onCancel={() => setConfirmId(null)}
      />
    </div>
  );
}
