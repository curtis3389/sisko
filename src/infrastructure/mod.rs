pub mod acoustid;
mod cursive_extensions;
pub mod database;
mod entity;
pub mod file;
pub mod musicbrainz;
pub mod cursive_react;
mod table_view_extensions;

pub use cursive_extensions::*;
pub use entity::*;
pub use table_view_extensions::*;

use anyhow::Result;
use itertools::Itertools;
use log::error;
use std::cmp::{max, min};
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

pub fn linear_combination_of_weights(parts: &[(f64, f64)]) -> f64 {
    let mut total = 0.0;
    let mut sum_of_products = 0.0;

    for (value, weight) in parts {
        // TODO: enforce 0 <= value <= 1 & weight >= 0
        total += weight;
        sum_of_products += value * weight;
    }

    sum_of_products / total
}
/*
pub fn get_score(release: &Release) -> f64 {
    release.score / 100.0
}

// This function and those it calls are ripped from picard:
// https://github.com/metabrainz/picard/blob/95c8b72be586379f86bb853ed3b251e99b23f687/picard/metadata.py#L8
pub fn compare_to_release(
    tag_fields: &[TagField],
    release: &Release,
    weights: &HashMap<TagFieldType, f64>,
) -> f64 {
    let parts = compare_to_release_parts(tag_fields, release, weights);
    let sim = linear_combination_of_weights(&parts) * get_score(release);
    sim
}

pub fn compare_to_release_parts(
    tag_fields: &[TagField],
    release: &Release,
    weights: &HashMap<TagFieldType, f64>,
) -> Vec<(f64, f64)> {
    let parts = vec![];
    let field_types: Vec<TagFieldType> = vec![]; // TODO
    for field_type in field_types {
        if tag_fields.iter().any(|f| f.tag_field_type() == field_type)
            && weights.contains_key(field_type)
        {
            parts.push((
                similarity2(
                    tag_fields
                        .iter()
                        .find(|f| f.tag_field_type() == field_type)
                        .unwrap(),
                    release[field_type],
                ),
                weights[field_type],
            ));
        }
    }
    parts
}

pub fn similarity2(a: &str, b: &str) -> f64 {
    let SPLIT_WORDS_REGEX: Regex = Regex::new(r"\W+").unwrap();

    if a == b {
        return 1.0;
    }

    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let a: Vec<&str> = SPLIT_WORDS_REGEX
        .split(&a)
        .filter(|s| !s.is_empty())
        .collect();
    let b: Vec<&str> = SPLIT_WORDS_REGEX
        .split(&b)
        .filter(|s| !s.is_empty())
        .collect();
    let (a, mut b) = if a.len() > b.len() { (b, a) } else { (a, b) };

    if a.len() == 0 || b.len() == 0 {
        return 0.0;
    }

    let mut score = 0.0;
    for a_word in &a {
        let mut ms = 0.0;
        let mut mp: Option<usize> = None;
        for (index, b_word) in b.iter().enumerate() {
            let s = levenshtein_distance(a_word, b_word);
            if s > ms {
                ms = s;
                mp = Some(index);
            }
        }
        if let Some(index) = mp {
            score += ms;
            if ms > 0.6 {
                b.remove(index);
            }
        }
    }

    // a.len() must be > 0 here, so no divide by zero
    score / (a.len() as f64 + (b.len() as f64 * 0.4))
}*/

pub fn levenshtein_distance(a: &str, b: &str) -> f64 {
    let (a, b) = if a.len() > b.len() { (b, a) } else { (a, b) };

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut current: Vec<usize> = (0..a.len() + 1).collect();
    for i in 1..b.len() + 1 {
        let previous = current;
        current = vec![0_usize; a.len() + 1];
        current.insert(0, i);
        for j in 1..a.len() + 1 {
            let (add, delete) = (previous[j] + 1, current[j - 1] + 1);
            let mut change = previous[j - 1];
            if a.chars().nth(j - 1) != b.chars().nth(i - 1) {
                change += 1;
            }
            current[j] = min(add, min(delete, change));
        }
    }

    // current[a.len()] is the edit distance; we return % similarity
    1.0 - (current[a.len()] as f64 / max(a.len(), b.len()) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_distance_works() {
        assert_eq!(levenshtein_distance("book", "back"), 0.5);
        assert_eq!(levenshtein_distance("test", "grammar"), 0.0);
        assert_eq!(
            levenshtein_distance("washington", "lincoln"),
            (1.0 - 7.0 / 10.0)
        );
        assert_eq!(levenshtein_distance("stink", "link"), 0.6);
    }
}
