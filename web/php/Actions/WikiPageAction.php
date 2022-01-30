<?php
declare(strict_types=1);

namespace Wikidot\Actions;

use Ds\Set;
use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;
use Wikidot\Utils\Deleter;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;
use Wikidot\Yaml;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\Page;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageMetadata;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\DB\PageMetadataPeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\AdminPeer;
use Wikijump\Models\TagSettings;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;

class WikiPageAction extends SmartyAction
{

    protected static $AUTOINCREMENT_PAGE = 'autoincrementpage';

    public function perform($runData)
    {
    }

    public function savePageEvent($runData)
    {

        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");

        if ($pl->getParameterValue("form")) {
            $data = [];
            foreach ($runData->getParameterList()->asArray() as $name => $val) {
                $m = [];
                if (preg_match("/^field_(.*)$/", $name, $m)) {
                    $data[$m[1]] = $val;
                }
            }
            $source = Yaml::dump($data);
        } else {
            $source = trim($pl->getParameterValue("source"));
        }

        $comments = trim($pl->getParameterValue("comments"));
        $title = trim($pl->getParameterValue("title"));

        $userId = $runData->getUserId();

        if ($title === '') {
            $title = null;
        }

        $unixName = $pl->getParameterValue("wiki_page");
        $unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)

        $site = $runData->getTemp("site");

        // validate input first

        $db = Database::connection();
        $db->begin();

        if (strlen($title) > 128) {
            throw new ProcessException(_("Title of the page should not be longer than 128 characters."), "title_too_long");
        }
        // if page source not too long...
        if (strlen($source) > 200000) {
            throw new ProcessException(_("Source of the page should not be longer than 200 000 characters which is large enough. Pages longer than that can indicate improper usage 	of the Wiki site."), "source_too_long");
        }
        // if comment too long
        if (strlen($comments) > 210) {
            throw new ProcessException(_("The changes comment is longer than 200 characters. Please keep this description short and informative. And no longer than this limit please..."), "comment_too_long");
        }

        $autoincrement = false;

        $nowDate = new ODate();
        if ($pageId === null || $pageId === '') {
            if (preg_match(';^([a-z0-9]+:)?'.self::$AUTOINCREMENT_PAGE.'$;', $unixName)) {
                $autoincrement = true;
            }
            // a page should be created!

            // extract category name
            if (strpos($unixName, ':') != false) {
                // ok, there is category!
                $exp = explode(':', $unixName);
                $categoryName = $exp[0];
            } else {
                // no category name, "_default" assumed
                $categoryName = "_default";
            }

            // check if category exists. if not - create it!
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
                $category->save();
            }

            // first look at permissions!

            WDPermissionManager::instance()->hasPagePermission('create', $runData->getUser(), $category);

            /* Change unixName to integer. */
            if ($autoincrement) {
                /* Check max number taken. */
                $db = Database::connection();
                $q = "select max(substring(unix_name from '[0-9]+')::integer) + 1 as max from page where category_id={$category->getCategoryId()} AND unix_name ~ '^([a-z0-9]+:)?[0-9]+$'";
                $r = $db->query($q);
                $row = $r->nextRow();
                $unixName = $row['max'];
                if ($category->getName() != '_default') {
                    $unixName = $category->getName() . ':' . $unixName;
                }
                $runData->ajaxResponseAdd('pageUnixName', $unixName);
            }

            $page = new Page();
            $page->obtainPK();

            $pageRevision = new PageRevision();
            $pageRevision->setSiteId($site->getSiteId());
            $pageRevision->setPageId($page->getPageId());
            $pageRevision->setFlagNew(true);
            $pageRevision->setComments($comments);
            $pageRevision->obtainPK();
            $pageRevision->setDateLastEdited($nowDate);

            $pageRevision->setPageId($page->getPageId());
            $page->setRevisionId($pageRevision->getRevisionId());

            $source_hash = DeepwellService::getInstance()->addText($source);
            $pageRevision->setWikitextHash($source_hash);
            // HACK: We need to insert now but Outdater runs later,
            // so for now we have the hash for an empty string
            $pageRevision->setCompiledHash('cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e');
            $pageRevision->setCompiledGenerator('');

