---
name: quadlet-units
description: How the Podman Quadlet units in quadlet/ work, and how to verify and locally test a change to them. Use when editing a file under quadlet/, the "Podman Quadlet" section of README.md, or a part of the stack the units share with Compose (Caddyfile, Dockerfiles, services, ports, volumes, backend environment).
---

# Podman Quadlet units

The units in `quadlet/` run the proxy, backend and frontend as rootless systemd
user units on Podman 6.1. `README.md` ("Podman Quadlet") is the user-facing
description; keep it in step with every change here.

## How they work

- **Installation is a symlink** from
  `~/.config/containers/systemd/pocketratings` to `quadlet/` of a checkout.
  Quadlet resolves the relative paths (`../backend`, `../frontend`,
  `../Caddyfile`) against the checkout. `podman quadlet install` copies the
  units away from the repository, which breaks those paths.
- **Host settings live in drop-in files** in
  `~/.config/containers/systemd/<unit>.container.d/*.conf`; the units hold only
  generic values. The backend gets `APP_BASE_URL` with `Environment=` and
  `JWT_SECRET` from the podman secret `pocketratings-jwt` with `Secret=`. The
  units take no `EnvironmentFile=`: `podman run --env-file` fails when the file
  is missing and has no optional form.
- **Every start rebuilds**: a `.build` unit is `Type=oneshot` without
  `RemainAfterExit`, so each start of the container unit that uses it runs
  `podman build` (seconds with the layer cache). Restarting
  `pocketratings-backend` or `pocketratings-frontend` also restarts the proxy
  `pocketratings`, which requires both.
- **Shared with Compose**: the `Caddyfile` and the Dockerfiles. The Caddyfile
  reaches the containers by the host names `backend` and `frontend`, which the
  units set with `NetworkAlias=`. A change to the stack (a service, a port, a
  volume, a required backend variable) goes into `compose.yaml`,
  `compose.bastion.yaml` and the units in the same change.

## Verify a change

1. **Dry run** on a scratch clone that carries the uncommitted changes and has
   no `.env`, with `quadlet/` linked into a scratch unit directory and the
   drop-ins of README step 1 next to it:

   ```bash
   git clone -q -b BRANCH . SCRATCH/clone && git diff | git -C SCRATCH/clone apply
   mkdir -p SCRATCH/units && ln -s SCRATCH/clone/quadlet SCRATCH/units/pocketratings
   QUADLET_UNIT_DIRS=SCRATCH/units /usr/lib/podman/quadlet -dryrun -user
   ```

   Done when the generator exits 0, prints no error, and each `ExecStart=` line
   carries the flags the change intends (ports, `--env`, `--secret`, `-v`).

2. **Local test**, only when the developer asks for one in this session: it
   creates units, images, volumes, a network and a listening port in their
   account. Install by the symlink, publish `127.0.0.1:3080:80` through a
   drop-in (`PublishPort=` then `PublishPort=127.0.0.1:3080:80`), and create the
   secret only when `podman secret exists pocketratings-jwt` fails. Done when:
   - `ss -ltn` shows only `127.0.0.1:3080`;
   - a user registered with
     `podman exec pocketratings-backend /app/pocketratings user register` logs
     in through the browser and creates a category;
   - the category survives `systemctl --user restart pocketratings-backend` and
     a stop and start of `pocketratings*`;
   - after `podman kill pocketratings-backend`, `NRestarts` of the unit rises
     and it is `active` again;
   - the journal of the three units shows no errors.

3. **Clean-up** after the local test:
   - `systemctl --user stop 'pocketratings*'`; remove the symlink and the
     drop-in directories of the test; `systemctl --user daemon-reload`.
   - `podman volume rm pocketratings-db pocketratings-caddy-data`,
     `podman network rm pocketratings`, and
     `podman rmi localhost/pocketratings-backend:latest localhost/pocketratings-frontend:latest`
     together with the untagged builder images the test created, by ID
     (`podman images -f dangling=true`, created during the test). Other
     projects' images live on the same machine, so remove by name or ID only.
   - The secret, `~/.config/containers/systemd` and `~/.config/containers` only
     when the test created them (`stat -c %w` shows when a directory was
     created).
   - Keep the base images and any `pocketratings_*` Compose containers and
     volumes that existed before the test.
