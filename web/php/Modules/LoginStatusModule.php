<?php

namespace Wikidot\Modules;
use Illuminate\Support\Facades\Auth;
use Ozone\Framework\Module;
use Wikidot\Utils\GlobalProperties;

class LoginStatusModule extends Module
{

    public function render($runData)
    {
        // use non-templating solution to have some optimisation here. not to call
        // Smarty if not required!

        $user = $runData->getUser();

        if (GlobalProperties::$WIKI_FARM) {
            $url_prefix = GlobalProperties::$HTTP_SCHEMA.'://' . GlobalProperties::$URL_HOST;
        } else {
            $url_prefix = '';
        }

        if($user === null)
        {
            $user = Auth::user();
        }

        if ($user == null) {
            $site = $runData->getTemp('site');

            $originalUrl = $_SERVER['REQUEST_URI'];
            if (GlobalProperties::$WIKI_FARM) {
                $originalUrl = $_SERVER['HTTP_HOST'] . $originalUrl;
            }

            if (preg_match('/\?origUrl=.*$/', $originalUrl)) {
                $o = array();
                parse_str(preg_replace('/^.*?\?/', '', $_SERVER['REQUEST_URI']), $o);
                $originalUrl = $o['origUrl'];
            }

            if ($site->getLanguage() != 'en') {
                $loginDomain = $site->getLanguage();
            }

            $out  = '<a href="' . route('register') . '">' . _('Register') . '</a> '._('or') . ' ';
            $out .= '<a href="' . route('login') . '">' . _('Log In') . '</a>';

            //$out = '<a href="javascript:;" onclick="Wikijump.page.listeners.createAccount(event)">'._('create account').'</a> '._('or').' <a href="javascript:;" onclick="Wikijump.page.listeners.loginClick(event)">'._('login').'</a>';
        } else {
            $lang = $user->language;

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

            $userId = $user->id;
            $linkInner = 'href="' . $url_prefix . '/user:info/'.$user->unix_name.'" onclick="Wikijump.page.listeners.userInfo('.$user->id.'); return false;" ';

            $out = '<span class="printuser"><a '.$linkInner.'><img class="small" src="' . $user->avatarSmall() . '" alt="avatar"';
            /* karma: */
            $out .= ' style="background-image:url(' . $url_prefix . '/user--karma/' .$userId  . ')"';
            /* end of karma */
            $out .= '/></a>';
            $out .= $user->username.'</span>'.
                    ' | <a href="' . $url_prefix .'/account:you">'._('my account').'</a>' .
                    '<a  id="account-topbutton" href="javascript:;">&nabla;</a>';
            $out .= '<div id="account-options">' .
                    '<ul>' .
                    '<li><a href="' . $url_prefix . '/account:you">'._('account summary').'</a></li>' .
                    '<li><a href="' . $url_prefix . '/account:you/start/messages">'._('private messages').'</a></li>' .
                    '<li><a href="' . $url_prefix . '/account:you/start/contacts">'._('my contacts').'</a></li>' .
                    '<li><a href="' . $url_prefix . '/account:you/start/notifications">'._('notifications').'</a></li>'.
                    '<li><a href="' . $url_prefix . '/account:you/start/watched-changes">'._('watched pages').'</a></li>'.
                    '<li><a href="' . $url_prefix . '/account:you/start/watched-forum">'._('watched discussions').'</a></li>'.
                    '<li><a href="' . route('logout') . '">'._('Log Out').'</a></li>' .
                    '</ul></div>';

            // back the language!

            $lang = $GLOBALS['lang'];

            switch ($lang) {
                case 'pl':
                    $glang = "pl_PL";
                    break;
                case 'en':
                    $glang = "en_US";
                    break;
            }

            putenv("LANG=$glang");
            putenv("LANGUAGE=$glang");
            setlocale(LC_ALL, $glang . '.UTF-8');
        }

        return $out;
    }
}
