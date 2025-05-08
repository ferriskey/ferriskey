import { Icon } from "@iconify/react";

type ReadMoreProps = {
  to: string;
  text?: string;
};

export default function ReadMore(props: ReadMoreProps) {
  const urlFragments = props.to.split('/')
    .filter(Boolean)
    .map(fragment => `${fragment.charAt(0).toUpperCase()}${fragment.slice(1)}`)

  return (
    <a
      href={props.to}
      className="group relative inline-flex items-center gap-1.5 px-4 py-3 w-full rounded-md text-sm/6 my-2 transition-colors border bg-muted/50 text-default border-dashed hover:border-primary"
      target={(props.to.startsWith(window.location.origin) || props.to.startsWith('/')) ? '_self' : '_blank'}
      style={{
        textDecoration: 'none',
      }}
    >
      <Icon icon="lucide:bookmark" width={16} aria-hidden="true" />
      {!props.text ? (
        <div className="flex items-center gap-1">
          <span>Read more in</span>
          <div className="inline-flex items-center font-bold">
            {urlFragments.map((fragment, index) => (
              <span key={index} className="flex items-center">
                {fragment}
                {index < urlFragments.length - 1 && (
                  <Icon icon="lucide:chevron-right" width={16} aria-hidden="true" />
                )}
              </span>
            ))}
          </div>
        </div>
      ) : (
        <span>{props.text}</span>
      )}
    </a>
  );
} 