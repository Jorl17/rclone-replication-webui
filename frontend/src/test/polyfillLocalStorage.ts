const memory = new Map<string, string>();
const storage: Storage = {
  get length() {
    return memory.size;
  },
  clear() {
    memory.clear();
  },
  getItem(key) {
    return memory.has(key) ? memory.get(key)! : null;
  },
  key(index) {
    return [...memory.keys()][index] ?? null;
  },
  removeItem(key) {
    memory.delete(key);
  },
  setItem(key, value) {
    memory.set(key, value);
  },
};

Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: storage });
if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'localStorage', { configurable: true, value: storage });
}
