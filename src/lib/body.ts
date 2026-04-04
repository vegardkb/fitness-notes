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
