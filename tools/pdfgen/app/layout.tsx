import type { Metadata } from "next";
import "./globals.css";
import "../themes/agentik.css";

export const metadata: Metadata = {
  title: "Agentik PDF",
  description: "Unified PDF generator"
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
