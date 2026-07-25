-- Wikidot exposes Favicon, iOS, and Windows 8 Tile controls per site, each
-- accepting a local upload or an existing URL. These columns record the
-- configured source; a site without one keeps declaring no icon.
ALTER TABLE site
    ADD COLUMN favicon_source TEXT
        CHECK (favicon_source IS NULL OR length(favicon_source) BETWEEN 1 AND 2048),
    ADD COLUMN ios_icon_source TEXT
        CHECK (ios_icon_source IS NULL OR length(ios_icon_source) BETWEEN 1 AND 2048),
    ADD COLUMN windows_tile_source TEXT
        CHECK (windows_tile_source IS NULL OR length(windows_tile_source) BETWEEN 1 AND 2048);
