import { glob } from "astro/loaders";
import { defineCollection, z } from "astro:content";

export const docSchema = z.object({
  title: z.string(),
  description: z.string(),
  permalink: z.string().optional(),
  order: z.number(),
  icon: z.string().optional(),
})

const syntax = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/syntax" }),
  schema: docSchema
})

const framework = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/framework" }),
  schema: docSchema
})

const blog = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/blog" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    permalink: z.string().optional(),
    thumbnail: z.string().optional(),
    authors: z.array(z.string()).optional(),
    publishedAt: z.string().optional()
  }),
})

export const collections = { blog, framework, syntax };