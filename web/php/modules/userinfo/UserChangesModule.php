<?php
class UserChangesModule extends SmartyLocalizedModule
{

    public function build($runData)
    {

        $userId = $runData->getParameterList()->getParameterValue("user_id");

        if ($userId === null) {
            $userId = $runData->getUserId();
        }

        $runData->contextAdd("userId", $userId);
    }
}
