function normalizeLabel(label: string): string {
  const normalized = label.trim();
  if (!normalized) throw new Error("invalid_volume_label");
  return normalized;
}

export function makeVolumeSortKey(label: string): string {
  const normalized = normalizeLabel(label);
  const numericMatch = /^(\d+)(?:\.(\d{1,3}))?$/.exec(normalized);

  if (numericMatch) {
    const [, integer, decimal = ""] = numericMatch;
    return `0:${integer.padStart(6, "0")}.${decimal.padEnd(3, "0")}`;
  }

  if (normalized === "上" || normalized === "下") return `1:${normalized}`;
  if (normalized === "全一冊") return "2:全一冊";
  return `9:${normalized}`;
}

export function compareVolumeLabels(left: string, right: string): number {
  const leftKey = makeVolumeSortKey(left);
  const rightKey = makeVolumeSortKey(right);

  if (leftKey < rightKey) return -1;
  if (leftKey > rightKey) return 1;
  return 0;
}
