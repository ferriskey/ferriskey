import clsx from 'clsx';

export type BentoGridProps = {
  title: string;
  description: string;
  items: {
    title: string;
    description: string;
    subtitle: string;
    image: string;
  }[];
}

export function HomeBentogrid(props: BentoGridProps) {
  const firstLine = props.items.slice(0, 2);
  const secondLine = props.items.slice(2, 5);

  return (
    <div className="bg-background py-24 sm:py-32">
      <div className="mx-auto max-w-2xl px-6 lg:max-w-7xl lg:px-8">
        <h2 className="text-base/7 font-semibold text-primary">
          {props.title}
        </h2>
        <p className="mt-2 max-w-lg text-pretty text-4xl font-semibold tracking-tight text-gray-950 dark:text-accent-foreground sm:text-5xl">
          {props.description}
        </p>
        <div className="mt-10 grid grid-cols-1 gap-4 sm:mt-16 lg:grid-cols-6 lg:grid-rows-2">
          {firstLine.map((item, index) => (
            <div key={item.title} className="relative lg:col-span-3">
              <div className="absolute inset-px rounded-lg bg-background max-lg:rounded-t-[2rem]" />
              <div
                className={clsx(
                  'absolute inset-px rounded-lg bg-background max-lg:rounded-t-[2rem]',
                  index === 0 && 'lg:rounded-tl-[2rem]',
                  index === firstLine.length - 1 && 'lg:rounded-tr-[2rem]',
                )}
              />
              <div
                className={clsx(
                  'relative flex h-full flex-col overflow-hidden rounded-[calc(theme(borderRadius.lg)+1px)] max-lg:rounded-t-[calc(2rem+1px)]',
                  index === 0 && 'lg:rounded-tl-[calc(2rem+1px)]',
                  index === firstLine.length - 1 && 'lg:rounded-tr-[calc(2rem+1px)]',
                )}
              >
                <img
                  alt={item.subtitle}
                  src={item.image}
                  className="h-80 object-cover object-left"
                />
                <div className="p-10 pt-4">
                  <h3 className="text-sm/4 font-semibold text-primary">{item.title}</h3>
                  <p className="mt-2 text-lg font-medium tracking-tight text-gray-950 dark:text-accent-foreground">
                    {item.subtitle}
                  </p>
                  <p className="mt-2 max-w-lg text-sm/6 text-gray-600">
                    {item.description}
                  </p>
                </div>
              </div>
              <div
                className={clsx(
                  'pointer-events-none absolute inset-px rounded-lg shadow ring-1 ring-black/5',
                  index === 0 && 'lg:rounded-tl-[2rem]',
                  index === firstLine.length - 1 && 'lg:rounded-tr-[2rem]',
                )}
              />
            </div>
          ))}

          {secondLine.map((item, index) => (
            <div key={item.title} className="relative lg:col-span-2">
              <div className="absolute inset-px rounded-lg bg-background" />
              <div className="relative flex h-full flex-col overflow-hidden rounded-[calc(theme(borderRadius.lg)+1px)] lg:rounded-bl-[calc(2rem+1px)]">
                <img
                  alt={item.subtitle}
                  src={item.image}
                  className="h-80 object-cover object-left"
                />
                <div className="p-10 pt-4">
                  <h3 className="text-sm/4 font-semibold text-primary">{item.title}</h3>
                  <p className="mt-2 text-lg font-medium tracking-tight text-gray-950 dark:text-accent-foreground">
                    {item.subtitle}
                  </p>
                  <p className="mt-2 max-w-lg text-sm/6 text-gray-600">
                    {item.description}
                  </p>
                </div>
              </div>
              <div
                className={clsx(
                  'pointer-events-none absolute inset-px rounded-lg shadow ring-1 ring-black/5',
                  index === 0 && 'lg:rounded-bl-[2rem]',
                  index === secondLine.length - 1 && 'lg:rounded-br-[2rem]',
                )}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}