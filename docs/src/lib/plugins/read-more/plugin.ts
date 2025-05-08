import type { Root } from 'hast';
import { visit } from 'unist-util-visit';

const rehypeReadMoreReact = () => {
  return (tree: Root) => {
    visit(tree, (node: any, index?: number, parent?: any) => {
      if (
        node.type === 'element' &&
        node.tagName === 'read-more'
      ) {
        node.tagName = 'ReadMoreWrapper';

        node.properties = {
          ...(node.properties || {}),
          isJsxComponent: true
        };

        node.children = [];
      }
    });
  };
};

export default rehypeReadMoreReact; 