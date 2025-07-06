const coreTeam = [
  {
    name: "Nathael Bonnal",
    description: "Creator and lead maintainer of Ferriskey.",
    image:
      "https://media.licdn.com/dms/image/v2/D4E03AQFqelIxg95W-w/profile-displayphoto-shrink_400_400/profile-displayphoto-shrink_400_400/0/1718339990545?e=1755734400&v=beta&t=zaBDYALu7GB4S6W9npEp1fOIaHfkbjOg8vlKKulrpXo",
    social: {
      github: "https://github.com/nathaelb",
      linkedin: "https://www.linkedin.com/in/nathael-bonnal/",
    },
  },
  {
    name: "Baptiste Parmantier",
    description: "Lead Software Engineer at Natalia.",
    image:
      "https://media.licdn.com/dms/image/v2/D4E03AQF3NefwIYKvKQ/profile-displayphoto-shrink_400_400/profile-displayphoto-shrink_400_400/0/1708508199106?e=1755734400&v=beta&t=C-5VPf6wPNEXq2peFT7oBHUn9zGfvXTcl9pX0GWJja0",
    social: {
      github: "https://github.com/leadcodedev",
      linkedin: "https://www.linkedin.com/in/baptiste-parmantier/",
    },
  },
  {
    name: "Joris Vilardell",
    description: "DevOps Engineer",
    image:
      "https://media.licdn.com/dms/image/v2/D4E03AQHzm2tJWI22Wg/profile-displayphoto-shrink_800_800/profile-displayphoto-shrink_800_800/0/1706510255978?e=1755734400&v=beta&t=Q4qw6i7vrAm8Z1V6PgddehRMlFm8TNawuYGWjCwQZN0",
    social: {
      github: "https://github.com/ZUHOWKS",
      linkedin: "https://www.linkedin.com/in/joris-vilardell-76050427b/",
    },
  },
];

export default function CoreTeam() {
  return (
    <div className="py-20">
      <div className="mx-auto">
        <div className="">
          <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
            Core Team
          </h2>
          <p className="mt-4 text-md text-muted-foreground max-w-2xl">
            Meet the core team behind Ferriskey, dedicated to building a secure
            and user-friendly platform for managing your secrets.
          </p>
        </div>

        <ul
          role="list"
          className="mx-auto mt-20 grid max-w-2xl grid-cols-1 gap-x-8 gap-y-14 sm:grid-cols-2 lg:mx-0 lg:max-w-none lg:grid-cols-3 xl:grid-cols-3"
        >
          {coreTeam.map((member) => (
            <li key={member.name} className="flex flex-col h-full">
              <img
                src={member.image}
                alt=""
                className="aspect-[14/13] w-full rounded-md object-cover"
              />

              <div className="flex flex-col flex-1 justify-between mt-2">
                <h3 className="text-lg/8 font-semibold tracking-tight ">
                  {member.name}
                </h3>
                <p className="text-base/7 text-gray-600/80 dark:text-gray-400">
                  {member.description}
                </p>

                <div className="flex flex-row gap-4 mt-4">
                  {member.social.github && (
                    <a href={member.social.github} target="_blank" className="">
                      <img src="/github-mark.svg" className="w-6 " />
                    </a>
                  )}
                  {member.social.linkedin && (
                    <a
                      href={member.social.linkedin}
                      target="_blank"
                      className=""
                    >
                      <img src="/linkedin.svg" className="w-6 " />
                    </a>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
