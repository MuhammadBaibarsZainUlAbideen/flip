import { NextRequest, NextResponse } from "next/server";
import { execFile } from "child_process";
import { writeFile, unlink, mkdir } from "fs/promises";
import { join } from "path";
import { tmpdir } from "os";
import { randomUUID } from "crypto";

const FLIP_BIN = process.env.FLIP_BIN || "flip";
const SOFFICE_BIN = process.env.SOFFICE_BIN || findLibreOffice();

function findLibreOffice(): string {
  const fs = require("fs");
  const candidates = [
    "/usr/bin/libreoffice",
    "/usr/bin/soffice",
    "/usr/local/bin/libreoffice",
    "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
    "C:\\Program Files (x86)\\LibreOffice\\program\\soffice.exe",
  ];
  for (const p of candidates) {
    if (fs.existsSync(p)) return p;
  }
  return "libreoffice";
}

const DOCX_CONVERSIONS: Record<string, boolean> = {
  pdf: true,
  html: true,
  htm: true,
};

const ALLOWED_EXTENSIONS: Record<string, string> = {
  md: "md", markdown: "md",
  html: "html", htm: "html",
  pdf: "pdf",
  csv: "csv",
  json: "json",
  yaml: "yaml", yml: "yaml",
  tex: "tex", latex: "tex",
  svg: "svg",
  txt: "txt", text: "txt",
  odt: "odt",
  epub: "epub",
  docx: "docx",
  xlsx: "xlsx",
  pptx: "pptx",
  ods: "ods",
  odp: "odp",
  png: "png",
  jpg: "jpg", jpeg: "jpg",
  webp: "webp",
  gif: "gif",
  bmp: "bmp",
  tiff: "tiff",tif: "tiff",
};

const OUTPUT_EXTENSIONS: Record<string, string> = {
  md: "md", html: "html", pdf: "pdf", csv: "csv", json: "json",
  yaml: "yaml", tex: "tex", svg: "svg", txt: "txt", odt: "odt",
  epub: "epub", docx: "docx", xlsx: "xlsx", pptx: "pptx", ods: "ods", odp: "odp",
  png: "png", jpg: "jpg", jpeg: "jpg", webp: "webp", gif: "gif", bmp: "bmp", tiff: "tiff",
};

