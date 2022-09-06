<?php

namespace Wikidot\Modules\ManageSite;

use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ManageSiteBaseModule;

class ManageSiteRecentModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $uri = GlobalProperties::$MODULES_JS_URL.'/changes/SiteChangesModule.js';
        $this->extraJs[] = $uri;

        $this->extraCss[] = GlobalProperties::$MODULES_CSS_URL.'/changes/SiteChangesModule.css';
    }
}
