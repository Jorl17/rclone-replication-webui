import { describe, expect, it } from 'vitest';
import { beginInertiaLock, decideLogWheel, killScrollMomentum, logEdgePx } from './logScroll';

const mid = {
  scrollTop: 80,
  scrollHeight: 4000,
  clientHeight: 288,
  canFetchOlder: true,
  canFetchNewer: true,
};

describe('logScroll', () => {
  it('opens the edge slot only when that end can still page', () => {
    expect(logEdgePx(false)).toBe(0);
    expect(logEdgePx(true)).toBe(40);
  });

  it('holds the wheel only while a page is in flight or that edge can still fetch', () => {
    expect(
      decideLogWheel(null, { ...mid, deltaY: -20, scrollTop: 0, loading: null, canFetchOlder: true }).prevent,
    ).toBe(true);
    expect(
      decideLogWheel(null, { ...mid, deltaY: -20, scrollTop: 0, loading: null, canFetchOlder: false }).prevent,
    ).toBe(false);
    expect(decideLogWheel(null, { ...mid, deltaY: -20, loading: null }).prevent).toBe(false);
    expect(decideLogWheel(null, { ...mid, deltaY: -20, loading: 'older' }).prevent).toBe(true);
    expect(
      decideLogWheel(null, {
        ...mid,
        deltaY: 20,
        scrollTop: 3712,
        loading: null,
        canFetchOlder: false,
        canFetchNewer: true,
      }).prevent,
    ).toBe(true);
    expect(
      decideLogWheel(null, {
        ...mid,
        deltaY: 20,
        scrollTop: 3712,
        loading: null,
        canFetchOlder: false,
        canFetchNewer: false,
      }).prevent,
    ).toBe(false);
  });

  it('swallows the decaying tail of a paging flick and lets a new flick through', () => {
    let lock = beginInertiaLock('older');
    lock = decideLogWheel(lock, { ...mid, deltaY: -80, loading: 'older' }).lock;
    const tail = decideLogWheel(lock, { ...mid, deltaY: -40, loading: null });
    expect(tail.prevent).toBe(true);
    expect(tail.lock?.lastAbs).toBe(40);

    const weaker = decideLogWheel(tail.lock, { ...mid, deltaY: -20, loading: null });
    expect(weaker.prevent).toBe(true);

    const opposite = decideLogWheel(lock, { ...mid, deltaY: 30, loading: null });
    expect(opposite.prevent).toBe(false);
    expect(opposite.lock).toBeNull();

    const stronger = decideLogWheel(lock, { ...mid, deltaY: -120, loading: null });
    expect(stronger.prevent).toBe(false);

    expect(decideLogWheel(beginInertiaLock('older'), { ...mid, deltaY: -20, loading: null }).prevent).toBe(false);
  });

  it('clears native momentum without moving the scroller', () => {
    const el = document.createElement('div');
    el.style.overflowY = 'auto';
    Object.defineProperty(el, 'scrollTop', { value: 64, writable: true });
    killScrollMomentum(el);
    expect(el.style.overflowY).toBe('auto');
    expect(el.scrollTop).toBe(64);
  });
});
