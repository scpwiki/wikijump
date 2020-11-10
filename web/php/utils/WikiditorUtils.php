<?php
class WikiditorUtils
{

    public static function getPageEditToolbar()
    {
        $smarty = OZONE::getSmartyPlain();

        $toolbarTemplateFile = WIKIJUMP_ROOT.'/templates/misc/wikiditor/PageEditToolbar.tpl';

        $out = $smarty->fetch($toolbarTemplateFile);

        return $out;
    }
}
