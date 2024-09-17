pub mod acoustid;
mod cursive_extensions;
pub mod database;
mod entity;
pub mod file;
pub mod musicbrainz;
mod table_view_extensions;

pub use cursive_extensions::*;
pub use entity::*;
pub use table_view_extensions::*;

use anyhow::Result;
use itertools::Itertools;
use log::error;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;

pub type Am<T> = Arc<Mutex<T>>;
pub type Ram<T> = Result<Am<T>>;

pub trait DurationExtensions {
    fn to_pretty_string(&self) -> String;
}

impl DurationExtensions for Duration {
    fn to_pretty_string(&self) -> String {
        let seconds = self.as_secs();
        let hours = seconds / 3600;
        let seconds = seconds % 3600;
        let minutes = seconds / 60;
        let minute_part = match hours {
            0 => format!("{}", minutes),
            _ => format!("{:02}", minutes),
        };
        let seconds = seconds % 60;
        let hour_part = match hours {
            0 => String::new(),
            _ => format!("{}:", hours),
        };
        format!("{}{}:{:02}", hour_part, minute_part, seconds)
    }
}

pub enum MappingType<MultiType, PartialGroup>
where
    PartialGroup: Eq + PartialEq + PartialOrd + Ord,
{
    Multi(MultiType),
    Partial(PartialGroup),
    Single,
}

pub fn multi_map<From, To, MultiType, PartialGroup>(
    froms: &[From],
    map_single: impl Fn(&From) -> Result<To>,
    map_partial: impl Fn(&PartialGroup, Vec<&From>) -> Result<To>,
    map_multi: impl Fn(&MultiType, &From) -> Result<Vec<To>>,
) -> Result<Vec<To>>
where
    MappingType<MultiType, PartialGroup>: for<'a> core::convert::From<&'a From>,
    PartialGroup: Eq + PartialEq + PartialOrd + Ord,
{
    let mut singles = vec![];
    let mut multis = vec![];
    let mut partials = vec![];
    froms
        .iter()
        .for_each(|f| match MappingType::<MultiType, PartialGroup>::from(f) {
            MappingType::Multi(t) => multis.push((t, f)),
            MappingType::Partial(g) => partials.push((g, f)),
            MappingType::Single => singles.push(f),
        });
    let singles = singles.iter().map(|f| map_single(f)).collect_vec();
    let multis = multis
        .iter()
        .map(|(t, f)| -> Result<Vec<To>> { map_multi(t, *f) })
        .flat_map(|r| -> Vec<Result<To>> {
            match r {
                Ok(v) => v.into_iter().map(|t| Ok(t)).collect(),
                Err(e) => vec![Err(e)],
            }
        })
        .collect_vec();
    let partials = partials
        .iter()
        .sorted_by(|(g1, _), (g2, _)| g1.cmp(g2))
        .chunk_by(|(g, _)| g)
        .into_iter()
        .map(|(g, v)| -> Result<To> {
            let v: Vec<&From> = v.map(|(_, f)| *f).collect();
            map_partial(g, v)
        })
        .collect_vec();
    singles.into_iter().chain(multis).chain(partials).collect()
}

pub fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = future.await {
            error!("{}", e);
        }
    })
}

pub enum MergeAction<'a, E>
where
    E: Entity,
{
    Add(&'a E),
    None,
    Remove(&'a E),
    Update(&'a E),
}

pub fn merge<'a, E>(old: &'a [E], new: &'a [E]) -> Vec<MergeAction<'a, E>>
where
    E: Entity,
    <E as Entity>::Id: EntityId,
{
    let mut results = vec![];

    for o in old {
        if let Some(n) = new.iter().find(|n| n.id() == o.id()) {
            if o != n {
                results.push(MergeAction::Update(n));
            } else {
                results.push(MergeAction::None);
            }
        } else {
            results.push(MergeAction::Remove(o));
        }
    }

    for n in new {
        if !old.iter().any(|o| o.id() == n.id()) {
            results.push(MergeAction::Add(n));
        }
    }

    results
}
