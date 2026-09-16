import type { RefreshActiveHours, RefreshHoursPreset } from "./types";

export const REFRESH_HOURS_PRESETS: Record<
  Exclude<RefreshHoursPreset, "custom">,
  { start: string; end: string; label: string }
> = {
  business: { start: "08:00", end: "18:00", label: "Business (8am–6pm)" },
  personal: { start: "06:00", end: "23:00", label: "Personal (6am–11pm)" },
};

export const DEFAULT_REFRESH_ACTIVE_HOURS: RefreshActiveHours = {
  enabled: false,
  preset: "business",
  start: REFRESH_HOURS_PRESETS.business.start,
  end: REFRESH_HOURS_PRESETS.business.end,
};

export function parseClock(value: string): number | null {
  const match = value.trim().match(/^(\d{1,2}):(\d{2})(?::\d{2}(?:\.\d+)?)?$/);
  if (!match) {
    return null;
  }
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour > 23 || minute > 59) {
    return null;
  }
  return hour * 60 + minute;
}

export function formatClockValue(minutes: number): string {
  const hour = Math.floor(minutes / 60) % 24;
  const minute = minutes % 60;
  return `${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}`;
}

export function formatClockLabel(value: string): string {
  const minutes = parseClock(value);
  if (minutes == null) {
    return value;
  }
  const date = new Date();
  date.setHours(Math.floor(minutes / 60), minutes % 60, 0, 0);
  return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}

export function resolvedRefreshHours(hours: RefreshActiveHours): { start: string; end: string } {
  if (hours.preset === "business" || hours.preset === "personal") {
    return REFRESH_HOURS_PRESETS[hours.preset];
  }
  return {
    start: parseClock(hours.start) == null ? REFRESH_HOURS_PRESETS.business.start : hours.start,
    end: parseClock(hours.end) == null ? REFRESH_HOURS_PRESETS.business.end : hours.end,
  };
}

export function normalizeRefreshActiveHours(
  hours?: Partial<RefreshActiveHours> | null,
): RefreshActiveHours {
  const preset: RefreshHoursPreset =
    hours?.preset === "personal" || hours?.preset === "custom" ? hours.preset : "business";
  const customStart = hours?.start ? parseClock(hours.start) : null;
  const customEnd = hours?.end ? parseClock(hours.end) : null;
  const resolved =
    preset === "custom"
      ? {
          start: customStart != null ? formatClockValue(customStart) : REFRESH_HOURS_PRESETS.business.start,
          end: customEnd != null ? formatClockValue(customEnd) : REFRESH_HOURS_PRESETS.business.end,
        }
      : REFRESH_HOURS_PRESETS[preset];
  return {
    enabled: Boolean(hours?.enabled),
    preset,
    start: resolved.start,
    end: resolved.end,
  };
}

export function isWithinActiveHours(now: Date, hours: RefreshActiveHours): boolean {
  if (!hours.enabled) {
    return true;
  }
  const { start, end } = resolvedRefreshHours(hours);
  const startMinutes = parseClock(start);
  const endMinutes = parseClock(end);
  if (startMinutes == null || endMinutes == null || startMinutes === endMinutes) {
    return true;
  }
  const current = now.getHours() * 60 + now.getMinutes();
  if (startMinutes < endMinutes) {
    return current >= startMinutes && current < endMinutes;
  }
  return current >= startMinutes || current < endMinutes;
}

export function nextActiveHoursStart(now: Date, hours: RefreshActiveHours): Date {
  if (isWithinActiveHours(now, hours)) {
    return now;
  }
  const { start } = resolvedRefreshHours(hours);
  const startMinutes = parseClock(start) ?? 8 * 60;
  const current = now.getHours() * 60 + now.getMinutes();
  const next = new Date(now);
  next.setSeconds(0, 0);
  next.setHours(Math.floor(startMinutes / 60), startMinutes % 60, 0, 0);
  if (current >= startMinutes) {
    next.setDate(next.getDate() + 1);
  }
  return next;
}
