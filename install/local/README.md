## LOCAL docker-compose

This `docker-compose.yaml` (and corresponding `docker-compose.dev.yaml`) file are used in standing up local instances of Wikijump. The convenience script `./deploy-helper.py` is provided to make management easier, providing options for common variations.

There are two important things to note about the local tier:
1. It runs its containers in "watch mode". This means that building the service takes place after container start, not at container build time, and that if you modify local watched files, the service will rebuild and restart.
2. Which is related to the fact that several directories are instead *mapped* into the container rather than copied into it. This way, any local changes are reflected in the container.

See `docs/development.md` for more information on local deployments.