            $pageMetadata = new PageMetadata();
            $pageMetadata->setTitle($title);

            $pageMetadata->setUnixName($unixName);
            if ($userId) {
                $pageMetadata -> setOwnerUserId($userId);
            }
            $pageMetadata->save();
            $pageRevision->setMetadataId($pageMetadata->getMetadataId());

            // update the page object

            $page->setUnixName($unixName);
            $page->setDateCreated($nowDate);
            $page->setSiteId($site->getSiteId());
            $page->setMetadataId($pageMetadata->getMetadataId());
            $page->setTitle($title);
            $page->setDateLastEdited($nowDate);
            $page->setTagsArray([]);

            $page->setCategoryId($category->getCategoryId());

            // now set user_id, user_string
            if ($userId === null) {
                $userString = $runData->createIpString();
                $pageRevision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $pageRevision->setUserString($userString);
                $page->setLastEditUserString($userString);
            } else {
                $pageRevision->setUserId($userId);
                $page->setLastEditUserId($userId);
            }

            $page->setOwnerUserId($userId);

            $pageRevision->save();
            $page->setRevisionId($pageRevision->getRevisionId());
            $page->save();
            $db->commit();

            $outdater = new Outdater();
            $outdater->pageEvent("new_page", $page);
        } else {
            // THE PAGE ALREADY EXISTS

            $c = new Criteria();
            $c->add("page_id", $pageId);
            $c->setForUpdate(true);
            $page = PagePeer::instance()->selectOne($c);

            if ($page == null) {
                throw new ProcessException(_("Page does not exist."));
            }

            // check permissions
            $category = $page->getCategory();
            WDPermissionManager::instance()->hasPagePermission('edit', $runData->getUser(), $category, $page);

            // check if source or metadata has changed. if neither is changed - do nothing

            // get current revision
            $currentRevision = $page->getCurrentRevision();

            // compare source text
            $oldSourceText = $page->getSource();
            $sourceChanged = false;

            if ($oldSourceText !== $source) {
                $sourceChanged = true;
            }

            // create new revision
            $pageRevision = new PageRevision();
            $pageRevision->setSiteId($site->getSiteId());

            // compare metadata
            $metadataChanged = false;
            $oldMetadata = $page->getMetadata();
            // check only if the whole page is edited
            if ($title !== $oldMetadata->getTitle()) {
                $pageRevision->setFlagTitle(true);
                $metadataChanged = true;
            }

            // and act accordingly to the situation

            if ($sourceChanged == false && $metadataChanged == false) {
                $db->commit();
                return;
            }

            $pageRevision->setPageId($page->getPageId());
            $pageRevision->setDateLastEdited($nowDate);
            $pageRevision->setRevisionNumber($currentRevision->getRevisionNumber()+1);
            $pageRevision->setFlagText(true);

            if ($metadataChanged) {
                $pageMetadata = clone($oldMetadata);
                $pageMetadata->setNew(true);
                $pageMetadata->setMetadataId(null);
                $pageMetadata->setTitle($title);
                $pageMetadata->save();

                $pageRevision->setMetadataId($pageMetadata->getMetadataId());
            } else {
                // copy metadata id
                $pageRevision->setMetadataId($currentRevision->getMetadataId());
            }

            // now set user_id, user_string

            if ($userId) {
                $pageRevision->setUserId($userId);
                $page->setLastEditUserId($userId);
            } else {
                $userString = $runData->createIpString();
                $pageRevision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $pageRevision->setUserString($userString);
                $page->setLastEditUserString($userString);
            }

            $source_hash = DeepwellService::getInstance()->addText($source);

            $pageRevision->setComments($comments);
            $pageRevision->setWikitextHash($source_hash);
            // HACK: We need to insert now but Outdater runs later,
            // so for now we have the hash for an empty string
            $pageRevision->setCompiledHash('cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e');
            $pageRevision->setCompiledGenerator('');
            $pageRevision->save();
            $page->setRevisionId($pageRevision->getRevisionId());

            // update Page object

            $page->setTitle($title);
            $page->setDateLastEdited($nowDate);
            $page->setMetadataId($pageRevision->getMetadataId());
            $page->setRevisionNumber($pageRevision->getRevisionNumber());
            $page->save();
            $db->commit();

            // OUTDATING PARTY!!!
            $outdater = new Outdater();
            if ($sourceChanged) {
                $outdater->pageEvent("source_changed", $page);
            }
            if ($metadataChanged) {
                $outdater->pageEvent("title_changed", $page);
            }
        }
    }

    public function renamePageEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");
        $newName = trim($pl->getParameterValue("new_name"));
        $newName = WDStringUtils::toUnixName($newName); // purify! (for sure)

        $site = $runData->getTemp("site");

        if ($newName == null || $newName == '') {
            throw new ProcessException(_("Destination page name should be given."), "no_new_name");
        }

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->setForUpdate(true);
        $page = PagePeer::instance()->selectOne($c);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        if ($newName == $page->getUnixName()) {
            throw new ProcessException(_("The current and new names are the same."), "page_exists");
        }

        // check for permissions again
        $category = $page->getCategory();

        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('move', $user, $category, $page);

        // check if the new page exists or not.

        $conflictPage = PagePeer::instance()->selectByName($site->getSiteId(), $newName);
        if ($conflictPage != null) {
            throw new ProcessException(_("The destination page already exists."), "page_exists");
        }

        $oldName = $page->getUnixName();

        // check if new page exists!

        // success so far...

        // create new revision, new metadata and alter the page object too.
        $oldMetadata = $page->getMetadata();
        $metadata = clone($oldMetadata);
        $metadata->setNew(true);
        $metadata->setMetadataId(null);
        $metadata->setUnixName($newName);
        $metadata->save();

        $oldRevision = $page->getCurrentRevision();
        $revision = new PageRevision();
        $revision->setSiteId($site->getSiteId());
        $revision->setPageId($page->getPageId());
        $revision->setMetadataId($metadata->getMetadataId());
        $revision->setFlagRename(true);
        $revision->setRevisionNumber($oldRevision->getRevisionNumber()+1);

        $revision->setComments(_("Page name changed").": \"$oldName\" "._("to")." \"$newName\".");

        $revision->setWikitextHash($oldRevision->getWikitextHash());
        // HACK: Outdater is structured so this will be updated eventually, but the column can't be null
        // So for now we just set it to the hash for an empty string...
        // Eventually we want this to be done in a way that preserves the non-null constraint
        $revision->setCompiledHash('cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e');
        $revision->setCompiledGenerator('');

        $userId = $runData->getUserId();
        if ($userId === null) {
            $userString = $runData->createIpString();
            $revision->setUserId(User::ANONYMOUS_USER);
            $page->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $page->setLastEditUserString($userString);
        } else {
            $revision->setUserId($userId);
            $page->setLastEditUserId($userId);
        }

        $now = new ODate();
        $revision->setDateLastEdited($now);
        $revision->save();

        // alter the page info
        $page->setRevisionId($revision->getRevisionId());
        $page->setDateLastEdited($now);
        $page->setUnixName($newName);
        $page->setRevisionNumber($revision->getRevisionNumber());

        // handle the categories
        // extract category name
        if (strpos($newName, ':') != false) {
            // ok, there is category!
            $exp = explode(':', $newName);
            $categoryName = $exp[0];
        } else {
            // no category name, "_default" assumed
            $categoryName = "_default";
        }
        if (strpos($oldName, ':') != false) {
            // ok, there is category!
            $exp = explode(':', $oldName);
            $oldCategoryName = $exp[0];
        } else {
            // no category name, "_default" assumed
            $oldCategoryName = "_default";
        }
        $page->save();

        $outdater = new Outdater();

        if ($categoryName !== $oldCategoryName) {
            // check if new category exists. if not - create it!

            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId(), false);
            if ($category == null) {
                // create the category - just clone the default category!!!
                $category = CategoryPeer::instance()->selectByName("_default", $site->getSiteId(), false);
                $category->setName($categoryName);
                // fill with some important things - we assume the _default category exists!!! IT REALLY SHOULD!!!
                $category->setCategoryId(null);
                $category->setNew(true); // this will make it INSERT, not UPDATE on save()
                $category->setPermissionsDefault(true);
                $category->setThemeDefault(true);
                $category->setLicenseDefault(true);
                $category->setNavDefault(true);
                $category->save();
            } else {
                //also check if one has permissions to create new pages in
                // the new category!!!
                try {
                    WDPermissionManager::instance()->hasPagePermission('create', $user, $category);
                } catch (Exception $e) {
                    throw new ProcessException(_("You are not allowed to create new pages in the destination category")." \"".$category->getName()."\".", "not_allowed");
                }
            }
            $page->setCategoryId($category->getCategoryId());
            $page->save();

            // also see if the old category is empty - if yes - delete it!
            if ($oldCategoryName != "_default") {
                $category = CategoryPeer::instance()->selectByName($oldCategoryName, $site->getSiteId(), false);

                $c = new Criteria();
                $c->add("category_id", $category->getCategoryId());
                $count = PagePeer::instance()->selectCount($c);

                if ($count == 0) {
                    // delete the category
                    CategoryPeer::instance()->delete($c);
                    $outdater->categoryEvent('delete', $category, $site);
                }
            }
        }

        // outdate party!

        $outdater->pageEvent("rename", $page, $oldName);

        // index page

        // move files too
        $oldDir = WIKIJUMP_ROOT."/web/files--sites/".
            $site->getSlug()."/files/".$oldName;
        $newDir =  WIKIJUMP_ROOT."/web/files--sites/".
            $site->getSlug()."/files/".$newName;

        if (file_exists($oldDir)) {
            if (rename($oldDir, $newDir) == false) {
                throw new ProcessException(_("Error moving attached files."), "error_files");
            }
        }

        $oldRDir = WIKIJUMP_ROOT."/web/files--sites/".
                $site->getSlug()."/resized-images/".$oldName;
        $newRDir = WIKIJUMP_ROOT."/web/files--sites/".
                $site->getSlug()."/resized-images/".$newName;

        if (file_exists($oldRDir)) {
            if (rename($oldRDir, $newRDir) == false) {
                throw new ProcessException(_("Error moving attached (resized) files."), "error_files");
            }
        }

        // check any dependency left
        $c = new Criteria();
        $q = "SELECT page_id, title, unix_name FROM page_link, page " .
                "WHERE page_link.to_page_name='".db_escape_string($oldName)."' " .
                "AND page_link.from_page_id=page.page_id AND page.site_id={$site->getSiteId()} ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);

        $pages = PagePeer::instance()->select($c);

        $q = "SELECT page_id, title, unix_name FROM page, page_inclusion " .
                "WHERE page_inclusion.included_page_name='".db_escape_string($oldName)."' " .
                "AND page_inclusion.including_page_id=page.page_id AND page.site_id={$site->getSiteId()} ORDER BY COALESCE(title, unix_name)";

        $c->setExplicitQuery($q);
        $runData->ajaxResponseAdd("newName", $newName);
        $db->commit();

        sleep(1);
    }

    public function setParentPageEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId"); // originating page id.
        $ppName = WDStringUtils::toUnixName(trim($pl->getParameterValue("parentName")));

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->setForUpdate(true);
        $page = PagePeer::instance()->selectOne($c);

        if ($page == null) {
            throw new ProcessException(_("Error: original page does not exist any more...???"), "no_page");
        }

        // check permissions
        $user = $runData->getUser();
        $category = $page->getCategory();
        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        if ($ppName == null || $ppName === '') {
            $ppName = null;
            $ppId = null;
        } else {
            // get the page!
            $pp = PagePeer::instance()->selectByName($site->getSiteId(), $ppName);
            if ($pp == null) {
                // page does not exist. return error
                throw new ProcessException(_("The requested page does not exist. Please indicate a parent page that already exists."), "no_parent_page");
            }
            // check if not "self"
            if ($pp->getPageId() == $page->getPageId()) {
                throw new ProcessException(_("Cannot set parent page to this page."), "loop_error");
            }

            // check permissions to edit the parent page (???) - it somehow affects
            // the parrent page when listing childpages or making pagetree

            $category2 = $pp->getCategory();
            try {
                WDPermissionManager::instance()->hasPagePermission('edit', $user, $category2);
            } catch (Exception $e) {
                throw new ProcessException(_('You are not allowed to alter contents of the parent page. You should have the "edit" permission on the parent page too.'), "not_allowed");
            }
            $ppId = $pp->getPageId();
        }

        // now check if the parent_page_id has changed...

        if ($page->getParentPageId() != $ppId) {
            // need to change...

            // create a new revision!!!!!!!!!!!!!!!
            // create new revision, new metadata and alter the page object too.
            $oldMetadata = $page->getMetadata();
            $metadata = clone($oldMetadata);
            $metadata->setNew(true);
            $metadata->setMetadataId(null);
            $metadata->setParentPageId($ppId);
            $metadata->save();

            $revision = $page->getCurrentRevision();
            $revision->setNew(true);
            $revision->setRevisionId(null);
            $revision->resetFlags();
            $revision->setFlagMeta(true);
            $revision->setMetadataId($metadata->getMetadataId());

            $revision->setRevisionNumber($revision->getRevisionNumber() +1);
            $now = new ODate();
            $revision->setDateLastEdited($now);

            $revision->setComments(_("Parent page set to").": \"$ppName\".");

            $userId = $runData->getUserId();
            if ($userId == null) {
                $userString = $runData->createIpString();
            }
            if ($userId) {
                $revision->setUserId($userId);
                $page->setLastEditUserId($userId);
            } else {
                $revision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $revision->setUserString($userString);
                $page->setLastEditUserString($userString);
            }

            $revision->setDateLastEdited($now);
            $revision->save();

            // alter the page info
            $page->setRevisionId($revision->getRevisionId());
            $page->setRevisionNumber($revision->getRevisionNumber());
            $page->setDateLastEdited($now);
            $page->setParentPageId($ppId);

            $page->save();

            // outdate page
            $od = new Outdater();
            $od->pageEvent('parent_changed', $page);
        }

        $db->commit();
    }

    public function revertEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $revisionId = $pl->getParameterValue("revisionId");

        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->setForUpdate(true);
        $page = PagePeer::instance()->selectOne($c);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        // check for permissions again
        $category = $page->getCategory();

        $user = $runData->getUser();
        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // get the revision

        $toRevision = PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        $toMeta = PageMetadataPeer::instance()->selectByPrimaryKey($toRevision->getMetadataId());
        $currentRevision = $page->getCurrentRevision();

        $currentMeta = $currentRevision->getMetadata();

        // success so far...

        $titleChanged = false;
        if ($toMeta->getTitle() !== $currentMeta->getTitle()) {
            // change the title, need to create a new metadata...
            $metadata = clone($currentMeta);
            $metadata->setMetadataId(null);
            $metadata->setNew(true);
            $metadata->setTitle($toMeta->getTitle());
            $metadata->save();
            $titleChanged = true;
        }

        $userId = $runData->getUserId();
        if ($userId === null) {
            $userString = $runData->createIpString();
        }

        if ($toRevision->getSourceId() !== $currentRevision->getSourceId()) {
            $nSource = $toRevision->getSourceText();
            $oSource = $currentRevision->getSourceText();
            $sourceChanged = $nSource === $oSource;
        }

        $revision = clone($currentRevision);
        $revision->setNew(true);
        $revision->setRevisionId(null);
        $revision->resetFlags();
        if ($sourceChanged) {
            $revision->setFlagText(true);
        }
        if ($titleChanged) {
            $revision->setFlagTitle(true);
            $revision->setMetadataId($metadata->getMetadataId());
            $page->setTitle($toMeta->getTitle());
        }

        $revision->setComments(_("Reverted to page revision number")." ".$toRevision->getRevisionNumber());

        if ($userId) {
            $revision->setUserId($userId);
            $page->setLastEditUserId($userId);
        } else {
            $revision->setUserId(User::ANONYMOUS_USER);
            $page->setLastEditUserId(User::ANONYMOUS_USER);
            $revision->setUserString($userString);
            $page->setLastEditUserString($userString);
        }

        $revision->setRevisionNumber($revision->getRevisionNumber() + 1);
        $revision->setWikitextHash($toRevision->getWikitextHash());
        $revision->setCompiledHash($toRevision->getCompiledHash());
        $revision->setCompiledGenerator($toRevision->getCompiledGenerator());

        $now = new ODate();
        $revision->setDateLastEdited($now);

        $revision->save();
        $page->setRevisionId($revision->getRevisionId());
        $page->setDateLastEdited($now);
        $page->setRevisionNumber($revision->getRevisionNumber());
        $page->save();

        // outdate party!
        $outdater = new Outdater();
        if ($sourceChanged) {
            $outdater->pageEvent("source_changed", $page);
        }
        if ($titleChanged) {
            $outdater->pageEvent("title_changed", $page);
        }
        // index page
        $db->commit();

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveTagsEvent($runData)
    {
        // do not create any new revision... :-(
        // or create???

        $user = $runData->getUser();
        $pl = $runData->getParameterList();
        $tags = strtolower(trim($pl->getParameterValue("tags")));
        $page_id = $pl->getParameterValue("pageId");

        $site = $runData->getTemp("site");
        $page = PagePeer::instance()->selectByPrimaryKey($page_id);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $category = $page->getCategory();

        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // Turn the tags into a set.
        // We have a check here because preg_split() on an empty string yields ['']
        $current_tags = $tags === '' ? new Set() : new Set(preg_split('/[, ]+/', $tags));
        $previous_tags = PagePeer::getTags($page_id);

        // TODO: concept of roles
        $role_ids = new Set();

        // Get tag settings, and ensure the tags pass the current configuration

        // TODO: allow multiple tag configurations per site
        // Currently this finds the first configuration with the (nullable?) site ID matching
        // This is because I don't want to further mess with the site table until we refactor it

        $tag_settings = TagSettings::where(['site_id' => $site->getSiteId()])->first();
        if ($tag_settings !== null) {
            $tag_configuration = $tag_settings->getConfiguration();
            $tag_decision = TagEngine::validate($tag_configuration, $previous_tags, $current_tags, $role_ids);

            if (!$tag_decision->valid) {
                throw new ProcessException(__('processing.tags.errors.INVALID_TAGS'), 'form_error');
            }
        }

        // Save the tags.
        PagePeer::saveTags($page_id, $current_tags);

        $od = new Outdater();
        $od->pageEvent("tag_change", $page);

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveBlockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $user = $runData->getUser();

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if (!$pageId || $page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        // check if can!

        if ($this->canSetBlock($user, $page) == false) {
            throw new WDPermissionException(_("Sorry, only Site Admnistrators and selected Moderators can block a page."));
        }

        $block = (bool)$pl->getParameterValue("block");
        $page->setBlocked($block);
        $page->save();

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    private function canSetBlock($user, $page)
    {

        if ($user->id === User::ADMIN_USER) {
            return true;
        }

        if (!$user) {
            return false;
        }

        // still nothing. check if moderator of "pages".
        $c = new Criteria();
        $c->add("site_id", $page->getSiteId());
        $c->add("user_id", $user->id);
        $rel = ModeratorPeer::instance()->selectOne($c);
        if ($rel && strpos($rel->getPermissions(), 'p') !== false) {
            return true;
        }

        // still nothing. check if admin.
        $c = new Criteria();
        $c->add("site_id", $page->getSiteId());
        $c->add("user_id", $user->id);
        $rel = AdminPeer::instance()->selectOne($c);
        if ($rel) {
            return true;
        }

        return false;
    }

    public function deletePageEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");

        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->setForUpdate(true);
        $page = PagePeer::instance()->selectOne($c);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $user = $runData->getUser();
        $category = $page->getCategory();
        WDPermissionManager::instance()->hasPagePermission('delete', $user, $category, $page);

        // ok, delete... sad but true.

        $deleter = Deleter::instance();
        $deleter->deletePage($page, $site);

        $db->commit();

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }
}
