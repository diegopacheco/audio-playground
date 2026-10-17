export interface WavHeader {
  encoding: string;
  format_tag: number;
  sample_rate: number;
  channels: number;
  bits_per_sample: number;
  byte_rate: number;
  block_align: number;
  frames: number;
}

export interface Chunk {
  id: string;
  offset: number;
  size: number;
}

export interface Report {
  header: WavHeader;
  chunks: Chunk[];
  duration: number;
  peak: number;
  rms: number;
  ascii: string;
  envelope: number[];
}

export interface AsciiSize {
  width: number;
  height: number;
}

export interface Point {
  x: number;
  y: number;
}
