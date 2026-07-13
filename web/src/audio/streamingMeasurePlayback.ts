import type { PlaybackClock } from '../types'

const CHANNELS = 2

/**
 * A `PlaybackClock` driven by an `AudioContext`'s wall-clock time, standing in
 * for an `<audio>` element's `currentTime`/`paused` so it can be dropped into
 * `usePlayhead` (see `Preview.tsx`) without changing that hook.
 */
export class MeasureAudioStreamClock extends EventTarget implements PlaybackClock {
  private audioContext: AudioContext | null = null
  private startContextTime = 0
  private frozenCurrentTime = 0
  private isPaused = true

  get currentTime(): number {
    if (!this.audioContext || this.isPaused) return this.frozenCurrentTime
    return this.audioContext.currentTime - this.startContextTime
  }

  get paused(): boolean {
    return this.isPaused
  }

  /** @internal called by `MeasureAudioStreamPlayer` once playback begins. */
  _begin(audioContext: AudioContext): void {
    this.audioContext = audioContext
    this.startContextTime = audioContext.currentTime
    this.isPaused = false
    this.dispatchEvent(new Event('play'))
  }

  /** @internal called by `MeasureAudioStreamPlayer` when playback is stopped early. */
  _pause(): void {
    if (this.isPaused) return
    this.frozenCurrentTime = this.currentTime
    this.isPaused = true
    this.dispatchEvent(new Event('pause'))
  }

  /** @internal called by `MeasureAudioStreamPlayer` when the final chunk finishes. */
  _end(): void {
    this.frozenCurrentTime = this.currentTime
    this.isPaused = true
    this.dispatchEvent(new Event('ended'))
  }
}

function deinterleave(
  interleaved: Float32Array,
  channels: number,
): Float32Array<ArrayBuffer>[] {
  const frameCount = interleaved.length / channels
  return Array.from({ length: channels }, (_, channel) => {
    const channelData = new Float32Array(frameCount)
    for (let i = 0; i < frameCount; i++) {
      channelData[i] = interleaved[i * channels + channel] ?? 0
    }
    return channelData
  })
}

/**
 * Schedules streamed per-measure PCM chunks onto a Web Audio `AudioContext`
 * as they arrive, so playback can start on the first chunk instead of
 * waiting for the whole measure range to finish synthesizing.
 */
export class MeasureAudioStreamPlayer {
  private audioContext: AudioContext | null = null
  private readonly clock = new MeasureAudioStreamClock()
  private readonly sources: AudioBufferSourceNode[] = []
  private nextStartTime = 0
  private playbackStartTime = 0
  private readonly measureTimes: number[] = [0]
  private stopped = false

  /**
   * Lazily creates the underlying `AudioContext`. Must be called from a
   * user-gesture handler (e.g. a click) to satisfy autoplay policy.
   */
  start(): void {
    if (this.audioContext) return
    const audioContext = new AudioContext()
    this.audioContext = audioContext
    this.playbackStartTime = audioContext.currentTime
    this.nextStartTime = this.playbackStartTime
    this.clock._begin(audioContext)
  }

  getClock(): MeasureAudioStreamClock {
    return this.clock
  }

  getMeasureTimes(): number[] {
    return this.measureTimes
  }

  pushChunk(
    measureIndex: number,
    pcm: ArrayBuffer,
    sampleRate: number,
    isFinal: boolean,
  ): void {
    const audioContext = this.audioContext
    if (this.stopped || !audioContext) return

    const interleaved = new Float32Array(pcm)
    const frameCount = interleaved.length / CHANNELS
    const buffer = audioContext.createBuffer(CHANNELS, frameCount, sampleRate)
    const channelBuffers = deinterleave(interleaved, CHANNELS)
    channelBuffers.forEach((channelData, channel) => {
      buffer.copyToChannel(channelData, channel)
    })

    const source = audioContext.createBufferSource()
    source.buffer = buffer
    source.connect(audioContext.destination)
    source.start(this.nextStartTime)
    this.sources.push(source)

    this.nextStartTime += buffer.duration
    this.measureTimes[measureIndex + 1] =
      this.nextStartTime - this.playbackStartTime

    if (isFinal) {
      source.addEventListener('ended', () => {
        if (!this.stopped) this.clock._end()
      })
    }
  }

  stop(): void {
    if (this.stopped) return
    this.stopped = true
    for (const source of this.sources) {
      try {
        source.stop()
      } catch {
        // Already stopped/ended — nothing to do.
      }
      source.disconnect()
    }
    this.sources.length = 0
    const audioContext = this.audioContext
    this.audioContext = null
    if (audioContext) {
      audioContext.close().catch(() => {})
    }
    this.clock._pause()
  }
}
