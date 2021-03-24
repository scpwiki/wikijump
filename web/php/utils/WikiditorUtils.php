<?php

namespace Wikidot\Utils;

use Ozone\Framework\Ozone;

class WikiditorUtils
{

    public static function getPageEditToolbar()
    {
        $smarty = OZONE::getSmartyPlain();

        $toolbarTemplateFile = WIKIJUMP_ROOT.'/templates/Misc/wikiditor/PageEditToolbar.tpl';

        $out = $smarty->fetch($toolbarTemplateFile);

        return $out;
    }
}
