use simdnbt::owned::*;
use std::fmt::Debug;

/// Creates a text component message with a token as input  
pub trait MessageGenerator: Debug + Clone {
    fn create_message(self, token: &str) -> NbtTag;
}

/// The default message template displayed upon disconnect  
#[derive(Debug, Clone)]
pub struct Message;
impl MessageGenerator for Message {
    fn create_message(self, token: &str) -> NbtTag {
        let mut text = NbtCompound::new();
        text.insert("text", NbtTag::String("Token: ".into()));

        let mut token_txt = NbtCompound::new();
        token_txt.insert("text", NbtTag::String(token.into()));
        token_txt.insert("color", NbtTag::String("#36bf5a".into()));

        let mut desc = NbtCompound::new();
        desc.insert(
            "text",
            NbtTag::String("\n\nUse this to link your\nminecraft account".into()),
        );
        desc.insert("color", "#919191");

        NbtTag::List(NbtList::Compound(vec![text, token_txt, desc]))
    }
}
