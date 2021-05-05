<?php

namespace Wikidot\Wikitext;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Text_Wiki;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\DB\PageTagPeer;
use function Wikidot\Utils\Exception;

//use Text_Antiwiki;  # What is this? I can't even find "text_antiwiki on google.

//require_once(WIKIJUMP_ROOT."/vendor/scpwiki/text_wiki/Text/Wiki.php");

function wikiPageExists($pageName)
{

    if ($GLOBALS['site'] == null) {
        $runData = Ozone::getRunData();
        $siteId = $runData->getTemp("site")->getSiteId();
    } else {
        $siteId = $GLOBALS['site']->getSiteId();
    }
    $q = "SELECT page_id FROM page WHERE unix_name='".db_escape_string($pageName)."' AND site_id='".db_escape_string($siteId)."' LIMIT 1";
    $db = Database::connection();
    $r = $db->query($q);
    if ($row = $r->nextRow()) {
        return $row['page_id'];
    } else {
        return false;
    }
}
