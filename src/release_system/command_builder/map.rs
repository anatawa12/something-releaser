use std::any::TypeId;
use std::collections::HashMap;
use std::process::Command;

pub trait CommandBuilder: Send {
    fn create_command_to_exec(self) -> Command;
}

pub struct CommandBuilderMap {
    inner: HashMap<TypeId, Box<dyn CommandBuilder>>,
}

impl CommandBuilderMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn find_mut<T: CommandBuilder + Default + 'static>(&mut self) -> &mut T {
        if !self.inner.contains_key(&TypeId::of::<T>()) {
            self.inner.insert(TypeId::of::<T>(), Box::new(T::default()));
        }
        unsafe { unsafe_mut_cast::<_, T>(self.inner.get_mut(&TypeId::of::<T>()).unwrap().as_mut()) }
    }

    pub fn values(&self) -> impl Iterator<Item = &Box<dyn CommandBuilder>> {
        self.inner.values()
    }

    pub fn into_values(self) -> impl Iterator<Item = Box<dyn CommandBuilder>> {
        self.inner.into_iter().map(|x| x.1)
    }
}

unsafe fn unsafe_mut_cast<T: ?Sized, U>(from: &mut T) -> &mut U {
    &mut *(from as *mut T as *mut U)
}

#[test]
fn api_test() {
    #[derive(Default)]
    struct CommandBuilder1 {}
    impl CommandBuilder for CommandBuilder1 {
        fn create_command_to_exec(self) -> Command {
            panic!()
        }
    }

    let mut map = CommandBuilderMap::new();
    let _: &mut CommandBuilder1 = map.find_mut::<CommandBuilder1>();
    for _ in map.values() {}
    for _ in map.into_values() {}
}

#[test]
fn unsafe_mut_cast_safety() {
    struct CommandBuilder1 {
        test: String,
    }
    impl CommandBuilder for CommandBuilder1 {
        fn create_command_to_exec(self) -> Command {
            panic!()
        }
    }

    let mut builder = CommandBuilder1 {
        test: "test string".to_owned(),
    };
    let mut_ref =
        unsafe { unsafe_mut_cast::<_, CommandBuilder1>(&mut builder as &mut dyn CommandBuilder) };
    assert_eq!(mut_ref.test, "test string".to_owned());
}
