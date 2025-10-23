// FS adapter adapts notify events from the file system to NimbusProtocol
// Every FS event that should be reflected in the server will generate a proper
// NimbusProtocol instance. This is a SYNC module (at least by now)

use anyhow::{Ok, bail};
use core::{
    AnyResult, logger,
    fs_handler,
    nimbus_protocol::{InterApplicationRequest, NimbusProtocol},
};
use std::path::PathBuf;
use notify::{
    Event,
    EventKind::{Create, Modify, Remove},
    event::{ModifyKind, RenameMode},
};

pub async fn create_request_from_event(event: &Event) -> AnyResult<NimbusProtocol> {
    let request_protocol = match event.kind {
        Create(_) => build_request_from_create_event(&event).await?,
        Modify(modify_kind) => build_request_from_modify_event(&modify_kind, &event).await?,
        Remove(_) => build_request_from_remove_event(event),
        _ => bail!(
            "This event was neither Create, Modify or Remove, therefore it doesn't have equivalent NimbusProtocol variant"
        ),
    };
    Ok(request_protocol)
}

async fn build_request_from_create_event(event: &Event) -> AnyResult<NimbusProtocol> {
    Ok(NimbusProtocol::Request(InterApplicationRequest::CREATE {
        path: format!("{:?}", event.paths[0]),
        data: fs_handler::fetch_fs_data(&event.paths[0]).await?,
    }))
}

async fn build_request_from_modify_event(
    modify_kind: &ModifyKind,
    event: &Event,
) -> AnyResult<NimbusProtocol> {
    let protocol = match modify_kind {
        ModifyKind::Metadata(_) => bail!("Invalid modify event kind: ModifyKind::Metadata"),
        ModifyKind::Name(rename_mode) => {
            if let RenameMode::Both = rename_mode {
                NimbusProtocol::Request(InterApplicationRequest::RENAME {
                    current_path: format!("{:?}", event.paths[0]),
                    new_path: format!("{:?}", event.paths[1]),
                })
            } else {
                if let RenameMode::From = rename_mode {
                    logger::info(String::from(
                        ">>> DETECTED Modify(Name(From)), possible delete by moving to trash <<<",
                    ));
                }
                bail!(
                    "Cannot generate rename request for an event that doesn't hold all needed values: {:?}",
                    rename_mode
                )
            }
        }
        ModifyKind::Data(_) => NimbusProtocol::Request(InterApplicationRequest::UPDATE {
            path: format!("{:?}", event.paths[0]),
            data: fs_handler::fetch_fs_data(&event.paths[0]).await?,
        }),
        _ => bail!("Irrelevant modify event. Doesn't have equivalent NimbusProtocol variant"),
    };
    Ok(protocol)
}

fn build_request_from_remove_event(event: &Event) -> NimbusProtocol {
    NimbusProtocol::Request(
        InterApplicationRequest::DELETE(PathBuf::from(&event.paths[0]))
    )
}
