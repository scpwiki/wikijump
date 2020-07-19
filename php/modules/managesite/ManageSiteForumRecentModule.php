<?php
class ManageSiteForumRecentModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $uri = GlobalProperties::$MODULES_JS_URL.'/forum/ForumRecentPostsModule.js';
        $this->extraJs[] = $uri;
    }
}
