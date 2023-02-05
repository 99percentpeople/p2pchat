pub trait MessageStore {
    type Message;
    type Error;

    fn get(&self, id: &str) -> Result<Option<Self::Message>, Self::Error>;
    fn put(&mut self, id: &str, message: Self::Message) -> Result<(), Self::Error>;
    fn remove(&mut self, id: &str) -> Result<Option<Self::Message>, Self::Error>;
}
