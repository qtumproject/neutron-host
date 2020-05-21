extern crate neutron_star_constants;
use std::collections::HashMap;
//use std::collections::HashSet;
use neutron_star_constants::*;

pub const NEUTRONDB_USER_SPACE: u8 = '_' as u8;

#[derive(Debug)]
pub enum NeutronDBError{
    Recoverable, // ever used?
    Unrecoverable
}
pub trait NeutronDB{
    fn read_key(&mut self, address: &NeutronShortAddress, key: &[u8]) -> Result<Vec<u8>, NeutronDBError>;
    fn write_key(&mut self, address: &NeutronShortAddress, key: &[u8], value: &[u8]) -> Result<(), NeutronDBError>;
    /// Creates a new checkpoint which enables the ability to revert back to the current state
    /// Returns the number of current checkpoints within the database context
    fn checkpoint(&mut self) -> Result<u32, NeutronDBError>;
    /// Collapses all outstanding checkpoints into a single top level checkpoint
    fn collapse_checkpoints(&mut self) -> Result<(), NeutronDBError>;
    /// Reverts the current state to the previous checkpoint, discarding the modifications made since that checkpoint
    fn revert_checkpoint(&mut self) -> Result<u32, NeutronDBError>;
    fn clear_checkpoints(&mut self);
    /// Commits all state to the database 
    /// TBD: should this be left as a non-trait function??
    fn commit(&mut self) -> Result<(), NeutronDBError>;
    //fn compute_new_proofs(&mut self, )
    // Automatically will execute `collapse_checkpoints`. Returns the keys and values which were read in this context as well as the keys which were written to
    //fn compute_state_differences(&mut self, reads: HashMap<NeutronShortAddress, HashMap<Vec<u8>, Vec<u8>>>, writes: HashMap<NeutronShortAddress, HashMap<Vec<u8>, Vec<u8>>>)
    //    -> Result<(), NeutronDBError>;
}
#[derive(Default,  Debug, Clone)]
pub struct ProtoDB{
    storage: HashMap<NeutronShortAddress, HashMap<Vec<u8>, Vec<u8>>>,
    /// This only tracks keys which are read from storage, and ignores checkpoint-only data and reverts
    //touched: HashMap<NeutronShortAddress, Vec<u8>>,
    //rents: HashMap<Vec<u8>, u32>,
    checkpoints: Vec<HashMap<NeutronShortAddress, HashMap<Vec<u8>, Vec<u8>>>>
}
impl NeutronDB for ProtoDB{
    fn read_key(&mut self, address: &NeutronShortAddress, key: &[u8]) -> Result<Vec<u8>, NeutronDBError>{
        for checkpoint in self.checkpoints.iter().rev(){
            match checkpoint.get(address){
                Some(kv) => {
                    match kv.get(key){
                        Some(v) => {return Ok(v.to_vec());},
                        None => {}
                    }
                },
                None => {}
            }
        }
        match self.storage.get(address){
            Some(kv) => {
                match kv.get(key){
                    Some(v) => {return Ok(v.to_vec());},
                    None => {}
                }
            },
            None => {}
        }
        Err(NeutronDBError::Unrecoverable)
    }
    fn write_key(&mut self, address: &NeutronShortAddress, key: &[u8], value: &[u8]) -> Result<(), NeutronDBError>{
        if self.checkpoints.len() == 0{
            return Err(NeutronDBError::Recoverable);
        }
        let c = self.checkpoints.last_mut().unwrap();
        match c.get_mut(address){
            Some(kv) => {
                kv.insert(key.to_vec(), value.to_vec());
            },
            None => {
                let mut t = HashMap::new();
                t.insert(key.to_vec(), value.to_vec());
                c.insert(*address, t);
            }
        }
        Ok(())
    }
    fn checkpoint(&mut self) -> Result<u32, NeutronDBError>{
        self.checkpoints.push(HashMap::new());
        Ok(self.checkpoints.len() as u32)
    }
    fn revert_checkpoint(&mut self) -> Result<u32, NeutronDBError>{
        if self.checkpoints.pop().is_none(){
            Err(NeutronDBError::Unrecoverable)
        }else{
            Ok(self.checkpoints.len() as u32)
        }
    }
    fn collapse_checkpoints(&mut self) -> Result<(), NeutronDBError>{
        let mut collapsed = HashMap::new();
        for kv in self.checkpoints.iter_mut(){
            for (key, value) in kv.drain(){
                collapsed.insert(key, value);
            }
        }
        self.checkpoints.clear();
        self.checkpoints.push(collapsed);
        
        Ok(())
    }
    fn commit(&mut self) -> Result<(), NeutronDBError>{
        self.collapse_checkpoints()?;
        for (key, value) in self.checkpoints.last_mut().unwrap().drain(){
            self.storage.insert(key, value);
        }
        self.clear_checkpoints();
        Ok(())
    }
    fn clear_checkpoints(&mut self){
        self.checkpoints.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic(){
        let mut a = NeutronShortAddress::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.write_key(&a, &[1], &[9, 9, 9, 9]).is_ok());
        let v = db.read_key(&a, &[1]).unwrap();
        assert!(v == vec![9, 9, 9, 9]);
    }
    
    #[test]
    fn test_checkpoints(){
        let mut a = NeutronShortAddress::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&a, &[1], &[9, 9, 9, 9]).is_ok());
        assert!(db.revert_checkpoint().is_ok());
        let v = db.read_key(&a, &[1]).unwrap();
        assert!(v == vec![8, 8, 8, 8]);
    }
    
    #[test]
    fn test_storage(){
        let mut a = NeutronShortAddress::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.commit().is_ok());
        assert!(db.revert_checkpoint().is_err());
        let v = db.read_key(&a, &[1]).unwrap();
        assert!(v == vec![8, 8, 8, 8]);
    }
    
}