# FerrisKey

FerrisKey is an open-source IAM (Identity and Access Management) solution designed for modern cloud-native environments. With its high-performance API written in Rust and its intuitive web interface developed in Typescript/React, FerrisKey offerrs a robust and flexible alternative to tradtional IAM solutions.

## 🚀 Features

- **Modern and high-performance**: Built with Rust and React, ensuring optimal performance and a smooth user experience.
- **Cloud-native**: Optimised for container deployments and Kubernetes clusters.
- **Hexagonal architecture**: An API structured around ports and adapters facilitating extensibility and collaboration.
- **Intuitive user interface**: A comprehensive administration portal developed with React, TypeScript and Tailwind CSS.
- **Highly secure**: Leveraging Rust's memory safety guarantees and best practices in authentication.
- **Open Source**: Developed transparently with the community, for the community.

## 🏗️ Architecture

FerrisKey is based on a hexagonal architecture (or "ports and adapters") that clearly separates business logic from technical infrastructure. This approach facilitates application maintenance, extension and testing.

### API (Rust)

This FerrisKey API is structured according to hexagonal architecture principles:

- **Domain**: Contains business entities and core logic
- **Application**: Coordinates workflows between the domain and adapters.
- **Infrastructure(adapters)**: Connect the system to external technologies (databases, web services, etc..).

![API Architecture](./docs_old/api_architecture.png)

This modular design allows:

- Easy addition of new features
- Component replacement without modifying business logic
- Robust unit and integration testing
- Efficient collaboration between developers

### Frontend (TypeScript/React)

The user interface is developed with modern web technologies:

- React: For a reactive and modular interface
- TypeScript: For strong typing and better maintainability
- Tailwind CSS: For elegant and responsive design
- React Query: For efficient data management and API calls
- Zustand: For state management

## 🌱 Why FerrisKey?

Unlike historical solutions such as Keycloak (11 years of Java/Quarkus codebase), FerrisKey is designed from the outset to address modern identity management challenges:

- **Performance**: Reduced memory footprint and minimal latency thanks to Rust.
- **Security**: Protection against common security vulnerabilities.
- **Adaptability**: Ease of extension and integration with modern ecosystems.
- **Simplified deployment**: Optimised for Kubernetes and containerised environments.

## 🚦 Getting Started with FerrisKey

We working to prepare the development environnement

## 🧩 Contributions

FerrisKey is an open-source project that welcomes community contributions. Whether you want to fix a bug, add a feature or improve documentation, your contributions are welcome!

## 🙏 Acknowledgements

FerrisKey builds upon numerous open-source projects and draws inspiration from community best practices. We thank all contributors who make this project possible.
