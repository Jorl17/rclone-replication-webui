import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Loader2 } from 'lucide-react';
import { Virtuoso } from 'react-virtuoso';
import { getRunLogs } from '../../api/runs';
import { formatBytes } from '../../i18n/format';
import { useWindowedLog, type LogLoad } from '../../hooks/useWindowedLog';
import { LOG_PANE_PX, logEdgePx } from './logScroll';
import { useLogScroller } from './useLogScroller';

interface Props {
  runId: string;
}

type LogListContext = {
  loading: LogLoad;
  label: string;
  canFetchOlder: boolean;
  canFetchNewer: boolean;
};

function LogSpinner({ label }: { label: string }) {
  return (
    <div role="status" aria-label={label} className="flex h-full w-full items-center justify-center">
      <Loader2 className="h-4 w-4 animate-spin text-brand-400" aria-hidden />
    </div>
  );
}

export function LogListHeader({ context }: { context?: LogListContext }) {
  const open = context?.loading === 'older' || !!context?.canFetchOlder;
  return (
    <div data-log-edge="start" style={{ height: logEdgePx(open) }}>
      {context?.loading === 'older' ? <LogSpinner label={context.label} /> : null}
    </div>
  );
}

export function LogListFooter({ context }: { context?: LogListContext }) {
  const open = context?.loading === 'newer' || !!context?.canFetchNewer;
  return (
    <div data-log-edge="end" style={{ height: logEdgePx(open) }}>
      {context?.loading === 'newer' ? <LogSpinner label={context.label} /> : null}
    </div>
  );
}

const logListComponents = {
  Header: LogListHeader,
  Footer: LogListFooter,
};

export function RunLogViewer({ runId }: Props) {
  const { t } = useTranslation();
  const fetchPage = useCallback((query: Parameters<typeof getRunLogs>[1]) => getRunLogs(runId, query), [runId]);
  const log = useWindowedLog(fetchPage, !!runId);
  const loadingLabel = t('tasks.detail.loadingLogs');
  const scrollerRef = useLogScroller({
    loading: log.loading,
    canFetchOlder: log.canFetchOlder,
    canFetchNewer: log.canFetchNewer,
  });
  const jumping = log.loading === 'jump' || (log.loading !== null && log.lines.length === 0);

  return (
    <div className="bg-surface-950 font-mono text-xs text-surface-300">
      <div className="flex items-center justify-between gap-3 px-5 py-2 border-b border-surface-800 text-[11px]">
        <span className="text-surface-500">
          {t('tasks.detail.logSize', { size: formatBytes(log.totalBytes, t) })}
        </span>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={log.jumpStart}
            disabled={log.atStart && log.lines.length > 0}
            className="cursor-pointer text-brand-400 hover:text-brand-300 disabled:cursor-default disabled:text-surface-600 disabled:hover:text-surface-600"
          >
            {t('tasks.detail.logJumpStart')}
          </button>
          <button
            type="button"
            onClick={log.jumpEnd}
            disabled={log.atEnd && log.lines.length > 0}
            className="cursor-pointer text-brand-400 hover:text-brand-300 disabled:cursor-default disabled:text-surface-600 disabled:hover:text-surface-600"
          >
            {t('tasks.detail.logJumpEnd')}
          </button>
        </div>
      </div>
      {log.error && <div className="px-5 py-4 text-red-400">{t('tasks.detail.logError')}</div>}
      {!log.error && (
        <div className="relative" style={{ height: LOG_PANE_PX }} data-log-pane>
          {log.lines.length > 0 && (
            <Virtuoso
              key={log.windowEpoch}
              style={{ height: LOG_PANE_PX }}
              data={log.lines}
              context={{
                loading: log.loading,
                label: loadingLabel,
                canFetchOlder: log.canFetchOlder,
                canFetchNewer: log.canFetchNewer,
              }}
              components={logListComponents}
              scrollerRef={scrollerRef}
              firstItemIndex={log.firstItemIndex}
              computeItemKey={(_index, line) => line.offset}
              initialTopMostItemIndex={log.atEnd ? Math.max(log.lines.length - 1, 0) : 0}
              increaseViewportBy={200}
              startReached={log.loadOlder}
              endReached={log.loadNewer}
              itemContent={(_index, line) => (
                <div className="px-5 whitespace-pre-wrap break-all">{line.text || ' '}</div>
              )}
            />
          )}
          {jumping && (
            <div className="absolute inset-0 flex items-center justify-center bg-surface-950/80">
              <LogSpinner label={loadingLabel} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}
