<?php
class ManageSiteForumLayoutModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get all groups and categories, prepare them in a suitable form
        $site = $runData->getTemp("site");

        $fsettings = $site->getForumSettings();

        if (!$fsettings) {
            throw new ProcessException(_("Forum not activated (yet)."));
        }
    }
}
