import { getCollection, type CollectionKey } from "astro:content";
import config from "../../../explainer.config";

export function useDocs() {
  async function getDocResumes(segments?: string[]) {
    return Promise.all((segments ?? config.docs.).map(async (segment) => {
      const docs = await getCollection(segment as CollectionKey);
      return docs.map((doc) => ({
        title: doc.data.title,
        description: doc.data.description,
        permalink: `/docs/${segment}/${doc.data.permalink}`,
      }));
    }));
  }

  return {
    getDocResumes
  }
}

