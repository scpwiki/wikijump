<?php
use DB\SitePeer;
use DB\ThemePeer;

class LoginIframeScreen extends SmartyScreen
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $url = $pl->getParameterValue('url');
        $siteId = $pl->getParameterValue('siteId');
        if ($siteId && is_numeric($siteId)) {
            $site = SitePeer::instance()->selectByPrimaryKey($siteId);
        }
        if (!$site) {
            throw new ProcessException(_('Invalid site'));
        }

        $runData->setLanguage($site->getLanguage());
        $GLOBALS['lang'] = $site->getLanguage();

        // and for gettext too:

        $lang = $site->getLanguage();

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        // Set the text domain as 'messages'
        $gdomain = 'messages';
        bindtextdomain($gdomain, WIKIDOT_ROOT.'/locale');
        textdomain($gdomain);

        $themeId = $pl->getParameterValue('themeId');

        if ($themeId && is_numeric($themeId)) {
            $theme = ThemePeer::instance()->selectByPrimaryKey($themeId);
        }
        if (!$theme) {
            throw new ProcessException(_('Invalid theme'));
        }

        $runData->contextAdd('site', $site);
        $runData->contextAdd('theme', $theme);
        $runData->contextAdd('url', $url);

        $seed = CryptUtils::generateSeed(4);

        // put seed into session!
        $runData->sessionStart();
        $runData->sessionAdd("login_seed", $seed);

        $runData->contextAdd("key", CryptUtils::modulus());
        $runData->contextAdd("seed", $seed);

        // clear welcome cookie?
        if ($pl->getParameterValue("clearwelcome")) {
            $runData->contextAdd('reset', true);
        }
    }
}
