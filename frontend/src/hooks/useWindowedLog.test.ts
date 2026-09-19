import { describe, expect, it } from 'vitest';
import { FIRST_ITEM_INDEX, MAX_LOG_LINES, mergeLogWindow, nextFirstItemIndex } from './useWindowedLog';
import type { RunLogLine, RunLogPage } from '../types/taskRun';

function page(texts: string[], startOffset = 0): RunLogPage {
  let offset = startOffset;
  const lines: RunLogLine[] = texts.map(text => {
    const line = { offset, text };
    offset += text.length + 1;
    return line;
  });
  return {
    lines,
    total_bytes: offset,
    at_start: startOffset === 0,
    at_end: true,
    prev_offset: startOffset === 0 ? null : startOffset,
    next_offset: null,
  };
}

describe('mergeLogWindow', () => {
  it('replaces the buffer on a jump and resets firstItemIndex', () => {
    const merged = mergeLogWindow(page(['old']).lines, page(['a', 'b']), { from: 'end' });
    expect(merged.lines.map(line => line.text)).toEqual(['a', 'b']);
    expect(nextFirstItemIndex(999_000, { from: 'end' }, merged)).toBe(FIRST_ITEM_INDEX);
  });

  it('prepends older lines, drops the tail, and shifts firstItemIndex by the kept prepend', () => {
    const first = page(Array.from({ length: MAX_LOG_LINES }, (_, i) => `l${i}`), 10_000);
    const merged = mergeLogWindow(first.lines, page(['old-a', 'old-b'], 0), { before: 10_000 });
    expect(merged.lines.length).toBe(MAX_LOG_LINES);
    expect(merged.droppedFromEnd).toBe(true);
    expect(merged.lines[0]?.text).toBe('old-a');
    expect(merged.lines[merged.lines.length - 1]?.text).not.toBe(`l${MAX_LOG_LINES - 1}`);
    expect(merged.prepended).toBe(2);
    expect(nextFirstItemIndex(FIRST_ITEM_INDEX, { before: 10_000 }, merged)).toBe(FIRST_ITEM_INDEX - 2);
  });

  it('appends newer lines, drops the head, and advances firstItemIndex', () => {
    const first = page(Array.from({ length: MAX_LOG_LINES }, (_, i) => `l${i}`));
    const merged = mergeLogWindow(first.lines, page(['new-a', 'new-b'], 50_000), { after: 1 });
    expect(merged.lines.length).toBe(MAX_LOG_LINES);
    expect(merged.droppedFromStart).toBe(true);
    expect(merged.droppedStartCount).toBe(2);
    expect(merged.lines[merged.lines.length - 1]?.text).toBe('new-b');
    expect(merged.lines[0]?.text).not.toBe('l0');
    expect(nextFirstItemIndex(FIRST_ITEM_INDEX, { after: 1 }, merged)).toBe(FIRST_ITEM_INDEX + 2);
  });

  it('stays within MAX_LOG_LINES across many pages', () => {
    let current: RunLogLine[] = [];
    for (let i = 0; i < 40; i += 1) {
      const texts = Array.from({ length: 200 }, (_, n) => `batch-${i}-${n}`);
      current = mergeLogWindow(current, page(texts, i * 10_000), { after: i }).lines;
      expect(current.length).toBeLessThanOrEqual(MAX_LOG_LINES);
    }
    expect(current.length).toBe(MAX_LOG_LINES);
  });
});
