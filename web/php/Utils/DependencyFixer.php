<?php

namespace Wikidot\Utils;

use Ozone\Framework\ODate;
use Wikidot\DB\PageSource;

/**
 * This Class handles source changes looking for inslusions and links and
 * tries to change them upon destinatin (or included) page name change.
 */
class DependencyFixer
{
    private $page;
    private $oldPageName;
    private $newPageName;

    private $user;

    public function __construct($page, $oldPageName, $newPageName)
    {
        $this->page = $page;
        $this->oldPageName = $oldPageName;
        $this->newPageName = $newPageName;
    }

    public function fixLinks()
    {
        // get the current source. for sure check the lock.
        // also note that $page should be selected with "FOR UPDATE" clause

        $oldSourceText = $this->page->getSource();
        $sourceChanged = false;

        $source = $oldSourceText;
        $source = preg_replace_callback(
            '/
            (\[\[\[)             # Detect freelinks: opening brackets
            ([^\]\|]+?)          # Link location
            (
                (\s*\|[^\]]*?)?  # Pipe to split link then link text (not ])
            \]\]\])              # Closing brackets
            /ix',
            array(&$this, 'fixLink'),
            $source
        );
        $source = preg_replace_callback(
            '/
            ^
            \[\[include ([a-zA-Z0-9\s\-]+?)(?:\]\])
            $
            /imx',
            array(&$this, 'fixInclusion'),
            $source
        );
        if ($source != $oldSourceText) {
            $page = $this->page;
            $currentRevision = $page->getCurrentRevision();
            //save it! wooohaaa! should we not clean the page source saving a bit?
            $revision = clone($currentRevision);
            $revision->setNew(true);
            $revision->setRevisionId(null);
            $revision->resetFlags();
            $revision->setFlagText(true);
            $revision->setPageId($page->getPageId());
            $revision->setRevisionNumber($currentRevision->getRevisionNumber()+1);

            $now = new ODate();
            $revision->setDateLastEdited($now);

            $pageSource = new PageSource();
            $pageSource->setText($source);
            $revision->setDiffSource(false);
            $revision->setSinceFullSource(0);
            $pageSource->save();

            $revision->setSourceId($pageSource->getSourceId());

            $revision->setComments(sprintf(_('Automatic update related to page rename: "%s" to "%s".'), $this->oldPageName, $this->newPageName));

            $userId = $this->user->id;

            if ($userId) {
                $revision->setUserId($userId);
                $page->setLastEditUserId($userId);
            }

            $revision->save();
            $page->setRevisionId($revision->getRevisionId());
            $page->setDateLastEdited($now);
            $page->setRevisionNumber($revision->getRevisionNumber());
            $page->save();

            // force page compilation
        }
    }

    public function fixInclusion($matches)
    {
        $pageName =  WDStringUtils::toUnixName(trim($matches[1]));
        if ($pageName != $this->oldPageName) {
            return $matches[0];
        } else {
            return  '[[include '.$this->newPageName.']]';
        }
    }

    private function fixLink($matches)
    {

        $pageName = WDStringUtils::toUnixName($matches[2]);
        $start = $matches[1];
        $rest = $matches[3];
        if ($pageName != $this->oldPageName) {
            return $matches[0];
        } else {
            return $start.$this->newPageName.$rest;
        }
    }

    public function setUser($user)
    {
        $this->user = $user;
    }
}
