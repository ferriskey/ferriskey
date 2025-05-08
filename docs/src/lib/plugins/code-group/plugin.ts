import type { Element, Root } from 'hast';
import { toHtml } from 'hast-util-to-html';
import type { Plugin } from 'unified';
import { visit } from 'unist-util-visit';

const rehypeCodeGroupReact: Plugin<[], Root> = () => {
  return (tree: Root) => {
    const codeGroups: {
      parent: any;
      index: number;
      endIndex: number;
      labels: string[] | null;
      languages: string[]
    }[] = [];

    // First pass: identify code groups
    visit(tree, 'element', (node, index, parent) => {
      if (
        node.type === 'element' &&
        node.tagName === 'p' &&
        node.children &&
        node.children[0] &&
        node.children[0].type === 'text'
      ) {
        const textContent = node.children[0].value;

        const matchWithoutLabels = textContent.match(/^:::code-group auto$/);
        const matchWithLabels = textContent.match(/^:::code-group\s+labels=\[(.*?)\]$/);

        if ((matchWithLabels || matchWithoutLabels) && parent && typeof index === 'number') {
          let endIndex = -1;

          let labels = []
          let languages: string[] = []
          for (let i = index + 1; i < parent.children.length; i++) {
            const child = parent.children[i]

            if (child.type === 'element' && child.tagName === 'pre' &&
              'properties' in child && child.properties?.dataLanguage) {
              labels.push(child.properties!.dataLanguage as string)
              languages.push(child.properties!.dataLanguage as string)
            }

            if (
              child.type === 'element' &&
              child.tagName === 'p' &&
              child.children &&
              child.children[0] &&
              child.children[0].type === 'text' &&
              child.children[0].value.trim() === ':::'
            ) {
              endIndex = i;
              break;
            }
          }

          if (endIndex !== -1) {
            if (matchWithLabels) {
              matchWithLabels[1].split(',').forEach((label, index) => {
                if (label.length) {
                  labels[index] = label.trim()
                }
              })
            }

            codeGroups.push({
              parent,
              index,
              endIndex,
              labels,
              languages
            });
          }
        }
      }
    });

    for (let i = codeGroups.length - 1; i >= 0; i--) {
      const { parent, index, endIndex, labels: providedLabels, languages: providedLanguages } = codeGroups[i];

      const codeBlocks = parent.children.slice(index + 1, endIndex)
        .filter((node: any) => node.type === 'element' && node.tagName === 'pre');

      const codes: string[] = [];
      const extractedLabels: string[] = [];

      codeBlocks.forEach((codeBlock: Element) => {
        const codeElement = codeBlock.children[0] as Element;

        if (codeElement && codeElement.type === 'element' && codeElement.tagName === 'code') {
          const codeHtml = toHtml(codeBlock);
          codes.push(codeHtml);
        }
      });

      const finalLabels = providedLabels || extractedLabels;
      if (providedLanguages.length > 0 && codes.length > 0) {
        const newNode = {
          type: 'mdxJsxFlowElement',
          name: 'CodeGroupWrapper',
          attributes: [
            { type: 'mdxJsxAttribute', name: 'labels', value: JSON.stringify(finalLabels) },
            { type: 'mdxJsxAttribute', name: 'languages', value: JSON.stringify(providedLanguages) },
            { type: 'mdxJsxAttribute', name: 'codes', value: JSON.stringify(codes) }
          ],
          children: [],
          data: { _mdxExplicitJsx: true }
        };

        parent.children.splice(index, endIndex - index + 1, newNode);
      }
    }
  };
};

export default rehypeCodeGroupReact;
