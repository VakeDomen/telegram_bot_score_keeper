use teloxide::types::{Message, MessageKind, MediaKind};

pub fn extract_message_text(message: &Message) -> Option<String> {
    let mes = match &message.kind {
        MessageKind::Common(mes) => mes,
        _ => return None,
    };
    // extract media (text)
    let media = match &mes.media_kind {
        MediaKind::Text(media) => media,
        _ => return None,
    };
    Some(media.text.clone())
}