#[derive(Debug, Clone)]
pub struct TokenStream {
    source: String,
}

pub fn tokenize(source: &str) -> TokenStream {
    TokenStream {
        source: source.to_owned(),
    }
}

impl TokenStream {
    pub fn into_source(self) -> String {
        self.source
    }
}
