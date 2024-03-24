use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppErrors<'a> {
    #[error("Missing enviroment variables: {0}")]
    MissingEvironmentVariables(String),
    #[error("Failed parsing variable `{0}` with reason: {1}")]
    InvalidEvironmentVariable(String, &'a str),
    #[error("Too many query params: `{0}`, expected 0")]
    TooManyQueryParams(usize),
    #[error("Missing header: `{0}`")]
    MissingHeader(&'a str),
    #[error("Header invalid format: `{0}`")]
    HeaderInvalidFormatError(&'a str),
    #[error("Header parsing failed: `{0}`")]
    HeaderParsingError(&'a str),
    #[error("Failed to validate signature: `{0}`")]
    SignatureError(&'a str),
    #[error("Failed to parse payload")]
    InvalidPayload(),
    #[error("Could not deserialise installation file: `{0}`")]
    InvalidDeserializationInstallationFile(String),
    #[error("Could save installation file: `{0}`")]
    FailedToSaveInstallationFile(String),
    #[error("Could not process JWT: `{0}`")]
    FailedToProcessJWD(String),
    #[error("Found problems during API call: `{0}`")]
    ApiFailure(String),
    //#[error("invalid header (expected {expected:?}, found {found:?})")]
    //InvalidHeader { expected: String, found: String },
    //#[error("unknown data store error")]
    //Unknown,
}
