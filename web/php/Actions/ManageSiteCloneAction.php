<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;
use Wikidot\DB\SitePeer;
use Wikidot\Utils\Duplicator;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;

class ManageSiteCloneAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));

        return true;
    }

    public function perform($r)
    {
    }

    public function cloneSiteEvent($runData)
    {

        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        WDPermissionManager::instance()->canBecomeAdmin($runData->getUser());

        $name = trim($pl->getParameterValue("name"));
        $unixName = trim($pl->getParameterValue("unixname"));
        $tagline = trim($pl->getParameterValue("tagline"));
        $description = trim($pl->getParameterValue("description"));

        $private = (bool) $pl->getParameterValue("private");

        // validate form data:

        $errors = array();
        if (strlen($name)<1) {
            $errors['name'] = _("Site name must be present.");
        } elseif (strlen8($name)>30) {
            $errors['name']  = _("Site name should not be longer than 30 characters.");
        }

        // site unix name *************
        if ($unixName === null || strlen($unixName)<3) {
            $errors['unixname'] = _("Web address must be present and should be at least 3 characters long.");
        } elseif (strlen($unixName)>30) {
            $errors['unixname']  = _("Web address name should not be longer than 30 characters.");
        } elseif (preg_match("/^[a-z0-9\-]+$/", $unixName) == 0) {
            $errors['unixname'] = _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.');
        } elseif (preg_match("/\-\-/", $unixName) !== 0) {
            $errors['unixname'] = _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.');
        } else {
            $unixName = WDStringUtils::toUnixName($unixName);

            if (!$runData->getUser()->getSuperAdmin()) {
                //  handle forbidden names
                foreach (ForbiddenNames::$sites as $regex) {
                    if (preg_match($regex, $unixName) > 0) {
                        $errors['unixname'] = _('This web address is not allowed or reserved.');
                    }
                }
            }

            // check if the domain is not taken.
            $c = new Criteria();
            $c->add("unix_name", $unixName);
            $ss = SitePeer::instance()->selectOne($c);
            if ($ss) {
                $errors['unixname'] = _('Sorry, this web address is already used by another site.');
            }
        }


        if (strlen8($tagline)>50) {
            $errors['tagline']   = _("Tagline should not be longer than 50 characters");
        }

        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        // and now... CREATE THE SITE!!!!!!!!!!!!!!!!

        $siteProps = array(
            'name' => $name,
            'subtitle' => $tagline,
            'unixname' => $unixName,
            'description' => $description,
            'private' => $private
        );
        $dup = new Duplicator();
        $dup->cloneSite($site, $siteProps);
    }
}
