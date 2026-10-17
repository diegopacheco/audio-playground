export const FLASH_SEC = 0.12;

export function lastBeatIndex(beats: number[], time: number): number {
  let lo = 0;
  let hi = beats.length - 1;
  let found = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (beats[mid] <= time) {
      found = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return found;
}

export function isFlashing(beats: number[], time: number, flashSec: number = FLASH_SEC): boolean {
  const index = lastBeatIndex(beats, time);
  return index >= 0 && time - beats[index] < flashSec;
}

export function lagToBpm(lag: number, frameRate: number): number {
  return (60 * frameRate) / lag;
}

export function bpmToLag(bpm: number, frameRate: number): number {
  return (60 * frameRate) / bpm;
}
