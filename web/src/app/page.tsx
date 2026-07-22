import Link from "next/link";
import {
  Zap,
  ArrowRight,
  FileText,
  FileSpreadsheet,
  FileJson,
  Globe,
  Lock,
  Cpu,
  Star,
  Image,
} from "lucide-react";
import { formats, popularConversions, categories } from "@/lib/formats";

function Hero() {
  return (
    <section className="relative overflow-hidden">
      <div className="absolute inset-0 -z-10">
        <div className="absolute left-1/2 top-0 h-[600px] w-[600px] -translate-x-1/2 rounded-full bg-primary/5 blur-3xl" />
        <div className="absolute bottom-0 right-0 h-[400px] w-[400px] rounded-full bg-accent/5 blur-3xl" />
      </div>

      <div className="mx-auto max-w-6xl px-4 pb-20 pt-24 sm:px-6 sm:pt-32">
        <div className="text-center">
          <div className="animate-fade-in mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-white px-4 py-1.5 text-base font-medium text-muted">
            <Star size={12} className="text-amber-500" fill="currentColor" />
            Free &middot; Open Source &middot; No Sign Up
          </div>

          <h1 className="animate-fade-in-delay-1 text-5xl font-bold tracking-tight sm:text-7xl lg:text-8xl">
            Convert files
            <br />
            <span className="gradient-text">instantly</span>
          </h1>

          <p className="animate-fade-in-delay-2 mx-auto mt-6 max-w-2xl text-xl leading-relaxed text-muted sm:text-2xl">
            Flip between Markdown, PDF, HTML, CSV, JSON, YAML, DOCX, XLSX,
            PowerPoint and more. Powered by a blazing-fast Rust engine. 100% free, no watermarks.
          </p>

          <div className="animate-fade-in-delay-3 mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
            <Link
              href="/converter"
              className="animate-pulse-glow flex items-center gap-2 rounded-full bg-primary px-8 py-3.5 text-base font-semibold text-white transition-all hover:bg-primary-light hover:shadow-lg"
            >
              Start Converting
              <ArrowRight size={16} />
            </Link>
            <Link
              href="https://github.com/MuhammadBaibarsZainUlAbideen/flip"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 rounded-full border border-border px-8 py-3.5 text-base font-medium transition-colors hover:border-foreground/20 hover:bg-foreground/5"
            >
              View on GitHub
            </Link>
          </div>
        </div>

        {/* Quick conversion chips */}
        <div className="animate-fade-in-delay-3 mx-auto mt-16 flex max-w-3xl flex-wrap items-center justify-center gap-2">
          {popularConversions.slice(0, 8).map((c) => {
            const fromFmt = formats.find((f) => f.id === c.from);
            const toFmt = formats.find((f) => f.id === c.to);
            return (
              <Link
                key={`${c.from}-${c.to}`}
                href={`/converter?from=${c.from}&to=${c.to}`}
                className="group flex items-center gap-1.5 rounded-full border border-border bg-white px-3 py-1.5 text-base font-medium text-muted transition-all hover:border-primary/30 hover:bg-primary/5 hover:text-primary"
              >
                <span
                  className="rounded px-1"
                  style={{ backgroundColor: (fromFmt?.color || "#666") + "15", color: fromFmt?.color }}
                >
                  {fromFmt?.name}
                </span>
                <ArrowRight size={10} />
                <span
                  className="rounded px-1"
                  style={{ backgroundColor: (toFmt?.color || "#666") + "15", color: toFmt?.color }}
                >
                  {toFmt?.name}
                </span>
              </Link>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function Features() {
  const features = [
    {
      icon: Zap,
      title: "Blazing Fast",
      description: "Rust-powered engine converts files in milliseconds, not seconds.",
      color: "#eab308",
    },
    {
      icon: Lock,
      title: "100% Private",
      description: "Files are processed server-side and deleted immediately. We never store your data.",
      color: "#16a34a",
    },
    {
      icon: Cpu,
      title: "16+ Formats",
      description: "Markdown, PDF, HTML, CSV, JSON, YAML, DOCX, XLSX, PPTX, LaTeX, SVG and more.",
      color: "#6d28d9",
    },
    {
      icon: Globe,
      title: "Works Everywhere",
      description: "Use in your browser, via CLI, or integrate into your own apps. Fully open source.",
      color: "#06b6d4",
    },
  ];

  return (
    <section id="features" className="border-t border-border bg-white py-24">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="text-center">
          <h2 className="text-4xl font-bold tracking-tight sm:text-5xl">
            Why <span className="gradient-text">flip</span>?
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-lg text-muted">
            No sign-ups. No watermarks. No limitations. Just fast, free file conversion.
          </p>
        </div>

        <div className="mt-16 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {features.map((f) => (
            <div
              key={f.title}
              className="group rounded-2xl border border-border bg-background p-6 transition-all hover:border-primary/20 hover:shadow-lg hover:shadow-primary/5"
            >
              <div
                className="mb-4 flex h-10 w-10 items-center justify-center rounded-xl"
                style={{ backgroundColor: f.color + "12", color: f.color }}
              >
                <f.icon size={20} />
              </div>
              <h3 className="font-semibold">{f.title}</h3>
              <p className="mt-2 text-base leading-relaxed text-muted">
                {f.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function FormatGrid() {
  const catIcons: Record<string, React.ReactNode> = {
    document: <FileText size={14} />,
    spreadsheet: <FileSpreadsheet size={14} />,
    presentation: <FileJson size={14} />,
    data: <FileJson size={14} />,
    web: <Globe size={14} />,
    text: <FileText size={14} />,
    image: <Image size={14} />,
  };

  return (
    <section id="formats" className="py-24">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="text-center">
          <h2 className="text-4xl font-bold tracking-tight sm:text-5xl">
            Supported <span className="gradient-text">Formats</span>
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-lg text-muted">
            All formats support bidirectional conversion through our universal document IR.
          </p>
        </div>

        <div className="mt-16 space-y-12">
          {categories.map((cat) => {
            const catFormats = formats.filter((f) => f.category === cat.id);
            if (catFormats.length === 0) return null;
            return (
              <div key={cat.id}>
                <div className="mb-4 flex items-center gap-2 text-base font-medium text-muted">
                  {catIcons[cat.id]}
                  {cat.label}
                </div>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
                  {catFormats.map((f) => (
                    <Link
                      key={f.id}
                      href={`/converter?to=${f.id}`}
                      className="group flex items-center gap-3 rounded-xl border border-border bg-white p-3 transition-all hover:border-primary/20 hover:shadow-md hover:shadow-primary/5"
                    >
                      <div
                        className="flex h-8 w-8 items-center justify-center rounded-lg text-xs font-bold text-white"
                        style={{ backgroundColor: f.color }}
                      >
                        {f.icon}
                      </div>
                      <div>
                        <div className="text-base font-medium">{f.name}</div>
                        <div className="text-base text-muted">.{f.extension}</div>
                      </div>
                    </Link>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function CTA() {
  return (
    <section className="border-t border-border py-24">
      <div className="mx-auto max-w-3xl px-4 text-center sm:px-6">
        <h2 className="text-4xl font-bold tracking-tight sm:text-5xl">
          Ready to <span className="gradient-text">flip</span>?
        </h2>
        <p className="mx-auto mt-4 max-w-lg text-lg text-muted">
          No sign-ups. No limits. No strings attached. Just drop your file and go.
        </p>
        <Link
          href="/converter"
          className="mt-8 inline-flex items-center gap-2 rounded-full bg-primary px-8 py-3.5 text-base font-semibold text-white transition-all hover:bg-primary-light hover:shadow-lg"
        >
          Open Converter
          <ArrowRight size={16} />
        </Link>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <>
      <Hero />
      <Features />
      <FormatGrid />
      <CTA />
    </>
  );
}
