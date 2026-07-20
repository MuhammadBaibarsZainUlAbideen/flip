import { Zap } from "lucide-react";
import Link from "next/link";

export default function Footer() {
  return (
    <footer className="border-t border-border bg-white">
      <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6">
        <div className="grid gap-8 md:grid-cols-3">
          <div>
            <Link href="/" className="flex items-center gap-2 font-bold text-xl">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-white">
                <Zap size={16} />
              </div>
              <span className="gradient-text">flip</span>
            </Link>
            <p className="mt-3 text-base leading-relaxed text-muted">
              Free, fast file conversion powered by Rust. Convert between 16+ formats instantly in your browser.
            </p>
          </div>
          <div>
            <h3 className="mb-3 font-semibold text-base">Quick Links</h3>
            <ul className="space-y-2 text-base text-muted">
              <li><Link href="/converter" className="hover:text-foreground transition-colors">Converter</Link></li>
              <li><Link href="/#formats" className="hover:text-foreground transition-colors">Supported Formats</Link></li>
              <li><Link href="https://github.com/MuhammadBaibarsZainUlAbideen/flip" target="_blank" rel="noopener noreferrer" className="hover:text-foreground transition-colors">GitHub</Link></li>
            </ul>
          </div>
          <div>
            <h3 className="mb-3 font-semibold text-base">Categories</h3>
            <ul className="space-y-2 text-base text-muted">
              <li>Documents &amp; PDFs</li>
              <li>Spreadsheets &amp; Data</li>
              <li>Web &amp; Markup</li>
              <li>Presentations</li>
            </ul>
          </div>
        </div>
        <div className="mt-10 flex flex-col items-center justify-between gap-2 border-t border-border pt-6 text-base text-muted sm:flex-row">
          <span>Built with Rust + Next.js. Open source under MIT.</span>
          <span>flip v0.1.0</span>
        </div>
      </div>
    </footer>
  );
}
