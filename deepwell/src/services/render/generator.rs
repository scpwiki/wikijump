/*
 * services/render/generator.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::sync::LazyLock;

const DEEPWELL_RENDERER_EPOCH: u32 = 1;

pub(super) static COMPILED_GENERATOR: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}; deepwell-render/v{DEEPWELL_RENDERER_EPOCH}",
        ftml::info::VERSION.as_str(),
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_identifies_ftml_and_deepwell_renderer_semantics() {
        assert!(COMPILED_GENERATOR.starts_with(ftml::info::VERSION.as_str()));
        assert!(COMPILED_GENERATOR.ends_with("; deepwell-render/v1"));
    }
}
