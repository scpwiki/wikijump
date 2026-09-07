mod prelude;

use deepwell_relation_impl_derive::impl_relation;
use self::prelude::*;

impl_relation! {
    name => PageWatch,
    dest => page_id: Page,
    from => user_id: User,
}

fn main() {}
