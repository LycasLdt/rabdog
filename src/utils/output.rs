pub struct OutputSession;

pub enum Notification<'a> {
    Setup(&'a str, &'a str),
    StartGettingInfo,
    FinishGettingInfo(&'a str, Vec<&'a str>),
}