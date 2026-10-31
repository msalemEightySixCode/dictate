// Local mod: the raw meter barely moves above the lowest frequency buckets, so
// the bars follow loudness above the room's own noise instead.
const BAR_SHAPE = [0.45, 0.7, 0.9, 1, 0.85, 1, 0.9, 0.7, 0.45];
const VOICE_RANGE = 0.2; // loudness above the noise floor that fills the bars
const NOISE_MARGIN = 0.03; // keeps room noise from twitching the bars
const FLOOR_RISE = 0.0005; // per update; slow, so long speech stays visible

/** Returns the bar heights (0-1) and the updated noise floor. */
export function barLevels(buckets: number[], floor: number) {
  const loudness = Math.max(0, ...buckets);
  const nextFloor = Math.min(loudness, floor + FLOOR_RISE);
  const voice = Math.min(
    1,
    Math.max(0, loudness - nextFloor - NOISE_MARGIN) / VOICE_RANGE,
  );
  const bars = BAR_SHAPE.map((shape, i) => {
    const detail = loudness > 0 ? (buckets[i] ?? 0) / loudness : 0;
    return voice * shape * (0.6 + 0.4 * detail);
  });
  return { bars, floor: nextFloor };
}
