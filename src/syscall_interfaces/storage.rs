use crate::neutronmanager::*;
use crate::neutronerror::*;
use crate::neutronerror::NeutronError::*;
/*
## Global Storage

ID: 1

Functions:

* store_state(key, value) -> ()
* load_state(key) -> (value)
* key_exists(key) -> (bool)
*/

const GLOBAL_STORAGE_FEATURE: u32 = 1;

#[derive(FromPrimitive)]
pub enum GlobalStorageFunctions{
    Available = 0, //reserved??
    StoreState = 1,
    LoadState,
    KeyExists
}

pub trait GlobalStorage{
    fn try_syscall(&mut self, stack: &mut NeutronManager, feature: u32, function: u32) -> Result<bool, NeutronError>{
        if feature != GLOBAL_STORAGE_FEATURE{
            return Ok(false);
        }
        let f = num::FromPrimitive::from_u32(function);
        if f.is_none(){
            return Err(Recoverable(RecoverableError::InvalidSystemFunction));
        }
        let f=f.unwrap();
        let result = match f{
            GlobalStorageFunctions::KeyExists => {
                self.key_exists(stack)
            },
            GlobalStorageFunctions::LoadState => {
                self.load_state(stack)
            },
            GlobalStorageFunctions::StoreState => {
                self.store_state(stack)
            }
            GlobalStorageFunctions::Available => {
                Ok(())
            }
        };
        if result.is_err(){
            Err(result.unwrap_err())
        }else{
            Ok(true)
        }
    }
    fn store_state(&mut self, stack: &mut NeutronManager) -> Result<(), NeutronError>;
    fn load_state(&mut self, stack: &mut NeutronManager) -> Result<(), NeutronError>;
    fn key_exists(&mut self, stack: &mut NeutronManager) -> Result<(), NeutronError>;
}

