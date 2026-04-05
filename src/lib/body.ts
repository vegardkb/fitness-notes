import { todayStr } from "./date";

export type DayMeasurement = {
  date: string;
  measurements: Measurement[];
};

export type Measurement = {
  value: number;
  metric: Metric;
  date: string;
  id: number | null;
};

export type Metric = {
  name: string;
  unit: string;
  id: number;
};

export function bodyHrefs(fromDate: string) {
  const from = fromDate ? `?from=${fromDate}` : "";
  return {
    feedHref: fromDate ? `/?date=${fromDate}` : "/",
    logHref: fromDate ? `/body/${fromDate}` : `/body/${todayStr()}`,
    historyHref: `/body/history${from}`,
    graphHref: `/body/graph${from}`,
    prsHref: `/body/prs${from}`,
  };
}
