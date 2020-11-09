<?php
class ModeratorBaseModule extends SmartyModule
{

    /**
     * Returns true only if current user is a moderator or aministrator
     */
    public function isAllowed($runData)
    {
        $runData->getUser();
        if ($user) {
            $c = new Criteria();
        }
    }

    public function build($r)
    {
    }
}
