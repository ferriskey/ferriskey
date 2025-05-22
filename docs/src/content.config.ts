import { glob } from "astro/loaders";
import { defineCollection, z } from "astro:content";

export const docSchema = z.object({
  title: z.string(),
  description: z.string(),
  permalink: z.string().optional(),
  order: z.number(),
  icon: z.string().optional(),
})

const framework = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/framework" }),
  schema: docSchema
})

const welcome = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/welcome" }),
  schema: docSchema
})

const concepts = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/01-concepts" }),
  schema: docSchema
})

const tutorials = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/02-tutorials" }),
  schema: docSchema
})

const tasks = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/03-tasks" }),
  schema: docSchema
})

const reference = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs/04-reference" }),
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

export const collections = { blog, framework, concepts, tutorials, tasks, reference, welcome };