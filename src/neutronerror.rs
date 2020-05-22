use std::fmt;
use std::error;



#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnrecoverableError{
    NotImplemented,
    StateOutOfRent,
    ContextIndexEmpty,
    UnknownVM,
    DatabaseCommitError,
    DatabaseWritingError,
    ErrorInitializingVM

}

//TODO: this later needs to be moved/copied to neutron-constants for sharing with neutron-star
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoverableError{
    StackIndexDoesntExist = 0x8000_0001,
    StackItemTooLarge,
    InvalidSystemFunction,
    InvalidSystemFeature,
    ErrorCopyingIntoVM,
    ErrorCopyingFromVM,
    ContractSignaledError,
    ContractExecutionError,
    InvalidHypervisorInterrupt

}

//TODO: add error codes for recoverable failures
/// The primary error structure of NeutronAPI calls
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutronError{
	/// An error has occured, but if the VM implements an error handling system, it is appropriate to allow this error
    /// to be handled by the smart contract and for execution to continue
	Recoverable(RecoverableError),
    /// An error has occured and the VM should immediately terminate, not allowing the smart contract to detect or handle this error in any capacity
    Unrecoverable(UnrecoverableError)
}

impl fmt::Display for NeutronError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NeutronError::Recoverable(e) => {
                write!(f, "Recoverable Failure! {:?}", e)
            },
            NeutronError::Unrecoverable(e) => {
                write!(f, "Unrecoverable Failure! {:?}", e)
            }
        }
    }
}

impl error::Error for NeutronError{
}



