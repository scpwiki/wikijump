<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;
use Wikidot\DB\SitePeer;
use Wikidot\DB\Site;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\Duplicator;
use Wikidot\Utils\FriendlyCaptchaHandler;
use Wikidot\Utils\Indexer;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;

class NewSiteAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        if ($runData->getUser() == null) {
            throw new WDPermissionException(_("You should be logged in to create a new site."));
        }
        return true;
    }

    public function perform($runData)
    {
    }

    public function createSiteEvent($runData)
    {

        WDPermissionManager::instance()->canBecomeAdmin($runData->getUser());

        $pl = $runData->getParameterList();

        $name = trim($pl->getParameterValue("name"));
        $unixName = trim($pl->getParameterValue("unixname"));
        $tagline = trim($pl->getParameterValue("tagline"));

        $templateId = $pl->getParameterValue("template");

        $private = (bool) $pl->getParameterValue("private");

        $captcha = $pl->getParameterValue("frc-captcha-solution");

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

            if ($runData->getUser()->id != 1) {
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

        // template
        if (!$templateId) {
            $errors['template'] = _('Please choose a template for your site');
        }

        if (strlen8($tagline)>50) {
            $errors['tagline']   = _("Tagline should not be longer than 50 characters");
        }

        // captcha
        $captchaValid = FriendlyCaptchaHandler::verifySolution($captcha);
        if (!$captchaValid) {
            $errors['captcha'] = _("Account creation failed: CAPTCHA was invalid.");
        }

        // TOS
        if (!$pl->getParameterValue("tos")) {
            $errors['tos'] = _("Please read and agree to the Terms of Service.");
        }

        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        // and now... CREATE THE SITE!!!!!!!!!!!!!!!!

        $dup = new Duplicator();
        $dup->setOwner($runData->getUser());

        $db = Database::connection();
        $db->begin();

        $templateSite = SitePeer::instance()->selectByPrimaryKey($templateId);
        if (!preg_match('/^template\-/', $templateSite->getUnixName())) {
            throw new ProcessException('Error');
        }

        $site = new Site();
        $site->setName($name);
        $site->setSubtitle($tagline);
        $site->setUnixName($unixName);
        $site->setLanguage($templateSite->getLanguage());
        $site->setDateCreated(new ODate());

        $site->setPrivate($private);

        if ($private) {
            // change file flag too
            $flagDir = WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName().'/flags';
            $flagFile = $flagDir.'/private';
            mkdirfull($flagDir); //just to make sure

            if (!file_exists($flagFile)) {
                file_put_contents($flagFile, "private");
            }
        }

        $site->save();

        $dup->addExcludedCategory("forum"); // should be initialized independently
        $dup->addExcludedCategory("profile");
        $dup->duplicateSite($templateSite, $site);

        $db->commit();

        $runData->ajaxResponseAdd("siteUnixName", $unixName);
    }
}
