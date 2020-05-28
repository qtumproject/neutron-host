use crate::callstack::*;
use crate::neutronerror::*;
use crate::neutronerror::NeutronError::*;
/*
## Logging

ID: 2

Functions:

* log_debug(string)
* log_info(string)
* log_warning(string)
* log_error(string)

Note in neutron-star, log_info is used by default for println!
*/

const LOGGING_FEATURE: u32 = 2;

#[derive(FromPrimitive)]
pub enum LoggingFunctions{
    Available = 0, //reserved??
    LogDebug = 1,
    LogInfo,
    LogWarning,
    LogError
}

pub trait LoggingInterface{
    fn try_syscall(&mut self, stack: &mut ContractCallStack, feature: u32, function: u32) -> Result<bool, NeutronError>{
        if feature != LOGGING_FEATURE {
            return Ok(false);
        }
        let f = num::FromPrimitive::from_u32(function);
        if f.is_none(){
            return Err(Recoverable(RecoverableError::InvalidSystemFunction));
        }
        let f=f.unwrap();
        let result = match f{
            LoggingFunctions::LogDebug => {
                self.log_debug(stack)
            },
            LoggingFunctions::LogInfo => {
                self.log_info(stack)
            },
            LoggingFunctions::LogWarning => {
                self.log_warning(stack)
            },
            LoggingFunctions::LogError => {
                self.log_error(stack)
            }
            LoggingFunctions::Available => {
                Ok(())
            }
        };
        if result.is_err(){
            Err(result.unwrap_err())
        }else{
            Ok(true)
        }
    }
    fn log_debug(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>;
    fn log_info(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>;
    fn log_warning(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>;
    fn log_error(&mut self, stack: &mut ContractCallStack) -> Result<(), NeutronError>;
}

