use std::collections::HashMap;

#[derive(Clone, Default)]
struct ProtoDB {
    storage: HashMap<Vec<u8>, Vec<u8>>,
    //rents: HashMap<Vec<u8>, u32>,
    checkpoints: Vec<HashMap<Vec<u8>, Vec<u8>>>
}

impl ProtoDB{ 
    fn build_key(&self, prefix: &[u8], key: &[u8]) -> Vec<u8>{
        let mut result: Vec<u8> = prefix.to_vec();
        result.append(&mut key.to_vec());
        result
    }
    fn read_key_internal(&self, key: &Vec<u8>) -> Result<Vec<u8>, NeutronDBError>{
        for checkpoint in self.checkpoints.iter().rev(){
            match checkpoint.get(key){
                Some(v) => return Ok(v),
                None => {}
            }
        }
        match self.storage.get(key){
            Some(v) => Ok(v),
            None => Err(NeutronDBError::Unrecoverable)
        }
    }
    fn write_key_internal(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), NeutronDBError>{
        if self.checkpoints.len() == 0{
            return Err(NeutronDBError::Recoverable);
        }
        let mut c = self.checkpoints.last_mut().unwrap();
        c.insert(key, value);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutronDBError{
	/// Success, no error has occured
	Success,
	/// An error has occured, but if the VM implements an error handling system, it is appropriate to allow this error
    /// to be handled by the smart contract and for execution to continue
	Recoverable,
    /// An error has occured and the VM should immediately terminate, not allowing the smart contract to detect or handle this error in any capacity
    Unrecoverable
}

impl NeutronDB for ProtoDB {
	fn read_key(&mut self, address: &[u8], key: &[u8]) -> Result<&mut Vec<u8>, NeutronDBError> {
		let prefixed_key = "unprotected_".as_bytes().to_vec();
		prefixed_key.append(&mut address.to_vec()); 
		prefixed_key.append(&mut key.to_vec());
		return self.read_key_internal(&prefixed_key);
	}
	fn read_protected_key(&mut self, address: &[u8], key: &[u8]) -> Result<Vec<u8>, NeutronDBError> {
		let prefixed_key = "protected_".as_bytes().to_vec();
		prefixed_key.append(&mut address.to_vec()); 
		prefixed_key.append(&mut key.to_vec());
		return self.read_key_internal(prefixed_key);
	}
	fn write_key(&mut self, key: &[u8], address: &[u8], value: &[u8]) -> Result<(), NeutronDBError> {
		let prefixed_key = "unprotected_".as_bytes().to_vec();
		prefixed_key.append(&mut address.to_vec()); 
		prefixed_key.append(&mut key.to_vec());
		return self.write_key_internal(prefixed_key, value.to_vec());		
	}
    fn write_protected_key(&mut self, address: &[u8], key: &[u8], value: &[u8]) -> Result<(), NeutronDBError> {
		let prefixed_key = "protected_".as_bytes().to_vec();
		prefixed_key.append(&mut address.to_vec()); 
		prefixed_key.append(&mut key.to_vec());
		return self.write_key_internal(prefixed_key, value.to_vec());
	}
}

pub trait NeutronDB{
    fn read_key(&mut self, address: &[u8], key: &[u8]) -> Result<&mut Vec<u8>, NeutronDBError>;
    fn read_protected_key(&mut self, address: &[u8], key: &[u8]) -> Result<Vec<u8>, NeutronDBError>;
    fn write_key(&mut self, key: &[u8], address: &[u8], value: &[u8]) -> Result<(), NeutronDBError>;
    fn write_protected_key(&mut self, address: &[u8], key: &[u8], value: &[u8]) -> Result<(), NeutronDBError>;
    /// Creates a new checkpoint which enables the ability to revert back to the current state
    /// Returns the number of current checkpoints within the database context
    fn checkpoint(&mut self) -> Result<u32, NeutronDBError>;
    /// Collapses all outstanding checkpoints into a single top level checkpoint
    fn collapse_checkpoints(&mut self) -> Result<(), NeutronDBError>;
    /// Reverts the current state to the previous checkpoint, discarding the modifications made since that checkpoint
    fn revert_checkpoint(&mut self) -> Result<u32, NeutronDBError>;
    /// Commits all state to the database
    /// TBD: should this be left as a non-trait function??
    fn commit(&mut self) -> Result<(), NeutronDBError>;
    /// Automatically will execute `collapse_checkpoints`. Returns the keys and values which were read in this context as well as the keys which were written to
    fn compute_state_differences(&mut self, reads: HashMap<String, String>, writes: HashMap<String, String>) -> Result<(), NeutronDBError>;
}