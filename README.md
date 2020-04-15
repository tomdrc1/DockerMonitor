# Docker Monitor

This is a simple POC for a docker monitor container.

## Overview
When designed properly, Docker containers are meant to do just one thing: serve a single service and maintain a consistent set of files.

However, in today’s environment, there are often malicious actors attempting to compromise containers and take control of them.

This tool allows you to monitor both your current containers and any new containers added to your environment, running alongside your existing containers.

It works by first retrieving the base image of each container on the host machine and analyzing it. It then maps out the files and processes that should be present in each container. If any unauthorized changes are detected, such as a process running that shouldn’t be, it will trigger an alert (still to be implemented), and the container will automatically reset.

Currently only tested on Linux!

## Build

```
cargo build --release

docker build -t docker-monitor .

docker run -d --name docker-monitor \
  -v /var/run/docker.sock:/var/run/docker.sock \
  docker-monitor
```