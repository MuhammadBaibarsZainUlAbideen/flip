import Link from "next/link";
import { notFound } from "next/navigation";
import { ArrowRight, Zap, Shield, Clock } from "lucide-react";
import { formats, getFormatById } from "@/lib/formats";
import { getConversionMeta } from "@/lib/conversions";
import { ConversionPageSchema, FAQSchema } from "@/components/StructuredData";

export async function generateStaticParams() {
  const pairs: Array<{ slug: string }> = [];
  for (const from of formats) {
    for (const to of formats) {
      if (from.id !== to.id) {
        pairs.push({ slug: `${from.id}-to-${to.id}` });
      }
    }
  }
  return pairs;
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const [fromId, toId] = slug.split("-to-");
  if (!fromId || !toId) return null;

  const meta = getConversionMeta(fromId, toId);
  if (!meta) return null;

  return {
    title: meta.title,
    description: meta.description,
    openGraph: {
      title: meta.title,
      description: meta.description,
      url: `https://flip.engineer/convert/${slug}`,
    },
    alternates: {
      canonical: `https://flip.engineer/convert/${slug}`,
    },
  };
}

export default async function ConversionPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const [fromId, toId] = slug.split("-to-");
  if (!fromId || !toId) notFound();

  const meta = getConversionMeta(fromId, toId);
  if (!meta) notFound();

  const fromFmt = getFormatById(fromId);
  const toFmt = getFormatById(toId);
  if (!fromFmt || !toFmt) notFound();

  const relatedPairs = formats
    .filter((f) => f.id !== fromId)
    .slice(0, 6)
    .map((f) => ({
      from: fromId,
      to: f.id,
      label: `${fromFmt.name} → ${f.name}`,
    }));

  return (
    <>
      <ConversionPageSchema from={fromId} to={toId} fromName={fromFmt.name} toName={toFmt.name} />
      <FAQSchema items={meta.faq} />

      <div className="mx-auto max-w-4xl px-4 py-12 sm:px-6 sm:py-20">
        {/* Hero */}
        <div className="text-center">
          <div className="mb-4 flex items-center justify-center gap-3">
            <div
              className="flex h-10 w-10 items-center justify-center rounded-xl text-sm font-bold text-white"
              style={{ backgroundColor: fromFmt.color }}
            >
              {fromFmt.icon}
            </div>
            <ArrowRight size={20} className="text-muted" />
            <div
              className="flex h-10 w-10 items-center justify-center rounded-xl text-sm font-bold text-white"
              style={{ backgroundColor: toFmt.color }}
            >
              {toFmt.icon}
            </div>
          </div>
          <h1 className="text-4xl font-bold tracking-tight sm:text-5xl">{meta.h1}</h1>
          <p className="mt-4 text-lg text-muted max-w-2xl mx-auto">{meta.description}</p>
          <Link
            href={`/converter?from=${fromId}&to=${toId}`}
            className="mt-8 inline-flex items-center gap-2 rounded-xl bg-primary px-8 py-3.5 text-base font-semibold text-white transition-all hover:bg-primary-light"
          >
            <Zap size={16} />
            Convert Now
          </Link>
        </div>

        {/* Content */}
        <div className="mt-16 grid gap-8 lg:grid-cols-3">
          <div className="lg:col-span-2 space-y-6">
            <div className="rounded-2xl border border-border bg-white p-8 shadow-sm">
              <h2 className="text-xl font-semibold mb-4">How to Convert {fromFmt.name} to {toFmt.name}</h2>
              <ol className="space-y-3 text-muted">
                <li className="flex gap-3">
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-[11px] font-bold text-white">1</span>
                  <span>Click &ldquo;Convert Now&rdquo; and select your {fromFmt.name} file</span>
                </li>
                <li className="flex gap-3">
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-[11px] font-bold text-white">2</span>
                  <span>{toFmt.name} is automatically selected as the output format</span>
                </li>
                <li className="flex gap-3">
                  <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary text-[11px] font-bold text-white">3</span>
                  <span>Download your converted {toFmt.extension} file instantly</span>
                </li>
              </ol>
            </div>

            <div className="rounded-2xl border border-border bg-white p-8 shadow-sm prose prose-gray max-w-none">
              {meta.paragraphs.map((p, i) => (
                <p key={i}>{p}</p>
              ))}
            </div>

            {/* FAQ */}
            <div className="rounded-2xl border border-border bg-white p-8 shadow-sm">
              <h2 className="text-xl font-semibold mb-6">Frequently Asked Questions</h2>
              <div className="space-y-6">
                {meta.faq.map((item, i) => (
                  <div key={i}>
                    <h3 className="font-medium text-foreground">{item.question}</h3>
                    <p className="mt-2 text-muted">{item.answer}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            <div className="rounded-2xl border border-border bg-white p-6 shadow-sm">
              <h3 className="font-semibold mb-4">Why flip?</h3>
              <div className="space-y-4">
                <div className="flex gap-3">
                  <Zap size={18} className="shrink-0 text-primary mt-0.5" />
                  <div>
                    <div className="font-medium text-sm">Blazing Fast</div>
                    <div className="text-sm text-muted">Powered by a Rust engine</div>
                  </div>
                </div>
                <div className="flex gap-3">
                  <Shield size={18} className="shrink-0 text-primary mt-0.5" />
                  <div>
                    <div className="font-medium text-sm">100% Private</div>
                    <div className="text-sm text-muted">Files are never stored</div>
                  </div>
                </div>
                <div className="flex gap-3">
                  <Clock size={18} className="shrink-0 text-primary mt-0.5" />
                  <div>
                    <div className="font-medium text-sm">No Sign-up</div>
                    <div className="text-sm text-muted">Just upload and convert</div>
                  </div>
                </div>
              </div>
            </div>

            <div className="rounded-2xl border border-border bg-white p-6 shadow-sm">
              <h3 className="font-semibold mb-4">Related Conversions</h3>
              <div className="space-y-2">
                {relatedPairs.map((pair) => (
                  <Link
                    key={pair.to}
                    href={`/convert/${pair.from}-to-${pair.to}`}
                    className="block rounded-lg px-3 py-2 text-sm text-muted transition-colors hover:bg-primary/5 hover:text-primary"
                  >
                    {pair.label}
                  </Link>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
