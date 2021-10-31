<?php
declare(strict_types=1);

namespace Wikidot\Modules\ManageSite;

use Wikidot\Utils\ManageSiteBaseModule;


class ManageSiteGeneralModule extends ManageSiteBaseModule
{
    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $runData->contextAdd("site", $site);
    }
}
