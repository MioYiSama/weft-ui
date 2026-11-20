import type { ColorScheme, Insets, Screen } from "./index.d.ts";

export interface EnvironmentData {
  colorScheme?: ColorScheme;
  screen?: Partial<Screen>;
  safeArea?: Partial<Insets>;
  orientation?: "portrait" | "landscape";
}

type Snapshot = Required<Omit<EnvironmentData, "screen" | "safeArea">> & { screen: Screen; safeArea: Insets };

function snapshot(data: EnvironmentData): Snapshot {
  const { width = 0, height = 0, scale = 1 } = data.screen ?? {};
  const { top = 0, bottom = 0, start = 0, end = 0 } = data.safeArea ?? {};
  return {
    colorScheme: data.colorScheme ?? "light",
    screen: Object.freeze({ width, height, scale }),
    safeArea: Object.freeze({ top, bottom, start, end }),
    orientation: data.orientation ?? (width > height ? "landscape" : "portrait"),
  };
}

function equal(a: Snapshot, b: Snapshot): boolean {
  return a.colorScheme === b.colorScheme && a.orientation === b.orientation
    && a.screen.width === b.screen.width && a.screen.height === b.screen.height && a.screen.scale === b.screen.scale
    && a.safeArea.top === b.safeArea.top && a.safeArea.bottom === b.safeArea.bottom
    && a.safeArea.start === b.safeArea.start && a.safeArea.end === b.safeArea.end;
}

/** Lazily reads one snapshot per environment and notifies listeners of changes. */
export class Environment {
  #read: () => EnvironmentData;
  #observe?: (notify: () => void) => void;
  #cached?: Snapshot;
  #listeners = new Set<() => void>();

  constructor(read: () => EnvironmentData, observe?: (notify: () => void) => void) {
    this.#read = read;
    this.#observe = observe;
  }

  #current(): Snapshot {
    if (this.#observe) {
      const observe = this.#observe;
      this.#observe = undefined;
      observe(() => this.notify());
    }
    return this.#cached ??= snapshot(this.#read());
  }

  get colorScheme(): ColorScheme { return this.#current().colorScheme; }
  get screen(): Screen { return this.#current().screen; }
  get safeArea(): Insets { return this.#current().safeArea; }
  get orientation(): "portrait" | "landscape" { return this.#current().orientation; }

  onChange(listener: () => void): () => void {
    this.#current();
    const subscription = () => listener();
    this.#listeners.add(subscription);
    return () => { this.#listeners.delete(subscription); };
  }

  notify(): void {
    const previous = this.#cached;
    this.#cached = undefined;
    if (this.#listeners.size === 0) return;
    if (previous && equal(previous, this.#current())) {
      this.#cached = previous;
      return;
    }
    for (const listener of [...this.#listeners]) {
      if (!this.#listeners.has(listener)) continue;
      try {
        listener();
      } catch (error) {
        console.error(error);
      }
    }
  }
}
