import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "flip — Free Online File Converter",
  description:
    "Convert files between Markdown, PDF, HTML, CSV, JSON, YAML, LaTeX, DOCX, XLSX, PPTX and more. Free, fast, and private — powered by Rust.",
  keywords: [
    "file converter",
    "online converter",
    "markdown to pdf",
    "csv to json",
    "convert documents",
    "free converter",
    "rust converter",
  ],
  openGraph: {
    title: "flip — Free Online File Converter",
    description:
      "Convert files between 16+ formats instantly. Markdown, PDF, HTML, CSV, JSON, YAML, DOCX, XLSX — all free.",
    url: "https://flip.dev",
    siteName: "flip",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "flip — Free Online File Converter",
    description:
      "Convert files between 16+ formats instantly. Free, fast, and private.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="flex min-h-full flex-col">
        <Header />
        <main className="flex-1">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
