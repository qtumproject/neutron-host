use neutronify::{neutronify};

pub trait TestContract {
    #[neutronify]
    fn my_func(&self, x: u8, y: String) -> u32 {
        return 0;
    }
    #[neutronify]
    fn empty_func(&mut self) {
        
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[derive(Default)]
    struct MyContract{
        pub a: u8
    }
    impl TestContract for MyContract{
        fn my_func(&self, _x: u8, _y: String) -> u32 {
            return 0;
        }
        fn empty_func(&mut self) {
            self.a = 5;
        }
    }
    #[test]
    fn test_empty_func() {
        let mut contract = MyContract::default() as TestContract;
        contract.empty_func();
        assert_eq!(5, contract.a);
    }

    #[test]
    fn test_macros() {
            let mut contract = MyContract::default();
            let zero = contract.my_func(8, "hello".to_string());
    }
}