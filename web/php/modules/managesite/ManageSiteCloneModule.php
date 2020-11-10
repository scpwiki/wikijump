<?php
class ManageSiteCloneModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $runData->contextAdd("site", $site);
    }
}
