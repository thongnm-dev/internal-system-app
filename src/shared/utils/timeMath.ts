import type { MinuteTotals } from "@/_/types/analysis";

export function emptyTotals(): MinuteTotals {
  return {
    regular_minutes: 0,
    normal_overtime_minutes: 0,
    legal_holiday_overtime_minutes: 0,
    legal_public_holiday_overtime_minutes: 0,
    late_night_overtime_minutes: 0,
  };
}

export function addTotals(target: MinuteTotals, source: MinuteTotals): void {
  target.regular_minutes += source.regular_minutes;
  target.normal_overtime_minutes += source.normal_overtime_minutes;
  target.legal_holiday_overtime_minutes += source.legal_holiday_overtime_minutes;
  target.legal_public_holiday_overtime_minutes += source.legal_public_holiday_overtime_minutes;
  target.late_night_overtime_minutes += source.late_night_overtime_minutes;
}

export function totalMinutes(totals: MinuteTotals): number {
  return (
    totals.regular_minutes +
    totals.normal_overtime_minutes +
    totals.legal_holiday_overtime_minutes +
    totals.legal_public_holiday_overtime_minutes +
    totals.late_night_overtime_minutes
  );
}

export function overtimeMinutes(totals: MinuteTotals): number {
  return totalMinutes(totals) - totals.regular_minutes;
}

export function formatHours(minutes: number): string {
  const value = formatHourValue(minutes);
  return value === "-" ? value : `${value}h`;
}

export function formatHourValue(minutes: number): string {
  if (minutes === 0) {
    return "-";
  }

  const roundedHours = Math.round((minutes / 60) * 100) / 100;
  return roundedHours.toLocaleString("en-US", {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
  });
}
