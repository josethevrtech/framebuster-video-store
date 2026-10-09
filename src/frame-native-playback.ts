export interface FramePlaybackState {
  id: string;
  position: number;
  paused: boolean;
  running: boolean;
  ended: boolean;
  error: boolean;
}

export class FrameNativePlayback {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private active = true;
  private constructor(private id: string, private progress: (state: FramePlaybackState) => void,
    private finished: (state: FramePlaybackState) => void) {}

  static async available(): Promise<boolean> {
    if (typeof location === 'undefined' || new URLSearchParams(location.search).get('frame') !== '1') return false;
    try {
      const response = await fetch('/__frame/native', { signal: AbortSignal.timeout(3000) });
      return response.ok && (await response.json()).available === true;
    } catch { return false; }
  }

  static async start(source: string, token: string, startSeconds: number,
    progress: (state: FramePlaybackState) => void, finished: (state: FramePlaybackState) => void) {
    const response = await fetch('/__frame/native/play', { method: 'POST',
      headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ source, token, startSeconds }),
      signal: AbortSignal.timeout(15000) });
    if (!response.ok) throw new Error('The headset cinema could not start. Close any movie already running and retry.');
    const state: FramePlaybackState = await response.json();
    const session = new FrameNativePlayback(state.id, progress, finished);
    session.schedule();
    return session;
  }

  stop(): void {
    this.active = false;
    if (this.timer) clearTimeout(this.timer);
    void fetch('/__frame/native/stop', { method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: this.id }), signal: AbortSignal.timeout(3000) }).catch(() => {});
  }

  private schedule(): void {
    this.timer = setTimeout(() => { void this.poll(); }, 1000);
  }

  private async poll(): Promise<void> {
    if (!this.active) return;
    try {
      const response = await fetch('/__frame/native', { signal: AbortSignal.timeout(5000) });
      if (!response.ok) throw new Error('Playback status unavailable');
      const { state }: { state: FramePlaybackState | null } = await response.json();
      if (!this.active) return;
      if (state?.id === this.id) {
        this.progress(state);
        if (!state.running) { this.active = false; this.finished(state); return; }
      }
    } catch { /* Keep playback running through a brief connection interruption. */ }
    if (this.active) this.schedule();
  }
}
