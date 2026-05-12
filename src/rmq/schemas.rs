use lapin::ExchangeKind;

/// Struct to construct exchange info
#[derive(Debug)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub exchange_type: ExchangeKind,
}

// Method converts exchange_type to an Enum ExchangeKind for Lapin handling
impl<'a> Exchange<'a> {
    pub fn new(
        name: &'a str,
        exchange_type: &str,
    ) -> Self {
        let lowercased_exchange_type = exchange_type.to_ascii_lowercase();
        let exchange_type = match lowercased_exchange_type.as_str() {
            "topic" => ExchangeKind::Topic,
            "fanout" => ExchangeKind::Fanout,
            "headers" => ExchangeKind::Headers,
            _ => ExchangeKind::Direct,
        };
        Self {
            name,
            exchange_type,
        }
    }
}

/// Struct to construct queue info
#[derive(Debug, Default)]
pub struct Queue<'a> {
    pub name: &'a str,
    pub routing_key: &'a str,
}

impl<'a> Queue<'a> {
    pub fn new(
        name: &'a str,
        routing_key: &'a str,
    ) -> Self {
        Self { name, routing_key }
    }
}
