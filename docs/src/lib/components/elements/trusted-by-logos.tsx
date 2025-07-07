import { cn } from "@/utils";

enum SPONSOR_TYPE {
  PLATINUM = "Platinum Sponsor",
  GOLD = "Gold Sponsor",
}

const sponsors = [
  {
    name: "Cloud-IAM",
    logo: "/sponsors/cloudiam.png",
    link: "https://cloud-iam.com",
    type: SPONSOR_TYPE.PLATINUM,
  },
];

export default function TrustedByLogos() {
  return (
    <section className="py-20">
      <div className="mx-auto max-w-7xl">
        <h2 className="mb-4 text-3xl font-bold tracking-tight sm:text-4xl">
          Powered by Our Sponsors
        </h2>
        <p className="mb-12 text-lg text-muted-foreground">
          Ferriskey is made possible by the generous support of our sponsors.
        </p>
        <div className="grid grid-cols-1 gap-3 overflow-hidden sm:rounded-2xl md:grid-cols-2">
          {sponsors.map((sponsor) => (
            <a
              href={sponsor.link}
              target="_blank"
              rel="noopener noreferrer"
              key={sponsor.name}
              className="group relative overflow-hidden rounded-2xl  bg-gradient-to-br from-blue-50 to-indigo-50  p-8 border border-blue-200"
            >
              <div className="absolute top-4 right-4">
                <div
                  className={cn(
                    "flex items-center gap-1 bg-gradient-to-r text-white px-3 py-1 rounded-full text-xs font-medium",
                    sponsor.type === SPONSOR_TYPE.PLATINUM &&
                      "from-purple-600 to-indigo-700",
                    sponsor.type === SPONSOR_TYPE.GOLD &&
                      "from-yellow-400 to-orange-500",
                    "transition-all duration-300 group-hover:scale-105"
                  )}
                >
                  <span>{sponsor.type}</span>
                </div>
              </div>

              <div className="block">
                <div className="mb-4">
                  <img
                    src={sponsor.logo}
                    alt={sponsor.name}
                    className="max-h-24 min-h-24 w-full object-contain"
                  />
                </div>
              </div>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}
