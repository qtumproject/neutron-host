extern crate neutron_star_constants;
use std::collections::HashMap;
//use std::collections::HashSet;
use neutron_star_constants::*;
use crate::callstack::*;
use crate::neutronerror::*;
use crate::syscall_interfaces::storage::GLOBAL_STORAGE_FEATURE;

pub const NEUTRONDB_USER_SPACE: u8 = '_' as u8;

//Note, these are not the costs, these are the identifiers used within GasCharger
enum DBGasCost{
    ReadUncached = 1,
    ReadUncachedByte,
    ReadCachedByte,
    WriteUncached,
    WriteUncachedByte,
    WriteKeyByte,
    WriteCached,
    WriteCachedByte,
    RefundUncachedByte,
    RefundCachedByte,
}


pub trait NeutronDB{
    fn read_key(&mut self, stack: &mut ContractCallStack, address: &NeutronShortAddress, key: &[u8]) -> Result<Vec<u8>, NeutronError>;
    fn write_key(&mut self, stack: &mut ContractCallStack, address: &NeutronShortAddress, key: &[u8], value: &[u8]) -> Result<(), NeutronError>;
    /// Creates a new checkpoint which enables the ability to revert back to the current state
    /// Returns the number of current checkpoints within the database context
    fn checkpoint(&mut self) -> Result<u32, NeutronError>;
    /// Collapses all outstanding checkpoints into a single top level checkpoint
    fn collapse_checkpoints(&mut self) -> Result<(), NeutronError>;
    /// Reverts the current state to the previous checkpoint, discarding the modifications made since that checkpoint
    fn revert_checkpoint(&mut self) -> Result<u32, NeutronError>;
    fn clear_checkpoints(&mut self);
    /// Commits all state to the database 
    /// TBD: should this be left as a non-trait function??
    fn commit(&mut self) -> Result<(), NeutronError>;
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

fn gas_cost(stack: &ContractCallStack, costid: DBGasCost) -> i64{
    stack.gas_cost(GLOBAL_STORAGE_FEATURE, costid as u32)
}

fn gas_cost_read_uncached(stack: &ContractCallStack, size: usize) -> i64{
    gas_cost(stack, DBGasCost::ReadUncached) 
    + (gas_cost(stack, DBGasCost::ReadUncachedByte) * size as i64)
}
fn gas_cost_read_cached(stack: &ContractCallStack, size: usize) -> i64{
    gas_cost(stack, DBGasCost::ReadCachedByte) * size as i64
}
fn gas_cost_write_uncached(stack: &ContractCallStack, key_size: usize, value_size: usize, old_size: usize) -> i64{
    gas_cost(stack, DBGasCost::WriteUncached) 
    + (gas_cost(stack, DBGasCost::WriteUncachedByte) * value_size as i64)
    + (gas_cost(stack, DBGasCost::WriteKeyByte) * key_size as i64)
    - (gas_cost(stack, DBGasCost::RefundUncachedByte) * old_size as i64)
}
fn gas_cost_write_cached(stack: &ContractCallStack, value_size: usize, old_size: usize) -> i64{
    gas_cost(stack, DBGasCost::WriteCached) 
    + (gas_cost(stack, DBGasCost::WriteCachedByte) * value_size as i64)
    - (gas_cost(stack, DBGasCost::RefundCachedByte) * old_size as i64)
}

impl NeutronDB for ProtoDB{
    fn read_key(&mut self, stack: &mut ContractCallStack, address: &NeutronShortAddress, key: &[u8]) -> Result<Vec<u8>, NeutronError>{
        for checkpoint in self.checkpoints.iter().rev(){
            match checkpoint.get(address){
                Some(kv) => {
                    match kv.get(key){
                        Some(v) => {
                            let tmp = v.to_vec();
                            //charge for cached read
                            let cost = gas_cost_read_cached(stack, tmp.len());
                            stack.charge_gas(cost)?;
                            return Ok(tmp);
                        },
                        None => {}
                    }
                },
                None => {
                }
            }
        }
        match self.storage.get(address){
            Some(kv) => {
                match kv.get(key){
                    Some(v) => {
                        let tmp = v.to_vec();
                        //charge for uncached read
                        let cost = gas_cost_read_uncached(stack, tmp.len());
                        stack.charge_gas(cost)?;
                        return Ok(tmp);
                    },
                    None => {
                    }
                }
            },
            None => {
            }
        }
        Err(NeutronError::Unrecoverable(UnrecoverableError::StateOutOfRent))
    }
    fn write_key(&mut self, stack: &mut ContractCallStack, address: &NeutronShortAddress, key: &[u8], value: &[u8]) -> Result<(), NeutronError>{
        if self.checkpoints.len() == 0{
            return Err(NeutronError::Unrecoverable(UnrecoverableError::DatabaseWritingError));
        }
        let c = self.checkpoints.last_mut().unwrap();
        let k = key.to_vec();
        let v = value.to_vec();
        match c.get_mut(address){
            Some(kv) => {
                let old_size = match kv.get(&k){
                    None => {
                        0
                    },
                    Some(old) =>{
                        old.len()
                    }
                };
                let cost = gas_cost_write_cached(stack, v.len(), old_size);
                stack.charge_gas(cost)?;
                kv.insert(k, v);
            },
            None => {
                //charge for previous/cached write
                let old_size = 0; //TODO
                let cost = gas_cost_write_uncached(stack, k.len(), v.len(), old_size);
                stack.charge_gas(cost)?;
                let mut t = HashMap::new();
                t.insert(key.to_vec(), value.to_vec());
                c.insert(*address, t);
            }
        }
        Ok(())
    }
    fn checkpoint(&mut self) -> Result<u32, NeutronError>{
        self.checkpoints.push(HashMap::new());
        Ok(self.checkpoints.len() as u32)
    }
    fn revert_checkpoint(&mut self) -> Result<u32, NeutronError>{
        if self.checkpoints.pop().is_none(){
            return Err(NeutronError::Unrecoverable(UnrecoverableError::DatabaseCommitError));
        }else{
            Ok(self.checkpoints.len() as u32)
        }
    }
    fn collapse_checkpoints(&mut self) -> Result<(), NeutronError>{
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
    fn commit(&mut self) -> Result<(), NeutronError>{
        self.collapse_checkpoints()?;
        for (key, value) in self.checkpoints.last_mut().unwrap().drain(){
            match self.storage.get_mut(&key){
                None => {
                    self.storage.insert(key, value);
                },
                Some(kv) => {
                    for(k2, v2) in value{
                        kv.insert(k2, v2);
                    }
                }
            }
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
        let mut stack = ContractCallStack::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.write_key(&mut stack, &a, &[1], &[9, 9, 9, 9]).is_ok());
        let v = db.read_key(&mut stack, &a, &[1]).unwrap();
        assert!(v == vec![9, 9, 9, 9]);
    }
    
    #[test]
    fn test_checkpoints(){
        let mut a = NeutronShortAddress::default();
        let mut stack = ContractCallStack::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[1], &[9, 9, 9, 9]).is_ok());
        assert!(db.revert_checkpoint().is_ok());
        let v = db.read_key(&mut stack, &a, &[1]).unwrap();
        assert!(v == vec![8, 8, 8, 8]);
    }
    
