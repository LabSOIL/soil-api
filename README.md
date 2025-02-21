# soil-api
A rewrite of [soil-api-python](https://github.com/LabSOIL/soil-api-python) in Rust. It is
a component of the [Sensor Map project](https://github.com/LabSOIL/sensormap-ui).


## Building

Refer to the [docker-compose.yaml](https://github.com/LabSOIL/sensormap-ui/blob/main/docker-compose.yaml) file in the Sensor Map project for an example of how to build and run the project.

Its incorporation into the Sensormap project can be configured by the addition of
a SOIL_API_SECONDARY_URL into the BFF (as seen in the docker-compose.yaml file), and
then forwarding individual routes via the BFF reverse proxy in [proxy.py](https://github.com/LabSOIL/sensormap-bff/blob/main/app/tools/proxy.py).
