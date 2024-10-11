# Marino

Marino is a small wrapper around `docker compose` that monitors the services
after a defined period of time and returns their status with json.

## Use case

This is used when, for example, you automatically deploy services but need
information about the status of each one after the actual deployment, to learn
whether it succeeded or failed (ie, the services are `healthy` or not).

Kubernetes would be the more correct solution for a high-availability service,
but for system that don't need distributed orchestration, this suffices.

## Example

Consider the following `docker-compose.yml`

```
services:
  dependentee_service:
    image: alpine
    container_name: dependentee_service
    command: ["sh", "-c", "while true; do sleep 30; done"]
    healthcheck:
      test: ["CMD", "ping", "-c", "1", "localhost"]
      interval: 1s
      timeout: 1s
      retries: 3
      start_period: 1s

  dependent_service:
    image: alpine
    container_name: dependent_service
    command: ["sh", "-c", "echo 'Dependent service started' && sleep 60"]
    depends_on:
      dependentee_service:
        condition: service_healthy

```

Marino will start these services and monitor the state for 10 seconds,
returning the health information afterwards.

```
marino --compose-file docker-compose.yml --monitor_duration 10 -- up -d
```

Sample response:

```
Monitoring the following containers: ["dependent_service", "dependentee_service"]
[
  {
    "id": "b65573dee1622db08b29494e1c4aa85a91be5966e7454f0a7bf522a10f5ce080",
    "image": "alpine",
    "names": [
      "/dependent_service"
    ],
    "status": "Up Less than a second"
  },
  {
    "id": "4822dba335871c6c262d7aefdf73e59eb522215cf9df7b575bab24e92d21b910",
    "image": "alpine",
    "names": [
      "/dependentee_service"
    ],
    "status": "Up 5 seconds (healthy)"
  },
```

Note that Marino will match the service `name` property and monitor only those
defined in the `docker-compose.yml` file.

## Project Status

Marino is experimental and developed as part of the Torizon Innovation
initiative. Feedback and contributions are welcomed.