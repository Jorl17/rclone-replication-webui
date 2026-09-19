import type { LogLoad } from '../../hooks/useWindowedLog';

export const LOG_PANE_PX = 288;
export const LOG_EDGE_PX = 40;

export function logEdgePx(open: boolean): number {
  return open ? LOG_EDGE_PX : 0;
}

export type InertiaLock = {
  direction: 'older' | 'newer';
  lastAbs: number;
  sawWheel: boolean;
};

export function beginInertiaLock(direction: 'older' | 'newer'): InertiaLock {
  return { direction, lastAbs: 0, sawWheel: false };
}

function noteInertiaWheel(lock: InertiaLock, deltaY: number): InertiaLock {
  return { direction: lock.direction, lastAbs: Math.abs(deltaY), sawWheel: true };
}

function continueInertiaLock(lock: InertiaLock | null, deltaY: number): InertiaLock | null {
  if (!lock?.sawWheel) return null;
  const sameSign = lock.direction === 'older' ? deltaY < 0 : deltaY > 0;
  if (!sameSign) return null;
  const abs = Math.abs(deltaY);
  if (abs > lock.lastAbs + 1) return null;
  return { direction: lock.direction, lastAbs: abs, sawWheel: true };
}

export function decideLogWheel(
  lock: InertiaLock | null,
  input: {
    deltaY: number;
    scrollTop: number;
    scrollHeight: number;
    clientHeight: number;
    loading: LogLoad;
    canFetchOlder: boolean;
    canFetchNewer: boolean;
  },
): { prevent: boolean; lock: InertiaLock | null } {
  if (input.loading === 'older' || input.loading === 'newer') {
    return {
      prevent: true,
      lock: lock ? noteInertiaWheel(lock, input.deltaY) : beginInertiaLock(input.loading),
    };
  }

  const continued = continueInertiaLock(lock, input.deltaY);
  if (continued) return { prevent: true, lock: continued };

  const atTop = input.deltaY < 0 && input.scrollTop <= 0;
  const atBottom = input.deltaY > 0 && input.scrollTop + input.clientHeight >= input.scrollHeight - 1;
  if (atTop && input.canFetchOlder) return { prevent: true, lock: null };
  if (atBottom && input.canFetchNewer) return { prevent: true, lock: null };
  return { prevent: false, lock: null };
}

export function killScrollMomentum(el: HTMLElement) {
  const top = el.scrollTop;
  const overflowY = el.style.overflowY;
  el.style.overflowY = 'hidden';
  void el.offsetHeight;
  el.style.overflowY = overflowY;
  if (el.scrollTop !== top) el.scrollTop = top;
}
