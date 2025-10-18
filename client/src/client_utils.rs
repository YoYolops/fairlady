// Event adapter is a module that provides the translation between an application event
// and the NimbusProtocol, so it 

use core::{
    logger,
    Result,
    nimbus_protocol::{InterApplicationRequest, NimbusProtocol},
};
use anyhow::{bail};
use notify::{
    Event,
    event::{ModifyKind, RenameMode},
    EventKind::{Create, Modify, Remove},
};

pub async fn create_request_from_event(event: Event) -> Result<NimbusProtocol> {
    let request_protocol = match event.kind {
        Create(_) => {
            NimbusProtocol::Request(
                InterApplicationRequest::CREATE {
                    path: String::from("path"),
                    data: Vec::from(b"hello")
                }
            )
        },
        Modify(modify_kind) => {
            match modify_kind {
                ModifyKind::Metadata(_) => bail!("Invalid modify event kind: ModifyKind::Metadata"),
                ModifyKind::Name(rename_mode) => {
                    if let RenameMode::Both = rename_mode {
                        NimbusProtocol::Request(
                            InterApplicationRequest::RENAME {
                                current_path: format!("{:?}", event.paths[0]),
                                new_path: format!("{:?}", event.paths[1].to_str())
                            }
                        )
                    } else {
                        if let RenameMode::From = rename_mode {
                            logger::info(String::from("DETECTED Modify(Name(From)), possible dele by moving to trash"));
                        }
                        bail!("Cannot generate rename request for an event that doesn't hold all needed values: {:?}", rename_mode)
                    }
                },
                ModifyKind::Data(_) => {
                    NimbusProtocol::Request(
                        InterApplicationRequest::UPDATE {
                            path: String::from("path"),
                            data: Vec::from(b"hello")
                        }
                    )
                },
                _ => bail!("Irrelevant modify event. Doesn't have equivalent NimbusProtocol variant"),
            }

        },
        Remove(_) => {
            NimbusProtocol::Request(
                InterApplicationRequest::DELETE(format!("{:?}", event.paths[0]))
            )
        },
        _ => bail!("This event was neither Create, Modify or Remove, therefore it doesn't have equivalent NimbusProtocol variant"),
    };
    Ok(request_protocol)
}

// pub async fn encode_request_protocol() -> Vec<u8> {
//     Vec::from(b"hello")
// }

// pub async fn encrypt_data() {

// }