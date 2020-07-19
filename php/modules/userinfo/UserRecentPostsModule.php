<?php
class UserRecentPostsModule extends SmartyLocalizedModule
{

    protected $timeOut = 300;

    public function build($runData)
    {

        $userId = $runData->getParameterList()->getParameterValue("user_id");

        if ($userId === null) {
            $userId = $runData->getUserId();
        }

        $runData->contextAdd("userId", $userId);
    }
}
