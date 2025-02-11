mod rocksdb;
use crate::Error;
use mezmo::MezmoContext;

pub(crate) use rocksdb::RocksDBConnection;
pub(crate) use rocksdb::RocksDBPersistenceConnection;

/// The [PersistenceConnection] trait defines the specifics on how to create the state that connects
/// to the persistence layer, e.g. a DB connection, that can then be used for individual operations.
/// Objects that implement this trait should expect to live for the life of the component that owns
/// them.
pub(crate) trait PersistenceConnection: Send + Sync + std::fmt::Debug {
    /// An associated function that creates a new [PersistenceConnection] given a connection string
    /// to the specific data store and a [MezmoContext] that restricts data storage to a given named
    /// component. Components without a valid [MezmoContext] are currently not eligible for persistence.
    fn new(base_path: &str, ctx: &MezmoContext) -> Result<Self, Error>
    where
        Self: Sized;

    /// An associated function that creates a new [PersistenceConnection] given a connection string
    /// to the specific data store and a [MezmoContext] that restricts data storage to a given named
    /// component. Components without a valid [MezmoContext] are currently not eligible for persistence.
    fn new_with_ttl(base_path: &str, ctx: &MezmoContext, ttl: u64) -> Result<Self, Error>
    where
        Self: Sized;

    /// Fetches the value associated with key. An implied namespace is enforced from the values in
    /// the MezmoContext instance supplied as part of the [new] function - i.e. the account_id,
    /// pipeline_id, and component_id. Sharing data across components is not permitted. If the key
    /// does not exist in the store, Ok(None) is returned instead of an error.
    fn get(&self, key: &str) -> Result<Option<String>, Error>;

    /// Stores a value and associates it with key. An implied namespace is enforced from the values
    /// in the MezmoContext instance supplied as part of the [new] function - i.e. the account_id,
    /// pipeline_id, and component_id. Sharing data across components is not permitted.
    fn set(&self, key: &str, value: &str) -> Result<(), Error>;

    /// Deletes a key within the rocks db. An implied namespace is enforced from the values
    /// in the MezmoContext instance supplied as part of the [new] function - i.e. the account_id,
    /// pipeline_id, and component_id. Sharing data across components is not permitted.
    fn delete(&self, key: &str) -> Result<(), Error>;
}
