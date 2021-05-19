<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\JSONService;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ThemePeer;
use Wikidot\DB\Theme;
use Wikidot\DB\SiteTagPeer;
use Wikidot\DB\SiteTag;
use Wikidot\DB\SitePeer;
use Wikidot\DB\DomainRedirectPeer;
use Wikidot\DB\DomainRedirect;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\SiteViewerPeer;
use Wikidot\DB\SiteViewer;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\AnonymousAbuseFlagPeer;
use Wikidot\DB\EmailInvitationPeer;
use Wikidot\DB\MemberApplicationPeer;
use Wikidot\DB\MemberInvitationPeer;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;

class ManageSiteAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));

        return true;
    }

    public function perform($r)
    {
    }

    public function saveAppearanceEvent($runData)
    {

        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        /* for each category
         *  - get a category from database
         *  - check if theme_id or theme_default has changed
         *  - if changed: update
         */
        $db = Database::connection();
        $db->begin();
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId); // for sure
            $dCategory = CategoryPeer::instance()->selectOne($c);

            // now compare
            $changed = false;
            if ($category['variant_theme_id']) {
                if ($category['variant_theme_id'] != $dCategory->getThemeId()) {
                    $dCategory->setThemeId($category['variant_theme_id']);
                    $changed = true;
                }
            } else {
                if ($category['theme_id'] != $dCategory->getThemeId()) {
                    $dCategory->setThemeId($category['theme_id']);
                    $changed = true;
                }
            }

            if ($category['theme_default'] != $dCategory->getThemeDefault()) {
                $dCategory->setThemeDefault($category['theme_default']);
                $changed = true;
            }

            if ($category['theme_external_url'] != $dCategory->getThemeExternalUrl()) {
                if ($category['theme_external_url'] && !preg_match(';^https?://;', $category['theme_external_url'])) {
                    throw new ProcessException('Url of the external theme for category '.$dCategory->getName(). ' is not valid.');
                }
                $dCategory->setThemeExternalUrl($category['theme_external_url']);
                $changed = true;
            }

            if ($changed) {
                $dCategory->save();
                // outdate category
                $outdater = new Outdater();
                $outdater->categoryEvent("category_save", $dCategory);
            }
            if ($changed && $dCategory->getName()=='_default') {
                // outdate all that depends somehow
                $c = new Criteria();
                $c->add("site_id", $dCategory->getSiteId());
                $c->add("theme_default", true);
                $c->add("name", "_default", '!=');
                $depcats = CategoryPeer::instance()->select($c);
                foreach ($depcats as $dc) {
                    $outdater = new Outdater();
                    $outdater->categoryEvent("category_save", $dc);
                }
            }
        }
        $db->commit();
    }

    public function importCssEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");

        $pageName = $pl->getParameterValue("pageName");
        if ($pageName == '') {
            throw new ProcessException(_("No page given."), "form_error");
        }
        $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
        if ($page == null) {
            throw new ProcessException(_("No page found with this name."), "form_error");
        }
        $source = $page->getSource();
        if (preg_match('/
            \[\[code(?:\s+type="css")?\]\]
            (.*?)
            \[\[\/code\]\]
            /six', $source) == 0) {
            throw new ProcessException(_("No code block could be found in the page source."), "form_error");
        }
        $code = trim(preg_replace('/
            .*?
            \[\[code(?:\s+type="css")?\]\]
            (.*?)
            \[\[\/code\]\]
            .*
            /six', "\\1", $source));
        $runData->ajaxResponseAdd("code", $code);
    }

    public function customThemeSaveEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");

        $themeId = $pl->getParameterValue("themeId") ?? null;

        $name = trim($pl->getParameterValue("name"));
        $parentThemeId = $pl->getParameterValue("parentTheme");
        $code =  trim($pl->getParameterValue("code"));
        $pageName = trim($pl->getParameterValue("cssImportPage"));

        $useSideBar = false;
        $useTopBar = false;
        if ($pl->getParameterValue("useSideBar")) {
            $useSideBar = true;
        }
        if ($pl->getParameterValue("useTopBar")) {
            $useTopBar = true;
        }

        if ($name == '') {
            throw new ProcessException(_("Theme name must be given."), "form_error");
        }
        if (strlen8($name) > 30) {
            throw new ProcessException(_("Theme name should not be longer than 30 characters."), "form_error");
        }
        if (strlen($code) > 50000) {
            throw new ProcessException(_("CSS code seems to be to long."), "form_error");
        }
        $parentTheme = ThemePeer::instance()->selectByPrimaryKey($parentThemeId);
        if ($parentTheme == null) {
            throw new ProcessException(_("Parent theme cannot be found."), "form_error");
        }

        if ($themeId == null) {
            // check if theme name is unique among custom themes for this site
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("name", $name);
            $th = ThemePeer::instance()->selectOne($c) ?? null;
            if ($th) {
                throw new ProcessException(_("Theme with this name already exists within this site."), "form_error");
            }
        }

        $db = Database::connection();
        $db->begin();
        if ($themeId) {
            // theme already exists
            $theme = ThemePeer::instance()->selectByPrimaryKey($themeId);
            if ($theme == null || $theme->getSiteId() !== $site->getSiteId()) {
                throw new ProcessException(_("Error selecting theme."), "wrong_theme");
            }
        } else {
            // new theme
            $theme = new Theme();
            $theme->setCustom(true);
            $theme->setSiteId($site->getSiteId());
        }
        $unixName = WDStringUtils::toUnixName($name);
        $theme->setName($name);

        $theme->setSyncPageName($pageName);
        $theme->setExtendsThemeId($parentThemeId);
        if ($unixName != $theme->getUnixName()) {
            $nameChanged = true;
            $oldName = $theme->getUnixName();
        }
        $theme->setUnixName($unixName);

        $theme->setUseSideBar($useSideBar);
        $theme->setUseTopBar($useTopBar);

        if ($nameChanged && $oldName != '') {
            $cmd = "rm -r ".escapeshellarg($site->getLocalFilesPath()."/theme/".$oldName);
            exec($cmd);
        }

        // handle code now
        $dir = WIKIJUMP_ROOT."/web/files--sites/".$site->getUnixName()."/theme/".$unixName;
        mkdirfull($dir);
        file_put_contents($dir."/style.css", $code);

        $theme->setRevisionNumber($theme->getRevisionNumber()+1);

        $theme->save();
        $outdater = new Outdater();
        $outdater->themeEvent("theme_save", $theme);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function customThemeDeleteEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $themeId = $pl->getParameterValue("themeId");
        $theme = ThemePeer::instance()->selectByPrimaryKey($themeId);
        if ($theme == null || $theme->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Error selecting theme."), "wrong_theme");
        }

        $db = Database::connection();
        $db->begin();

        // now check if theme is used by pages (categories)
        $c = new Criteria();
        $c->add("theme_id", $theme->getThemeId());
        $c->add("site_id", $site->getSiteId());
        $cats = CategoryPeer::instance()->select($c);
        if (count($cats)>0) {
            throw new ProcessException(_("This theme cannot be deleted because there are still pages that use it. Please check themes assigned to particular categories."), "can_not_delete");
        }
        // ok, delete now!
        ThemePeer::instance()->deleteByPrimaryKey($theme->getThemeId());

        $cmd = "rm -r ".escapeshellarg($site->getLocalFilesPath()."/theme/".$theme->getUnixName());
        exec($cmd);

        $db->commit();
    }

    public function saveTemplatesEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        $db = Database::connection();
        $db->begin();
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId);
            $dCategory = CategoryPeer::instance()->selectOne($c);
            if ($dCategory == null) {
                throw new ProcessException(_("Error saving changes - one of the categories could not be found."), "no_category");
            }
            // now compare
            $changed = false;
            if ($category['template_id'] != $dCategory->getTemplateId()) {
                $dCategory->setTemplateId($category['template_id']);
                $changed = true;
            }
            if ($changed) {
                $dCategory->save();
            }
        }
        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function savePermissionsEvent($runData)
    {

        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        $db = Database::connection();

        $db->begin();
        /* for each category
         *  - get a category from database
         *  - check if theme_id or theme_default has changed
         *  - if changed: update
         */
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId);
            $dCategory = CategoryPeer::instance()->selectOne($c);
            if ($dCategory == null) {
                throw new ProcessException("Invalid category.");
            }

            // now compare
            $changed = false;
            $permstring = $category['permissions'];

            //validate permstring
            $p2 = explode(";", $permstring);
            foreach ($p2 as $perm) {
                if (!$category['permissions_default'] && preg_match("/^[vecmdarzo]:[armo]{0,4}$/", $perm) == 0) {
                    throw new ProcessException(_("Error saving permissions - invalid internal format. Please try again and contact admins if the problem repeats."));
                }
            }

            if ($permstring != $dCategory->getPermissions()) {
                $dCategory->setPermissions($permstring);
                $changed = true;
            }
            if ($category['permissions_default'] != $dCategory->getPermissionsDefault()) {
                $dCategory->setPermissionsDefault($category['permissions_default']);
                $changed = true;
            }
            if ($changed) {
                $dCategory->save();
                // outdate category
                $outdater = new Outdater();
                $outdater->categoryEvent("category_save", $dCategory);
            }
        }
        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveLicenseEvent($runData)
    {

        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        /* for each category
     *  - get a category from database
     *  - check if license_id or license_default has changed
     *  - if changed: update
     */
        $db = Database::connection();
        $db->begin();
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId);
            $dCategory = CategoryPeer::instance()->selectOne($c);

            // now compare
            $changed = false;
            if ($category['license_id'] != $dCategory->getLicenseId()) {
                $dCategory->setLicenseId($category['license_id']);
                $changed = true;
            }
            if ($category['license_default'] != $dCategory->getLicenseDefault()) {
                $dCategory->setLicenseDefault($category['license_default']);
                $changed = true;
            }
            if ($category['license_other'] !== $dCategory->getLicenseOther()) {
                $ltext = trim($category['license_other']);
                if (strlen8($ltext)>300) {
                    throw new ProcessException(_("The custom license text should not be longer than 300 characters."));
                }
                $dCategory->setLicenseOther($ltext);
                $changed = true;
            }
            if ($changed) {
                $dCategory->save();
                // outdate category
                $outdater = new Outdater();
                $outdater->categoryEvent("category_save", $dCategory);
            }

            if ($changed && $dCategory->getName()=='_default') {
                // outdate all that depends somehow
                $c = new Criteria();
                $c->add("site_id", $dCategory->getSiteId());
                $c->add("license_default", true);
                $c->add("name", "_default", '!=');
                $depcats = CategoryPeer::instance()->select($c);
                foreach ($depcats as $dc) {
                    $outdater = new Outdater();
                    $outdater->categoryEvent("category_save", $dc);
                }
            }
        }
        $db->commit();
    }

    public function saveGeneralEvent($runData)
    {

        $pl = $runData->getParameterList();
        $name = trim($pl->getParameterValue("name"));
        $subtitle = trim($pl->getParameterValue("subtitle"));

        $description = trim($pl->getParameterValue("description"));
        $tags = strtolower(trim($pl->getParameterValue("tags")));

        $defaultPage = WDStringUtils::toUnixName($pl->getParameterValue("default_page"));

        $errors = array();
        if (strlen($name)<1) {
            $errors['name'] = _("Site name must be present.");
        } elseif (strlen8($name)>30) {
            $errors['name']  = _("Site name should not be longer than 30 characters.");
        }

        if (strlen8($subtitle)>50) {
            $errors['subtitle']  = _("Subtitle should not be longer than 50 characters");
        }
        if (strlen8($description)>300) {
            $errors['description']   = _("Description should not be longer than 300 characters");
        }
        if (strlen8($tags)>128) {
            $errors['tags']  = _('"Tags" field too long.');
        }

        if ($defaultPage == "") {
            $errors['defaultPage'] = _("Default landing page should be given and be somehow valid.");
        }
        if (strlen($defaultPage) >80) {
            $errors['defaultPage'] = _("Default landing page name is too long.");
        }

        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        $site = $runData->getTemp("site");
        $changed = false;
        if ($site->getName() !== $name) {
            $site->setName($name);
            $changed = true;
        }
        if ($site->getSubtitle() !== $subtitle) {
            $site->setSubtitle($subtitle);
            $changed = true;
        }
        if ($site->getDescription() !== $description) {
            $site->setDescription($description);
            $changed = true;
        }
        if ($site->getDefaultPage() !== $defaultPage) {
            $site->setDefaultPage($defaultPage);
            $changed = true;
        }

        $db = Database::connection();
        $db->begin();

        if ($changed) {
            $site->save();
            // outdate cache for sure
            $outdater = new Outdater();
            $outdater->siteEvent("sitewide_change");
        }

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());

        $dbTags = SiteTagPeer::instance()->select($c);
        $tags = preg_split("/[ ,]+/", $tags);
        $tags = array_unique($tags);

        foreach ($dbTags as $dbTag) {
            if (in_array($dbTag->getTag(), $tags)) {
                unset($tags[array_search($dbTag->getTag(), $tags)]);
            } else {
                SiteTagPeer::instance()->deleteByPrimaryKey($dbTag->getTagId());
            }
        }
        // insert all other
        foreach ($tags as $tag) {
            if (trim($tag) != '') {
                $dbTag = new SiteTag();
                $dbTag->setSiteId($site->getSiteId());
                $dbTag->setTag($tag);
                $dbTag->save();
            }
        }

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveDomainEvent($runData)
    {
        $pl = $runData->getParameterList();
        $domain = trim($pl->getParameterValue("domain"));
        $site = $runData->getTemp("site");

        $redirects = $pl->getParameterValue('redirects');
        $redirects = explode(';', $redirects);

        $db = Database::connection();
        $db->begin();

        if ($domain !== "" && preg_match("/^[a-z0-9\-]+(\.[a-z0-9\-]+)+$/i", $domain) != 1) {
            throw new ProcessException(sprintf(_('"%s" is not a valid domain name.'), $domain), "form_error");
        }
        foreach ($redirects as $r) {
            if ($r !== "" && preg_match("/^[a-z0-9\-]+(\.[a-z0-9\-]+)+$/i", $r) != 1) {
                throw new ProcessException(sprintf(_('"%s" is not a valid domain name.'), $r), "form_error");
            }
        }

        if ($redirects && count($redirects)>10) {
            throw new ProcessException("You an create max 10 redirects.");
        }

        if (preg_match("/\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $domain) !== 0 || $domain == GlobalProperties::$URL_DOMAIN) {
            throw new ProcessException(sprintf(_('Sorry, "%s" domain is not allowed.'), $domain), "not_allowed");
        }

        foreach ($redirects as $r) {
            if (preg_match("/\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $r) !== 0 || $r == GlobalProperties::$URL_DOMAIN) {
                throw new ProcessException(sprintf(_('Sorry, "%s" domain is not allowed.'), $r), "not_allowed");
            }
        }

        if ($domain != '') {
            // check if domain taken.
            $c = new Criteria();
            $c->add("custom_domain", $domain);
            $s = SitePeer::instance()->selectOne($c);

            if ($s && $s->getSiteId() !== $site->getSiteId()) {
                throw new ProcessException(sprintf(_('Another Wiki already maps the "%s" domain.'), $domain));
            }
            // check any redirects conflict
            $c = new Criteria();
            $c->add("url", $domain);
            $s = DomainRedirectPeer::instance()->selectOne($c);
            if ($s && $s->getSiteId() !== $site->getSiteId()) {
                throw new ProcessException(sprintf(_('Another Wiki already redirects the "%s" domain.'), $domain));
            }
        }

        // check if anyone else uses the redirects.

        foreach ($redirects as $r) {
            if ($r != '') {
                // check if domain taken.
                $c = new Criteria();
                $c->add("custom_domain", $r);
                $s = SitePeer::instance()->selectOne($c);

                if ($s && $s->getSiteId() !== $site->getSiteId()) {
                    throw new ProcessException(sprintf(_('Another Wiki already maps the "%s" domain.'), $r));
                }
                // check any redirects conflict
                $c = new Criteria();
                $c->add("url", $r);
                $s = DomainRedirectPeer::instance()->selectOne($c);
                if ($s && $s->getSiteId() !== $site->getSiteId()) {
                    throw new ProcessException(sprintf(_('Another Wiki already redirects the "%s" domain.'), $r));
                }
            }
        }

        // save redirects
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());

        $dbRedirects = DomainRedirectPeer::instance()->select($c);

        $memcache = Ozone::$memcache;

        foreach ($dbRedirects as $dbr) {
            if (in_array($dbr->getUrl(), $redirects)) {
                unset($redirects[array_search($dbr->getUrl(), $redirects)]);
            } else {
                $key = 'domain_redirect..'.$dbr->getUrl();
                $memcache->delete($key);
                DomainRedirectPeer::instance()->deleteByPrimaryKey($dbr->getRedirectId());
            }
        }
        // insert all other
        foreach ($redirects as $redirect) {
            if (trim($redirect) != '') {
                $dbRedirect = new DomainRedirect();
                $dbRedirect->setSiteId($site->getSiteId());
                $dbRedirect->setUrl($redirect);
                $dbRedirect->save();
            }
        }

        // check IP address

        if ($domain != '' && gethostbyname($domain) !== gethostbyname(GlobalProperties::$URL_DOMAIN)) {
            throw new ProcessException(_("Sorry, it seams that the new domain does not resolve to a valid IP address. See the tips at the bottom of this page."));
        }

        if ($site->getCustomDomain() != $domain) {
            // change the domain
            $oldDomain = $site->getCustomDomain();
            $site->setCustomDomain($domain);
            $cdLinkDir = WIKIJUMP_ROOT.'/web/custom--domains/';
            if ($oldDomain != '' && $oldDomain != null) {
                // unlink the link
                unlink($cdLinkDir.$oldDomain);
            }
            if ($domain != '') {
                symlink(
                    WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName(),
                    $cdLinkDir.$domain
                );
            }
            $site->save();
            $outdater = new Outdater();
            $outdater->siteEvent("sitewide_change");
        }
        $db->commit();

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveNavigationEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        $db = Database::connection();

        $db->begin();
        /* for each category
         *  - get a category from database
         *  - check if theme_id or theme_default has changed
         *  - if changed: update
         */
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);

            $c->add("site_id", $siteId);
            $dCategory = CategoryPeer::instance()->selectOne($c);

            // now compare
            $changed = false;
            if ($category['nav_default'] != $dCategory->getNavDefault()) {
                $dCategory->setNavDefault($category['nav_default']);
                $changed = true;
            }
            if ($category['top_bar_page_name'] !== $dCategory->getTopBarPageName()) {
                $dCategory->setTopBarPageName($category['top_bar_page_name']);
                $changed = true;
            }
            if ($category['side_bar_page_name'] !== $dCategory->getSideBarPageName()) {
                $dCategory->setSideBarPageName($category['side_bar_page_name']);
                $changed = true;
            }

            if ($changed) {
                $dCategory->save();
                // outdate category
                $outdater = new Outdater();
                $outdater->categoryEvent("category_save", $dCategory);
            }
            if ($changed && $dCategory->getName()=='_default') {
                // outdate all that depends somehow
                $c = new Criteria();
                $c->add("site_id", $dCategory->getSiteId());
                $c->add("nav_default", true);
                $c->add("name", "_default", '!=');
                $depcats = CategoryPeer::instance()->select($c);
                foreach ($depcats as $dc) {
                    $outdater = new Outdater();
                    $outdater->categoryEvent("category_save", $dc);
                }
            }
        }
        $db->commit();
    }

    public function savePageRateSettingsEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $json = new JSONService(SERVICES_JSON_LOOSE_TYPE);
        $cats0 = $json->decode($pl->getParameterValue("categories"));

        $db = Database::connection();
        $db->begin();

        $outdater = new Outdater();
        foreach ($cats0 as $category) {
            $categoryId = $category['category_id'];
            $c = new Criteria();
            $c->add("category_id", $categoryId);
            $c->add("site_id", $siteId);
            $dCategory = CategoryPeer::instance()->selectOne($c);

            // now compare
            $changed = false;

            if ($category['rating'] !== $dCategory->getRating()) {
                $dCategory->setRating($category['rating']);
                $changed = true;
            }
            if ($changed) {
                $dCategory->save();
                // outdate category too
                $outdater->categoryEvent("category_save", $dCategory);
            }
        }

        $db->commit();

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function savePrivateSettingsEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");

        $private = (bool) $pl->getParameterValue("private") ?? null;
        $landing = trim($pl->getParameterValue("landingPage"));

        $hideNav = (bool) $pl->getParameterValue("hideNav");

        $viewers = $pl->getParameterValue("viewers");
        $viewers = explode(',', $viewers);

        $settings = $site->getSettings();
        $maxMembers = $settings->getMaxPrivateMembers();
        $maxViewers = $settings->getMaxPrivateViewers();

        // check if not >=10 members
        if ($private) {
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $cmem = MemberPeer::instance()->selectCount($c);
            if ($cmem > $maxMembers) {
                throw new ProcessException(sprintf(_('Sorry, at the moment max %d member limit apply for private Wikis. The Site would have to be upgraded to allow more members.'), $maxMembers));
            }
        }

        if (count($viewers)>=$maxViewers) {
            throw new ProcessException(sprintf(_('Sorry, at the moment max %d viewer limit apply.'), $maxViewers));
        }
        // check landing
        if ($landing == "" || strlen($landing)>80) {
            throw new ProcessException(_('Landing page is not valid'));
        }

        $db = Database::connection();
        $db->begin();

        if ($site->getPrivate() != $private) {
            $site->setPrivate($private);

            $site->save();

            // change file flag too
            $flagDir = $site->getLocalFilesPath().'/flags';
            $flagFile = $flagDir.'/private';
            if ($private) {
                mkdirfull($flagDir); //just to make sure

                if (!file_exists($flagFile)) {
                    file_put_contents($flagFile, "private");
                }
            } else {
                if (file_exists($flagFile)) {
                    unlink($flagFile);
                }
            }
        }

        $settings = $site->getSettings();

        if ($settings->getPrivateLandingPage() != $landing) {
            $settings->setPrivateLandingPage($landing);
            $settings->save();
        }

        if ($settings->getHideNavigationUnauthorized() != $hideNav) {
            $settings->setHideNavigationUnauthorized($hideNav);
            $settings->save();
        }

        // handle viewers
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());

        $dbViewers = SiteViewerPeer::instance()->select($c);
        $viewers = array_unique($viewers);

        foreach ($dbViewers as $dbViewer) {
            if (in_array($dbViewer->getUserId(), $viewers)) {
                unset($viewers[array_search($dbViewer->getUserId(), $viewers)]);
            } else {
                SiteViewerPeer::instance()->deleteByPrimaryKey($dbViewer->getViewerId());
            }
        }
        // insert all other
        foreach ($viewers as $viewer) {
            if (trim($viewer) != '') {
                $dbViewer = new SiteViewer();
                $dbViewer->setSiteId($site->getSiteId());
                $dbViewer->setUserId($viewer);
                $dbViewer->save();
            }
        }

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveSecureAccessEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $secureMode0 = $pl->getParameterValue("secureMode");

        // we have removed the "paranoid" setting. maybe later.
        $allowedValues = array(null, '', 'ssl', 'ssl_only');
        if (!in_array($secureMode0, $allowedValues)) {
            throw new ProcessException(_("SSL mode value not allowed."));
        }

        $settings = $site->getSettings();

        $settings->setSslMode($secureMode0);
        $settings->save();
    }

    /**
     * Marks the site as "deleted" and invalidates all the cache related to the site.
     *
     * @param mixed $runData
     */
    public function deleteSiteEvent($runData)
    {
        $site = $runData->getTemp("site");

        $user = $runData->getUser();

        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $c->add("site_id", $site->getSiteId());
        $c->add("founder", true);
        $rel = AdminPeer::instance()->selectOne($c);

        if (!$rel) {
            throw new ProcessException(_("Sorry, you have no permissions to delete this site."));
        }

        $db = Database::connection();
        $db->begin();
        $oldUnixName = $site->getUnixName();
        $site->setDeleted(true);

        // remove some data.
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());

        AnonymousAbuseFlagPeer::instance()->delete($c);
        DomainRedirectPeer::instance()->delete($c);
        EmailInvitationPeer::instance()->delete($c);
        MemberApplicationPeer::instance()->delete($c);
        MemberInvitationPeer::instance()->delete($c);

        // now clear cache!

        $keys = array();
        $keys[] = 'site..'.$site->getUnixName();
        $keys[] = 'site_cd..'.$site->getCustomDomain();

        $mc = OZONE::$memcache;
        foreach ($keys as $k) {
            $mc->delete($k);
        }

        $outdater = new Outdater();
        $outdater->siteEvent('delete', $site);
        $outdater->siteEvent('sitewide_change', $site);

        // change site name!!!
        $site->setUnixName($site->getUnixName().'..del..'.time());

        $site->save();

        // remove custom domain link

        // rename the files
        @rename(WIKIJUMP_ROOT.'/web/files--sites/'.$oldUnixName, WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName());
        // delete custom domain link

        if ($site->getCustomDomain()) {
            @unlink(WIKIJUMP_ROOT.'/web/custom--domains/'.$site->getCustomDomain());
            $site->setCustomDomain(null);
        }
        $db->commit();
    }

    /**
     * Changes the "unix name" of the site and effectively its URL address.
     *
     * @param mixed $runData
     */
    public function renameSiteEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $user = $runData->getUser();
        $unixName = trim($pl->getParameterValue('unixName'));

        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $c->add("site_id", $site->getSiteId());
        $c->add("founder", true);
        $rel = AdminPeer::instance()->selectOne($c);

        if (!$rel) {
            throw new ProcessException(_("Sorry, you have no permissions to change URL of this site."));
        }

        $db = Database::connection();
        $db->begin();
        $oldUnixName = $site->getUnixName();

    // validate unix name
        $errors = array();
        if ($unixName == $site->getUnixName()) {
            $errors['unixname'] = _('The new and current addresses are the same.');
        } elseif ($unixName === null || strlen($unixName)<3 || strlen(WDStringUtils::toUnixName($unixName))<3) {
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
            $ss = SitePeer::instance()->selectOne($c) ?? null;
            if ($ss) {
                $errors['unixname'] = _('Sorry, this web address is already used by another site.');
            }
        }

        if (isset($errors['unixname'])) {
            throw new ProcessException($errors['unixname']);
        }

        // remove some data.
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());

        // now clear cache!

        $keys = array();
        $keys[] = 'site..'.$site->getUnixName();
        $keys[] = 'site_cd..'.$site->getCustomDomain();

        $mc = OZONE::$memcache;
        foreach ($keys as $k) {
            $mc->delete($k);
        }

        $outdater = new Outdater();
        $outdater->siteEvent('delete', $site);
        $outdater->siteEvent('sitewide_change', $site);

        // change site name!!!
        $site->setUnixName($unixName);

        $site->save();

        // remove custom domain link

        // rename the files
        @rename(WIKIJUMP_ROOT.'/web/files--sites/'.$oldUnixName, WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName());
        // delete custom domain link

        if ($site->getCustomDomain()) {
            @unlink(WIKIJUMP_ROOT.'/web/custom--domains/'.$site->getCustomDomain());
            symlink(
                WIKIJUMP_ROOT.'/web/files--sites/'.$site->getUnixName(),
                WIKIJUMP_ROOT.'/web/custom--domains/'.$site->getCustomDomain()
            );
        }
        $db->commit();

        $runData->ajaxResponseAdd("unixName", $site->getUnixName());
    }
}
