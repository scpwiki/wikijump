<?php
use DB\ForumGroupPeer;
use DB\ForumCategoryPeer;

class ManageSiteGetForumLayoutModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        // get all groups and categories, prepare them in a suitable form
        $site = $runData->getTemp("site");

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("sort_index");

        $groups = ForumGroupPeer::instance()->select($c);

        $g0 = array();
        $c0 = array();
        $gcount = 0;
        foreach ($groups as $group) {
            $grow = array();
            $grow['name']=$group->getName();
            $grow['description']=$group->getDescription();
            $grow['group_id']=$group->getGroupId();
            $grow['visible']=$group->getVisible();

            $g0[$gcount] = $grow;

            // now get categories...
            $c0[$gcount] = array();
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->addOrderAscending("sort_index");
            $c->add("group_id", $group->getGroupId());
            $categories = ForumCategoryPeer::instance()->select($c);
            $ccount = 0;
            foreach ($categories as $cat) {
                $crow = array();
                $crow['name'] = $cat->getName();
                $crow['description'] = $cat->getDescription();
                $crow['category_id'] = $cat->getCategoryId();
                $crow['posts'] = $cat->getNumberPosts();
                $crow['number_threads'] = $cat->getNumberThreads();
                $crow['permissions'] = $cat->getPermissions();
                $crow['max_nest_level'] = $cat->getMaxNestLevel();

                $c0[$gcount][$ccount] = $crow;
                $ccount++;
            }

            $gcount++;
        }

        $runData->ajaxResponseAdd("groups", $g0);
        $runData->ajaxResponseAdd("categories", $c0);

        //get default nesting
        $fs = $site->getForumSettings();
        $runData->ajaxResponseAdd("defaultNesting", $fs->getMaxNestLevel());
    }
}
