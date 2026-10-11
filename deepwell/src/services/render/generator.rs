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

pub(crate) const DEEPWELL_RENDERER_EPOCH: u32 = 14;

pub(super) static COMPILED_GENERATOR: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{} v{}; deepwell-render/v{DEEPWELL_RENDERER_EPOCH}",
        ftml::info::PKG_NAME,
        ftml::info::PKG_VERSION,
    )
});

pub(crate) fn compiled_generator_is_current(generator: &str) -> bool {
    generator == COMPILED_GENERATOR.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_identifies_ftml_and_deepwell_renderer_semantics() {
        assert_eq!(
            COMPILED_GENERATOR.as_str(),
            format!(
                "{} v{}; deepwell-render/v14",
                ftml::info::PKG_NAME,
                ftml::info::PKG_VERSION,
            ),
        );
        assert!(compiled_generator_is_current(&COMPILED_GENERATOR));
        assert!(!compiled_generator_is_current(
            "fixture-ftml; deepwell-render/v9"
        ));
    }
}
