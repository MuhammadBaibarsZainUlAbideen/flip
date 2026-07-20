export interface FormatInfo {
  id: string;
  name: string;
  extension: string;
  icon: string;
  category: "document" | "spreadsheet" | "presentation" | "text" | "data" | "web" | "image";
  color: string;
}

export const formats: FormatInfo[] = [
  { id: "md", name: "Markdown", extension: "md", icon: "M", category: "document", color: "#6d28d9" },
  { id: "html", name: "HTML", extension: "html", icon: "</>", category: "web", color: "#e44d26" },
  { id: "pdf", name: "PDF", extension: "pdf", icon: "P", category: "document", color: "#dc2626" },
  { id: "csv", name: "CSV", extension: "csv", icon: "C", category: "data", color: "#16a34a" },
  { id: "json", name: "JSON", extension: "json", icon: "{", category: "data", color: "#ca8a04" },
  { id: "yaml", name: "YAML", extension: "yaml", icon: "Y", category: "data", color: "#2563eb" },
  { id: "latex", name: "LaTeX", extension: "tex", icon: "L", category: "document", color: "#006600" },
  { id: "svg", name: "SVG", extension: "svg", icon: "S", category: "web", color: "#ff6600" },
  { id: "txt", name: "Plain Text", extension: "txt", icon: "T", category: "text", color: "#71717a" },
  { id: "odt", name: "OpenDocument Text", extension: "odt", icon: "W", category: "document", color: "#185abd" },
  { id: "epub", name: "EPUB", extension: "epub", icon: "E", category: "document", color: "#9333ea" },
  { id: "docx", name: "Word (DOCX)", extension: "docx", icon: "W", category: "document", color: "#2b579a" },
  { id: "xlsx", name: "Excel (XLSX)", extension: "xlsx", icon: "X", category: "spreadsheet", color: "#217346" },
  { id: "pptx", name: "PowerPoint (PPTX)", extension: "pptx", icon: "P", category: "presentation", color: "#d24726" },
  { id: "ods", name: "OpenDocument Sheet", extension: "ods", icon: "S", category: "spreadsheet", color: "#008000" },
  { id: "odp", name: "OpenDocument Presentation", extension: "odp", icon: "O", category: "presentation", color: "#981228" },
  { id: "png", name: "PNG", extension: "png", icon: "I", category: "image", color: "#0ea5e9" },
  { id: "jpg", name: "JPEG", extension: "jpg", icon: "I", category: "image", color: "#f97316" },
  { id: "webp", name: "WebP", extension: "webp", icon: "I", category: "image", color: "#8b5cf6" },
  { id: "gif", name: "GIF", extension: "gif", icon: "I", category: "image", color: "#ec4899" },
  { id: "bmp", name: "BMP", extension: "bmp", icon: "I", category: "image", color: "#64748b" },
  { id: "tiff", name: "TIFF", extension: "tiff", icon: "I", category: "image", color: "#78716c" },
];

export function getFormatById(id: string): FormatInfo | undefined {
  return formats.find((f) => f.id === id || f.extension === id);
}

export function getFormatsByCategory(category: FormatInfo["category"]): FormatInfo[] {
  return formats.filter((f) => f.category === category);
}

export const categories = [
  { id: "document" as const, label: "Documents", description: "Written content and publications" },
  { id: "spreadsheet" as const, label: "Spreadsheets", description: "Tabular data and calculations" },
  { id: "presentation" as const, label: "Presentations", description: "Slides and visual content" },
  { id: "data" as const, label: "Data Formats", description: "Structured data interchange" },
  { id: "web" as const, label: "Web", description: "Web-ready formats" },
  { id: "text" as const, label: "Text", description: "Plain text formats" },
  { id: "image" as const, label: "Images", description: "Raster and vector image formats" },
];

// Popular conversion paths for the landing page
export const popularConversions = [
  { from: "md", to: "pdf", label: "Markdown to PDF" },
  { from: "md", to: "html", label: "Markdown to HTML" },
  { from: "csv", to: "html", label: "CSV to HTML" },
  { from: "csv", to: "json", label: "CSV to JSON" },
  { from: "json", to: "yaml", label: "JSON to YAML" },
  { from: "yaml", to: "json", label: "YAML to JSON" },
  { from: "html", to: "pdf", label: "HTML to PDF" },
  { from: "latex", to: "pdf", label: "LaTeX to PDF" },
  { from: "md", to: "docx", label: "Markdown to Word" },
  { from: "csv", to: "xlsx", label: "CSV to Excel" },
  { from: "md", to: "pptx", label: "Markdown to PowerPoint" },
  { from: "json", to: "csv", label: "JSON to CSV" },
];

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export function getInputFormatsFor(outputId: string): string[] {
  // All formats can convert to any other format via the IR hub
  return formats.filter((f) => f.id !== outputId).map((f) => f.id);
}

export function getOutputFormatsFor(inputId: string): string[] {
  return formats.filter((f) => f.id !== inputId).map((f) => f.id);
}
