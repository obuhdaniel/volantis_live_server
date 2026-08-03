#[derive(Debug, Clone)]
pub struct Config {
    pub livekit_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub bind_addr: String,
    pub s3_access_key: String,
    pub s3_secret: String,
    pub s3_region: String,
    pub s3_bucket: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            livekit_url:  std::env::var("LIVEKIT_URL")?,
            api_key:      std::env::var("LIVEKIT_API_KEY")?,
            api_secret:   std::env::var("LIVEKIT_API_SECRET")?,
            bind_addr:    std::env::var("BIND_ADDR")
                              .unwrap_or_else(|_| "0.0.0.0:3000".into()),
            s3_access_key: std::env::var("S3_ACCESS_KEY")?,
            s3_secret:     std::env::var("S3_SECRET")?, 
            s3_region:     std::env::var("S3_REGION")?,
            s3_bucket:     std::env::var("S3_BUCKET")?,
        })
    }
}