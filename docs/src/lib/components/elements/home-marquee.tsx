import { cn } from "@/utils";
import { GridPattern } from '../ui/grid-pattern';
import { Marquee } from "../ui/marquee";

export type MarqueeProps = {
  title: string;
  description: string;
  reviewers: {
    name: string;
    username: string;
    body: string;
    img: string;
  }[];
}

export function HomeMarquee(props: MarqueeProps) {
  const firstRow = props.reviewers.slice(0, props.reviewers.length / 2);
  const secondRow = props.reviewers.slice(props.reviewers.length / 2);

  return (
    <div className="relative mx-auto max-w-2xl px-6 lg:max-w-7xl lg:px-8 pb-20">
      <GridPattern
        width={25}
        height={25}
        x={-1}
        y={-1}
        strokeDasharray={"4 2"}
        className={cn(
          "[mask-image:radial-gradient(600px_circle_at_center,white,transparent)]",
        )}
      />
      <div className="relative z-10">
        <h2 className="text-base/7 font-semibold text-primary">
          {props.title}
        </h2>
        <p className="mt-2 max-w-lg text-pretty text-4xl font-semibold tracking-tight text-gray-950 dark:text-accent-foreground sm:text-5xl">
          {props.description}
        </p>

        <div className="mt-10 relative flex w-full flex-col items-center justify-center overflow-hidden">
          <Marquee pauseOnHover className="[--duration:20s]">
            {firstRow.map((review) => (
              <ReviewCard key={review.username} {...review} />
            ))}
          </Marquee>
          <Marquee reverse pauseOnHover className="[--duration:20s]">
            {secondRow.map((review) => (
              <ReviewCard key={review.username} {...review} />
            ))}
          </Marquee>
          <div className="pointer-events-none absolute inset-y-0 left-0 w-1/4 bg-gradient-to-r from-background"></div>
          <div className="pointer-events-none absolute inset-y-0 right-0 w-1/4 bg-gradient-to-l from-background"></div>
        </div>
      </div>
    </div>
  );
}

type ReviewCardProps = {
  img: string;
  name: string;
  username: string;
  body: string;
}

function ReviewCard(props: ReviewCardProps) {
  const { img, name, username, body } = props;

  return (
    <figure
      className={cn(
        "relative h-full w-64 bg-background cursor-pointer overflow-hidden rounded-xl border p-4",
      )}
    >
      <div className="flex flex-row items-center gap-2">
        <img className="rounded-full" width="32" height="32" alt="" src={img} />
        <div className="flex flex-col">
          <figcaption className="text-sm font-medium dark:text-white">
            {name}
          </figcaption>
          <p className="text-xs font-medium dark:text-white/40">{username}</p>
        </div>
      </div>
      <blockquote className="mt-2 text-sm">{body}</blockquote>
    </figure>
  );
};