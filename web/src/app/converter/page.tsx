"use client";

import { Suspense, useCallback, useState, useRef, useEffect } from "react";
import { useSearchParams } from "next/navigation";
import {
  Upload,
  ArrowRight,
  ArrowDown,
  FileText,
  Download,
  Loader2,
  Check,
  AlertCircle,
  X,
  Zap,
} from "lucide-react";
import { formats, getFormatById, formatFileSize } from "@/lib/formats";

type Status = "idle" | "converting" | "done" | "error";

function ConverterInner() {
  const searchParams = useSearchParams();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [file, setFile] = useState<File | null>(null);
  const [fromFormat, setFromFormat] = useState(searchParams.get("from") || "");
  const [toFormat, setToFormat] = useState(searchParams.get("to") || "");
  const [status, setStatus] = useState<Status>("idle");
  const [error, setError] = useState("");
  const [dragOver, setDragOver] = useState(false);
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [downloadName, setDownloadName] = useState("");

  // Auto-detect format from filename
  useEffect(() => {
    if (file && !fromFormat) {
      const ext = file.name.split(".").pop()?.toLowerCase();
      if (ext) {
        const fmt = formats.find((f) => f.extension === ext);
        if (fmt) setFromFormat(fmt.id);
      }
    }
  }, [file, fromFormat]);

  const handleFile = useCallback((f: File) => {
    setFile(f);
    setStatus("idle");
    setError("");
    setDownloadUrl(null);
    setDownloadName("");
    // Reset from format detection
    const ext = f.name.split(".").pop()?.toLowerCase();
    if (ext) {
      const fmt = formats.find((f2) => f2.extension === ext);
      if (fmt) setFromFormat(fmt.id);
    }
  }, []);

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const f = e.dataTransfer.files[0];
      if (f) handleFile(f);
    },
    [handleFile]
  );

  const onFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const f = e.target.files?.[0];
      if (f) handleFile(f);
    },
    [handleFile]
  );

  const handleConvert = async () => {
    if (!file || !toFormat) return;

    setStatus("converting");
    setError("");

    try {
      const formData = new FormData();
      formData.append("file", file);
      formData.append("to", toFormat);
      if (fromFormat) formData.append("from", fromFormat);

      const res = await fetch("/api/convert", {
        method: "POST",
        body: formData,
      });

      if (!res.ok) {
        const body = await res.json().catch(() => ({ error: "Conversion failed" }));
        throw new Error(body.error || `Server error: ${res.status}`);
      }

      const blob = await res.blob();
      const cd = res.headers.get("Content-Disposition");
      const name = cd
        ? cd.split("filename=")[1]?.replace(/"/g, "")
        : `converted.${getFormatById(toFormat)?.extension || toFormat}`;

      const url = URL.createObjectURL(blob);
      setDownloadUrl(url);
      setDownloadName(name || `converted.${toFormat}`);
      setStatus("done");
    } catch (err: unknown) {
      setStatus("error");
      setError(err instanceof Error ? err.message : "Unknown error occurred");
    }
  };

  const handleDownload = () => {
    if (!downloadUrl) return;
    const a = document.createElement("a");
    a.href = downloadUrl;
    a.download = downloadName;
    a.click();
  };

  const reset = () => {
    setFile(null);
    setFromFormat("");
    setToFormat("");
    setStatus("idle");
    setError("");
    setDownloadUrl(null);
    setDownloadName("");
  };

  const selectedFrom = getFormatById(fromFormat);
  const selectedTo = getFormatById(toFormat);
  const outputFormats = formats.filter((f) => f.id !== fromFormat);

  return (
    <div className="mx-auto max-w-3xl px-4 py-12 sm:px-6 sm:py-20">
      <div className="text-center">
        <h1 className="text-4xl font-bold tracking-tight sm:text-5xl">
          Convert your <span className="gradient-text">files</span>
        </h1>
        <p className="mt-4 text-lg text-muted">
          Drop your file, pick a format, done. It&apos;s that simple.
        </p>
      </div>

      <div className="mt-10 space-y-6">
        {/* Step 1: Upload */}
        <div className="rounded-2xl border border-border bg-white p-6 shadow-sm">
          <div className="mb-4 flex items-center gap-2 text-base font-medium text-muted">
            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-[11px] font-bold text-white">
              1
            </span>
            {file ? "Your file" : "Drop a file here"}
          </div>

          {!file ? (
            <div
              onDrop={onDrop}
              onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
              onDragLeave={() => setDragOver(false)}
              onClick={() => fileInputRef.current?.click()}
              className={`flex cursor-pointer flex-col items-center rounded-xl border-2 border-dashed p-10 transition-all ${
                dragOver
                  ? "border-primary bg-primary/5"
                  : "border-border hover:border-primary/30 hover:bg-primary/5"
              }`}
            >
              <Upload size={32} className="text-muted" />
              <p className="mt-3 text-base font-medium">
                Click to browse or drag a file here
              </p>
              <p className="mt-1 text-base text-muted">
                Any file up to 50MB
              </p>
            </div>
          ) : (
            <div className="flex items-center gap-3 rounded-xl border border-border bg-background p-4">
              <FileText size={20} className="shrink-0 text-primary" />
              <div className="flex-1 overflow-hidden">
                <div className="truncate text-base font-medium">{file.name}</div>
                <div className="text-base text-muted">{formatFileSize(file.size)}</div>
              </div>
              <button
                onClick={reset}
                className="rounded-lg p-1 text-muted transition-colors hover:bg-foreground/5 hover:text-foreground"
              >
                <X size={16} />
              </button>
            </div>
          )}

          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            onChange={onFileChange}
          />
        </div>

        {/* Step 2: Pick format */}
        <div className="rounded-2xl border border-border bg-white p-6 shadow-sm">
          <div className="mb-4 flex items-center gap-2 text-base font-medium text-muted">
            <span className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-[11px] font-bold text-white">
              2
            </span>
            Convert to
          </div>

          <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-5">
            {outputFormats.map((f) => (
              <button
                key={f.id}
                onClick={() => {
                  setToFormat(f.id);
                  if (status === "done" || status === "error") {
                    setStatus("idle");
                    setDownloadUrl(null);
                    setDownloadName("");
                    setError("");
                  }
                }}
                className={`flex flex-col items-center gap-1.5 rounded-xl border p-3 text-base font-medium transition-all ${
                  toFormat === f.id
                    ? "border-primary bg-primary/5 text-primary shadow-sm"
                    : "border-border hover:border-primary/20 hover:bg-primary/5"
                }`}
              >
                <div
                  className="flex h-7 w-7 items-center justify-center rounded-lg text-[10px] font-bold text-white"
                  style={{ backgroundColor: f.color }}
                >
                  {f.icon}
                </div>
                <span className="truncate w-full text-center">{f.name}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Step 3: Convert */}
        <div className="rounded-2xl border border-border bg-white p-6 shadow-sm">
          {status === "idle" && (
            <button
              onClick={handleConvert}
              disabled={!file || !toFormat}
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-primary py-3.5 text-base font-semibold text-white transition-all hover:bg-primary-light disabled:cursor-not-allowed disabled:opacity-40"
            >
              <Zap size={16} />
              Convert
              {selectedFrom && selectedTo && (
                <span className="text-white/70">
                  ({selectedFrom.name} &rarr; {selectedTo.name})
                </span>
              )}
            </button>
          )}

          {status === "converting" && (
            <div className="flex flex-col items-center gap-3 py-6">
              <Loader2 size={24} className="animate-spin text-primary" />
              <span className="text-base text-muted">Converting your file...</span>
            </div>
          )}

          {status === "done" && (
            <div className="flex flex-col items-center gap-4 py-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-green-100 text-green-600">
                <Check size={24} />
              </div>
              <span className="text-base font-medium text-green-700">
                Conversion complete!
              </span>
              <div className="flex gap-3">
                <button
                  onClick={handleDownload}
                  className="flex items-center gap-2 rounded-xl bg-primary px-6 py-2.5 text-base font-semibold text-white transition-all hover:bg-primary-light"
                >
                  <Download size={14} />
                  Download {downloadName}
                </button>
                <button
                  onClick={reset}
                  className="rounded-xl border border-border px-6 py-2.5 text-base font-medium transition-colors hover:bg-foreground/5"
                >
                  Convert Another
                </button>
              </div>
            </div>
          )}

          {status === "error" && (
            <div className="flex flex-col items-center gap-4 py-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-red-100 text-red-600">
                <AlertCircle size={24} />
              </div>
              <span className="text-base text-red-700">{error}</span>
              <button
                onClick={() => { setStatus("idle"); setError(""); }}
                className="rounded-xl border border-border px-6 py-2.5 text-base font-medium transition-colors hover:bg-foreground/5"
              >
                Try Again
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default function ConverterPage() {
  return (
    <Suspense
      fallback={
        <div className="mx-auto max-w-3xl px-4 py-12 sm:px-6 sm:py-20">
          <div className="flex items-center justify-center py-20">
            <Loader2 size={24} className="animate-spin text-primary" />
          </div>
        </div>
      }
    >
      <ConverterInner />
    </Suspense>
  );
}