export async function POST(request: NextRequest) {
  const tmpDir = join(tmpdir(), `flip-web-${randomUUID()}`);

  try {
    await mkdir(tmpDir, { recursive: true });

    const formData = await request.formData();
    const file = formData.get("file") as File | null;
    const toFormat = (formData.get("to") as string)?.toLowerCase().trim();
    const fromFormat = (formData.get("from") as string)?.toLowerCase().trim();

    if (!file) {
      return NextResponse.json({ error: "No file provided" }, { status: 400 });
    }
    if (!toFormat || !OUTPUT_EXTENSIONS[toFormat]) {
      return NextResponse.json({ error: `Unsupported output format: ${toFormat}` }, { status: 400 });
    }

    const origName = file.name || "upload";
    const ext = origName.split(".").pop()?.toLowerCase() || "";
    const detectedExt = ALLOWED_EXTENSIONS[ext] || ext;

    if (!detectedExt) {
      return NextResponse.json({ error: "Cannot detect file format" }, { status: 400 });
    }

    const inputPath = join(tmpDir, `input.${detectedExt}`);
    const outputPath = join(tmpDir, `output.${OUTPUT_EXTENSIONS[toFormat]}`);

    // Write uploaded file to disk
    const arrayBuffer = await file.arrayBuffer();
    const buffer = Buffer.from(arrayBuffer);
    await writeFile(inputPath, buffer);

    // Use LibreOffice for DOCX conversions (Word-perfect formatting)
    const useLibreOffice =
      (detectedExt === "docx" || detectedExt === "odt") && DOCX_CONVERSIONS[toFormat];

    if (useLibreOffice) {
      // LibreOffice outputs to the same directory with the same name but different extension
      const loResult = await new Promise<{ stdout: string; stderr: string; code: number }>(
        (resolve) => {
          execFile(
            SOFFICE_BIN,
            ["--headless", "--convert-to", toFormat, "--outdir", tmpDir, inputPath],
            { timeout: 60000, maxBuffer: 50 * 1024 * 1024 },
            (error, stdout, stderr) => {
              resolve({
                stdout: stdout || "",
                stderr: stderr || "",
                code: error ? Number(error.code ?? 1) : 0,
              });
            }
          );
        }
      );

      if (loResult.code !== 0) {
        let errMsg = loResult.stderr || loResult.stdout || "LibreOffice conversion failed";
        errMsg = errMsg.replace(/\x1B\[[0-9;]*[a-zA-Z]/g, "").trim();
        return NextResponse.json({ error: errMsg }, { status: 422 });
      }

      // LibreOffice outputs as inputBaseName.ext in the output dir (uses input filename, not original)
      const loOutputPath = join(tmpDir, `input.${toFormat}`);
      const { readFile } = await import("fs/promises");
      const outputBuffer = await readFile(loOutputPath);

      await unlink(inputPath).catch(() => {});

      const outputExt = OUTPUT_EXTENSIONS[toFormat];
      const baseName = origName.replace(/\.[^.]+$/, "");
      const outputFileName = `${baseName}.${outputExt}`;

      const response = new NextResponse(outputBuffer);
      const mimeMap: Record<string, string> = {
        pdf: "application/pdf", html: "text/html", csv: "text/csv",
        json: "application/json", yaml: "text/yaml", svg: "image/svg+xml",
        txt: "text/plain", md: "text/markdown",
      };
      response.headers.set("Content-Type", mimeMap[outputExt] || "application/octet-stream");
      response.headers.set("Content-Disposition", `attachment; filename="${outputFileName}"`);
      return response;
    }

    // Build flip command for all other conversions
    const args = [inputPath, "--to", toFormat, "-o", outputPath];
    if (fromFormat && fromFormat !== detectedExt) {
      args.splice(1, 0, "--from", fromFormat);
    }

    // Execute flip
    const result = await new Promise<{ stdout: string; stderr: string; code: number }>((resolve) => {
      execFile(
        FLIP_BIN,
        args,
        { timeout: 30000, maxBuffer: 50 * 1024 * 1024 },
        (error, stdout, stderr) => {
          resolve({
            stdout: stdout || "",
            stderr: stderr || "",
            code: error ? Number(error.code ?? 1) : 0,
          });
        }
      );
    });

    if (result.code !== 0) {
      let errMsg = result.stderr || "Conversion failed";
      errMsg = errMsg.replace(/\x1B\[[0-9;]*[a-zA-Z]/g, "").trim();
      return NextResponse.json({ error: errMsg }, { status: 422 });
    }

    // Read output file
    const { readFile } = await import("fs/promises");
    const outputBuffer = await readFile(outputPath);

    // Clean up input file
    await unlink(inputPath).catch(() => {});

    const baseName = origName.replace(/\.[^.]+$/, "");
    const outputExt = OUTPUT_EXTENSIONS[toFormat];
    const outputFileName = `${baseName}.${outputExt}`;

    // Return file
    const response = new NextResponse(outputBuffer);
    const mimeMap: Record<string, string> = {
      pdf: "application/pdf",
      html: "text/html",
      csv: "text/csv",
      json: "application/json",
      yaml: "text/yaml",
      svg: "image/svg+xml",
      txt: "text/plain",
      md: "text/markdown",
      docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      tex: "application/x-latex",
      odt: "application/vnd.oasis.opendocument.text",
      ods: "application/vnd.oasis.opendocument.spreadsheet",
      odp: "application/vnd.oasis.opendocument.presentation",
      epub: "application/epub+zip",
      png: "image/png",
      jpg: "image/jpeg",
      jpeg: "image/jpeg",
      webp: "image/webp",
      gif: "image/gif",
      bmp: "image/bmp",
      tiff: "image/tiff",
    };

    response.headers.set("Content-Type", mimeMap[outputExt] || "application/octet-stream");
    response.headers.set("Content-Disposition", `attachment; filename="${outputFileName}"`);

    return response;
  } catch (err: unknown) {
    console.error("Conversion error:", err);
    return NextResponse.json(
      { error: err instanceof Error ? err.message : "Internal server error" },
      { status: 500 }
    );
  } finally {
    // Clean up temp directory
    const { rm } = await import("fs/promises");
    await rm(tmpDir, { recursive: true, force: true }).catch(() => {});
  }
}
