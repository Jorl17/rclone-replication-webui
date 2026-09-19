import { useCallback, useEffect, useRef } from 'react';
import type { LogLoad } from '../../hooks/useWindowedLog';
import { beginInertiaLock, decideLogWheel, killScrollMomentum, type InertiaLock } from './logScroll';

export function useLogScroller(opts: {
  loading: LogLoad;
  canFetchOlder: boolean;
  canFetchNewer: boolean;
}) {
  const state = useRef({
    loading: opts.loading,
    canFetchOlder: opts.canFetchOlder,
    canFetchNewer: opts.canFetchNewer,
    lock: null as InertiaLock | null,
    paging: false,
    el: null as HTMLElement | null,
    detach: null as (() => void) | null,
  });
  state.current.loading = opts.loading;
  state.current.canFetchOlder = opts.canFetchOlder;
  state.current.canFetchNewer = opts.canFetchNewer;

  useEffect(() => {
    if (opts.loading === 'older' || opts.loading === 'newer') {
      state.current.paging = true;
      state.current.lock = beginInertiaLock(opts.loading);
      return;
    }
    if (opts.loading === 'jump') {
      state.current.paging = false;
      state.current.lock = null;
      return;
    }
    if (!state.current.paging) return;
    state.current.paging = false;
    if (state.current.el && state.current.lock?.sawWheel) killScrollMomentum(state.current.el);
    if (!state.current.lock?.sawWheel) state.current.lock = null;
  }, [opts.loading]);

  useEffect(() => () => state.current.detach?.(), []);

  return useCallback((el: HTMLElement | Window | null) => {
    state.current.detach?.();
    state.current.detach = null;
    const node = el && 'style' in el ? el : null;
    state.current.el = node;
    if (!node) return;

    node.style.overscrollBehavior = 'none';
    const onWheel = (event: WheelEvent) => {
      const next = decideLogWheel(state.current.lock, {
        deltaY: event.deltaY,
        scrollTop: node.scrollTop,
        scrollHeight: node.scrollHeight,
        clientHeight: node.clientHeight,
        loading: state.current.loading,
        canFetchOlder: state.current.canFetchOlder,
        canFetchNewer: state.current.canFetchNewer,
      });
      state.current.lock = next.lock;
      if (next.prevent) event.preventDefault();
    };
    node.addEventListener('wheel', onWheel, { passive: false });
    state.current.detach = () => node.removeEventListener('wheel', onWheel);
  }, []);
}
