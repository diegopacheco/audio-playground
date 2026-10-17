import { STEPS } from "../pattern";
import type { Cursor, ScheduledStep, ScheduleWindow } from "../types";

export function stepDuration(bpm: number): number {
  return 60 / bpm / 4;
}

export function swingOffset(step: number, bpm: number, swing: number): number {
  return step % 2 === 1 ? swing * stepDuration(bpm) : 0;
}

export function stepsInWindow(cursor: Cursor, until: number, bpm: number, swing: number): ScheduleWindow {
  const steps: ScheduledStep[] = [];
  let { nextStep, nextBaseTime } = cursor;
  while (nextBaseTime < until) {
    steps.push({ step: nextStep, time: nextBaseTime + swingOffset(nextStep, bpm, swing) });
    nextBaseTime += stepDuration(bpm);
    nextStep = (nextStep + 1) % STEPS;
  }
  return { steps, cursor: { nextStep, nextBaseTime } };
}
