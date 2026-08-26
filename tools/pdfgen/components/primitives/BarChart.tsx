"use client";

import {
  BarChart as RBarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  ResponsiveContainer,
  LabelList
} from "recharts";

type Datum = {
  label: string;
  value: number;
};

type Props = {
  data: Datum[];
  height?: number;
  caption?: string;
};

export function BarChart({ data, height = 220, caption }: Props) {
  return (
    <figure className="no-break my-6">
      <div style={{ width: "100%", height }}>
        <ResponsiveContainer>
          <RBarChart
            data={data}
            margin={{ top: 24, right: 0, bottom: 0, left: 0 }}
            barCategoryGap="35%"
          >
            <CartesianGrid stroke="var(--hairline)" vertical={false} />
            <XAxis
              dataKey="label"
              tickLine={false}
              axisLine={{ stroke: "var(--ink)" }}
              tick={{
                fontFamily: "var(--mono)",
                fontSize: 8,
                fill: "var(--mute)"
              }}
            />
            <YAxis
              tickLine={false}
              axisLine={false}
              tick={{
                fontFamily: "var(--mono)",
                fontSize: 8,
                fill: "var(--mute)"
              }}
              width={28}
            />
            <Bar
              dataKey="value"
              fill="var(--ink)"
              isAnimationActive={false}
            >
              <LabelList
                dataKey="value"
                position="top"
                style={{
                  fontFamily: "var(--mono)",
                  fontSize: 8,
                  fill: "var(--ink)"
                }}
              />
            </Bar>
          </RBarChart>
        </ResponsiveContainer>
      </div>
      {caption && (
        <figcaption className="caption mt-1" style={{ color: "var(--mute)" }}>
          {caption}
        </figcaption>
      )}
    </figure>
  );
}
