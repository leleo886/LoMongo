#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Mongo(#[from] mongodb::error::Error),
    #[error(transparent)]
    MongoDateTime(#[from] mongodb::bson::datetime::Error),
    #[error(transparent)]
    MongoSer(#[from] mongodb::bson::ser::Error),
    #[error(transparent)]
    MongoBSON(#[from] mongodb::bson::de::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Json5(#[from] json5::Error),
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    #[error(transparent)]
    Aes(#[from] aes_gcm::aes::cipher::InvalidLength),
    #[error(transparent)]
    AsyncSsh2(#[from] async_ssh2_tokio::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
