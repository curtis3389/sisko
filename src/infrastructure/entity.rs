use std::fmt::Debug;

pub trait EntityId: Clone + Debug + Eq {
    fn to_string(&self) -> String;
}

pub trait Entity: Clone + Debug + Eq {
    type Id;
    fn id(&self) -> &Self::Id
    where
        Self::Id: EntityId;
}

pub trait Value: Clone + Debug + Eq {}
