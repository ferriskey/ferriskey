const sponsors = [
    {
        name: 'Cloud-IAM',
        logo: '/sponsors/cloudiam.png'
    },
]

export default function TrustedByLogos() {
    return (
        <section className="py-20">
            <div className="mx-auto max-w-7xl">
                <div className="-mx-6 grid grid-cols-2 gap-0.5 overflow-hidden sm:mx-0 sm:rounded-2xl md:grid-cols-3">
                    {sponsors.map((sponsor) => (
                        <div
                            key={sponsor.name}
                            className="bg-gray-400/5 p-8 sm:p-10"
                        >
                            <img
                                src={sponsor.logo}
                                alt={sponsor.name}
                                className="max-h-24 min-h-24 w-full object-contain grayscale transition hover:grayscale-0"
                            />
                        </div>
                    ))}
                </div>
            </div>
        </section>
    )
}