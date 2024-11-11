use clap::Parser;

#[derive(Parser, Debug)]
pub struct Options {
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    #[arg(short, long, default_value_t = 8001)]
    pub port: u16,
}

impl Options {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
