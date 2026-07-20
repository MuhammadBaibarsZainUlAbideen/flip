"use client";

import Link from "next/link";
import { useState } from "react";
import { Menu, X, Zap } from "lucide-react";

export default function Header() {
  const [open, setOpen] = useState(false);

  return (
    <header className="sticky top-0 z-50 border-b border-border bg-white/80 backdrop-blur-lg">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
        <Link href="/" className="flex items-center gap-2 font-bold text-xl">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-white">
            <Zap size={16} />
          </div>
          <span className="gradient-text">flip</span>
        </Link>

        <nav className="hidden items-center gap-8 md:flex">
          <Link href="/#features" className="text-base text-muted transition-colors hover:text-foreground">
            Features
          </Link>
          <Link href="/#formats" className="text-base text-muted transition-colors hover:text-foreground">
            Formats
          </Link>
          <Link href="/converter" className="rounded-full bg-primary px-4 py-2 text-base font-medium text-white transition-colors hover:bg-primary-light">
            Start Converting
          </Link>
        </nav>

        <button onClick={() => setOpen(!open)} className="flex items-center justify-center md:hidden">
          {open ? <X size={20} /> : <Menu size={20} />}
        </button>
      </div>

      {open && (
        <div className="border-t border-border bg-white px-4 py-4 md:hidden">
          <nav className="flex flex-col gap-3">
            <Link href="/#features" onClick={() => setOpen(false)} className="text-base text-muted hover:text-foreground">
              Features
            </Link>
            <Link href="/#formats" onClick={() => setOpen(false)} className="text-base text-muted hover:text-foreground">
              Formats
            </Link>
            <Link href="/converter" onClick={() => setOpen(false)} className="rounded-full bg-primary px-4 py-2 text-center text-base font-medium text-white">
              Start Converting
            </Link>
          </nav>
        </div>
      )}
    </header>
  );
}
