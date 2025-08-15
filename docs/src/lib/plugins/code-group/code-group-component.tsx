import { Icon } from "@iconify/react";
import clsx from 'clsx';
import { useState } from 'react';

export type CodeGroupProps = {
  labels: string;
  languages: string;
  codes: string;
};

export function CodeGroup(props: CodeGroupProps) {
  const [activeTab, setActiveTab] = useState(0);

  // Parse the JSON strings into arrays
  const languages = JSON.parse(props.languages || '[]');
  const labels = JSON.parse(props.labels || '[]');
  const codes = JSON.parse(props.codes || '[]');

  if (!languages.length || !codes.length) {
    return null;
  }

  function getCurrentIcon(label?: string, language?: string) {
    if (!label && !language) return 'mdi:code-tags';

    const labelBase = label?.toLowerCase();
    const languageBase = language?.toLowerCase();

    return icons[labelBase as keyof typeof icons]
      || icons[languageBase as keyof typeof icons]
      || 'mdi:code-tags';
  }

  return (
    <div className="code-group border rounded-md overflow-hidden mb-5 bg-background dark:bg-muted/20">
      <div className="flex p-2 border-b gap-2">
        {languages.map((language: string, i: number) => (
          <button
            key={i}
            onClick={() => setActiveTab(i)}
            className={clsx(
              'relative inline-flex items-center gap-1.5 text-default px-2 py-1.5 text-sm rounded-md disabled:cursor-not-allowed disabled:opacity-75 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary focus:outline-none transition-colors cursor-pointer',
              activeTab === i && 'bg-muted'
            )}
          >
            <Icon icon={getCurrentIcon(labels[i], language)} width={16} /> {labels[i] ?? language}
          </button>
        ))}
      </div>
      <div dangerouslySetInnerHTML={{ __html: codes[activeTab] || '' }} />
    </div>
  );
}

const icons = {
  markdown: 'devicon:markdown',
  mdx: 'devicon:markdown',
  html: 'devicon:html5',
  css: 'devicon:css3',
  javascript: 'devicon:javascript',
  typescript: 'devicon:typescript',
  python: 'devicon:python',
  dart: 'devicon:dart',
  rust: 'catppuccin:rust',
  rs: 'catppuccin:rust',
  npm: 'devicon:npm',
  yarn: 'devicon:yarn',
  pnpm: 'devicon:pnpm',
  bun: 'devicon:bun',
  vite: 'devicon:vite',
  'tailwind.config.js': 'devicon:tailwindcss',
  'tailwind.config.ts': 'devicon:tailwindcss',
  react: 'devicon:react',
  nextjs: 'devicon:nextjs',
  svelte: 'devicon:svelte',
  vue: 'devicon:vuejs',
  go: 'devicon:go',
  ts: 'devicon:typescript',
  bash: 'devicon:bash',
  sql: 'devicon:azuresqldatabase',
  yaml: 'devicon:yaml',
  json: 'devicon:json',
  dockerfile: 'devicon:docker',
  git: 'devicon:git',
  github: 'devicon:github',
  gitlab: 'devicon:gitlab',
}
