<?php
use DB\ForumGroupPeer;
use DB\ForumCategoryPeer;

class ManageSiteForumPermissionsModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $fsettings = $site->getForumSettings();

        if (!$fsettings) {
            throw new ProcessException(_("Forum not activated (yet)."));
        }

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("sort_index");

        $groups = ForumGroupPeer::instance()->select($c);

        $catout = array();
        $catout2 = array();

        foreach ($groups as $group) {
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->addOrderAscending("sort_index");
            $c->add("group_id", $group->getGroupId());
            $categories = ForumCategoryPeer::instance()->select($c);
            $catout[$group->getGroupId()] = $categories;
            foreach ($categories as $cat) {
                $catout2[] = $cat->getFieldValuesArray();
            }
        }

        $defaultPermissions = $fsettings->getPermissions();

        $runData->contextAdd("groups", $groups);
        $runData->contextAdd("categories", $catout);
        $runData->ajaxResponseAdd("categories", $catout2);
        $runData->contextAdd("defaultPermissions", $defaultPermissions);
    }
}
