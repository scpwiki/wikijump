<?php
use DB\SitePeer;

class NewWikiWidgetAction extends SmartyAction
{

    public function perform($runData)
    {
    }

    public function newWikiEvent($runData)
    {
        $pl = $runData->getParameterList();

        $siteName = $pl->getParameterValue('siteName');

        // validate even more
        $unixName = WDStringUtils::toUnixName($siteName);

        if ($unixName === null || strlen($unixName)<3) {
            throw new ProcessException(_("Web address must be at least 3 characters long."));
        }
        if (strlen($unixName)>30) {
            throw new ProcessException(_("Web address name should not be longer than 30 characters."));
        }
        if (preg_match("/^[a-z0-9\-]+$/", $unixName) == 0) {
            throw new ProcessException(_('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.'));
        }
        if (preg_match("/\-\-/", $unixName) !== 0) {
            throw new ProcessException(_('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.'));
        }

        $unixName = WDStringUtils::toUnixName($unixName);

        if (!$runData->getUser() || !$runData->getUser()->getSuperAdmin()) {
            //  handle forbidden names
            $forbiddenUnixNames = explode("\n", file_get_contents(WIKIJUMP_ROOT.'/conf/forbidden_site_names.conf'));
            foreach ($forbiddenUnixNames as $f) {
                if (preg_match($f, $unixName) >0) {
                    throw new ProcessException(_('For some reason this web address is not allowed or is reserved for future use.'));
                }
            }
        }

        // check if the domain is not taken.
        $c = new Criteria();
        $c->add("unix_name", $unixName);
        $ss = SitePeer::instance()->selectOne($c);
        if ($ss) {
            throw new ProcessException(_('Sorry, this web address is already used by another wiki.'));
        }

        $runData->ajaxResponseAdd('unixName', $unixName);
    }
}
