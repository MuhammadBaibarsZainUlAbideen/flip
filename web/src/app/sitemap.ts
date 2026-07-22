import type { MetadataRoute } from "next";
import { formats, popularConversions } from "@/lib/formats";

const BASE_URL = "https://flip.engineer";

const conversionPairs: Array<{ from: string; to: string }> = [];

for (const from of formats) {
  for (const to of formats) {
    if (from.id !== to.id) {
      conversionPairs.push({ from: from.id, to: to.id });
    }
  }
}

export default function sitemap(): MetadataRoute.Sitemap {
  const now = new Date().toISOString();

  const staticPages: MetadataRoute.Sitemap = [
    {
      url: BASE_URL,
      lastModified: now,
      changeFrequency: "weekly",
      priority: 1,
    },
    {
      url: `${BASE_URL}/converter`,
      lastModified: now,
      changeFrequency: "weekly",
      priority: 0.9,
    },
  ];

  const popularPairs: MetadataRoute.Sitemap = popularConversions.map((c) => ({
    url: `${BASE_URL}/convert/${c.from}-to-${c.to}`,
    lastModified: now,
    changeFrequency: "monthly" as const,
    priority: 0.8,
  }));

  const allPairs: MetadataRoute.Sitemap = conversionPairs.map((c) => ({
    url: `${BASE_URL}/convert/${c.from}-to-${c.to}`,
    lastModified: now,
    changeFrequency: "monthly" as const,
    priority: 0.5,
  }));

  return [...staticPages, ...popularPairs, ...allPairs];
}
