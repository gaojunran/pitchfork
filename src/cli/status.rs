use crate::Result;
use crate::cli::list::build_proxy_url;
use crate::cli::picker;
use crate::daemon_list::get_all_daemons;
use crate::ipc::client::IpcClient;
use crate::pitchfork_toml::PitchforkToml;
use crate::settings::settings;
use crate::state_file::StateFile;

/// Display the status of a daemon
#[derive(Debug, clap::Args)]
#[clap(
    visible_alias = "stat",
    verbatim_doc_comment,
    long_about = "\
Display the status of a daemon

Shows detailed information about a single daemon including its PID and
current status (running, stopped, failed, etc.).

Example:
  pitchfork status api

Output:
  Name: api
  PID: 12345
  Status: running"
)]
pub struct Status {
    /// Name of the daemon to check
    pub id: Option<String>,
}

impl Status {
    pub async fn run(&self) -> Result<()> {
        let qualified_id = if let Some(ref id) = self.id {
            PitchforkToml::resolve_id(id)?
        } else {
            // No argument provided — show interactive picker
            let client = IpcClient::connect(true).await?;
            let entries = get_all_daemons(&client).await?;
            let items = picker::items_from_entries(&entries);
            match picker::select_single(
                "Select a daemon",
                "↑/↓ to navigate, enter to confirm",
                &items,
            )? {
                Some(id) => id.clone(),
                None => return Ok(()),
            }
        };
        let global_slugs = settings()
            .proxy
            .enable
            .then(PitchforkToml::read_global_slugs)
            .unwrap_or_default();

        let daemon = StateFile::get().daemons.get(&qualified_id);
        if let Some(daemon) = daemon {
            println!("Name: {qualified_id}");
            if let Some(pid) = &daemon.pid {
                println!("PID: {pid}");
            }
            println!("Status: {}", daemon.status.style());
            // Show active port if available
            if let Some(port) = daemon.active_port {
                println!("Port: {port} (active)");
            } else if !daemon.resolved_port.is_empty() {
                let ports = daemon
                    .resolved_port
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("Port: {ports}");
            }
            // Show proxy URL only when the proxy server is globally enabled AND the daemon
            // has a port (active or resolved).  Without a port the proxy cannot route to this
            // daemon, so printing a URL would be misleading — matching the behaviour of `list`.
            let s = settings();
            if s.proxy.enable && (daemon.active_port.is_some() || !daemon.resolved_port.is_empty())
            {
                let slug =
                    PitchforkToml::find_slug_for_daemon_in_registry(&qualified_id, &global_slugs);
                if let Some(url) = build_proxy_url(slug.as_deref(), s) {
                    println!("Proxy: {url}");
                }
            }
        } else {
            miette::bail!("Daemon {} not found", qualified_id);
        }
        Ok(())
    }
}
