#!/bin/bash

docker start rinha24-api01
docker start rinha24-api02

docker cp ./src/* rinha24-api01:/rinha24/src/
docker cp ./src/* rinha24-api02:/rinha24/src/

docker stop rinha24-api01
docker stop rinha24-api02