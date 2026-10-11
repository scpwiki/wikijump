//! Emit include references recognized by the same FTML parser used at runtime.
//!
//! This small stdin/stdout adapter is intended for verification tooling that
//! needs parser-owned include discovery without reimplementing Wikidot syntax
//! in JavaScript.

use ftml::includes::include;
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use serde::Serialize;
use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

#[derive(Serialize)]
struct Output {
    ftml_version: &'static str,
    includes: Vec<ftml::includes::IncludeRef<'static>>,
}

#[derive(Debug)]
struct CollectingIncluder {
    includes: Rc<RefCell<Vec<ftml::includes::IncludeRef<'static>>>>,
}

impl<'t> ftml::prelude::Includer<'t> for CollectingIncluder {
    type Error = std::convert::Infallible;

    fn include_pages(
        &mut self,
        includes: &[ftml::includes::IncludeRef<'t>],
    ) -> Result<Vec<ftml::includes::FetchedPage<'t>>, Self::Error> {
        self.includes
            .borrow_mut()
            .extend(includes.iter().map(|include| {
                ftml::includes::IncludeRef::new(
                    include.page_ref().clone(),
                    include
                        .variables()
                        .iter()
                        .map(|(key, value)| {
                            (
                                std::borrow::Cow::Owned(key.to_string()),
                                std::borrow::Cow::Owned(value.to_string()),
                            )
                        })
                        .collect(),
                )
                .with_spaced_empty_separator(include.has_spaced_empty_separator())
            }));
        Ok(includes
            .iter()
            .map(|include| ftml::includes::FetchedPage {
                page_ref: include.page_ref().clone(),
                content: Some(std::borrow::Cow::Borrowed("")),
            })
            .collect())
    }

    fn no_such_include(
        &mut self,
        _page_ref: &ftml::data::PageRef,
    ) -> Result<std::borrow::Cow<'t, str>, Self::Error> {
        Ok(std::borrow::Cow::Borrowed(""))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let includes = Rc::new(RefCell::new(Vec::new()));
    let includer = CollectingIncluder {
        includes: Rc::clone(&includes),
    };
    let _ = include(&source, &settings, includer, || {
        unreachable!("include collector returns exactly one result per include")
    })?;
    let output = Output {
        ftml_version: ftml::info::VERSION.as_str(),
        includes: includes.borrow().clone(),
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
