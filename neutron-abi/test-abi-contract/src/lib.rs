use neutronify::{my_macro};

pub trait TestTrait {
    #[my_macro]
    fn my_func(&self, x: u8, y: String) -> u32 {
        return 0;
    }
    #[my_macro]
    fn empty_func(&mut self);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[derive(Default)]
    struct MyStruct{
        pub a: u8
    }
    impl TestTrait for MyStruct{
        #[my_macro]
        fn my_func(&self, _x: u8, _y: String) -> u32 {
            return 0;
        }
        #[my_macro]
        fn empty_func(&mut self) {
            self.a = 5;
        }
    }
    #[test]
    fn test_empty_func() {
        let mut mystruc = MyStruct::default();
        let interface = &mut mystruc as &mut dyn TestTrait;
        interface.empty_func();
        assert_eq!(5, mystruc.a);
    }

    #[test]
    fn test_macros() {
            let mut mystruct = MyStruct::default();
            let interface = &mut mystruct as &mut dyn TestTrait;
            let zero = interface.my_func(8, "hello".to_string());
    }
}