    #[test]
    fn test_storage(){
        let mut a = NeutronShortAddress::default();
        let mut stack = ContractCallStack::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[1], &[8, 8, 8, 8]).is_ok());
        assert!(db.commit().is_ok());
        assert!(db.revert_checkpoint().is_err());
        db.clear_checkpoints();
        let v = db.read_key(&mut stack, &a, &[1]).unwrap();
        assert!(v == vec![8, 8, 8, 8]);
        db.clear_checkpoints();
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[1, 2, 3], &[9, 9, 9, 9]).is_ok());
        assert!(db.commit().is_ok());
        assert!(db.revert_checkpoint().is_err());
        assert!(db.checkpoint().is_ok());
        let v = db.read_key(&mut stack, &a, &[1, 2, 3]).unwrap();
        assert!(v == vec![9, 9, 9, 9]);
    }
    #[test]
    fn replicate_checkpoint_bug(){
        let mut a = NeutronShortAddress::default();
        let mut stack = ContractCallStack::default();
        a.version=100;
        a.data[5] = 20;
        let mut db = ProtoDB::default(); 
        //deploy
        assert!(db.checkpoint().is_ok());
        assert!(db.write_key(&mut stack, &a, &[2, 1, 0], &[10]).is_ok());
        assert!(db.commit().is_ok());
        //first call
        assert!(db.checkpoint().is_ok());
        let v = db.read_key(&mut stack, &a, &[2, 1, 0]).unwrap();
        assert!(v == vec![10]);
        db.write_key(&mut stack, &a, &[95, 0, 1, 2, 3], &[10, 20, 30, 40]).unwrap();
        db.commit().unwrap();
        //second call
        db.checkpoint().unwrap();
        let v = db.read_key(&mut stack, &a, &[2, 1, 0]).unwrap();
        assert!(v == vec![10]);
        
    }
    
}