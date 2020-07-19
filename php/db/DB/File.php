<?php
namespace DB;

use \FileHelper;
use GlobalProperties;

/**
 * Object Model class.
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
        return WIKIDOT_ROOT."/web/files--sites/".
            $site->getUnixName()."/files/".$page->getUnixName().'/'.$this->getFilename();
    }

    public function getResizedDir()
    {
        $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
        $site = SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
        return WIKIDOT_ROOT."/web/files--sites/".
                        $site->getUnixName()."/resized-images/".$page->getUnixName().
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
        if ($this->getUserId() == 0) {
            return null;
        }
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
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
