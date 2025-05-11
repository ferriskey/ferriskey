import type { CollectionKey } from 'astro:content';
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export type HeadingNode = {
  depth: number;
  slug: string;
  text: string;
  children: HeadingNode[];
};

type NavbarCollection = {
  label: string
  items?: NavbarItem[]
  href?: string
}

type NavbarItem = {
  label: string
  description: string
  href: string
}

type ExplainerMeta = {
  title: string
  description: string
  thumbnail: string
}

type ExplainerDocs = {
  icon: string
  label: string
  href: string
  baseUrl: string
}

type ExplainerBlog = {
  defaults: {
    thumbnail?: string
  }
  authors: {
    [key: string]: {
      name: string
      avatar: string
      href: string
    }
  }
}

const SocialLink = {
  github: 'Github',
  twitter: 'Twitter',
  linkedin: 'LinkedIn',
  facebook: 'Facebook',
  instagram: 'Instagram',
  youtube: 'YouTube',
  tiktok: 'TikTok',
  twitch: 'Twitch',
} as const

type ExplainerSocial = {
  [key in keyof typeof SocialLink]?: {
    href: string
    icon: string
  }
}

type ExplainerConfig = {
  meta: ExplainerMeta
  docs: { [key in CollectionKey]?: ExplainerDocs }
  urls: {
    github?: string
    getStarted?: string
    documentation?: string
  },
  navbar: NavbarCollection[],
  blog: ExplainerBlog,
  social: ExplainerSocial
}

export function defineExplainerConfig(config: ExplainerConfig) {
  return config
}
