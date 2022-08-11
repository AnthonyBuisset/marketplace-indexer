<div align="center">
  <h1 align="center">Marketplace indexer</h1>
  <p align="center">
    <a href="https://discord.gg/onlydust">
        <img src="https://img.shields.io/badge/Discord-6666FF?style=for-the-badge&logo=discord&logoColor=white">
    </a>
    <a href="https://twitter.com/intent/follow?screen_name=onlydust_xyz">
        <img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white">
    </a>
    <a href="https://contributions.onlydust.xyz/">
        <img src="https://img.shields.io/badge/Contribute-6A1B9A?style=for-the-badge&logo=notion&logoColor=white">
    </a>
  </p>
  
  <h3 align="center">Contribution marketplace - On-chain events indexing service.</h3>

</h3>
</div>

> ## ⚠️ WARNING! ⚠️
>
> This repo contains highly experimental code.
> Expect rapid iteration.

## 🎟️ Description

This repository contains everything related to on-chain event indexing. It uses [apibara](http://apibara.com/) as indexing server.

## 🎗️ Prerequisites

### 1. Setup your environment

Create the `.env` file with the correct environment variables.
Copy the `.env.example` file and modify the values according to your setup.

### 2. Start the docker container

Make sure `docker-compose` is installed (see [Installation instructions](https://docs.docker.com/compose/install/)).

```
docker-compose up -d
```

## 📦 Installation

To build the project, run the following command:

```sh
cargo build
```

## 🔬 Usage

To launch the backend, just run:
```sh
cargo run
``` 

## 🌡️ Testing

```
cargo test
```

## 🫶 Contributing

## 📄 License

**marketplace-indexer** is released under the [MIT](LICENSE).
