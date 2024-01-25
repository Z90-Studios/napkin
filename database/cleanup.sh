#!/bin/bash

sudo docker compose -f docker-compose.yml down
sudo rm -rfv ../.napkin/postgres-data
