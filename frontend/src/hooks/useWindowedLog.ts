import { useCallback, useEffect, useRef, useState } from 'react';
import type { RunLogLine, RunLogPage, RunLogQuery } from '../types/taskRun';

// FIXME: this caps line count, not bytes. A few huge lines (or 2500 long ones)
// still blow the tab. Bound the buffer by decoded size as well.
export const MAX_LOG_LINES = 2500;
export const FIRST_ITEM_INDEX = 1_000_000;

export type LogLoad = 'jump' | 'older' | 'newer' | null;

function loadKind(query: RunLogQuery): Exclude<LogLoad, null> {
  if ('from' in query) return 'jump';
  if ('before' in query) return 'older';
  return 'newer';
}

export type LogMerge = {
  lines: RunLogLine[];
  droppedFromStart: boolean;
  droppedFromEnd: boolean;
  prepended: number;
  droppedStartCount: number;
};

export function mergeLogWindow(
  current: RunLogLine[],
  page: RunLogPage,
  query: RunLogQuery,
  maxLines: number = MAX_LOG_LINES,
): LogMerge {
  if ('from' in query) {
    const lines = page.lines.slice(0, maxLines);
    return {
      lines,
      droppedFromStart: page.lines.length > maxLines,
      droppedFromEnd: false,
      prepended: 0,
      droppedStartCount: 0,
    };
  }

  const seen = new Set(current.map(line => line.offset));
  const incoming = page.lines.filter(line => !seen.has(line.offset));

  if ('after' in query) {
    const next = [...current, ...incoming];
    if (next.length > maxLines) {
      const droppedStartCount = next.length - maxLines;
      return {
        lines: next.slice(droppedStartCount),
        droppedFromStart: true,
        droppedFromEnd: false,
        prepended: 0,
        droppedStartCount,
      };
    }
    return {
      lines: next,
      droppedFromStart: false,
      droppedFromEnd: false,
      prepended: 0,
      droppedStartCount: 0,
    };
  }

  const next = [...incoming, ...current];
  if (next.length > maxLines) {
    const lines = next.slice(0, maxLines);
    const kept = new Set(lines.map(line => line.offset));
    return {
      lines,
      droppedFromStart: false,
      droppedFromEnd: true,
      prepended: incoming.filter(line => kept.has(line.offset)).length,
      droppedStartCount: 0,
    };
  }
  return {
    lines: next,
    droppedFromStart: false,
    droppedFromEnd: false,
    prepended: incoming.length,
    droppedStartCount: 0,
  };
}

/** Virtuoso keeps the viewport pinned if this moves by the items that appeared/disappeared at the top. */
export function nextFirstItemIndex(
  current: number,
  query: RunLogQuery,
  merged: Pick<LogMerge, 'prepended' | 'droppedStartCount'>,
): number {
  if ('from' in query) return FIRST_ITEM_INDEX;
  if ('before' in query) return current - merged.prepended;
  return current + merged.droppedStartCount;
}

export function useWindowedLog(
  fetchPage: (query: RunLogQuery) => Promise<RunLogPage>,
  enabled: boolean,
) {
  const [lines, setLines] = useState<RunLogLine[]>([]);
  const [totalBytes, setTotalBytes] = useState(0);
  const [atStart, setAtStart] = useState(true);
  const [atEnd, setAtEnd] = useState(true);
  const [loading, setLoading] = useState<LogLoad>(null);
  const [error, setError] = useState(false);
  const [firstItemIndex, setFirstItemIndex] = useState(FIRST_ITEM_INDEX);
  const [windowEpoch, setWindowEpoch] = useState(0);
  const inflight = useRef(false);
  const linesRef = useRef<RunLogLine[]>([]);

  const applyPage = useCallback((page: RunLogPage, query: RunLogQuery) => {
    const merged = mergeLogWindow(linesRef.current, page, query);
    linesRef.current = merged.lines;
    setLines(merged.lines);
    setFirstItemIndex(index => nextFirstItemIndex(index, query, merged));
    setAtStart(page.at_start && !merged.droppedFromStart);
    setAtEnd(page.at_end && !merged.droppedFromEnd);
    setTotalBytes(page.total_bytes);
    if ('from' in query) setWindowEpoch(epoch => epoch + 1);
  }, []);

  const load = useCallback(
    async (query: RunLogQuery) => {
      if (!enabled || inflight.current) return;
      inflight.current = true;
      setLoading(loadKind(query));
      setError(false);
      try {
        const page = await fetchPage(query);
        applyPage(page, query);
      } catch {
        setError(true);
      } finally {
        inflight.current = false;
        setLoading(null);
      }
    },
    [applyPage, enabled, fetchPage],
  );

  useEffect(() => {
    if (!enabled) {
      linesRef.current = [];
      setLines([]);
      setTotalBytes(0);
      setAtStart(true);
      setAtEnd(true);
      setFirstItemIndex(FIRST_ITEM_INDEX);
      setWindowEpoch(0);
      setLoading(null);
      setError(false);
      return;
    }
    void load({ from: 'end' });
  }, [enabled, load]);

  const jumpStart = useCallback(() => {
    inflight.current = false;
    void load({ from: 'start' });
  }, [load]);

  const jumpEnd = useCallback(() => {
    inflight.current = false;
    void load({ from: 'end' });
  }, [load]);

  const loadOlder = useCallback(() => {
    if (atStart || lines.length === 0 || inflight.current) return;
    void load({ before: lines[0].offset });
  }, [atStart, lines, load]);

  const loadNewer = useCallback(() => {
    if (atEnd || lines.length === 0 || inflight.current) return;
    const last = lines[lines.length - 1];
    void load({ after: last.offset + last.text.length + 1 });
  }, [atEnd, lines, load]);

  return {
    lines,
    totalBytes,
    atStart,
    atEnd,
    loading,
    error,
    firstItemIndex,
    windowEpoch,
    canFetchOlder: !atStart && lines.length > 0,
    canFetchNewer: !atEnd && lines.length > 0,
    jumpStart,
    jumpEnd,
    loadOlder,
    loadNewer,
  };
}
