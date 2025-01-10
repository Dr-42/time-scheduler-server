# Time scheduler server

Server for the [Time Sceduler app](https://github.com/Dr-42/time-scheduler-client)

## Usage

```console
cargo install time-scheduler-server
time-scheduler-server <port>
```

## Migration from 0.\*

The server api and data base have new format. To migrate run

```sh
cd <path-to-time-scheduler-data-you-had>
time-scheduler-server migrate
```

This will create a new migrations folder

If you trust me enough

```sh
cd <path-to-time-scheduler-data-you-had>
time-scheduler-server migrate --overwrite
```
