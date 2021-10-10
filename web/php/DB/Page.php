<?php

namespace Wikidot\DB;

use Illuminate\Support\Facades\DB;
use Wikidot\DB\PagePeer;
use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\ProcessException;
use Ozone\Framework\Database\Database;
use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class Page extends PageBase
{

    protected static $_titleTemplate = array();

    public function getSource()
    {
        return $this->getCurrentRevision()->getSourceText();
    }

    public function getMetadata()
    {
        return $this->getCurrentRevision()->getMetadata();
    }

    public function getCompiled()
    {
        $c = new Criteria();
        $c->add("page_id", $this->getPageId());
        $compiled = PageCompiledPeer::instance()->selectOne($c);
        if ($compiled == null) {
            throw new ProcessException("Error getting compiled version of the page.");
        }
        return $compiled;
    }

    public function getCurrentRevision()
    {
        $c = new Criteria();
        $c->add("revision_id", $this->getRevisionId());
        return PageRevisionPeer::instance()->selectOne($c);
    }

    public function outdateCompiled()
    {
        $q = "UPDATE page_compiled SET date_compiled=(now() - interval '1 week') " . "WHERE page_id='" . db_escape_string($this->getPageId()) . "'";
        $db = Database::connection();
        $db->query($q);
    }

    public function getFiles()
    {
        $q = "SELECT * FROM file WHERE page_id='" . $this->getPageId() . "' ORDER BY filename, file_id DESC";
        $c = new Criteria();
        $c->setExplicitQuery($q);

        return FilePeer::instance()->select($c);
    }

    public function getCategoryName()
    {
        $unixName = $this->getUnixName();
        if (strpos($unixName, ":") != false) {
            $tmp0 = explode(':', $unixName);
            $categoryName = $tmp0[0];
        } else {
            $categoryName = "_default";
        }
        return $categoryName;
    }

    public function getCategory()
    {
        $categoryId = $this->getCategoryId();
        $siteId = $this->getSiteId();

        return CategoryPeer::instance()->selectById($categoryId, $siteId);
    }

    public function getTitleOrUnixName()
    {
        $title = $this->getTitle();
        if ($title === null || $title === '') {
            $title = ucfirst(str_replace("-", " ", preg_replace("/^[a-z0-9\-]+:/i", '', $this->getUnixName())));
        }
        return $title;
    }

    public function getPreview($length = 200)
    {
        if (is_array($this->prefetched)) {
            if (in_array('page_compiled', $this->prefetched)) {
                if (in_array('page_compiled', $this->prefetchedObjects)) {
                    $compiled = $this->prefetchedObjects['page_compiled'];
                } else {
                    $obj = new PageCompiled($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['page_compiled'] = $obj;
                    $compiled = $obj;
                }
            }
        }
        if ($compiled == null) {
            $c = new Criteria();
            $c->add("page_id", $this->getPageId());
            $compiled = PageCompiledPeer::instance()->selectOne($c);
        }
        $text = $compiled->getText();
        $text = preg_replace(';<table style=".*?id="toc".*?</table>;s', '', $text, 1);
        $stripped = strip_tags($text);
        $d = utf8_encode("\xFE");
        $stripped = preg_replace("/" . $d . "module \"([a-zA-Z0-9\/_]+?)\"(.+?)?" . $d . "/", '', $stripped);
        $stripped = str_replace($d, '', $stripped);
        // get last position of " "
        if (strlen8($stripped) > $length) {
            $substr = substr($stripped, 0, $length);
            $length = strrpos($substr, " ");
            $substr = trim(substr($substr, 0, $length));
            $substr .= '...';
        } else {
            $substr = $stripped;
        }
        return $substr;
    }

    public function getLastEditUserOrString()
    {
        $user = $this->getLastEditUser();
        if ($user == null) {
            return $this->getLastEditUserString();
        } else {
            return $user;
        }
    }

    public function getLastEditUser()
    {
        if ($this->getLastEditUserId() == User::ANONYMOUS_USER) {
            return null;
        }
        return User::find($this->getLastEditUserId());
    }

    public function getSite()
    {
        return SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
    }

    public function getTags()
    {
        return PagePeer::getTags($this->getPageId());
    }

    public function getTitle()
    {
        $categoryId = $this->getCategoryId();
        if ($categoryId) {
            if (!array_key_exists($categoryId, self::$_titleTemplate)) {
                /* Check for template. */
                $c = new Criteria();
                $templateUnixName = '_titletemplate';
                if ($this->getCategoryName() != '_default') {
                    $templateUnixName = $this->getCategoryName() . ':' . $templateUnixName;
                }

                $c->add('unix_name', $templateUnixName);
                $c->add('site_id', $this->getSiteId());
                $templatePage = PagePeer::instance()->selectOne($c);
                if ($templatePage) {
                    $templateSource = $templatePage->getSource();
                    if (strlen($templateSource) > 0 && strlen($templateSource) < 200 && !strpos($templateSource, "\n")) {
                        self::$_titleTemplate[$categoryId] = $templateSource;
                    } else {
                        self::$_titleTemplate[$categoryId] = false;
                    }
                } else {
                    self::$_titleTemplate[$categoryId] = false;
                }
            }
            $titleTemplate = self::$_titleTemplate[$categoryId];
            if ($titleTemplate) {
                /* Process the template. */
                $b = $titleTemplate;
                $b = str_replace('%%page_unix_name%%', preg_replace('/^[a-z0-9]+:/', '', $this->getUnixName()), $b);
                $b = str_replace('%%title%%', parent::getTitle(), $b);
                return $b;
            }
        }
        return parent::getTitle();
    }

    public function getTitleRaw()
    {
        return parent::getTitle();
    }
}
