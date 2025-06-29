const contributors = [
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/64804778?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/8946317?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/73846641?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/151126350?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/809982?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/95345091?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/1043863?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/60852187?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/79365734?s=120&v=4",
  },
  {
    avatarUrl: "https://avatars.githubusercontent.com/u/129272769?s=120&v=4",
  },
];

export default function Contributors() {
  return (
    <div className="py-20">
      <div>
        <h2 className="text-34l text-balance font-semibold tracking-tight sm:text-5xl">
          Contributors
        </h2>
        <p className="mt-6 text-md text-gray-600 dark:text-gray-400 max-w-2xl">
          FerrisKey is not possible without the help and support of our
          outstanding contributors.
        </p>
      </div>

      <div className="mt-12">
        <div className="grid gap-1 grid-cols-5 md:grid-cols-10">
          {contributors.map((contributor) => (
            <div className="rounded-full border border-sand4 px-1 py-1">
              <img
                src={contributor.avatarUrl}
                alt={"Contributor Avatar"}
                className="w-full rounded-full"
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
