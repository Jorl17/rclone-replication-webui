import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import i18n from '../../i18n';
import type { RunLogPage, RunLogQuery } from '../../types/taskRun';

const getRunLogs = vi.fn<(runId: string, query: RunLogQuery) => Promise<RunLogPage>>();

vi.mock('../../api/runs', () => ({
  getRunLogs: (runId: string, query: RunLogQuery) => getRunLogs(runId, query),
}));

import { LogListFooter, LogListHeader, RunLogViewer } from './RunLogViewer';

class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

if (!globalThis.ResizeObserver) {
  globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;
}

function page(from: 'start' | 'end'): RunLogPage {
  const texts = from === 'start' ? ['first-line', 'second-line'] : ['almost-last', 'last-line'];
  return {
    lines: texts.map((text, i) => ({ offset: i * 20, text })),
    total_bytes: 4 * 1024 * 1024,
    at_start: from === 'start',
    at_end: from === 'end',
    prev_offset: from === 'start' ? null : 20,
    next_offset: from === 'end' ? null : 40,
  };
}

const idleContext = {
  loading: null,
  label: 'Loading logs...',
  canFetchOlder: false,
  canFetchNewer: false,
} as const;

describe('RunLogViewer', () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    getRunLogs.mockReset();
    getRunLogs.mockImplementation(async (_id, query) => {
      if ('from' in query && query.from === 'start') return page('start');
      return page('end');
    });
  });

  it('opens on the end of the log and can jump to the start', async () => {
    const user = userEvent.setup();
    render(
      <I18nextProvider i18n={i18n}>
        <RunLogViewer runId="run-1" />
      </I18nextProvider>,
    );

    expect(await screen.findByText('4.0 MB')).toBeInTheDocument();
    expect(getRunLogs).toHaveBeenCalledWith('run-1', { from: 'end' });

    const start = screen.getByRole('button', { name: 'Start' });
    expect(start).toHaveClass('cursor-pointer');
    expect(screen.getByRole('button', { name: 'End' })).toHaveClass('cursor-pointer');
    await user.click(start);
    expect(getRunLogs).toHaveBeenCalledWith('run-1', { from: 'start' });
  });

  it('keeps a paging gap only on the end that can still load', () => {
    const { rerender } = render(<LogListHeader context={idleContext} />);
    expect(document.querySelector('[data-log-edge="start"]')).toHaveStyle({ height: '0px' });

    rerender(<LogListHeader context={{ ...idleContext, canFetchOlder: true }} />);
    expect(document.querySelector('[data-log-edge="start"]')).toHaveStyle({ height: '40px' });

    rerender(<LogListHeader context={{ ...idleContext, loading: 'older' }} />);
    expect(document.querySelector('[data-log-edge="start"]')).toHaveStyle({ height: '40px' });
    expect(screen.getByRole('status', { name: /loading logs/i })).toBeInTheDocument();

    rerender(<LogListFooter context={idleContext} />);
    expect(document.querySelector('[data-log-edge="end"]')).toHaveStyle({ height: '0px' });

    rerender(<LogListFooter context={{ ...idleContext, canFetchNewer: true }} />);
    expect(document.querySelector('[data-log-edge="end"]')).toHaveStyle({ height: '40px' });
  });

  it('shows a centered spinner while jumping', async () => {
    const user = userEvent.setup();
    let releaseStart!: (page: RunLogPage) => void;
    getRunLogs.mockImplementation(
      (_id, query) =>
        new Promise(resolve => {
          if ('from' in query && query.from === 'start') {
            releaseStart = resolve;
            return;
          }
          resolve(page('end'));
        }),
    );

    render(
      <I18nextProvider i18n={i18n}>
        <RunLogViewer runId="run-1" />
      </I18nextProvider>,
    );

    await screen.findByText('4.0 MB');
    await user.click(screen.getByRole('button', { name: 'Start' }));
    expect(await screen.findByRole('status', { name: /loading logs/i })).toBeInTheDocument();

    releaseStart(page('start'));
    await screen.findByRole('button', { name: 'Start' });
    expect(screen.queryByRole('status', { name: /loading logs/i })).not.toBeInTheDocument();
  });
});
