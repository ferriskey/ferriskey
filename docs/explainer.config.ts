import { defineExplainerConfig } from '@/utils'

export default defineExplainerConfig({
  meta: {
    title: 'FerrisKey',
    description: 'FerrisKey is an open-source IAM (Identity and Access Management) solution designed for modern cloud-native environments. ',
    thumbnail: 'https://placehold.co/1200x630',
  },
  urls: {
    github: 'https://github.com/ferriskey/ferriskey',
    getStarted: '/docs/welcome/introduction',
    documentation: '/docs/welcome/installation'
  },
  docs: {
    welcome: {
      icon: 'lucide:pencil-line',
      label: 'Welcome',
      href: '/docs/welcome/introduction',
      baseUrl: '/docs/welcome',
    },
    concepts: {
      icon: 'lucide:book',
      label: 'Concepts',
      href: '/docs/concepts/overview',
      baseUrl: '/docs/concepts',
    },
    tutorials: {
      icon: 'lucide:graduation-cap',
      label: 'Tutorials',
      href: '/docs/tutorials/quickstart',
      baseUrl: '/docs/tutorials',
    },
    reference: {
      icon: 'lucide:braces',
      label: 'Reference',
      href: '/docs/reference/api',
      baseUrl: '/docs/reference',
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
      nathael: {
        name: 'Nathael',
        avatar: 'https://avatars.githubusercontent.com/u/64804778?v=4',
        href: 'https://github.com/nathaelb',
      }
    }
  },
  navbar: [
    {
      label: 'Docs',
      items: [
        {
          label: 'Concepts',
          description: 'Learn about the core concepts of Explainer.',
          href: '/docs/concepts/overview',
        },
        {
          label: 'Tutorials',
          description: 'Learn how to use Explainer.',
          href: '/docs/tutorials/quickstart',
        },
        {
          label: 'Tasks',
          description: 'Learn how to use Explainer.',
          href: '/docs/tasks/manage-users',
        },
        {
          label: 'Reference',
          description: 'Learn how to use Explainer.',
          href: '/docs/reference/api',
        },
      ],
    },
    {
      label: 'About',
      href: '/about',
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