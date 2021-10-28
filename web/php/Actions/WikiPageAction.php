<?php

namespace Wikidot\Actions;

use Ds\Set;
use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\OzoneLogger;
use Ozone\Framework\SmartyAction;
use Wikidot\Utils\Deleter;
use Wikidot\Utils\DependencyFixer;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;
use Wikidot\Utils\WDStringUtils;
use Wikidot\Yaml;
use Wikidot\DB\PageEditLockPeer;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\PageEditLock;
use Wikidot\DB\Page;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageSource;
use Wikidot\DB\PageMetadata;
use Wikidot\DB\PageCompiled;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\DB\PageMetadataPeer;
use Wikidot\DB\AllowedTags;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\AdminPeer;
use Wikijump\Models\User;
use Illuminate\Support\Facades\DB;



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

        $mode = $pl->getParameterValue("mode");

        if ($pl->getParameterValue("form")) {
            $data = array();
            $newpages = array();
            foreach ($runData->getParameterList()->asArray() as $name => $val) {
                $m = array();
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
        if ($userId == null) {
            $userString = $runData->createIpString();
        }

        if ($title ==='') {
            $title = null;
        }

        $unixName = $pl->getParameterValue("wiki_page");
        $unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)

        $lockId = $pl->getParameterValue("lock_id");
        $lockSecret = $pl->getParameterValue("lock_secret");

        $site = $runData->getTemp("site");

        // validate input first

        $db = Database::connection();
        $db->begin();

        // remove old locks.
        if (strlen($title)>128) {
            throw new ProcessException(_("Title of the page should not be longer than 128 characters."), "title_too_long");
        }
        // if page source not too long...
        if (strlen($source)>200000) {
            throw new ProcessException(_("Source of the page should not be longer than 200 000 characters which is large enough. Pages longer than that can indicate improper usage 	of the Wiki site."), "source_too_long");
        }
        // if comment too long
        if (strlen($comments)>210) {
            throw new ProcessException(_("The changes comment is longer than 200 characters. Please keep this description short and informative. And no longer than this limit please..."), "comment_too_long");
        }

        $autoincrement = false;

        $nowDate = new ODate();
        if ($pageId === null || $pageId==='') {
            if (preg_match(';^([a-z0-9]+:)?'.self::$AUTOINCREMENT_PAGE.'$;', $unixName)) {
                $autoincrement = true;
            }
            if (!$autoincrement) {
                PageEditLockPeer::instance()->deleteOutdatedByPageName($site->getSiteId(), $unixName);
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

            // check the locks!
            // check if the lock still exists.
            if (!$autoincrement) {
                $c = new Criteria();
                $c->add("lock_id", $lockId);
                $c->add("secret", $lockSecret);

                $lock = PageEditLockPeer::instance()->selectOne($c);
                if ($lock == null) {
                    $page = PagePeer::instance()->selectByName($site->getSiteId(), $unixName);
                    if ($page != null) {
                        // page exists!!! error!
                        $runData->ajaxResponseAdd("noLockError", "other_locks");
                        $runData->ajaxResponseAdd("pageExists", true);
                        $runData->ajaxResponseAdd("locked", true); //well, it is somehow locked...
                        $runData->setModuleTemplate("Edit/NewPageExistsWinModule");
                        $runData->contextAdd("nonrecoverable", true);
                        $runData->ajaxResponseAdd("nonrecoverable", true);
                        $db->commit();
                        return;
                    }

                    // check if we can TRANSPARENTLY recreate the lock IF there is no
                    // conflicting lock and the revision_id has not changed.
                    $lock = new PageEditLock();

                    $lock->setPageUnixName($unixName);
                    $lock->setSiteId($site->getSiteId());
                    $lock->setUserId($runData->getUserId());
                    $lock->setUserString($runData->getSession()->getIpAddress());

                    $lock->setDateStarted(new ODate());
                    $lock->setDateLastAccessed(new ODate());
                    $lock->setMode("page");

                    $conflictLocks = $lock->getConflicts();
                    if ($conflictLocks == null) {
                        // safely recreate lock
                        $secret = md5(time().rand(1000, 9999));
                        $lock->setSecret($secret);
                        $lock->setSessionId($runData->getSession()->getSessionId());
                        $lock->save();
                        $lockId = $lock->getLockId();

                        // send back new lock information
                        $runData->ajaxResponseAdd("lockRecreated", true);
                        $runData->ajaxResponseAdd("lockId", $lockId);
                        $runData->ajaxResponseAdd("lockSecret", $secret);
                        $runData->ajaxResponseAdd('timeLeft', 60*15);
                    } else {
                        $runData->ajaxResponseAdd("noLockError", "other_locks");
                        $runData->setModuleTemplate("Edit/LockInterceptedWinModule");
                        $runData->contextAdd("locks", $conflictLocks);
                        $db->commit();
                        return;
                    }
                } else {
                    $lock->setDateLastAccessed(new ODate());
                    $lock->save();
                    $runData->ajaxResponseAdd('timeLeft', 60*15);
                }
            }

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

            $pageSource = new PageSource();
            $pageSource->setText($source);
            $pageSource->save();
            $pageRevision->setSourceId($pageSource->getSourceId());

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
            $page->setSourceId($pageSource->getSourceId());
            $page->setMetadataId($pageMetadata->getMetadataId());
            $page->setTitle($title);
            $page->setDateLastEdited($nowDate);
            $page->setTags([]);

            $pageCompiled = new PageCompiled();
            $pageCompiled->setPageId($page->getPageId());
            $pageCompiled->outdate();

            $page->setCategoryId($category->getCategoryId());

            // now set user_id, user_string
            if ($userId) {
                $pageRevision->setUserId($userId);
                $page->setLastEditUserId($userId);
            } else {
                $pageRevision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $pageRevision->setUserString($userString);
                $page->setLastEditUserString($userString);
            }

            $page->setOwnerUserId($userId);

            $pageRevision->save();
            $page->setRevisionId($pageRevision->getRevisionId());
            $page->save();

            $pageCompiled->save();

            $sourceChanged=true;

            $outdater = new Outdater();
            $outdater->pageEvent("new_page", $page);

            // index page
            if (!$autoincrement) {
                $c = new Criteria();
                $c->add("lock_id", $lockId);
                PageEditLockPeer::instance()->delete($c);
            }
        } else {
            // THE PAGE ALREADY EXISTS

            PageEditLockPeer::instance()->deleteOutdated($pageId);

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

            // check if the lock still exists.
            $c = new Criteria();
            $c->add("lock_id", $lockId);
            $c->add("secret", $lockSecret);

            $lock = PageEditLockPeer::instance()->selectOne($c);
            if ($lock == null) {
                OzoneLogger::instance()->debug("no lock");
                // no lock!!! not good.
                if ($page->getRevisionId() != $pl->getParameterValue("revision_id")) {
                    // this is nonrecoverable.
                    // author should stop editing now!!!
                    OzoneLogger::instance()->debug("page changed");
                    $runData->ajaxResponseAdd("noLockError", "page_changed");
                    $runData->setModuleTemplate("Edit/LockInterceptedWinModule");
                    $runData->contextAdd("nonrecoverable", true);
                    $runData->ajaxResponseAdd("nonrecoverable", true);
                    $db->commit();
                    return;
                }

                // check if we can TRANSPARENTLY recreate the lock IF there is no
                // conflicting lock and the revision_id has not changed.
                $lock = new PageEditLock();
                $lock->setPageId($page->getPageId());
                $lock->setPageUnixName($page->getUnixName());
                $lock->setSiteId($site->getSiteId());
                $lock->setUserId($runData->getUserId());
                $lock->setUserString($runData->getSession()->getIpAddress());

                $lock->setDateStarted(new ODate());
                $lock->setDateLastAccessed(new ODate());
                $lock->setMode($mode);
                if ($mode == "section") {
                    $rangeStart = $pl->getParameterValue("range_start");
                    $rangeEnd = $pl->getParameterValue("range_end");
                    $lock->setRangeStart($rangeStart);
                    $lock->setRangeEnd($rangeEnd);
                }
                $conflictLocks = $lock->getConflicts();
                if ($conflictLocks == null) {
                    // safely recreate lock
                    $secret = md5(time().rand(1000, 9999));
                    $lock->setSecret($secret);
                    $lock->setSessionId($runData->getSession()->getSessionId());
                    $lock->save();
                    $lockId = $lock->getLockId();
                    // send back new lock information
                    $runData->ajaxResponseAdd("lockRecreated", true);
                    $runData->ajaxResponseAdd("lockId", $lockId);
                    $runData->ajaxResponseAdd("lockSecret", $secret);
                    $runData->ajaxResponseAdd('timeLeft', 60*15);
                } else {
                    $runData->ajaxResponseAdd("noLockError", "other_locks");
                    $runData->setModuleTemplate("Edit/LockInterceptedWinModule");
                    $runData->contextAdd("locks", $conflictLocks);
                    $db->commit();
                    return;
                }
            } else {
                $lock->setDateLastAccessed(new ODate());
                $lock->save();
                $runData->ajaxResponseAdd('timeLeft', 60*15);

                // here is a good place to check conditions for
                // "save & continue" which when first called
                // creates new revision, but the subsequent calls
                // do not.
            }

            // check if source or metadata has changed. if neither is changed - do nothing

            // get current revision
            $currentRevision = $page->getCurrentRevision();

            // compare source text
            $oldSourceText = $page->getSource();
            $sourceChanged = false;

            if ($mode == "append") {
                $source = $oldSourceText."\n\n".$source;
            }
            if ($mode == "section") {
                $rangeStart = $lock->getRangeStart(); //$pl->getParameterValue("range_start");
                $rangeEnd = $lock->getRangeEnd(); //$pl->getParameterValue("range_end");
                $s2 = explode("\n", $oldSourceText);
                // fix source last empty line
                if (!preg_match("/\n$/", $source)) {
                    $source.="\n";
                }
                array_splice($s2, $rangeStart, $rangeEnd-$rangeStart+1, explode("\n", $source));
                $source = implode("\n", $s2);
            }

            if ($oldSourceText !== $source) {
                $sourceChanged = true;
            }

            // create new revision
            $pageRevision = new PageRevision();
            $pageRevision->setSiteId($site->getSiteId());

            // compare metadata
            $metadataChanged = false;
            $oldMetadata = $page->getMetadata();
            // title
            if ($mode == 'page') {
                // check only if the whole page is edited
                if ($title !== $oldMetadata->getTitle()) {
                    $pageRevision->setFlagTitle(true);
                    $metadataChanged = true;
                }
            }

            // and act accordingly to the situation

            if ($sourceChanged == false && $metadataChanged == false) {
                $c = new Criteria();
                $c->add("lock_id", $lockId);
                PageEditLockPeer::instance()->delete($c);
                $db->commit();
                return;
            }

            $pageRevision->setPageId($page->getPageId());
            $pageRevision->setDateLastEdited($nowDate);
            $pageRevision->setRevisionNumber($currentRevision->getRevisionNumber()+1);
            if ($sourceChanged) {
                $pageSource = new PageSource();
                $pageSource->setText($source);
                $pageSource->save();

                $pageRevision->setSourceId($pageSource->getSourceId());
                $pageRevision->setFlagText(true);
            } else {
                // copy source id
                $pageRevision->setSourceId($currentRevision->getSourceId());
                $pageRevision->setSinceFullSource($currentRevision->getSinceFullSource());
                $pageRevision->setDiffSource($currentRevision->getDiffSource());
            }
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
                $pageRevision->setUserId(User::ANONYMOUS_USER);
                $page->setLastEditUserId(User::ANONYMOUS_USER);
                $pageRevision->setUserString($userString);
                $page->setLastEditUserString($userString);
            }

            $pageRevision->setComments($comments);
            $pageRevision->save();
            $page->setRevisionId($pageRevision->getRevisionId());

            // update Page object

            $page->setSourceId($pageRevision->getSourceId());
            if ($mode == 'page') {
                $page->setTitle($title);
            }
            $page->setDateLastEdited($nowDate);
            $page->setMetadataId($pageRevision->getMetadataId());
            $page->setRevisionNumber($pageRevision->getRevisionNumber());
            $page->save();

            // also if "section edit" - find other locks that refer to
            // blocks with higher line numbers and change start/end accordingly

            if ($mode == "section") {
                $c = new Criteria();
                $c->add("page_id", $pageId);
                $c->add("range_start", $lock->getRangeEnd(), ">=");
                $c->add("mode", "section");
                $laterLocks = PageEditLockPeer::instance()->select($c);
                if (count($laterLocks)>0) {
                    // take the length of the current lock
                    $sectionLength = $lock->getRangeEnd() - $lock->getRangeStart() +1;
                    $newSourceLength = count(explode("\n", trim($pl->getParameterValue("source"))))+1; // +1 for the new line at the end
                    $lengthDifference = $newSourceLength - $sectionLength;
                    foreach ($laterLocks as $llock) {
                        $llock->setRangeStart($llock->getRangeStart()+$lengthDifference);
                        $llock->setRangeEnd($llock->getRangeEnd()+$lengthDifference);
                        $llock->save();
                    }
                }
            }

            // OUTDATING PARTY!!!
            $outdater = new Outdater();
            if ($sourceChanged) {
                $outdater->pageEvent("source_changed", $page);
            }
            if ($metadataChanged) {
                $outdater->pageEvent("title_changed", $page);
            }
        }

        // remove lock too?
        if (!$pl->getParameterValue("and_continue") && !$autoincrement) {
            $c = new Criteria();
            $c->add("lock_id", $lockId);
            PageEditLockPeer::instance()->delete($c);
            $runData->ajaxResponseAdd("revisionId", $pageRevision->getRevisionId());
        }

        $db->commit();
    }

    /**
     * Simply removes page edit lock from a page.
     */
    public function removePageEditLockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $lockId =  $pl->getParameterValue("lock_id");
        $secret = $pl->getParameterValue("lock_secret");
        $c = new Criteria();
        $c->add("lock_id", $lockId);
        $c->add("secret", $secret);

        PageEditLockPeer::instance()->delete($c);
    }

    public function updateLockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");

        $site = $runData->getTemp("site");

        $mode = $pl->getParameterValue("mode");

        $unixName = $pl->getParameterValue("wiki_page");
        $unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)

        $lockId = $pl->getParameterValue("lock_id");
        $lockSecret = $pl->getParameterValue("lock_secret");

        $site = $runData->getTemp("site");
        $sinceLastInput = $pl->getParameterValue("since_last_input");
        if ($sinceLastInput == null) {
            $sinceLastInput = 0;
        }

        $db = Database::connection();
        $db->begin();

        if ($pageId!= null) {
            PageEditLockPeer::instance()->deleteOutdated($pageId);
            $c = new Criteria();
            $c->add("page_id", $pageId);
            $c->setForUpdate(true);
            $page = PagePeer::instance()->selectOne($c);
            if ($page == null) {
                throw new ProcessException(_("Cannot find the page."). "no_page");
            }
        } else {
            PageEditLockPeer::instance()->deleteOutdatedByPageName($site->getSiteId(), $unixName);
        }

        // delete outdated locks...

        // check if the lock still exists.
        $c = new Criteria();
        $c->add("lock_id", $lockId);
        $c->add("secret", $lockSecret);

        $lock = PageEditLockPeer::instance()->selectOne($c);
        $dateLastAccessed = new ODate();
        $timeLeft = 15*60 - $sinceLastInput;
        $dateLastAccessed->subtractSeconds($sinceLastInput);
        if ($lock!=null) {
            // just update

            $lock->setDateLastAccessed($dateLastAccessed);
            $lock->save();
            $runData->ajaxResponseAdd('timeLeft', $timeLeft);
        } else {
            // no lock!!! not good.
            if ($page != null && $page->getRevisionId() != $pl->getParameterValue("revision_id")) {
                // this is nonrecoverable.
                // author should stop editing now!!!
                $runData->ajaxResponseAdd("noLockError", "page_changed");
                $runData->setModuleTemplate("Edit/LockInterceptedWinModule");
                $runData->contextAdd("nonrecoverable", true);
                $runData->ajaxResponseAdd("nonrecoverable", true);
            } elseif ($page == null && PagePeer::instance()->selectByName($site->getSiteId(), $unixName) != null) {
                // page exists!
                $runData->ajaxResponseAdd("noLockError", "page_exists");
                $runData->ajaxResponseAdd("nonrecoverable", true);
                $runData->setModuleTemplate("Edit/NewPageExistsWinModule");
            } else {
                // ok, see if there are conflicts and is it possible to
                // recreate the lock.
                $lock = new PageEditLock();
                if ($page != null) {
                    $lock->setPageId($page->getPageId());
                    $lock->setPageUnixName($page->getUnixName());
                } else {
                    $lock->setPageUnixName($unixName);
                }
                $lock->setSiteId($site->getSiteId());
                $lock->setUserId($runData->getUserId());
                $lock->setUserString($runData->getSession()->getIpAddress());

                $lock->setDateStarted($dateLastAccessed);
                $lock->setDateLastAccessed($dateLastAccessed);
                $lock->setMode($mode);
                if ($mode == "section") {
                    $rangeStart = $pl->getParameterValue("range_start");
                    $rangeEnd = $pl->getParameterValue("range_end");
                    $lock->setRangeStart($rangeStart);
                    $lock->setRangeEnd($rangeEnd);
                }
                $conflictLocks = $lock->getConflicts();
                if ($conflictLocks == null) {
                    // safely recreate lock
                    $secret = md5(time().rand(1000, 9999));
                    $lock->setSecret($secret);
                    $lock->setSessionId($runData->getSession()->getSessionId());
                    $lock->save();
                    $lockId = $lock->getLockId();
                    // send back new lock information
                    $runData->ajaxResponseAdd("lockRecreated", true);
                    $runData->ajaxResponseAdd("lockId", $lockId);
                    $runData->ajaxResponseAdd("lockSecret", $secret);
                    $runData->ajaxResponseAdd('timeLeft', $timeLeft);
                } else {
                    $runData->ajaxResponseAdd("noLockError", "other_locks");
                    $runData->setModuleTemplate("Edit/LockInterceptedWinModule");
                    $runData->contextAdd("locks", $conflictLocks);
                }
            }
        }

        $db->commit();
    }

    public function forceLockInterceptEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");

        $mode = $pl->getParameterValue("mode");

        $unixName = $pl->getParameterValue("wiki_page");
        $unixName = WDStringUtils::toUnixName($unixName); // purify! (for sure)

        $lockId = $pl->getParameterValue("lock_id");
        $lockSecret = $pl->getParameterValue("lock_secret");

        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();

        if ($pageId != null) {
            $c = new Criteria();
            $c->add("page_id", $pageId);
            $c->setForUpdate(true);
            $page = PagePeer::instance()->selectOne($c);
            if ($page == null) {
                throw new ProcessException(_("Cannot find the page."). "no_page");
            }
        }
        if ($page != null && $page->getRevisionId() != $pl->getParameterValue("revision_id")) {
            $runData->setModuleTemplate("Edit/LockPageChangedWinModule");
            $runData->ajaxResponseAdd("nonrecoverable", true);
            return;
        }

        if ($page == null && PagePeer::instance()->selectByName($site->getSiteId(), $unixName) != null) {
            $runData->ajaxResponseAdd("noLockError", "page_exists");
            $runData->ajaxResponseAdd("nonrecoverable", true);
            $runData->setModuleTemplate("Edit/NewPageExistsWinModule");
        }

        // delete outdated locks...
        if ($page != null) {
            PageEditLockPeer::instance()->deleteOutdated($pageId);
        } else {
            PageEditLockPeer::instance()->deleteOutdatedByPageName($site->getSiteId(), $unixName);
        }

        $lock = new PageEditLock();
        if ($page != null) {
            $lock->setPageId($page->getPageId());
            $lock->setPageUnixName($page->getUnixName());
        } else {
            $lock->setPageUnixName($unixName);
        }
        $lock->setSiteId($site->getSiteId());

        $lock->setUserId($runData->getUserId());
        $lock->setUserString($runData->getSession()->getIpAddress());

        $lock->setDateStarted(new ODate());
        $lock->setDateLastAccessed(new ODate());
        $lock->setMode($mode);
        if ($mode == "section") {
            $rangeStart = $pl->getParameterValue("range_start");
            $rangeEnd = $pl->getParameterValue("range_end");
            $lock->setRangeStart($rangeStart);
            $lock->setRangeEnd($rangeEnd);
        }
        $secret = md5(time().rand(1000, 9999));
        $lock->setSecret($secret);
        $lock->setSessionId($runData->getSession()->getSessionId());

        $lock->deleteConflicts();
        $lock->save();

        $db->commit();

        $runData->ajaxResponseAdd('lock_id', $lock->getLockId());
        $runData->ajaxResponseAdd('lock_secret', $secret);
        $runData->ajaxResponseAdd('timeLeft', 60*15);
    }

    public function recreateExpiredLockEvent($runData)
    {
        // it should be basicly the same as updateLockEvent.
        $pl = $runData->getParameterList();
        // make sure the lock is deleted!!!
        $lockId = $pl->getParameterValue("lock_id");
        $lockSecret = $pl->getParameterValue("lock_secret");
        $c = new Criteria();
        $c->add("lock_id", $lockId);
        $c->add("secret", $lockSecret);
        PageEditLockPeer::instance()->delete($c);

        $this->updateLockEvent($runData);

        // means page has changed...
        if ($runData->contextGet("nonrecoverable") == true) {
            $runData->setModuleTemplate("Edit/LockPageChangedWinModule");
            $runData->ajaxResponseAdd("nonrecoverable", true);
            $runData->ajaxResponseAdd("pageChanged", true);
            return;
        }

        // means there are conflicting locks
        if ($runData->getModuleTemplate() == "Edit/LockInterceptedWinModule") {
            $runData->setModuleTemplate("Edit/LockExpiredConflictWinModule");
            $runData->ajaxResponseAdd("conflicts", true);
            return;
        }

        // if nothing - the lock has been successfuly recreated.
        $runData->ajaxResponseAdd('timeLeft', 60*15);
    }

    public function renamePageEvent($runData)
    {
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");
        $newName = trim($pl->getParameterValue("new_name"));
        $newName = WDStringUtils::toUnixName($newName); // purify! (for sure)

        $fixDeps = $pl->getParameterValue('fixdeps');

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

        // check for locks first
        PageEditLockPeer::instance()->deleteOutdated($pageId);

        $c = new Criteria();
        $c->add("page_id", $page->getPageId());

        if ($pl->getParameterValue("force") === "yes") {
            PageEditLockPeer::instance()->delete($c);
        }

        $locks = PageEditLockPeer::instance()->select($c);

        if (count($locks)>0) {
            $runData->ajaxResponseAdd("locks", true);
            $runData->contextAdd("locks", $locks);
            $runData->setModuleTemplate("Rename/PageLockedWin");
            $db->rollback();
            return;
        }

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
        // copy source id
        $revision->setSourceId($oldRevision->getSourceId());
        $revision->setSinceFullSource($oldRevision->getSinceFullSource());
        $revision->setDiffSource($oldRevision->getDiffSource());

        $revision->setMetadataId($metadata->getMetadataId());
        $revision->setFlagRename(true);
        $revision->setRevisionNumber($oldRevision->getRevisionNumber()+1);

        $revision->setComments(_("Page name changed").": \"$oldName\" "._("to")." \"$newName\".");

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
            $site->getUnixName()."/files/".$oldName;
        $newDir =  WIKIJUMP_ROOT."/web/files--sites/".
            $site->getUnixName()."/files/".$newName;

        if (file_exists($oldDir)) {
            if (rename($oldDir, $newDir) == false) {
                throw new ProcessException(_("Error moving attached files."), "error_files");
            }
        }

        $oldRDir = WIKIJUMP_ROOT."/web/files--sites/".
                $site->getUnixName()."/resized-images/".$oldName;
        $newRDir = WIKIJUMP_ROOT."/web/files--sites/".
                $site->getUnixName()."/resized-images/".$newName;

        if (file_exists($oldRDir)) {
            if (rename($oldRDir, $newRDir) == false) {
                throw new ProcessException(_("Error moving attached (resized) files."), "error_files");
            }
        }

        // try to fix dependencies

        if ($fixDeps && preg_match('/^[0-9]+(,[0-9]+)*$/', $fixDeps)) {
            $fixPageIds = explode(',', $fixDeps);
            foreach ($fixPageIds as $pageId) {
                $page = PagePeer::instance()->selectByPrimaryKey($pageId);
                if ($page == null || $page->getSiteId() !== $site->getSiteId()) {
                    continue;
                }

                // check for any locks
                $c = new Criteria();
                $c->add("page_id", $pageId);
                $lock = PageEditLockPeer::instance()->selectOne($c);

                if ($lock) {
                    continue;
                }

                $fixer = new DependencyFixer($page, $oldName, $newName);
                $fixer->setUser($user);
                $fixer->fixLinks();

                $od = new Outdater();
                $od->pageEvent('source_changed', $page);
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

        $pagesI = PagePeer::instance()->select($c);

        if (count($pages)>0 || count($pagesI)>0) {
            $runData->setModuleTemplate("Rename/LeftDepsModule");
            $runData->contextAdd("pagesI", $pagesI);
            $runData->contextAdd("pages", $pages);

            $runData->ajaxResponseAdd("leftDeps", true);
        }

        $runData->ajaxResponseAdd("newName", $newName);

        $db->commit();

        sleep(0.5);
    }

    public function setParentPageEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId"); // originating page id.
        $ppName = trim($pl->getParameterValue("parentName"));

        $ppName =  WDStringUtils::toUnixName($ppName);

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
        } else {
            // no need to change!

            throw new ProcessException(_("Parent page has not been changed because the submitted and current values are identical."), "no_change");
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

        // compare title and source (ids and contents)
        if ($toMeta->getTitle() === $currentMeta->getTitle()
                && ($toRevision->getSourceId() === $currentRevision->getSourceId())) {
            throw new ProcessException(_("The title and content source of the current revision and the destination revision are identical. No change has been applied."), "no_change");
        }

        // check for locks first
        PageEditLockPeer::instance()->deleteOutdated($pageId);

        $c = new Criteria();
        $c->add("page_id", $page->getPageId());

        if ($pl->getParameterValue("force") === "yes") {
            PageEditLockPeer::instance()->delete($c);
        }

        $locks = PageEditLockPeer::instance()->select($c);

        if (count($locks)>0) {
            $runData->ajaxResponseAdd("locks", true);
            $runData->contextAdd("locks", $locks);
            $runData->setModuleTemplate("History/RevertPageLockedWin");
            $db->rollback();
            return;
        }

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
        if ($userId == null) {
            $userString = $runData->createIpString();
        }

        if ($toRevision->getSourceId() !== $currentRevision->getSourceId()) {
            $sourceChanged = true;
            $nSource = $toRevision->getSourceText();
            $oSource = $currentRevision->getSourceText();
            if ($nSource === $oSource) {
                $sourceChanged = false;
            }
        }

        if (!$sourceChanged && !$titleChanged) {
            throw new ProcessException(_("The title and content source of the current revision and the destination revision are identical. No change has been applied."), "no_change");
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

        if ($sourceChanged) {
            // first check if store new source as a diff or as a full-source.
            $pageSource = new PageSource();
            $pageSource->setText($nSource);
            $revision->setSinceFullSource(0);
            $revision->setDiffSource(false);
            $pageSource->save();

            $revision->setSourceId($pageSource->getSourceId());
        } else {
            // copy source id i.e. do nothing
        }

        $revision->setRevisionNumber($revision->getRevisionNumber() +1);
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
        $site_id = $site->getSiteId();
        $enable_tag_engine = DB::table('site')->where('site_id', $site_id)->value('enable_tag_engine');
        $page = PagePeer::instance()->selectByPrimaryKey($page_id);

        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $category = $page->getCategory();

        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // Turn the tags into a set.
        if ($tags !== '') {
            $tags = preg_split("/[ ,]+/", $tags);
            $tags = new Set($tags);
        } else {
            $tags = new Set();
        }

        // If Allowed Tags are enabled, ensure all tags are compliant, and return an error listing any non-compliant ones.
        if($enable_tag_engine && !$tags->isEmpty()) {
            $allowed_tags_list = AllowedTags::getAllowedTags($site_id);
            $forbidden_tags = $tags->diff($allowed_tags_list);
            if(!$forbidden_tags->isEmpty()) {
              $error_message = sprintf(_('The tags %s are not valid for this site.'), $forbidden_tags->join(", "));
              throw new ProcessException($error_message, "form_error");
            }
        }

        // Save the tags.
        PagePeer::saveTags($page_id, $tags);

        $od = new Outdater();
        $od->pageEvent("tag_change", $page);

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveBlockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

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
