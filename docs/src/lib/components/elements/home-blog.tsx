import type { CollectionEntry } from 'astro:content'
import BlogCard from './blog-card'

type Props = {
  posts: CollectionEntry<"blog">[]
}

export function HomeBlog(props: Props) {
  return (
    <div className="bg-background py-24 sm:py-32">
      <div className="mx-auto max-w-2xl px-6 lg:max-w-7xl lg:px-8">
        <h2 className="text-base/7 font-semibold text-primary">
          Follow our blog for updates and news
        </h2>
        <p className="mt-2 max-w-lg text-pretty text-4xl font-semibold tracking-tight text-gray-950 dark:text-accent-foreground sm:text-5xl">
          See the latest blog posts and follow us
        </p>
        <div className="mt-10 grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {props.posts.map((post) => <BlogCard key={post.data.permalink} post={post} />)}
        </div>
      </div>
    </div>
  )
}