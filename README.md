# WEB TERMINAL

A rust app for launching a web terminal that uses websocket.


## Run using ```cargo```
```sh
cargo run -- --host 0.0.0.0 --port 3032
```

# Build

```sh
cargo build --release
```

# Deploy

## Help Command:
```sh
cargo run -- --help

Parameters when running the web terminal app.

Usage: web-terminal [OPTIONS]

Options:
      --host <HOST>
          The ip of the server.
          
          [default: 127.0.0.1]

      --port <PORT>
          The port of the server.
          
          [default: 3030]

      --heartbeat-interval <HEARTBEAT_INTERVAL>
          The heartbeat interval.
          
          [default: 30]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Run using ```cargo```
```sh
cargo run -- --host 0.0.0.0 --port 3032
```

## Run using the compiled binary file.

Execute binary
```sh
./target/release/web-terminal --host 0.0.0.0 --port 3032
```

# Testing

You can simulate the web terminal using the provided frontend app.

1. Execute docker-compose for the frontend.
```sh
docker-compose -f docker-compose up -d
```

2. Look for the container of the frontend.
```sh
docker ps 

Example:
CONTAINER ID   IMAGE         COMMAND                  CREATED        STATUS        PORTS                  NAMES
85a2f260ffda   node:latest   "docker-entrypoint.s…"   20 hours ago   Up 20 hours   0.0.0.0:7060->80/tcp   web-terminal-termi-1
```

3. Grab the container id
```sh
docker exec -it e6b8bedb2193 "bash"
```

4. Run the development server.
```sh
yarn dev
```

5. Open browser then go to
```
http://localhost:7060
```

# Developer
JP Mateo (jpmateo022@gmail.com)