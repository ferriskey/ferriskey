import { defineExplainerConfig } from '@/utils'
import { CuboidIcon, PencilLineIcon } from 'lucide-react'

export default defineExplainerConfig({
  meta: {
    title: 'FerrysKey',
    description: 'FerrisKey is an open-source IAM (Identity and Access Management) solution designed for modern cloud-native environments. ',
    thumbnail: 'https://placehold.co/1200x630',
  },
  urls: {
    github: 'https://github.com/ferriskey/ferriskey',
    getStarted: '/docs/framework/getting-started',
    documentation: '/docs/framework/installation'
  },
  docs: {
    framework: {
      icon: CuboidIcon,
      label: 'Framework',
      href: '/docs/framework/getting-started',
      baseUrl: '/docs/framework',
    },
    syntax: {
      icon: PencilLineIcon,
      label: 'Syntax',
      href: '/docs/syntax/texts',
      baseUrl: '/docs/syntax',
    },
  },
  blog: {
    defaults: {
      thumbnail: 'https://placehold.co/1200x630',
    },
    authors: {
      leadcode_dev: {
        name: 'LeadcodeDev',
        avatar: 'https://avatars.githubusercontent.com/u/8946317?v=4',
        href: 'https://github.com/LeadcodeDev',
      },
    }
  },
  navbar: [
    {
      label: 'Docs',
      items: [
        {
          label: 'Framework',
          description: 'Discover Explainer framework guidelines and usages.',
          href: '/docs/framework/getting-started',
        },
        {
          label: 'Syntax',
          description: 'Learn markdown syntax and markdown components.',
          href: '/docs/syntax/texts',
        },
      ],
    },
    {
      label: 'API',
      href: '/api',
    },
    {
      label: 'Blog',
      href: '/blog',
    },
  ],
  social: {
    github: {
      href: 'https://github.com/ferriskey/ferriskey',
      icon: 'mdi:github',
    },
  }
})