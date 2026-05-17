use crate::Result;
use crate::daemon_id::DaemonId;
use crate::daemon_list::DaemonListEntry;
use crate::daemon_status::DaemonStatus;
use demand::{DemandOption, MultiSelect, Select};

/// Displayable item for the daemon picker.
pub struct PickerItem<'a> {
    pub id: &'a DaemonId,
    pub status: Option<&'a DaemonStatus>,
    pub description: Option<String>,
}

impl<'a> PickerItem<'a> {
    fn label(&self) -> String {
        match self.status {
            Some(status) => format!("{} ({})", self.id.qualified(), status),
            None => self.id.qualified(),
        }
    }
}

/// Prompt the user to select a single daemon via an interactive picker.
///
/// Returns `Ok(Some(id))` if a daemon was selected, `Ok(None)` if the user
/// cancelled (e.g. Esc), or an error if the terminal is not interactive.
pub fn select_single<'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    items: &[PickerItem<'a>],
) -> Result<Option<&'a DaemonId>> {
    if items.is_empty() {
        return Ok(None);
    }

    let title = title.into();
    let description = description.into();
    let mut select = Select::new(title)
        .description(&description)
        .filtering(true)
        .filterable(true);

    for item in items {
        let label = item.label();
        let opt = DemandOption::new(item.id.qualified()).label(&label);
        let opt = if let Some(desc) = &item.description {
            opt.description(desc)
        } else {
            opt
        };
        select = select.option(opt);
    }

    match select.run() {
        Ok(qualified) => {
            let found = items.iter().find(|i| i.id.qualified() == qualified);
            Ok(found.map(|i| i.id))
        }
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => Ok(None),
        Err(e) => Err(miette::miette!("picker failed: {e}")),
    }
}

/// Prompt the user to select multiple daemons via an interactive picker.
///
/// Returns `Ok(Some(ids))` if at least one daemon was selected, `Ok(None)` if
/// the user cancelled, or an error if the terminal is not interactive.
pub fn select_multiple<'a>(
    title: impl Into<String>,
    description: impl Into<String>,
    items: &[PickerItem<'a>],
) -> Result<Option<Vec<&'a DaemonId>>> {
    if items.is_empty() {
        return Ok(None);
    }

    let title = title.into();
    let description = description.into();
    let mut select = MultiSelect::new(title)
        .description(&description)
        .filtering(true)
        .filterable(true);

    for item in items {
        let label = item.label();
        let opt = DemandOption::new(item.id.qualified()).label(&label);
        let opt = if let Some(desc) = &item.description {
            opt.description(desc)
        } else {
            opt
        };
        select = select.option(opt);
    }

    match select.run() {
        Ok(qualifieds) => {
            let ids: Vec<&'a DaemonId> = qualifieds
                .iter()
                .filter_map(|q| items.iter().find(|i| i.id.qualified() == *q).map(|i| i.id))
                .collect();
            Ok(Some(ids))
        }
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => Ok(None),
        Err(e) => Err(miette::miette!("picker failed: {e}")),
    }
}

/// Build picker items from daemon list entries, showing status when available.
pub fn items_from_entries(entries: &[DaemonListEntry]) -> Vec<PickerItem<'_>> {
    entries
        .iter()
        .map(|e| PickerItem {
            id: &e.id,
            status: Some(&e.daemon.status),
            description: None,
        })
        .collect()
}
