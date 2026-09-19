import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Play, RotateCcw, Pencil, ChevronDown, ChevronRight, ArrowLeft, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useTask, useTriggerTask, useRestoreTask } from '../hooks/useTasks';
import { useRemotes } from '../hooks/useRemotes';
import { useTaskRuns } from '../hooks/useTaskRuns';
import { RunLogViewer } from '../components/tasks/RunLogViewer';
import { useTaskProgress } from '../hooks/useTaskProgress';
import { LiveProgressPanel } from '../components/progress/LiveProgressPanel';
import { StatusBadge } from '../components/ui/StatusBadge';
import { RestoreModal } from '../components/tasks/RestoreModal';
import { Tooltip } from '../components/ui/Tooltip';
import type { RcloneStats } from '../types/taskRun';
import { formatCronHuman } from '../i18n';
import { formatBytes, formatDateTime } from '../i18n/format';

function StatsCell({ stats }: { stats: RcloneStats | null }) {
  const { t } = useTranslation();
  if (!stats) return <span className="text-surface-300">—</span>;
  return (
    <div className="flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
      <Tooltip content={t('tasks.detail.statTransfersTooltip')}>
        <span className="cursor-help">
          <span className="text-surface-400">{t('tasks.detail.statTransfers')}</span>{' '}
          <span className="font-medium text-surface-700">{stats.transfers}/{stats.totalTransfers}</span>
        </span>
      </Tooltip>
      <Tooltip content={t('tasks.detail.statVolumeTooltip')}>
        <span className="cursor-help">
          <span className="text-surface-400">{t('tasks.detail.statVolume')}</span>{' '}
          <span className="font-medium text-surface-700">{formatBytes(stats.bytes, t)}</span>
        </span>
      </Tooltip>
      <Tooltip content={t('tasks.detail.statChecksTooltip')}>
        <span className="cursor-help">
          <span className="text-surface-400">{t('tasks.detail.statChecks')}</span>{' '}
          <span className="font-medium text-surface-700">{stats.checks}</span>
        </span>
      </Tooltip>
      {stats.deletes > 0 && (
        <Tooltip content={t('tasks.detail.statDeletesTooltip')}>
          <span className="cursor-help">
            <span className="text-surface-400">{t('tasks.detail.statDeletes')}</span>{' '}
            <span className="font-medium text-orange-600">{stats.deletes}</span>
          </span>
        </Tooltip>
      )}
      {stats.errors > 0 && (
        <Tooltip content={t('tasks.detail.statErrorsTooltip')}>
          <span className="cursor-help">
            <span className="text-surface-400">{t('tasks.detail.statErrors')}</span>{' '}
            <span className="font-medium text-red-600">{stats.errors}</span>
          </span>
        </Tooltip>
      )}
    </div>
  );
}

