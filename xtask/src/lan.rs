use std::path::{Path, PathBuf};

use master_protocol::PROTOCOL_VERSION;
use serde_json::json;

use crate::certs::{Ca, is_ipv4};
use crate::dotenv::Env;
use crate::shell::{Res, Step};
use crate::{bundle_zip, release, windows};

const DEFAULT_PORT: u16 = 4433;
const DEFAULT_PLACEHOLDER_IP: &str = "192.168.1.100";
const SERVER_NAME: &str = "iw4l-lan";

struct Options {
    port: u16,
    placeholder_ip: String,
}

pub fn run_cli(root: &Path, env: &Env, args: &[String]) -> Res<()> {
    let options = parse_args(args)?;
    let profile = windows::profile(env)?;
    let step = Step::start(
        "build.lan",
        &format!(
            "profile={profile} port={} placeholder_ip={}",
            options.port, options.placeholder_ip
        ),
    );

    let game = windows::build(&profile)?.game;
    let master = release::build_master(root, &profile)?;
    let out = root.join("dist/lan");
    let identity = Ca::new(out.join("identity"));
    identity.ensure_dns(SERVER_NAME)?;
    let (players, host, archive) = prepare_layout(&out)?;
    let ca_pem = read_trimmed(&identity.ca_cert())?;

    write_players(
        &players,
        &game,
        &ca_pem,
        &options.placeholder_ip,
        options.port,
    )?;
    write_host(
        root,
        &host,
        &game,
        &master,
        &identity,
        &ca_pem,
        options.port,
    )?;
    write_player_archive(&archive, &players)?;

    set_mode(&host, 0o700)?;
    identity_guard(&out)?;
    step.done(&format!(
        "players={} host={} archive={}",
        players.display(),
        host.display(),
        archive.display()
    ));
    println!(
        "LAN players: {} (archive password: {})",
        archive.display(),
        String::from_utf8_lossy(release::ARCHIVE_PASSWORD)
    );
    println!(
        "LAN host: {} (contains server-key.pem; do not distribute)",
        host.display()
    );
    Ok(())
}

fn parse_args(args: &[String]) -> Res<Options> {
    let mut port = DEFAULT_PORT;
    let mut placeholder_ip = DEFAULT_PLACEHOLDER_IP.to_string();
    let mut index = 0;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--port" => {
                port = value
                    .parse()
                    .map_err(|_| format!("--port must be 1..=65535 (got {value:?})"))?;
                if port == 0 {
                    return Err("--port must be 1..=65535".into());
                }
            }
            "--placeholder-ip" => {
                if !is_ipv4(value) {
                    return Err(format!(
                        "--placeholder-ip must be an IPv4 address (got {value:?})"
                    ));
                }
                placeholder_ip = value.clone();
            }
            other => return Err(format!("unknown option {other}")),
        }
        index += 2;
    }
    Ok(Options {
        port,
        placeholder_ip,
    })
}

fn prepare_layout(out: &Path) -> Res<(PathBuf, PathBuf, PathBuf)> {
    let players = out.join("players");
    let host = out.join("host");
    let archive = out.join("iw4l-lan-players.zip");
    for path in [&players, &host] {
        if path.exists() {
            std::fs::remove_dir_all(path)
                .map_err(|error| format!("remove {}: {error}", path.display()))?;
        }
    }
    let _ = std::fs::remove_file(&archive);
    std::fs::create_dir_all(&players)
        .map_err(|error| format!("create {}: {error}", players.display()))?;
    std::fs::create_dir_all(host.join("updates"))
        .map_err(|error| format!("create {}: {error}", host.display()))?;
    set_mode(&players, 0o755)?;
    set_mode(&host, 0o700)?;
    Ok((players, host, archive))
}

fn write_players(
    players: &Path,
    game: &Path,
    ca_pem: &str,
    placeholder_ip: &str,
    port: u16,
) -> Res<()> {
    copy(game, &players.join("iw4l.exe"))?;
    write(
        &players.join(".env"),
        b"# Generated IW4L LAN player configuration.\nIW4L_GAMES=.\nIW4L_COMMUNITY=lan.iw4l-server\n",
    )?;
    let descriptor = descriptor(ca_pem, placeholder_ip, port, Some(placeholder_ip));
    updater::Community::parse(descriptor.as_bytes())?;
    write(&players.join("lan.iw4l-server"), descriptor.as_bytes())?;
    write(
        &players.join("LAN-README.txt"),
        player_readme(placeholder_ip, port).as_bytes(),
    )?;
    Ok(())
}

