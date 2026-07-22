export function WebAppSchema() {
  const schema = {
    "@context": "https://schema.org",
    "@type": "WebApplication",
    name: "flip",
    url: "https://flip.engineer",
    description:
      "Free online file converter. Convert between Markdown, PDF, HTML, CSV, JSON, YAML, DOCX, XLSX, PPTX, LaTeX, SVG and more. Blazing fast, powered by Rust.",
    applicationCategory: "UtilitiesApplication",
    operatingSystem: "All",
    offers: {
      "@type": "Offer",
      price: "0",
      priceCurrency: "USD",
    },
    featureList: [
      "Convert between 20+ file formats",
      "Blazing fast Rust-powered engine",
      "100% private — files never stored",
      "No sign-up required",
      "No watermarks",
      "Free forever",
    ],
    screenshot: "https://flip.engineer/og.png",
    softwareVersion: "0.1.0",
    browserRequirements: "Requires a modern web browser",
    permissions: "none",
  };

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(schema) }}
    />
  );
}

export function ConversionPageSchema({
  from,
  to,
  fromName,
  toName,
}: {
  from: string;
  to: string;
  fromName: string;
  toName: string;
}) {
  const schema = {
    "@context": "https://schema.org",
    "@type": "WebApplication",
    name: `flip — ${fromName} to ${toName} Converter`,
    url: `https://flip.engineer/convert/${from}-to-${to}`,
    description: `Convert ${fromName} to ${toName} instantly. Free, fast, and private — powered by Rust. No sign-up required.`,
    applicationCategory: "UtilitiesApplication",
    operatingSystem: "All",
    offers: {
      "@type": "Offer",
      price: "0",
      priceCurrency: "USD",
    },
  };

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(schema) }}
    />
  );
}

export function FAQSchema({
  items,
}: {
  items: Array<{ question: string; answer: string }>;
}) {
  const schema = {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: items.map((item) => ({
      "@type": "Question",
      name: item.question,
      acceptedAnswer: {
        "@type": "Answer",
        text: item.answer,
      },
    })),
  };

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(schema) }}
    />
  );
}
