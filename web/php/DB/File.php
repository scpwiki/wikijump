<?php

namespace Wikidot\DB;


use Wikidot\Utils\FileHelper;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class File extends FileBase
{

    private $cachedSite;

    public function getSizeString()
    {
        return FileHelper::formatSize($this->getSize());
    }

    public function getFilePath()
    {
        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
        return WIKIJUMP_ROOT."/web/files--sites/".
            $site->getSlug()."/files/".$page->getUnixName().'/'.$this->getFilename();
    }

    public function getResizedDir()
    {
        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
        return WIKIJUMP_ROOT."/web/files--sites/".
                        $site->getSlug()."/resized-images/".$page->getUnixName().
                        '/'.$this->getFilename();
    }

    public function getResizedURI($size = null)
    {

        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
        $out =  GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/local--resized-images/".
            $page->getUnixName().'/'.$this->getFilename();
        if ($size) {
            $out .= '/'.strtolower($size).'.jpg';
        }
        return $out;
    }

    public function getFileURI()
    {
        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = SitePeer::instance()->selectByPrimaryKey($this->getSiteId());

        return  GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/local--files/".
            $page->getUnixName()."/".$this->getFilename();
    }

    public function getUser()
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }

        return User::find($this->getUserId());
    }

    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }
}
