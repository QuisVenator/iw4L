# LAN bundle

```bash
make setup-windows                 # once
make lan                           # PROFILE=play
make lan PROFILE=release
make lan LAN_ARGS='--port 4433 --placeholder-ip 192.168.1.100'
```

`make lan` writes `dist/lan/`:

```text
dist/lan/
├── identity/                     private CA and keys; never distribute
├── players/                      unpacked player files
├── iw4l-lan-players.zip          password: t.me/contextrot
└── host/                         private master kit; never distribute
```

The player ZIP contains `iw4l.exe`, `.env`, `lan.iw4l-server` and
`LAN-README.txt`. `.env` keeps game discovery relative to the extracted folder.
Edit `lan.iw4l-server` in **two** places, replacing the placeholder with the
host's LAN IPv4:

```toml
[master]
address = "192.168.1.100:4433"
[updates]
url = "https://192.168.1.100:4433/updates/manifest.toml"
```

The included certificate names only `server_name = "iw4l-lan"`; it does not
contain the host IP, so changing the address does not invalidate the identity.
The descriptor keeps the same CA and server name, and the client verifies that
name after connecting to the edited address.

On the host, `host/run-master.sh` starts the Linux master on `0.0.0.0:4433`.
Allow both TCP and UDP `4433`. `host/lan.iw4l-server` points at `127.0.0.1`
for a game running on the master machine. `host/updates/` contains a manifest
and blob for the exact player executable, so launches do not wait on a failed
update check.

Keep `host/server-key.pem` and `identity/ca-key.pem` private. Delete the whole
`identity/` directory to rotate the LAN certificate; old player descriptors
will then stop trusting the new master.

There is no automatic LAN discovery. Players receive the modified descriptor
with the ZIP, and the host must be reachable by its LAN IPv4 address.
