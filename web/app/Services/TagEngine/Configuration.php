<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

/**
 * The Configuration Class stores information regarding a site's tag configuration, and passes that data to the Tag Engine to perform checks on.
 */

class Configuration {
    public int    $configurationId;
    public int    $siteId;
    public string $configurationName;
    public array  $allowedTags;

    public function __construct(
        int $configurationId,
        int $siteId,
        string $configurationName,
        array $allowedTags,
    )
    {
        $this->configurationId = $configurationId;
        $this->siteId = $siteId;
        $this->configurationName = $configurationName;
        $this->allowedTags = $allowedTags;
    }
}