export function TaskDetailPage() {
  const { t, i18n } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: task, isLoading } = useTask(id || '');
  const { data: runs } = useTaskRuns(id || '');
  const { data: remotes } = useRemotes();
  const triggerTask = useTriggerTask();
  const restoreTask = useRestoreTask();
  const [showRestoreConfirm, setShowRestoreConfirm] = useState(false);
  const [forceConnect, setForceConnect] = useState(false);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const progress = useTaskProgress(id || null, task?.running || false, forceConnect);

  if (forceConnect && (task?.running || progress.done)) {
    setForceConnect(false);
  }

  if (isLoading || !task) return <div className="p-8 text-surface-400">{t('common.loading')}</div>;

  const language = i18n.resolvedLanguage ?? i18n.language;
  const cronLabel = task.cron_expression ? (() => { try { return formatCronHuman(task.cron_expression, language); } catch { return task.cron_expression; } })() : t('tasks.detail.manualOnly');
  const triggerLabel = (triggeredBy: string) => {
    if (triggeredBy === 'manual') return t('tasks.detail.startedBy.manual');
    if (triggeredBy === 'scheduler') return t('tasks.detail.startedBy.scheduler');
    return t('tasks.detail.startedBy.restore');
  };

  return (
    <div className="p-8 space-y-6 max-w-5xl animate-fade-in">
      <button onClick={() => navigate('/tasks')} className="flex items-center gap-1.5 text-sm text-surface-500 hover:text-brand-600 transition-colors">
        <ArrowLeft size={14} /> {t('tasks.form.backToList')}
      </button>

      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">{task.name}</h1>
          <div className="mt-1.5 flex items-center gap-3">
            <StatusBadge status={task.running ? 'running' : !task.enabled ? 'disabled' : task.last_run?.status ?? 'unknown'} />
            <span className="text-sm text-surface-500">{cronLabel}</span>
            {task.encryption_enabled && (
              <Tooltip content={t('tasks.list.encryptedTooltip')}>
                <span className="inline-flex items-center gap-1 text-xs font-medium text-brand-700 bg-brand-50 border border-brand-200 rounded-full px-2 py-0.5 cursor-help">
                  <Lock size={11} /> {t('tasks.detail.encrypted')}
                </span>
              </Tooltip>
            )}
          </div>
        </div>
        <div className="flex gap-2">
          <Tooltip content={t('tasks.list.runNow')}>
            <button onClick={() => { triggerTask.mutate(task.id); setForceConnect(true); }} disabled={task.running}
              className="flex items-center gap-1.5 px-3.5 py-2 bg-emerald-600 text-white rounded-lg text-sm font-medium hover:bg-emerald-700 disabled:opacity-40 transition-colors shadow-sm">
              <Play size={14} /> {t('tasks.detail.run')}
            </button>
          </Tooltip>
          <Tooltip content={t('tasks.detail.restoreTooltip')}>
            <button onClick={() => setShowRestoreConfirm(true)} disabled={task.running}
              className="flex items-center gap-1.5 px-3.5 py-2 bg-orange-600 text-white rounded-lg text-sm font-medium hover:bg-orange-700 disabled:opacity-40 transition-colors shadow-sm">
              <RotateCcw size={14} /> {t('tasks.detail.restore')}
            </button>
          </Tooltip>
          <Tooltip content={t('tasks.detail.editTooltip')}>
            <button onClick={() => navigate(`/tasks/${task.id}/edit`)}
              className="flex items-center gap-1.5 px-3.5 py-2 border border-surface-300 text-surface-600 rounded-lg text-sm font-medium hover:bg-surface-50 transition-colors">
              <Pencil size={14} /> {t('common.edit')}
            </button>
          </Tooltip>
        </div>
      </div>

      <div className="bg-white rounded-xl border border-surface-200 p-5 shadow-sm">
        <p className="text-xs font-semibold text-surface-500 uppercase tracking-wider mb-3">{t('tasks.detail.configuration')}</p>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-surface-400 text-xs">{t('tasks.detail.source')}</span>
            <p className="font-mono mt-0.5 text-surface-800">{task.source_remote_name}:{task.source_path}</p>
          </div>
          <div>
            <span className="text-surface-400 text-xs">{t('tasks.detail.destination')}</span>
            <p className="font-mono mt-0.5 text-surface-800">{task.dest_remote_name}:{task.dest_path}</p>
          </div>
          {task.rclone_flags.length > 0 && (
            <div className="col-span-2">
              <span className="text-surface-400 text-xs">{t('tasks.form.rcloneFlags')}</span>
              <p className="font-mono mt-0.5 text-surface-800">{task.rclone_flags.join(' ')}</p>
            </div>
          )}
        </div>
      </div>

      {(task.running || progress.lines.length > 0) && (
        <LiveProgressPanel lines={progress.lines} done={progress.done} status={progress.status} />
      )}

      <div>
        <div className="flex items-center gap-2 mb-3">
          <h2 className="text-lg font-semibold text-surface-800">{t('tasks.detail.history')}</h2>
          <span className="text-xs text-surface-400">{t('tasks.detail.historyCount', { count: runs?.length ?? 0 })}</span>
        </div>
        <div className="bg-white rounded-xl border border-surface-200 shadow-sm overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-surface-50 border-b border-surface-200">
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('tasks.detail.colDate')}</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                  <Tooltip content={t('tasks.detail.colTriggerTooltip')} position="bottom">
                    <span className="cursor-help border-b border-dashed border-surface-400">{t('tasks.detail.colTrigger')}</span>
                  </Tooltip>
                </th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('tasks.detail.colStatus')}</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">{t('tasks.detail.colDuration')}</th>
                <th className="text-left px-5 py-3 text-xs font-semibold text-surface-500 uppercase tracking-wider">
                  <Tooltip content={t('tasks.detail.colStatsTooltip')} position="bottom">
                    <span className="cursor-help border-b border-dashed border-surface-400">{t('tasks.detail.colStats')}</span>
                  </Tooltip>
                </th>
                <th className="text-right px-5 py-3"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-100">
              {runs?.map(run => {
                const isExpanded = run.id === selectedRunId;
                return (
                  <React.Fragment key={run.id}>
                    <tr
                      onClick={() => setSelectedRunId(isExpanded ? null : run.id)}
                      className={`cursor-pointer transition-colors ${isExpanded ? 'bg-surface-100' : 'hover:bg-surface-50/50'}`}
                    >
                      <td className="px-5 py-3 text-surface-700 whitespace-nowrap">{formatDateTime(new Date(run.started_at), language)}</td>
                      <td className="px-5 py-3">
                        <span className="inline-flex px-2 py-0.5 rounded bg-surface-100 text-surface-600 text-xs font-medium">
                          {triggerLabel(run.triggered_by)}
                        </span>
                      </td>
                      <td className="px-5 py-3"><StatusBadge status={run.status as 'success' | 'failure' | 'running' | 'disabled' | 'unknown'} /></td>
                      <td className="px-5 py-3 text-surface-500 whitespace-nowrap">{run.duration_ms != null ? `${(run.duration_ms / 1000).toFixed(1)}s` : '—'}</td>
                      <td className="px-5 py-3"><StatsCell stats={run.stats} /></td>
                      <td className="px-5 py-3 text-right">
                        <span className="text-brand-600 text-xs flex items-center gap-0.5 ml-auto font-medium">
                          {t('tasks.detail.logs')} {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                        </span>
                      </td>
                    </tr>
                    {isExpanded && (
                      <tr>
                        <td colSpan={6} className="p-0 border-t-0">
                          <div className="animate-slide-down">
                            {selectedRunId && <RunLogViewer runId={selectedRunId} />}
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
              {!runs?.length && (
                <tr>
                  <td colSpan={6} className="px-5 py-12 text-center text-surface-400">
                    <p className="text-base font-medium mb-1">{t('tasks.detail.emptyTitle')}</p>
                    <p className="text-xs">{t('tasks.detail.emptyBody')}</p>
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {showRestoreConfirm && (
        <RestoreModal
          open
          task={task}
          remotes={remotes || []}
          onConfirm={(payload) => { restoreTask.mutate({ id: task.id, payload }); setShowRestoreConfirm(false); setForceConnect(true); }}
          onCancel={() => setShowRestoreConfirm(false)}
        />
      )}
    </div>
  );
}
