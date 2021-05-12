<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Database;

class FileHelper
{

    public static function totalSiteFilesSize($siteId)
    {
        $q = "SELECT sum(size) AS size FROM file WHERE site_id='".db_escape_string($siteId)."'	";
        $db = Database::connection();
        $r = $db->query($q)->nextRow();
        return $r['size'];
    }

    public static function totalPageFilesSize($pageId)
    {
        $q = "SELECT sum(size) as size FROM file WHERE page_id='".db_escape_string($pageId)."'	";
        $db = Database::connection();
        $r = $db->query($q)->nextRow();
        return $r['size'];
    }

    public static function totalSiteFileNumber($siteId)
    {
        $q = "SELECT count(*) AS count FROM file WHERE site_id='".db_escape_string($siteId)."'	";
        $db = Database::connection();
        $r = $db->query($q)->nextRow();
        return $r['count'];
    }

    public static function formatSize($size)
    {
        $filesizename = array(" Bytes", " kB", " MB", " GB", " TB", " PB");
        if ($size == 0) {
            return "0 Bytes";
        }
        return round($size/pow(1024, ($i = floor(log($size, 1024)))), 2) . $filesizename[$i];
    }
}