fn write_host(
    root: &Path,
    host: &Path,
    game: &Path,
    master: &Path,
    identity: &Ca,
    ca_pem: &str,
    port: u16,
) -> Res<()> {
    let master_dest = host.join("iw4l-master");
    copy(master, &master_dest)?;
    set_mode(&master_dest, 0o755)?;

    copy(&identity.server_cert(), &host.join("server-cert.pem"))?;
    copy(&identity.server_key(), &host.join("server-key.pem"))?;
    set_mode(&host.join("server-key.pem"), 0o600)?;
    write(&host.join("iw4l-ca.pem"), ca_pem.as_bytes())?;

    let descriptor = descriptor(ca_pem, "127.0.0.1", port, None);
    updater::Community::parse(descriptor.as_bytes())?;
    write(&host.join("lan.iw4l-server"), descriptor.as_bytes())?;

    let updates = host.join("updates");
    let blob = release::package_game(root, game, &updates)?;
    let client_identity = json!({
        "exe": blob.exe_sha,
        "size": blob.exe_size,
        "compressed_size": blob.blob_size,
        "protocol": PROTOCOL_VERSION,
    });
    let manifest = updater::Manifest {
        schema: 1,
        release: release::sha256_hex(client_identity.to_string().as_bytes())[..16].into(),
        protocol: PROTOCOL_VERSION,
        file: updater::ManifestFile {
            name: "iw4l.exe".into(),
            path: blob.name,
            sha256: blob.exe_sha,
            size: blob.exe_size,
            compressed_size: blob.blob_size,
        },
    };
    let mut manifest_text = toml::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    manifest_text.push('\n');
    write(&updates.join("manifest.toml"), manifest_text.as_bytes())?;

    let script = format!(
        "#!/bin/sh\nset -eu\ncd \"$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\"\nexec ./iw4l-master serve --bind \"0.0.0.0:{port}\" --cert server-cert.pem --key server-key.pem --updates updates\n"
    );
    let run = host.join("run-master.sh");
    write(&run, script.as_bytes())?;
    set_mode(&run, 0o755)?;
    write(&host.join("README.txt"), host_readme(port).as_bytes())?;
    Ok(())
}

fn descriptor(ca_pem: &str, host: &str, port: u16, placeholder: Option<&str>) -> String {
    let role = match placeholder {
        Some(placeholder) => format!(
            "Player descriptor. Replace EVERY {placeholder} with the host's LAN IPv4 in master.address and updates.url."
        ),
        None => "Host-side descriptor. It points at the master on this machine.".to_string(),
    };
    format!(
        "# IW4L LAN community descriptor.\n\
         # {role}\n\
         schema = 1\n\
         name = \"IW4L LAN\"\n\n\
         [master]\n\
         address = \"{host}:{port}\"\n\
         server_name = \"{SERVER_NAME}\"\n\n\
         [updates]\n\
         url = \"https://{host}:{port}/updates/manifest.toml\"\n\
         ca_pem = \"\"\"\n{ca_pem}\n\"\"\"\n"
    )
}

fn player_readme(placeholder_ip: &str, port: u16) -> String {
    format!(
        "IW4L LAN player bundle\n\
         ======================\n\n\
         1. Extract this folder into steamapps/common/IW4L, or any writable folder\n\
            beside the Modern Warfare 2 and Black Ops II installations.\n\
         2. Edit lan.iw4l-server in a text editor. Replace every\n\
            {placeholder_ip} with the host machine's LAN IPv4 address.\n\
            There are two occurrences: master.address and updates.url.\n\
         3. Launch iw4l.exe and join the host's room.\n\n\
         The host must allow TCP and UDP port {port} through its firewall.\n\
         The .env file is generated and keeps game paths relative to this folder.\n"
    )
}

fn host_readme(port: u16) -> String {
    format!(
        "IW4L LAN host kit\n\
         =================\n\n\
         1. Run ./run-master.sh on this Linux machine.\n\
         2. Allow TCP and UDP port {port} through the firewall.\n\
         3. Tell players the host's LAN IPv4. They replace the placeholder IP in\n\
            their lan.iw4l-server twice: master.address and updates.url.\n\
         4. If the game runs on this same machine, use the included\n\
            lan.iw4l-server; it points at 127.0.0.1.\n\n\
         The updates/ directory contains a manifest for the exact player executable.\n\
         Keep server-key.pem private. Never copy this host directory into a player zip.\n"
    )
}

fn write_player_archive(archive: &Path, players: &Path) -> Res<()> {
    let files = ["iw4l.exe", ".env", "lan.iw4l-server", "LAN-README.txt"]
        .into_iter()
        .map(|name| players.join(name).display().to_string())
        .collect::<Vec<_>>();
    bundle_zip::write_archive(archive, &files, release::ARCHIVE_PASSWORD)
}

fn identity_guard(out: &Path) -> Res<()> {
    write(
        &out.join("identity/README.txt"),
        b"PRIVATE LAN identity. Do not distribute this directory.\n\
          ca-key.pem can mint another server certificate accepted by the players.\n\
          Keep server-key.pem only in the host kit.\n",
    )
}

fn read_trimmed(path: &Path) -> Res<String> {
    std::fs::read_to_string(path)
        .map(|text| text.trim().to_string())
        .map_err(|error| format!("read {}: {error}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Res<()> {
    std::fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn copy(from: &Path, to: &Path) -> Res<()> {
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|error| format!("copy {} -> {}: {error}", from.display(), to.display()))
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Res<()> {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|error| format!("chmod {mode:o} {}: {error}", path.display()))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Res<()> {
    Ok(())
}
