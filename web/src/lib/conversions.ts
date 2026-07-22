import { formats } from "./formats";

interface ConversionMeta {
  title: string;
  description: string;
  h1: string;
  paragraphs: string[];
  faq: Array<{ question: string; answer: string }>;
}

export function getConversionMeta(fromId: string, toId: string): ConversionMeta | null {
  const from = formats.find((f) => f.id === fromId || f.extension === fromId);
  const to = formats.find((f) => f.id === toId || f.extension === toId);
  if (!from || !to) return null;

  return {
    title: `Convert ${from.name} to ${to.name} Online — Free | flip`,
    description: `Convert ${from.name} files to ${to.name} format instantly. Free, fast, and private — powered by Rust. No sign-up required.`,
    h1: `Convert ${from.name} to ${to.name}`,
    paragraphs: [
      `Need to convert a ${from.name} file to ${to.name}? flip makes it effortless. Upload your ${from.extension} file, select ${to.name} as the output format, and get your converted file in seconds.`,
      `flip is powered by a blazing-fast Rust engine that processes your files locally — your data never leaves your browser for client-side conversions, and server-side processing is instant.`,
      `No sign-up, no watermarks, no file size limits. Just drag, drop, and convert.`,
    ],
    faq: [
      {
        question: `Is it free to convert ${from.name} to ${to.name}?`,
        answer: `Yes. flip is completely free to use. There are no hidden fees, no watermarks, and no sign-up required.`,
      },
      {
        question: `Is my ${from.name} file safe when converting to ${to.name}?`,
        answer: `Absolutely. flip processes files server-side and never stores them. Your data is deleted immediately after conversion.`,
      },
      {
        question: `What is the maximum file size for ${from.name} to ${to.name} conversion?`,
        answer: `flip supports files up to 50MB. For most ${from.name} files, this is more than enough.`,
      },
    ],
  };
}

export function getAllConversionPairs(): Array<{ from: string; to: string }> {
  const pairs: Array<{ from: string; to: string }> = [];
  for (const from of formats) {
    for (const to of formats) {
      if (from.id !== to.id) {
        pairs.push({ from: from.id, to: to.id });
      }
    }
  }
  return pairs;
}
