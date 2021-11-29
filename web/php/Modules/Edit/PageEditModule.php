<?php

namespace Wikidot\Modules\Edit;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PageEditLock;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDEditUtils;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;
use Wikidot\Form;
use Wikidot\Form\Renderer;
use Wikidot\DB\PageEditLockPeer;

use Ozone\Framework\SmartyModule;

class PageEditModule extends SmartyModule
{

    protected static $AUTOINCREMENT_PAGE = 'autoincrementpage';

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $pageId = $pl->getParameterValue("page_id");

        $user = $runData->getUser();

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
            $runData->contextAdd("anonymousString", $userString);
        }

        $db = Database::connection();
        $db->begin();

        if ($pageId === null || $pageId==='') {
            // means probably creating a new page
            // no context is needed
            $runData->sessionStart();
            $runData->contextAdd("newPage", true);

            // first create if a page not already exists!
            $unixName = $pl->getParameterValue("wiki_page");
            $unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)

            if (!$unixName) {
                throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
            }

            $page = PagePeer::instance()->selectByName($site->getSiteId(), $unixName);
            if ($page != null) {
                // page exists!!! error!
                throw new ProcessException(_("The page you want to create already exists. Please refresh the page in your browser to see it."));
            /*  $runData->ajaxResponseAdd("pageExists", true);
                $runData->ajaxResponseAdd("locked", true); //well, it is somehow locked...
                $runData->setModuleTemplate("Edit/NewPageExistsWinModule");
                $db->commit();
                return; */
            }

            // extract category name
            if (strpos($unixName, ':') != false) {
                // ok, there is category!
                $exp = explode(':', $unixName);
                $categoryName = $exp[0];
                $suggestedTitle = ucwords(str_replace("-", " ", $exp[1]));
            } else {
                // no category name, "_default" assumed
                $categoryName = "_default";
                $suggestedTitle = ucwords(str_replace("-", " ", $unixName));
            }

            $stitle = $pl->getParameterValue("title");
            if ($stitle) {
                $suggestedTitle = $stitle;
            }

            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());

            if ($category == null) {
                // get the default!
                //$category = Wikidot_DB_CategoryPeer::instance()->selectByName('_default', $site->getSiteId());
                $category = $this->createTempCategory($categoryName, $site);
            }

            // now check for permissions!!!
            WDPermissionManager::instance()->hasPagePermission('create', $user, $category);
            $autoincrement = false;
            if (preg_match(';^([a-z0-9]+:)?'.self::$AUTOINCREMENT_PAGE.'$;', $unixName)) {
                $autoincrement = true;
            }
            if (!$autoincrement) {
                $lock = new PageEditLock();
                $lock->setPageUnixName($unixName);
                $lock->setSiteId($site->getSiteId());
                $lock->setUserId($runData->getUserId());
                $lock->setUserString($runData->getSession()->getIpAddress());

                $lock->setDateStarted(new ODate());
                $lock->setDateLastAccessed(new ODate());
                $lock->setMode("page");

                if ($pl->getParameterValue("force_lock") != null) {
                    $lock->deleteConflicts();
                } else {
                    // check for conflicts
                    $conflicts = $lock->getConflicts();
                    if ($conflicts != null) {
                        $runData->ajaxResponseAdd("locked", true);
                        $runData->setModuleTemplate("Edit/NewPageLockedWinModule");
                        $runData->contextAdd("locks", $conflicts);
                        return;
                    }
                }

                $secret = md5(time().rand(1000, 9999));
                $lock->setSecret($secret);
                $lock->setSessionId($runData->getSession()->getSessionId());
                $lock->save();
                $runData->ajaxResponseAdd('lock_id', $lock->getLockId());
                $runData->ajaxResponseAdd('lock_secret', $secret);
            } else {
                $runData->contextAdd('disableLocks', true);
                $runData->ajaxResponseAdd('disableLocks', true);
            }
            $runData->contextAdd("title", $suggestedTitle);


            /* Select available templates, but only if the category does not have a live template. */

            $templatePage = $category->getTemplatePage();

            if ($templatePage && $form = Form::fromSource($templatePage->getSource())) {
                $runData->contextAdd("form", new Renderer($form));
            } elseif (!$templatePage || !preg_match('/^={4,}$/sm', $templatePage->getSource())) {
                $templatesCategory = CategoryPeer::instance()->selectByName("template", $site->getSiteId());

                if ($templatesCategory != null) {
                    $c = new Criteria();
                    $c->add("category_id", $templatesCategory->getCategoryId());
                    $c->addOrderAscending("title");
                    $templates =  PagePeer::instance()->select($c);

                    $runData->contextAdd("templates", $templates);
                }

                // check if there is a default template...


                if ($category != null) {
                    if ($category->getTemplateId() != null) {
                        $runData->contextAdd("templateId", $category->getTemplateId());
                    }
                }
            } else {
                /* Has default template, try to populate the edit box with initial content. */
                $templateSource = $templatePage->getSource();
                $split = preg_split('/^={4,}$/sm', $templateSource);
                if (count($split) >= 2) {
                    /* Fine, there is some initial content. */
                    $templateSource = trim(preg_replace("/^.*?\n={4,}/s", '', $templateSource));
                } else {
                    $templateSource = '';
                }
                $runData->contextAdd('source', $templateSource);
            }


            $db->commit();
            return;
        }

        // now if editing an existing page...

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if (!$page || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $category = $page->getCategory();
        if ($category == null) {
            throw new ProcessException(_("Internal error - page category does not exist!!!"));
        }

        // now check for permissions!

        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // now check if form is defined

        $templatePage = $category->getTemplatePage();

        if (preg_match('/^[^:]*:[^_]|^[^_:][^:]*$/', $page->getUnixName())
            && $templatePage && $form = Form::fromSource($templatePage->getSource())
        ) {
            $form->setDataFromYaml($page->getSource());
            $runData->contextAdd("form", new Renderer($form));
        }

        // if we have not returned yet it means that the lock does not exist or is expired
        // if session is not started - start it!
        $runData->sessionStart();
        // create new page lock
        $lock = new PageEditLock();
        $lock->setPageId($page->getPageId());
        $lock->setPageUnixName($page->getUnixName());
        $lock->setSiteId($site->getSiteId());
        $lock->setUserId($runData->getUserId());
        $lock->setUserString($runData->getSession()->getIpAddress());

        $lock->setDateStarted(new ODate());
        $lock->setDateLastAccessed(new ODate());

        // delete outdated...
        PageEditLockPeer::instance()->deleteOutdated($pageId);
        // check for conflicts

        if ($pl->getParameterValue("force_lock") != null) {
            $lock->deleteConflicts();
        } else {
            $blocklocks = $lock->getConflicts();
            if ($blocklocks != null) {
                // conflicting locks exist.
                $runData->setModuleTemplate("Edit/LockExistsWinModule");
                $runData->ajaxResponseAdd("locked", true);
                $runData->contextAdd("locks", $blocklocks);
                return;
            }
        }

        $secret = md5(time().rand(1000, 9999));
        $lock->setSecret($secret);
        $lock->setSessionId($runData->getSession()->getSessionId());
        $lock->save();

        $runData->ajaxResponseAdd('lock_id', $lock->getLockId());
        $runData->ajaxResponseAdd('lock_secret', $secret);
        // also put current page revision in case one wants to regain lock after expired.

        $runData->ajaxResponseAdd('page_revision_id', $page->getRevisionId());

        // keep the session - i.e. put an object into session storage not to delete it!!!
        $runData->sessionAdd("keep", true);

        $pageSource = $page->getSource();
        $runData->contextAdd("source", $pageSource);
        $runData->contextAdd("title", $page->getTitleRaw());
        $runData->contextAdd("pageId", $page->getPageId());

        $runData->ajaxResponseAdd("timeLeft", 15*60);

        $db->commit();
    }

    protected function createTempCategory($categoryName, $site)
    {
        $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
        if ($category == null) {
            // create the category - just clone the default category!!!
            $category = CategoryPeer::instance()->selectByName("_default", $site->getSiteId(), false);
            $category->setName($categoryName);
            // fill with some important things - we assume the _default category exists!!! IT REALLY SHOULD!!!
            $category->setCategoryId(null);
            $category->setNew(true); // this will make it INSERT, not UPDATE on save()
            $category->setPerPageDiscussion(null); //default value
            // set default permissions theme and license
            $category->setPermissionsDefault(true);
            $category->setThemeDefault(true);
            $category->setLicenseDefault(true);
            $category->setNavDefault(true);
        }

        return $category;
    }
}
