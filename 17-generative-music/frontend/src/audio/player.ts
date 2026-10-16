import type { ScheduledTone, Song } from "../types";
import { buildSchedule, songSeconds } from "./schedule";

const LEAD_ATTACK = 0.01;
const PAD_ATTACK = 0.08;
const RELEASE = 0.12;
const START_DELAY = 0.1;

export interface Playback {
  startedAt: number;
  context: AudioContext;
  stop: () => void;
}

function scheduleTone(context: AudioContext, destination: AudioNode, tone: ScheduledTone, offset: number): OscillatorNode {
  const oscillator = context.createOscillator();
  const gain = context.createGain();
  const start = offset + tone.time;
  const end = start + tone.duration;
  const attack = tone.voice === "lead" ? LEAD_ATTACK : PAD_ATTACK;
  oscillator.type = tone.voice === "lead" ? "triangle" : "sine";
  oscillator.frequency.setValueAtTime(tone.frequency, start);
  gain.gain.setValueAtTime(0, start);
  gain.gain.linearRampToValueAtTime(tone.gain, start + attack);
  gain.gain.setValueAtTime(tone.gain, Math.max(start + attack, end));
  gain.gain.linearRampToValueAtTime(0, end + RELEASE);
  oscillator.connect(gain).connect(destination);
  oscillator.start(start);
  oscillator.stop(end + RELEASE + 0.01);
  return oscillator;
}

export function play(song: Song, onEnd: () => void): Playback {
  const context = new AudioContext();
  const master = context.createGain();
  master.gain.value = 0.8;
  master.connect(context.destination);
  const offset = context.currentTime + START_DELAY;
  buildSchedule(song).forEach((tone) => scheduleTone(context, master, tone, offset));
  const timer = window.setTimeout(onEnd, (songSeconds(song) + START_DELAY + RELEASE) * 1000);
  return {
    startedAt: offset,
    context,
    stop: () => {
      window.clearTimeout(timer);
      void context.close();
    },
  };
}
