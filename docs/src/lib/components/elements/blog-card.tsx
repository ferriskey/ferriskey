import type { CollectionEntry } from 'astro:content';
import config from '../../../../explainer.config';
import { Badge } from '../ui/badge';

type Props = {
  post: CollectionEntry<"blog">
}

export default function BlogCard(props: Props) {
  return (
    <a
      href={`/blog/${props.post.data.permalink}`}
      className="relative flex flex-col gap-2 border border-border hover:border-primary/75 bg-primary/5 transition-colors duration-200 rounded-lg p-4"
    >
      {import.meta.env.DEV && !props.post.data.publishedAt && (
        <div className="absolute z-50 top-1 right-2">
          <Badge variant="default">
            Invisible in production mode
          </Badge>
        </div>
      )}
      <div className="overflow-hidden rounded-lg">
        <img
          src={props.post.data?.thumbnail ?? config.blog.defaults.thumbnail}
          alt={props.post.data.title}
          className="w-full h-48 object-cover rounded-lg"
        />
      </div>

      <div className="flex flex-col gap-2 pt-2">
        <h3 className="text-lg font-semibold">{props.post.data.title}</h3>
        <p className="text-sm text-muted-foreground">
          {props.post.data.description}
        </p>

        <div className="pt-2">
          {props.post.data.authors && props.post.data.authors.length > 1 && (
            <div className="flex -space-x-1 overflow-hidden">
              {props.post.data.authors?.map((author) => {
                const authorData = config.blog.authors[author];
                return (
                  <img
                    alt={authorData.name}
                    src={authorData.avatar}
                    className="inline-block size-6 rounded-full ring-2 ring-white"
                  />
                );
              })}
            </div>
          )}

          {props.post.data.authors && props.post.data.authors.length === 1 && (
            <div
              data-orientation="horizontal"
              className="relative flex items-center gap-2"
            >
              <span className="inline-flex items-center justify-center select-none overflow-hidden rounded-full align-middle size-8 text-base shrink-0 transform transition-transform duration-200">
                <img
                  width="32"
                  height="32"
                  alt={props.post.data.authors[0]}
                  className="h-full w-full rounded-[inherit] object-cover"
                  src={config.blog.authors[props.post.data.authors[0]].avatar}
                />
              </span>
              <div>
                <p className="font-medium text-sm">
                  {config.blog.authors[props.post.data.authors[0]].name}
                </p>
                <p className="text-muted-foreground transition-colors text-xs">
                  @{props.post.data.authors[0]}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </a>
  )
}
